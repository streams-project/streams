//! The host: it steps the core and performs each effect with what a desktop has.
//!
//! The core is a Rust library here, so it is linked and stepped directly. The offline layer sits
//! around it, which is what makes the store port matter on a machine that has a disk.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use contract::client::{CoreMode, DeviceClass, Form};
use contract::core_msg::{
    BodyTarget, CoreConfig, CoreConfigTag, CoreIn, CoreOut, Effect, EffectBody, Event, EventBody,
    LayerConfig, RouteTarget, ScreenBody, SecretOp, StoreOp, TimerOp, View,
};
use contract::error::Notice;
use core_rules::Step;
use offline::Machine;
use rand::Rng as _;
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

use crate::adapters::{FileSecrets, FileStore, HttpNetwork, NoPlayer, NoReel};
use crate::ports::{Answer, Call, ETAG, Network, Player, Reel, Secrets, Store};

/// The app word every request carries. It names this family, not a product: the brand is not in
/// this code and the shell passes everything the brand decides.
pub const APP: &str = "desktop";

/// How many times one turn lets the core answer itself before the host stops. The other hosts hold
/// the same figure.
const STEPS: usize = 64;

/// Where the secrets and the layer's own files sit under the folder the shell names.
const SECRETS: &str = "secrets";
const STORE: &str = "store";

/// What the shell tells this host. Nothing here is a setting a viewer sees: a brand's name, its
/// folder and its server are the shell's to resolve, exactly as `launcher` resolves them today.
pub struct Settings {
    /// Where every request goes. The core only ever names a path.
    pub server: String,
    /// Where this host writes. Everything it writes stays under it.
    pub data_dir: PathBuf,
    /// Bytes the offline layer may keep. 0 keeps nothing across a restart.
    pub storage_bytes: u64,
}

/// The five ports, filled. `Host::open` fills them with what a desktop has; a test fills them with
/// what it wants to watch.
pub struct Ports {
    pub network: Box<dyn Network>,
    pub secrets: Box<dyn Secrets>,
    pub store: Box<dyn Store>,
    pub player: Box<dyn Player>,
    pub reel: Box<dyn Reel>,
}

/// Why a host did not open.
#[derive(Debug)]
pub enum Opening {
    /// No runtime for the network port and the timers.
    Runtime(std::io::Error),
    /// This machine's TLS could not be made ready, so no request could ever go out.
    Tls(reqwest::Error),
    /// The core refused the config this host wrote. A bug here, not a fault of the machine.
    Config,
}

impl std::fmt::Display for Opening {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(out, "no runtime: {error}"),
            Self::Tls(error) => write!(out, "no tls: {error}"),
            Self::Config => write!(out, "the core refused this host's config"),
        }
    }
}

impl std::error::Error for Opening {}

/// The answer to a request a surface prepared itself, by the id it gave the event. Status 0 means
/// no answer came at all. The core keeps the tokens and reads only an error body, so a surface that
/// asks for something of its own reads it here.
#[derive(Clone, Debug)]
pub struct Reply {
    pub request: String,
    pub status: u32,
    pub body: String,
}

/// What one turn of the core left for a surface to draw. Every part is a contract type or an
/// answer named by the request that asked for it, so nothing here is invented: a surface merges
/// the views in order and matches a page or a reply by the id it used.
#[derive(Debug, Default)]
pub struct Turn {
    /// Changed screen state, oldest first. A part that is absent did not change.
    pub views: Vec<View>,
    /// Page bodies the core or the layer marked for a screen, each named by its request.
    pub pages: Vec<ScreenBody>,
    /// Answers to requests a surface prepared with `Action::Request`.
    pub replies: Vec<Reply>,
    /// One time notices.
    pub notices: Vec<Notice>,
    /// Addresses the core opened by itself.
    pub opens: Vec<RouteTarget>,
    /// Monotonic time at which the host steps the core again. Absent means no wake.
    pub next_wake_ms: Option<u64>,
}

/// What an adapter hands back from another thread.
enum Back {
    Event(Event),
    Page(ScreenBody),
    Reply(Reply),
}

pub struct Host {
    machine: Machine,
    server: String,
    network: Box<dyn Network>,
    secrets: Box<dyn Secrets>,
    store: Box<dyn Store>,
    player: Box<dyn Player>,
    reel: Box<dyn Reel>,
    runtime: Runtime,
    timers: HashMap<String, JoinHandle<()>>,
    started: Instant,
    back: Sender<Back>,
    inbox: Receiver<Back>,
}

impl Host {
    /// Open the core with what this desktop has.
    ///
    /// # Errors
    /// No runtime, no TLS, or a config the core refused.
    pub fn open(settings: &Settings) -> Result<Self, Opening> {
        let runtime = runtime()?;
        let network = HttpNetwork::new(runtime.handle().clone()).map_err(Opening::Tls)?;
        let ports = Ports {
            network: Box::new(network),
            secrets: Box::new(FileSecrets::new(settings.data_dir.join(SECRETS))),
            store: Box::new(FileStore::new(settings.data_dir.join(STORE))),
            player: Box::new(NoPlayer),
            reel: Box::new(NoReel),
        };
        Self::new(settings, runtime, ports)
    }

    /// Open the core with ports a caller filled.
    ///
    /// # Errors
    /// No runtime, or a config the core refused.
    pub fn with_ports(settings: &Settings, ports: Ports) -> Result<Self, Opening> {
        Self::new(settings, runtime()?, ports)
    }

    fn new(settings: &Settings, runtime: Runtime, ports: Ports) -> Result<Self, Opening> {
        let machine = Machine::open_json(&open_text(settings)?).map_err(|_| Opening::Config)?;
        let (back, inbox) = channel();
        let Ports {
            network,
            secrets,
            store,
            player,
            reel,
        } = ports;
        Ok(Self {
            machine,
            server: settings.server.trim_end_matches('/').to_owned(),
            network,
            secrets,
            store,
            player,
            reel,
            runtime,
            timers: HashMap::new(),
            started: Instant::now(),
            back,
            inbox,
        })
    }

    /// True while a brand film could be shown. Nothing here can, and a surface asks before it
    /// holds room for one.
    #[must_use]
    pub fn reel_plays(&self) -> bool {
        self.reel.plays()
    }

    /// Hand the core these events and run it until nothing more can be answered at once. An empty
    /// list is how a run begins and how a wake is served.
    pub fn step(&mut self, events: Vec<Event>) -> Turn {
        let mut turn = Turn::default();
        self.pump(events, &mut turn);
        turn
    }

    /// Take whatever the adapters have handed back and step on it. Nothing waits.
    pub fn drain(&mut self) -> Turn {
        let back: Vec<Back> = self.inbox.try_iter().collect();
        self.run(back)
    }

    /// Wait up to `within` for the first thing an adapter hands back, then take the rest of what
    /// is there and step on all of it. A wake that nothing answers still steps the core, which is
    /// what `Turn::next_wake_ms` asked for.
    pub fn wait(&mut self, within: Duration) -> Turn {
        let mut back = Vec::new();
        if let Ok(first) = self.inbox.recv_timeout(within) {
            back.push(first);
            back.extend(self.inbox.try_iter());
        }
        self.run(back)
    }

    fn run(&mut self, back: Vec<Back>) -> Turn {
        let mut turn = Turn::default();
        let mut events = Vec::new();
        for message in back {
            match message {
                Back::Event(event) => events.push(event),
                Back::Page(page) => turn.pages.push(page),
                Back::Reply(reply) => turn.replies.push(reply),
            }
        }
        self.pump(events, &mut turn);
        turn
    }

    fn pump(&mut self, events: Vec<Event>, turn: &mut Turn) {
        let mut events = events;
        for _ in 0..STEPS {
            let input = self.input(events);
            let out = self.machine.step(input);
            events = self.apply(out, turn);
            if events.is_empty() {
                return;
            }
        }
        eprintln!("warn host.turn.long the core answered itself {STEPS} times in one turn");
    }

    fn input(&self, events: Vec<Event>) -> CoreIn {
        CoreIn {
            contract: contract::version::WireDate::default(),
            tag: contract::core_msg::CoreInTag::default(),
            now_ms: u64::try_from(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
            )
            .unwrap_or(u64::MAX),
            mono_ms: u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX),
            events,
        }
    }

    fn apply(&mut self, out: CoreOut, turn: &mut Turn) -> Vec<Event> {
        let CoreOut {
            effects,
            view,
            next_wake_ms,
            ..
        } = out;
        if let Some(view) = view {
            turn.views.push(view);
        }
        turn.next_wake_ms = next_wake_ms;
        let mut results = Vec::new();
        for effect in effects {
            self.perform(effect, turn, &mut results);
        }
        results
    }

    fn perform(&mut self, effect: Effect, turn: &mut Turn, results: &mut Vec<Event>) {
        let Effect { id, body } = effect;
        match body {
            EffectBody::Http {
                method,
                path,
                headers,
                body,
                deadline_ms,
                to,
                ..
            } => {
                let carried_token = headers.contains_key("Authorization");
                let call = Call {
                    method,
                    url: format!("{}{path}", self.server),
                    headers,
                    body,
                    deadline_ms,
                };
                let back = self.back.clone();
                self.network.send(
                    call,
                    Box::new(move |answer| {
                        for message in messages(id, to, carried_token, answer) {
                            let _ = back.send(message);
                        }
                    }),
                );
            }
            EffectBody::Secret { op, name, value } => {
                results.push(answered(id, self.secret(op, &name, value)));
            }
            EffectBody::Store { op, key, value } => {
                results.push(answered(id, self.stored(op, key, value)));
            }
            EffectBody::Timer { op, after_ms } => self.timer(id, op, after_ms),
            // A command nothing can perform is said to be unperformed, so the core moves on at its
            // start deadline instead of waiting for a driver that will never report.
            EffectBody::Player { command } => {
                if !self.player.command(command) {
                    results.push(answered(id, EventBody::Unsupported));
                }
            }
            EffectBody::View { notice, body } => {
                if let Some(notice) = notice {
                    turn.notices.push(notice);
                }
                if let Some(body) = body {
                    turn.pages.push(body);
                }
            }
            EffectBody::Open { target } => turn.opens.push(target),
            EffectBody::Log {
                level,
                code,
                detail,
            } => eprintln!("{level:?} {code} {}", detail.unwrap_or_default()),
            _ => results.push(answered(id, EventBody::Unsupported)),
        }
    }

    fn secret(&self, op: SecretOp, name: &str, value: Option<String>) -> EventBody {
        match op {
            SecretOp::Get => EventBody::SecretDone {
                value: self.secrets.get(name),
            },
            SecretOp::Put => {
                if self.secrets.put(name, &value.unwrap_or_default()) {
                    EventBody::SecretDone { value: None }
                } else {
                    EventBody::Unsupported
                }
            }
            _ => {
                self.secrets.delete(name);
                EventBody::SecretDone { value: None }
            }
        }
    }

    fn stored(&self, op: StoreOp, key: Option<String>, value: Option<String>) -> EventBody {
        let key = key.unwrap_or_default();
        match op {
            StoreOp::Get => EventBody::StoreDone {
                value: self.store.get(&key),
                keys: Vec::new(),
            },
            StoreOp::Put => {
                if self.store.put(&key, &value.unwrap_or_default()) {
                    EventBody::StoreDone {
                        value: None,
                        keys: Vec::new(),
                    }
                } else {
                    EventBody::Unsupported
                }
            }
            StoreOp::List => EventBody::StoreDone {
                value: None,
                keys: self.store.keys(&key),
            },
            _ => {
                self.store.delete(&key);
                EventBody::StoreDone {
                    value: None,
                    keys: Vec::new(),
                }
            }
        }
    }

    fn timer(&mut self, id: String, op: TimerOp, after_ms: Option<u64>) {
        // A timer that has fired holds nothing worth keeping, and the ids never repeat, so the
        // finished ones go here rather than growing for as long as the app runs.
        self.timers.retain(|_, running| !running.is_finished());
        if let Some(old) = self.timers.remove(&id) {
            old.abort();
        }
        if op == TimerOp::Clear {
            return;
        }
        let back = self.back.clone();
        let key = id.clone();
        let after = Duration::from_millis(after_ms.unwrap_or(0));
        let running = self.runtime.spawn(async move {
            tokio::time::sleep(after).await;
            let _ = back.send(Back::Event(answered(key, EventBody::TimerFired)));
        });
        self.timers.insert(id, running);
    }
}

fn runtime() -> Result<Runtime, Opening> {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(Opening::Runtime)
}

fn answered(id: String, body: EventBody) -> Event {
    Event { id: Some(id), body }
}

/// What an answer means to the core, to a surface that prepared the request, and to a screen when
/// the body was going to one.
fn messages(id: String, to: BodyTarget, carried_token: bool, answer: Answer) -> Vec<Back> {
    match answer {
        Answer::NoReply { reason } => vec![
            Back::Reply(Reply {
                request: id.clone(),
                status: 0,
                body: String::new(),
            }),
            Back::Event(answered(id, EventBody::HttpFailed { reason })),
        ],
        Answer::Reply { status, etag, text } => {
            let good = (200..300).contains(&status);
            let to_screen = to == BodyTarget::Screen && good;
            let mut headers = BTreeMap::new();
            if let Some(etag) = etag {
                headers.insert(ETAG.to_owned(), etag);
            }
            let mut out = Vec::new();
            if to_screen {
                out.push(Back::Page(ScreenBody {
                    request: id.clone(),
                    body: text.clone(),
                    stale: false,
                }));
            }
            // The core answers this 401 by refreshing and sending the same id again, so whoever
            // prepared the request waits for that answer. Handing it on now would show a signed in
            // viewer a refusal, and asking again from a surface would be a second request.
            if status != 401 || !carried_token {
                out.push(Back::Reply(Reply {
                    request: id.clone(),
                    status,
                    body: text.clone(),
                }));
            }
            out.push(Back::Event(answered(
                id,
                EventBody::HttpDone {
                    status,
                    headers,
                    body: (!to_screen && !text.is_empty()).then_some(text),
                },
            )));
            out
        }
    }
}

/// Hex digits from the machine's own randomness. The seed is the only random input the core ever
/// gets, and the boot id has to change whenever monotonic time restarts, which on a desktop is
/// once per run of the process.
fn hex(bytes: usize) -> String {
    let mut source = rand::rng();
    let mut digits = String::with_capacity(bytes * 2);
    for byte in (0..bytes).map(|_| source.random::<u8>()) {
        digits.push(char::from_digit(u32::from(byte >> 4), 16).unwrap_or('0'));
        digits.push(char::from_digit(u32::from(byte & 15), 16).unwrap_or('0'));
    }
    digits
}

/// The text `core_open` reads: the core's config, and a `layer` key beside it that turns the
/// offline layer on. The core never reads that key, so it never learns the layer is there.
///
/// No capability card is sent. A card has to state the screen in pixels and what the player opens,
/// and this host has neither a window nor a player; a card full of zeros would make the server
/// filter on figures nobody measured. Every other family sends none either.
fn open_text(settings: &Settings) -> Result<String, Opening> {
    let config = CoreConfig {
        contract: contract::version::WireDate::default(),
        tag: CoreConfigTag::default(),
        class: DeviceClass::Desktop,
        form: Form::Desktop,
        mode: CoreMode::Local,
        app: APP.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        os: std::env::consts::OS.to_owned(),
        seed: hex(16),
        card: None,
    };
    let layer = LayerConfig {
        boot_id: hex(8),
        storage_bytes: settings.storage_bytes,
    };
    let mut open = serde_json::to_value(&config).map_err(|_| Opening::Config)?;
    let keys = open.as_object_mut().ok_or(Opening::Config)?;
    keys.insert(
        "layer".to_owned(),
        serde_json::to_value(&layer).map_err(|_| Opening::Config)?,
    );
    Ok(open.to_string())
}

#[cfg(test)]
mod tests {
    use contract::core_msg::{BodyTarget, EventBody, NoAnswerReason};

    use super::{Answer, Back, Reply, Settings, messages, open_text};

    /// What one answer turned into: the page, the reply, and the event the core was given.
    fn parts(
        to: BodyTarget,
        carried_token: bool,
        answer: Answer,
    ) -> (Option<String>, Option<Reply>, Option<EventBody>) {
        let mut page = None;
        let mut reply = None;
        let mut event = None;
        for message in messages(String::from("ask-1"), to, carried_token, answer) {
            match message {
                Back::Page(body) => {
                    assert_eq!(body.request, "ask-1");
                    assert!(!body.stale);
                    page = Some(body.body);
                }
                Back::Reply(one) => {
                    assert_eq!(one.request, "ask-1");
                    reply = Some(one);
                }
                Back::Event(one) => {
                    assert_eq!(one.id.as_deref(), Some("ask-1"));
                    event = Some(one.body);
                }
            }
        }
        (page, reply, event)
    }

    fn reply(status: u32, text: &str) -> Answer {
        Answer::Reply {
            status,
            etag: Some(String::from("\"w-7\"")),
            text: String::from(text),
        }
    }

    #[test]
    fn a_body_for_the_core_reaches_the_core_and_whoever_asked() {
        let (page, reply, event) = parts(BodyTarget::Core, false, reply(200, "{\"ok\":1}"));
        assert_eq!(page, None);
        assert_eq!(
            reply.map(|one| (one.status, one.body)),
            Some((200, String::from("{\"ok\":1}")))
        );
        match event {
            Some(EventBody::HttpDone {
                status,
                headers,
                body,
            }) => {
                assert_eq!(status, 200);
                assert_eq!(headers.get("etag").map(String::as_str), Some("\"w-7\""));
                assert_eq!(body.as_deref(), Some("{\"ok\":1}"));
            }
            other => panic!("the core was given {other:?}"),
        }
    }

    #[test]
    fn a_body_for_a_screen_goes_to_the_screen_and_not_into_the_core() {
        let (page, reply, event) =
            parts(BodyTarget::Screen, false, reply(200, "{\"sections\":[]}"));
        assert_eq!(page.as_deref(), Some("{\"sections\":[]}"));
        assert!(reply.is_some());
        match event {
            Some(EventBody::HttpDone { body, .. }) => assert_eq!(body, None),
            other => panic!("the core was given {other:?}"),
        }
    }

    #[test]
    fn a_refusal_marked_for_a_screen_still_reaches_the_core_whole() {
        // Every status outside 200 to 299 is the core's: it reads the error code.
        let (page, _, event) = parts(BodyTarget::Screen, false, reply(409, "{\"code\":\"busy\"}"));
        assert_eq!(page, None);
        match event {
            Some(EventBody::HttpDone { status, body, .. }) => {
                assert_eq!(status, 409);
                assert_eq!(body.as_deref(), Some("{\"code\":\"busy\"}"));
            }
            other => panic!("the core was given {other:?}"),
        }
    }

    #[test]
    fn a_401_on_a_request_that_carried_a_token_waits_for_the_core_to_refresh() {
        let (_, held, event) = parts(BodyTarget::Core, true, reply(401, "{}"));
        assert!(
            held.is_none(),
            "a signed in viewer would have been shown a refusal"
        );
        assert!(matches!(
            event,
            Some(EventBody::HttpDone { status: 401, .. })
        ));
    }

    #[test]
    fn a_401_on_a_request_that_carried_no_token_is_the_answer_to_whoever_asked() {
        let (_, held, _) = parts(BodyTarget::Core, false, reply(401, "{}"));
        assert_eq!(held.map(|one| one.status), Some(401));
    }

    #[test]
    fn no_answer_at_all_is_told_to_whoever_asked_as_status_0() {
        let (page, held, event) = parts(
            BodyTarget::Screen,
            true,
            Answer::NoReply {
                reason: NoAnswerReason::Timeout,
            },
        );
        assert_eq!(page, None);
        assert_eq!(
            held.map(|one| (one.status, one.body)),
            Some((0, String::new()))
        );
        assert!(matches!(
            event,
            Some(EventBody::HttpFailed {
                reason: NoAnswerReason::Timeout
            })
        ));
    }

    fn settings() -> Settings {
        Settings {
            server: String::from("http://127.0.0.1:8080"),
            data_dir: std::path::PathBuf::from("."),
            storage_bytes: 1_000_000,
        }
    }

    #[test]
    fn the_config_says_desktop_class_and_desktop_form() {
        let text = open_text(&settings()).expect("the config is well formed");
        let read: serde_json::Value = serde_json::from_str(&text).expect("it reads back");
        assert_eq!(read["class"], "desktop");
        assert_eq!(read["form"], "desktop");
        assert_eq!(read["mode"], "local");
        assert_eq!(read["app"], "desktop");
        assert_eq!(read["card"], serde_json::Value::Null);
        assert_eq!(read["layer"]["storage_bytes"], 1_000_000);
        assert_eq!(read["seed"].as_str().map(str::len), Some(32));
    }

    #[test]
    fn every_run_gets_its_own_seed_and_boot_id() {
        let one: serde_json::Value =
            serde_json::from_str(&open_text(&settings()).expect("one")).expect("reads");
        let two: serde_json::Value =
            serde_json::from_str(&open_text(&settings()).expect("two")).expect("reads");
        assert_ne!(one["seed"], two["seed"]);
        assert_ne!(one["layer"]["boot_id"], two["layer"]["boot_id"]);
    }
}
