//! A simulated host and fake server. It performs every effect, advances scripted time, answers
//! from the script, and checks the rules that must hold in every session.

use std::collections::BTreeMap;

use contract::api::{self, Auth};
use contract::core_msg::{BodyTarget, CoreOut, Effect, EffectBody, HttpClass, HttpMethod, StoreOp};
use contract::version::{API_VERSION_HEADER, WIRE_VERSION};
use core_rules::net::class_of;
use core_rules::{Core, step_json};
use offline::{Config as LayerConfig, Layer};
use serde::Deserialize;
use serde_json::{Value, json};

const WALL_START_MS: u64 = 1_793_836_800_000;

/// How a session runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The core alone.
    LayerOff,
    /// The offline layer around the core.
    LayerOn,
}

/// What the device has.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Device {
    pub secret: bool,
    pub store: bool,
    pub storage_bytes: u64,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            secret: true,
            store: true,
            storage_bytes: 1_000_000,
        }
    }
}

/// One scripted answer of the fake server.
#[derive(Clone, Debug, Deserialize)]
pub struct Rule {
    pub method: String,
    pub path: String,
    #[serde(default = "ok")]
    pub status: u32,
    #[serde(default)]
    pub body: Value,
    #[serde(default)]
    pub etag: Option<String>,
    /// Answers this many times, then the next rule that fits.
    #[serde(default)]
    pub times: Option<u32>,
    /// Scripted time from which this rule applies.
    #[serde(default)]
    pub from_ms: u64,
}

const fn ok() -> u32 {
    200
}

/// What a step expects. A part that is absent is not checked. Lists hold what happened since the
/// last step, exactly. `view` is compared by part: every key named must match, `null` means absent.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Want {
    pub view: Option<Value>,
    pub sent: Option<Vec<String>>,
    /// Bearer values of the requests sent, in order, leaving out requests with none.
    pub bearers: Option<Vec<String>>,
    pub player: Option<Vec<Value>>,
    pub ops: Option<Vec<Value>>,
    pub routes: Option<Vec<Value>>,
    pub notices: Option<Vec<String>>,
    pub logs: Option<Vec<String>>,
    pub pages: Option<Vec<Value>>,
    pub wake: Option<Value>,
    pub queued: Option<usize>,
    pub kept_pages: Option<usize>,
    pub secrets: Option<Vec<String>>,
    /// Bodies of requests to a path, in order.
    pub bodies: Option<BTreeMap<String, Vec<Value>>>,
}

/// One scripted moment.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Moment {
    pub at_ms: u64,
    pub events: Vec<Value>,
    /// "down" or "up".
    pub net: Option<String>,
    /// Rebuild the core as a new run. True also changes the boot, so monotonic time restarts.
    pub restart: Option<bool>,
    /// The secret store comes back empty at this restart.
    pub forget_secrets: bool,
    pub want: Want,
    pub want_on: Want,
    pub want_off: Want,
}

/// One JSON session.
#[derive(Clone, Debug, Deserialize)]
pub struct Script {
    pub name: String,
    pub covers: Vec<String>,
    pub about: String,
    #[serde(default)]
    pub prologue: Option<String>,
    #[serde(default)]
    pub config: Option<Value>,
    #[serde(default)]
    pub device: Device,
    #[serde(default)]
    pub wall_skew_ms: i64,
    #[serde(default)]
    pub server: Vec<Rule>,
    pub moments: Vec<Moment>,
}

/// Shared parts of the sessions.
#[derive(Clone, Debug, Deserialize)]
pub struct Common {
    pub config: Value,
    pub server: Vec<Rule>,
    pub prologues: BTreeMap<String, Vec<Moment>>,
}

/// Sent to the fake server.
#[derive(Clone, Debug)]
pub struct Sent {
    pub line: String,
    pub auth: Option<String>,
    pub at_ms: u64,
}

/// What one run left behind.
#[derive(Debug, Default)]
pub struct Report {
    /// Every message pair, in order, as JSON text.
    pub transcript: Vec<(String, String)>,
    pub sent: Vec<Sent>,
    pub ops: Vec<Value>,
    pub view: Value,
    pub steps: usize,
}

enum Machine {
    Bare(Box<Core>),
    Wrapped(Box<Layer<Core>>),
}

impl Machine {
    fn open(script: &Script, config: &Value, mode: Mode, boot: u64) -> Result<Self, String> {
        let core =
            Core::open_json(&config.to_string()).map_err(|_| "config refused".to_string())?;
        Ok(match mode {
            Mode::LayerOff => Self::Bare(Box::new(core)),
            Mode::LayerOn => Self::Wrapped(Box::new(Layer::new(
                core,
                LayerConfig {
                    boot_id: format!("boot-{boot}"),
                    storage_bytes: script.device.storage_bytes,
                },
            ))),
        })
    }

    fn step(&mut self, input: &str) -> Result<String, String> {
        let bad = |_| format!("the machine refused a message: {input}");
        match self {
            Self::Bare(core) => step_json(core.as_mut(), input).map_err(bad),
            Self::Wrapped(layer) => step_json(layer.as_mut(), input).map_err(bad),
        }
    }
}

/// `want` must be inside `have`. An object key may be null to mean absent.
fn fits(have: &Value, want: &Value) -> bool {
    match (have, want) {
        (Value::Object(have), Value::Object(want)) => want.iter().all(|(key, part)| {
            part.is_null() && !have.contains_key(key)
                || have.get(key).is_some_and(|inner| fits(inner, part))
        }),
        (Value::Array(have), Value::Array(want)) => {
            have.len() == want.len() && have.iter().zip(want).all(|(a, b)| fits(a, b))
        }
        _ => have == want,
    }
}

/// `want` holds for each item, in order, and nothing else happened.
fn list_fits(name: &str, have: &[Value], want: &[Value]) -> Result<(), String> {
    if have.len() == want.len() && have.iter().zip(want).all(|(a, b)| fits(a, b)) {
        Ok(())
    } else {
        Err(format!(
            "{name}: wanted {}\n got {}",
            Value::from(want.to_vec()),
            Value::from(have.to_vec())
        ))
    }
}

#[derive(Default)]
struct Since {
    sent: Vec<String>,
    bearers: Vec<String>,
    player: Vec<Value>,
    ops: Vec<Value>,
    routes: Vec<Value>,
    notices: Vec<String>,
    logs: Vec<String>,
    pages: Vec<Value>,
    bodies: BTreeMap<String, Vec<Value>>,
}

struct Host<'a> {
    script: &'a Script,
    rules: Vec<Rule>,
    config: Value,
    mode: Mode,
    machine: Machine,
    boot: u64,
    /// Scripted time, which never restarts.
    clock_ms: u64,
    mono_base_ms: u64,
    net_up: bool,
    wake: Option<u64>,
    secrets: BTreeMap<String, String>,
    store: BTreeMap<String, String>,
    view: Value,
    since: Since,
    tokens: Vec<String>,
    source_open: bool,
    generation: u64,
    report: Report,
}

fn method_word(method: HttpMethod) -> Result<&'static str, String> {
    Ok(match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
        other => return Err(format!("unknown method {other:?}")),
    })
}

fn collect_tokens(value: &Value, found: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, part) in map {
                if (key == "access_token" || key == "refresh_token")
                    && let Some(text) = part.as_str()
                {
                    found.push(text.to_string());
                }
                collect_tokens(part, found);
            }
        }
        Value::Array(list) => list.iter().for_each(|part| collect_tokens(part, found)),
        _ => {}
    }
}

impl Host<'_> {
    fn mono(&self) -> u64 {
        self.clock_ms - self.mono_base_ms
    }

    fn wall(&self) -> u64 {
        WALL_START_MS
            .saturating_add(self.clock_ms)
            .saturating_add_signed(self.script.wall_skew_ms)
    }

    fn step(&mut self, mut events: Vec<Value>) -> Result<(), String> {
        for round in 0.. {
            if round > 64 {
                return Err(format!("no end at {} ms", self.clock_ms));
            }
            let input = json!({
                "contract": WIRE_VERSION,
                "type": "core_in",
                "now_ms": self.wall(),
                "mono_ms": self.mono(),
                "events": events,
            })
            .to_string();
            let output = self.machine.step(&input)?;
            let parsed: CoreOut = serde_json::from_str(&output)
                .map_err(|error| format!("bad out message: {error}: {output}"))?;
            self.report.transcript.push((input, output));
            self.report.steps += 1;
            if let Some(view) = &parsed.view {
                self.fold_view(view)?;
            }
            self.wake = parsed.next_wake_ms;
            if !parsed.unsupported.is_empty() {
                return Err(format!("unsupported events: {:?}", parsed.unsupported));
            }
            events = Vec::new();
            for effect in parsed.effects {
                self.perform(effect, &mut events)?;
            }
            if events.is_empty() {
                break;
            }
        }
        if self.wake.is_some_and(|wake| wake <= self.mono()) {
            return Err(format!(
                "busy loop: wake {:?} at mono {}",
                self.wake,
                self.mono()
            ));
        }
        Ok(())
    }

    fn fold_view(&mut self, view: &contract::core_msg::View) -> Result<(), String> {
        let value = serde_json::to_value(view).map_err(|error| error.to_string())?;
        let text = value.to_string();
        for token in &self.tokens {
            if text.contains(token.as_str()) {
                return Err(format!("a token reached the view: {text}"));
            }
        }
        if let Value::Object(parts) = value {
            for (part, content) in parts {
                self.view[part] = content;
            }
        }
        Ok(())
    }

    fn perform(&mut self, effect: Effect, results: &mut Vec<Value>) -> Result<(), String> {
        let Effect { id, body } = effect;
        match body {
            EffectBody::Http {
                method,
                path,
                headers,
                body,
                class,
                to,
                ..
            } => self.http(&id, method, &path, &headers, body, class, to, results),
            EffectBody::Store { op, key, value } => self.store_effect(&id, op, key, value, results),
            EffectBody::Secret { op, name, value } => {
                if !self.script.device.secret {
                    results.push(json!({"id": id, "type": "unsupported"}));
                    return Ok(());
                }
                let mut answer = json!({"id": id, "type": "secret_done"});
                match op {
                    contract::core_msg::SecretOp::Get => {
                        if let Some(held) = self.secrets.get(&name) {
                            answer["value"] = json!(held);
                        }
                    }
                    contract::core_msg::SecretOp::Put => {
                        self.secrets.insert(name, value.unwrap_or_default());
                    }
                    contract::core_msg::SecretOp::Delete => {
                        self.secrets.remove(&name);
                    }
                    other => return Err(format!("unknown secret op {other:?}")),
                }
                results.push(answer);
                Ok(())
            }
            EffectBody::Timer { .. } => {
                results.push(json!({"id": id, "type": "unsupported"}));
                Ok(())
            }
            EffectBody::Player { command } => self.player(&command),
            EffectBody::View { notice, body } => {
                if let Some(notice) = notice {
                    self.since.notices.push(notice.label);
                }
                if let Some(body) = body {
                    let page: Value = serde_json::from_str(&body.body)
                        .map_err(|error| format!("kept body is not JSON: {error}"))?;
                    self.since
                        .pages
                        .push(json!({"page": page, "stale": body.stale}));
                }
                Ok(())
            }
            EffectBody::Open { target } => {
                let target = serde_json::to_value(target).map_err(|error| error.to_string())?;
                self.since.routes.push(target);
                Ok(())
            }
            EffectBody::Log { code, .. } => {
                self.since.logs.push(code);
                Ok(())
            }
            other => Err(format!("unknown effect {other:?}")),
        }
    }

    fn player(&mut self, command: &contract::driver::DriverCommand) -> Result<(), String> {
        use contract::driver::DriverCommand as Driver;
        let value = serde_json::to_value(command).map_err(|error| error.to_string())?;
        match command {
            Driver::Open {
                generation,
                headers,
                ..
            } => {
                if self.source_open {
                    return Err(format!("open with a source still open: {value}"));
                }
                if u64::from(*generation) <= self.generation {
                    return Err(format!("generation went back: {value}"));
                }
                if headers
                    .keys()
                    .any(|name| name.eq_ignore_ascii_case("authorization"))
                {
                    return Err(format!("a token reached a video host: {value}"));
                }
                self.generation = u64::from(*generation);
                self.source_open = true;
            }
            Driver::Close => self.source_open = false,
            _ => {}
        }
        self.since.player.push(value);
        Ok(())
    }

    fn store_effect(
        &mut self,
        id: &str,
        op: StoreOp,
        key: Option<String>,
        value: Option<String>,
        results: &mut Vec<Value>,
    ) -> Result<(), String> {
        if !self.script.device.store {
            results.push(json!({"id": id, "type": "unsupported"}));
            return Ok(());
        }
        if self.mode == Mode::LayerOff {
            return Err("the core used the store".to_string());
        }
        let key = key.unwrap_or_default();
        let mut answer = json!({"id": id, "type": "store_done"});
        match op {
            StoreOp::Get => {
                if let Some(held) = self.store.get(&key) {
                    answer["value"] = json!(held);
                }
            }
            StoreOp::Put => {
                let value = value.unwrap_or_default();
                for token in &self.tokens {
                    if value.contains(token.as_str()) {
                        return Err(format!("a token reached the store: {value}"));
                    }
                }
                self.store.insert(key, value);
            }
            StoreOp::Delete => {
                self.store.remove(&key);
            }
            StoreOp::List => {
                let keys: Vec<&String> = self
                    .store
                    .keys()
                    .filter(|name| name.starts_with(&key))
                    .collect();
                answer["keys"] = json!(keys);
            }
            other => return Err(format!("unknown store op {other:?}")),
        }
        results.push(answer);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn http(
        &mut self,
        id: &str,
        method: HttpMethod,
        path: &str,
        headers: &BTreeMap<String, String>,
        body: Option<String>,
        class: HttpClass,
        to: BodyTarget,
        results: &mut Vec<Value>,
    ) -> Result<(), String> {
        let word = method_word(method)?;
        let bare = path.split('?').next().unwrap_or(path);
        if !path.starts_with("/v1/") || path.contains("://") {
            return Err(format!("not a path: {path}"));
        }
        let route = api::find(word, bare).ok_or_else(|| format!("no route {word} {path}"))?;
        let auth = headers.get("Authorization").cloned();
        if matches!(route.auth, Auth::Open) && auth.is_some() {
            return Err(format!("a token went to an open route: {path}"));
        }
        for name in [
            API_VERSION_HEADER,
            "Client-App",
            "Client-Version",
            "Client-Class",
            "Client-Os",
        ] {
            if !headers.contains_key(name) {
                return Err(format!("{path} lacks {name}"));
            }
        }
        let held_by_layer =
            class == HttpClass::Plain && class_of(method, path) == HttpClass::Command;
        if class != class_of(method, path) && !(held_by_layer && self.mode == Mode::LayerOn) {
            return Err(format!("{path} marked {class:?}"));
        }
        if self.mode == Mode::LayerOn && class == HttpClass::Command {
            return Err("a command got past the layer".to_string());
        }
        self.since.sent.push(format!("{word} {path}"));
        if let Some(bearer) = auth
            .as_deref()
            .and_then(|text| text.strip_prefix("Bearer "))
        {
            self.since.bearers.push(bearer.to_string());
        }
        if let Some(text) = &body {
            let value = serde_json::from_str(text).map_err(|error| error.to_string())?;
            self.since
                .bodies
                .entry(path.to_string())
                .or_default()
                .push(value);
        }
        self.report.sent.push(Sent {
            line: format!("{word} {path}"),
            auth,
            at_ms: self.clock_ms,
        });
        if !self.net_up {
            results.push(json!({"id": id, "type": "http_failed", "reason": "offline"}));
            return Ok(());
        }
        let found = self.rules.iter().position(|rule| {
            rule.method == word
                && rule.path == path
                && rule.from_ms <= self.clock_ms
                && rule.times != Some(0)
        });
        let Some(found) = found else {
            if word == "POST" && path == "/v1/commands" {
                return self.commands(id, body.as_deref(), results);
            }
            return Err(format!("no answer scripted for {word} {path}"));
        };
        if let Some(times) = &mut self.rules[found].times {
            *times -= 1;
        }
        let rule = self.rules[found].clone();
        if word == "POST" && path == "/v1/commands" && (200..300).contains(&rule.status) {
            return self.commands(id, body.as_deref(), results);
        }
        if let Some(etag) = &rule.etag
            && headers.get("If-None-Match") == Some(etag)
        {
            results.push(json!({"id": id, "type": "http_done", "status": 304}));
            return Ok(());
        }
        collect_tokens(&rule.body, &mut self.tokens);
        let mut answer = json!({"id": id, "type": "http_done", "status": rule.status});
        if let Some(etag) = rule.etag {
            answer["headers"] = json!({"etag": etag});
        }
        let good = (200..300).contains(&rule.status);
        if to == BodyTarget::Screen && good {
            self.since
                .pages
                .push(json!({"page": rule.body, "stale": false}));
        } else if !rule.body.is_null() {
            answer["body"] = json!(rule.body.to_string());
        }
        results.push(answer);
        Ok(())
    }

    fn commands(
        &mut self,
        id: &str,
        body: Option<&str>,
        results: &mut Vec<Value>,
    ) -> Result<(), String> {
        let parsed: Value = serde_json::from_str(body.unwrap_or_default())
            .map_err(|error| format!("bad batch: {error}"))?;
        let mut answers = Vec::new();
        for op in parsed["ops"].as_array().ok_or("batch has no ops")? {
            answers.push(json!({"op_id": op["op_id"], "outcome": "applied"}));
            self.since.ops.push(op.clone());
            self.report.ops.push(op.clone());
        }
        let answer = json!({"answers": answers}).to_string();
        results.push(json!({"id": id, "type": "http_done", "status": 200, "body": answer}));
        Ok(())
    }

    fn advance_to(&mut self, at_ms: u64) -> Result<(), String> {
        if at_ms < self.clock_ms {
            return Err(format!("time went back to {at_ms}"));
        }
        let mut wakes = 0;
        loop {
            match self.wake.map(|wake| wake + self.mono_base_ms) {
                Some(wake) if wake <= at_ms => {
                    wakes += 1;
                    if wakes > 2_000 {
                        return Err("too many wakes".to_string());
                    }
                    self.clock_ms = wake;
                    self.step(Vec::new())?;
                }
                _ => break,
            }
        }
        self.clock_ms = at_ms;
        Ok(())
    }

    fn restart(&mut self, new_boot: bool) -> Result<(), String> {
        if new_boot {
            self.boot += 1;
            self.mono_base_ms = self.clock_ms;
        }
        self.machine = Machine::open(self.script, &self.config, self.mode, self.boot)?;
        self.source_open = false;
        self.generation = 0;
        self.wake = None;
        self.view = json!({});
        Ok(())
    }

    fn check(&mut self, want: &Want, at_ms: u64) -> Result<(), String> {
        let name = |part: &str| format!("at {at_ms} ms, {part}");
        if let Some(view) = &want.view
            && !fits(&self.view, view)
        {
            return Err(format!(
                "{}: wanted {view}\n got {}",
                name("view"),
                self.view
            ));
        }
        if let Some(sent) = &want.sent
            && self.since.sent != *sent
        {
            return Err(format!(
                "{}: wanted {sent:?}\n got {:?}",
                name("sent"),
                self.since.sent
            ));
        }
        if let Some(bearers) = &want.bearers
            && self.since.bearers != *bearers
        {
            return Err(format!(
                "{}: wanted {bearers:?}\n got {:?}",
                name("bearers"),
                self.since.bearers
            ));
        }
        if let Some(player) = &want.player {
            list_fits(&name("player"), &self.since.player, player)?;
        }
        if let Some(ops) = &want.ops {
            list_fits(&name("ops"), &self.since.ops, ops)?;
        }
        if let Some(routes) = &want.routes {
            list_fits(&name("routes"), &self.since.routes, routes)?;
        }
        if let Some(pages) = &want.pages {
            list_fits(&name("pages"), &self.since.pages, pages)?;
        }
        if let Some(notices) = &want.notices
            && self.since.notices != *notices
        {
            return Err(format!(
                "{}: wanted {notices:?}\n got {:?}",
                name("notices"),
                self.since.notices
            ));
        }
        if let Some(logs) = &want.logs
            && self.since.logs != *logs
        {
            return Err(format!(
                "{}: wanted {logs:?}\n got {:?}",
                name("logs"),
                self.since.logs
            ));
        }
        if let Some(wake) = &want.wake
            && json!(self.wake) != *wake
        {
            return Err(format!(
                "{}: wanted {wake}, got {:?}",
                name("wake"),
                self.wake
            ));
        }
        if let Some(queued) = want.queued {
            let held = self
                .store
                .keys()
                .filter(|key| key.starts_with("q/"))
                .count();
            if held != queued {
                return Err(format!("{}: wanted {queued}, got {held}", name("queued")));
            }
        }
        if let Some(pages) = want.kept_pages {
            let held = self
                .store
                .keys()
                .filter(|key| key.starts_with("c/"))
                .count();
            if held != pages {
                return Err(format!(
                    "{}: wanted {pages}, got {held}",
                    name("kept pages")
                ));
            }
        }
        if let Some(bodies) = &want.bodies {
            for (path, wanted) in bodies {
                let have = self.since.bodies.get(path).map_or(&[][..], Vec::as_slice);
                list_fits(&name(path), have, wanted)?;
            }
        }
        if let Some(secrets) = &want.secrets {
            let held: Vec<&String> = self.secrets.keys().collect();
            if held
                .iter()
                .map(|name| name.as_str())
                .ne(secrets.iter().map(String::as_str))
            {
                return Err(format!(
                    "{}: wanted {secrets:?}, got {held:?}",
                    name("secrets")
                ));
            }
        }
        Ok(())
    }

    fn moment(&mut self, moment: &Moment) -> Result<(), String> {
        self.advance_to(moment.at_ms)?;
        if moment.forget_secrets {
            self.secrets.clear();
        }
        if let Some(new_boot) = moment.restart {
            self.restart(new_boot)?;
        }
        match moment.net.as_deref() {
            Some("down") => self.net_up = false,
            Some("up") => self.net_up = true,
            Some(other) => return Err(format!("unknown net state {other}")),
            None => {}
        }
        self.step(moment.events.clone())?;
        self.check(&moment.want, moment.at_ms)?;
        let by_mode = match self.mode {
            Mode::LayerOff => &moment.want_off,
            Mode::LayerOn => &moment.want_on,
        };
        self.check(by_mode, moment.at_ms)?;
        self.since = Since::default();
        Ok(())
    }
}

/// Run one session.
///
/// # Errors
/// The first thing that broke, in words.
pub fn run(script: &Script, common: &Common, mode: Mode) -> Result<Report, String> {
    let config = script
        .config
        .clone()
        .unwrap_or_else(|| common.config.clone());
    let mut rules = script.server.clone();
    rules.extend(common.server.iter().cloned());
    let prologue = match &script.prologue {
        Some(name) => common
            .prologues
            .get(name)
            .ok_or_else(|| format!("no prologue {name}"))?
            .clone(),
        None => Vec::new(),
    };
    let mut host = Host {
        machine: Machine::open(script, &config, mode, 0)?,
        script,
        rules,
        config,
        mode,
        boot: 0,
        clock_ms: 0,
        mono_base_ms: 0,
        net_up: true,
        wake: None,
        secrets: BTreeMap::new(),
        store: BTreeMap::new(),
        view: json!({}),
        since: Since::default(),
        tokens: Vec::new(),
        source_open: false,
        generation: 0,
        report: Report::default(),
    };
    for moment in prologue.iter().chain(&script.moments) {
        host.moment(moment)
            .map_err(|error| format!("{}: {error}", script.name))?;
    }
    host.report.view = host.view.clone();
    Ok(host.report)
}
