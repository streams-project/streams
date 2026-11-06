//! The core, the server and the finder, each the real thing, as one system. The core goes through
//! the same boundary a device host calls, and every request it asks for goes over a real socket.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use contract::core_msg::{CoreOut, Effect, EffectBody};
use contract::version::WIRE_VERSION;
use kit_core::boundary::Boundary;
use serde_json::{Value, json};

const TOKEN: &str = "a-service-token-for-this-run-only";

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a free port");
    listener.local_addr().expect("address").port()
}

fn built(name: &str) -> PathBuf {
    let status = Command::new(env!("CARGO"))
        .args([
            "build",
            "--quiet",
            "-p",
            "server",
            "-p",
            "finder-open-films",
        ])
        .status()
        .expect("cargo runs");
    assert!(status.success(), "the binaries build");
    let test = std::env::current_exe().expect("this test");
    test.parent()
        .and_then(Path::parent)
        .expect("the profile folder")
        .join(name)
}

fn start(binary: &Path, pairs: &[(&str, String)]) -> Running {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (name, value) in pairs {
        command.env(name, value);
    }
    Running(command.spawn().expect("the process starts"))
}

async fn wait_for(url: &str) {
    let began = Instant::now();
    while reqwest::get(url).await.is_err() {
        assert!(
            began.elapsed() < Duration::from_secs(30),
            "{url} did not answer"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// A host in the plainest form: it performs every effect for real and writes down what it saw.
struct Host {
    machine: Boundary,
    base: String,
    client: reqwest::Client,
    secrets: BTreeMap<String, String>,
    began: Instant,
    pages: Vec<Value>,
    routes: Vec<Value>,
    player: Vec<Value>,
    sent: Vec<String>,
    logs: Vec<String>,
    answers: Vec<String>,
    bodies: Vec<(String, String)>,
    view: Value,
}

impl Host {
    async fn step(&mut self, mut events: Vec<Value>) {
        for _ in 0..64 {
            let mono = u64::try_from(self.began.elapsed().as_millis()).expect("millis");
            let input = json!({
                "contract": WIRE_VERSION,
                "type": "core_in",
                "now_ms": 1_793_836_800_000_u64 + mono,
                "mono_ms": mono,
                "events": events,
            });
            let output = self.machine.step(&input.to_string()).expect("a step");
            let out: CoreOut = serde_json::from_str(&output).expect("the core's message");
            assert!(out.unsupported.is_empty(), "{:?}", out.unsupported);
            let view = out.view.as_ref().map(serde_json::to_value);
            if let Some(Ok(Value::Object(parts))) = view {
                for (part, content) in parts {
                    self.view[part] = content;
                }
            }
            events = Vec::new();
            for effect in out.effects {
                self.perform(effect, &mut events).await;
            }
            if events.is_empty() {
                return;
            }
        }
        panic!("the core never settled");
    }

    async fn perform(&mut self, effect: Effect, results: &mut Vec<Value>) {
        let Effect { id, body } = effect;
        match body {
            EffectBody::Http {
                method,
                path,
                headers,
                body,
                to,
                ..
            } => {
                let word = serde_json::to_value(method).expect("method");
                let word = word.as_str().expect("a word").to_uppercase();
                self.sent.push(format!("{word} {path}"));
                let mut request = self.client.request(
                    reqwest::Method::from_bytes(word.as_bytes()).expect("a method"),
                    format!("{}{path}", self.base),
                );
                for (name, value) in &headers {
                    request = request.header(name, value);
                }
                if let Some(body) = body {
                    request = request
                        .header("content-type", "application/json")
                        .body(body);
                }
                let answer = request.send().await;
                let answer = match answer {
                    Ok(answer) => answer,
                    Err(_) => {
                        results.push(json!({"id": id, "type": "http_failed", "reason": "offline"}));
                        return;
                    }
                };
                let status = answer.status().as_u16();
                let etag = answer
                    .headers()
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_owned);
                let text = answer.text().await.expect("a body");
                self.bodies.push((path.clone(), text.clone()));
                self.answers.push(format!(
                    "{status} {path} {}",
                    text.chars().take(300).collect::<String>()
                ));
                let mut event = json!({"id": id, "type": "http_done", "status": status});
                if let Some(etag) = etag {
                    event["headers"] = json!({"etag": etag});
                }
                let to_screen = serde_json::to_value(to).expect("target") == json!("screen");
                if to_screen && (200..300).contains(&status) {
                    self.pages
                        .push(serde_json::from_str(&text).expect("a page"));
                } else if !text.is_empty() {
                    event["body"] = json!(text);
                }
                results.push(event);
            }
            EffectBody::Secret { op, name, value } => {
                let word = serde_json::to_value(op).expect("op");
                let mut event = json!({"id": id, "type": "secret_done"});
                match word.as_str() {
                    Some("get") => {
                        if let Some(held) = self.secrets.get(&name) {
                            event["value"] = json!(held);
                        }
                    }
                    Some("put") => {
                        self.secrets.insert(name, value.unwrap_or_default());
                    }
                    _ => {
                        self.secrets.remove(&name);
                    }
                }
                results.push(event);
            }
            EffectBody::Store { .. } | EffectBody::Timer { .. } => {
                results.push(json!({"id": id, "type": "unsupported"}));
            }
            EffectBody::Player { command } => {
                self.player
                    .push(serde_json::to_value(command).expect("command"));
            }
            EffectBody::Open { target } => {
                self.routes
                    .push(serde_json::to_value(target).expect("target"));
            }
            EffectBody::View { body, .. } => {
                if let Some(body) = body {
                    self.pages
                        .push(serde_json::from_str(&body.body).expect("a kept page"));
                }
            }
            EffectBody::Log { code, .. } => self.logs.push(code),
            other => panic!("an effect this host does not know: {other:?}"),
        }
    }
}

/// The first title key anywhere in a page.
fn first_title(page: &Value) -> Option<Value> {
    match page {
        Value::String(text) if text.starts_with("movie:") || text.starts_with("tv:") => {
            Some(page.clone())
        }
        Value::Object(fields) => fields.values().find_map(first_title),
        Value::Array(items) => items.iter().find_map(first_title),
        _ => None,
    }
}

#[tokio::test]
async fn a_visitor_opens_the_app_sees_home_and_a_title_starts_playing() {
    let server_binary = built("server");
    let finder_binary = built("finder-open-films");
    let dir = std::env::temp_dir().join(format!("wired_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let manifest = server_catalog::sample::write_sample(&dir);
    let keys = server_catalog::sample::write_sample_key(&dir);

    // Every sample title gets an open film, so whichever title home leads with can play.
    let film = "https://films.example/open-film.m3u8";
    let list: Vec<Value> = (1..1000)
        .flat_map(|id| {
            ["movie", "tv"].map(|kind| json!({
            "title": format!("{kind}:{id}"), "url": film, "kind": "hls", "licence": "CC0-1.0",
        }))
        })
        .collect();
    let films = dir.join("films.json");
    std::fs::write(&films, Value::from(list).to_string()).expect("the film list");

    // A real database, migrated from nothing by the server's own command.
    let cache = Path::new(env!("CARGO_TARGET_TMPDIR")).join("database");
    let bin = launcher::postgres::install(&cache)
        .await
        .expect("the database installs");
    let database = launcher::postgres::Postgres::start(&bin, &dir.join("db/data"), free_port())
        .expect("the database starts");
    let migrated = Command::new(&server_binary)
        .arg("migrate")
        .env_clear()
        .env("DATABASE_URL", &database.url)
        .output()
        .expect("the server runs");
    assert!(
        migrated.status.success(),
        "{}",
        String::from_utf8_lossy(&migrated.stderr)
    );

    let finder_port = free_port();
    let server_port = free_port();
    let _finder = start(
        &finder_binary,
        &[
            ("FINDER_BIND", format!("127.0.0.1:{finder_port}")),
            ("FINDER_SERVICE_TOKEN", String::from(TOKEN)),
            ("FINDER_FILMS", films.to_string_lossy().into_owned()),
        ],
    );
    let base = format!("http://127.0.0.1:{server_port}");
    let _server = start(
        &server_binary,
        &[
            ("SERVER_LISTEN", format!("127.0.0.1:{server_port}")),
            ("DATABASE_URL", database.url.clone()),
            ("DATA_MANIFEST", manifest.to_string_lossy().into_owned()),
            ("DATA_PUBLIC_KEYS", keys.to_string_lossy().into_owned()),
            ("ACCOUNTS_REQUIRED", String::from("false")),
            ("VISITOR_TIER", String::from("15")),
            ("FINDER_URLS", format!("http://127.0.0.1:{finder_port}")),
            ("FINDER_TOKEN", String::from(TOKEN)),
            ("BRAND_NAME", String::from("Wired")),
        ],
    );
    wait_for(&format!("{base}/v1/ready")).await;

    let mut host = Host {
        machine: Boundary::open(&json!({
            "app": "wired", "class": "web", "contract": WIRE_VERSION, "form": "desktop", "mode": "local",
            "os": "test", "seed": "00112233445566778899aabbccddeeff", "type": "core_config", "version": "1.0.0",
        }).to_string()).expect("the core opens"),
        base,
        client: reqwest::Client::new(),
        secrets: BTreeMap::new(),
        began: Instant::now(),
        pages: Vec::new(),
        routes: Vec::new(),
        player: Vec::new(),
        sent: Vec::new(),
        logs: Vec::new(),
        answers: Vec::new(),
        bodies: Vec::new(),
        view: json!({}),
    };

    // The app opens, then the device asks for the address it was launched at.
    host.step(Vec::new()).await;
    host.step(vec![json!({"type": "address", "address": "/home"})])
        .await;
    assert!(
        host.sent.iter().any(|line| line.ends_with("/v1/hello")),
        "{:?}",
        host.sent
    );
    assert!(
        !host.pages.is_empty(),
        "no page came back. sent {:?} logs {:?} routes {:?} view {}",
        host.sent,
        host.logs,
        host.routes,
        host.view
    );
    let title = first_title(host.pages.last().expect("home")).expect("a title on home");

    // The visitor joins, signs in through the core, and enters the profile the account came with.
    let password = "a-password-for-this-run";
    let email = "viewer@example.test";
    let join =
        json!({"email": email, "password": password, "first_name": "Ada", "last_name": "Viewer"});
    host.step(vec![json!({"type": "action", "id": "join-1", "action": {
        "action": "request", "method": "POST", "path": "/v1/auth/signup", "body": join.to_string(),
    }})])
    .await;
    assert!(
        host.answers
            .iter()
            .any(|line| line.starts_with("200 /v1/auth/signup")),
        "{:?}",
        host.answers
    );
    host.step(vec![json!({"type": "action", "action": {
        "action": "sign_in", "email": email, "password": password,
    }})])
    .await;
    assert!(
        host.answers
            .iter()
            .any(|line| line.starts_with("200 /v1/auth/password")),
        "{:?}",
        host.answers
    );
    host.step(vec![json!({"type": "action", "id": "list-1", "action": {
        "action": "request", "method": "GET", "path": "/v1/profiles",
    }})])
    .await;
    let profiles = host
        .bodies
        .iter()
        .rev()
        .find(|(path, _)| path == "/v1/profiles")
        .map(|(_, body)| body.clone())
        .expect("the profile list");
    let profiles: Value = serde_json::from_str(&profiles).expect("profiles");
    let profile = profiles["profiles"][0]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("no profile in {profiles}"))
        .to_owned();
    host.step(vec![json!({"type": "action", "action": {
        "action": "enter_profile", "profile": profile,
    }})])
    .await;

    // The viewer presses play, and the finder's film reaches the player.
    host.step(vec![
        json!({"type": "action", "action": {"action": "play", "title": title}}),
    ])
    .await;
    assert!(
        host.sent.iter().any(|line| line.contains("/v1/play")),
        "play was not asked for. sent {:?} logs {:?} view {}",
        host.sent,
        host.logs,
        host.view
    );
    let opened = host.player.iter().find(|command| command["op"] == "open");
    let opened = opened.unwrap_or_else(|| {
        panic!(
            "nothing reached the player. answers {:?} player {:?} logs {:?} view {}",
            host.answers, host.player, host.logs, host.view
        )
    });
    assert!(opened.to_string().contains(film), "{opened}");

    // One film in progress is enough for Continue Watching, and it comes back on the next read.
    host.step(vec![
        json!({"type": "action", "action": {"action": "command", "command": {
            "command": "position", "title": title, "position_s": 61, "duration_s": 634,
        }}}),
    ])
    .await;
    host.step(vec![json!({"type": "address", "address": "/home"})])
        .await;
    let sections = host.pages.last().expect("home again")["sections"].clone();
    let sections = sections.as_array().expect("sections");
    let resumed = sections
        .iter()
        .find(|section| section["kind"] == "continue")
        .unwrap_or_else(|| panic!("no continue row in {sections:?}"));
    assert_eq!(resumed["tiles"][0]["title"], title, "{resumed}");

    let _ = std::fs::remove_dir_all(&dir);
}
