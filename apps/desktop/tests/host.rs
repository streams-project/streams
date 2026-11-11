//! The host, with the ports filled by this test. What is proved here is the wiring: which port is
//! asked in which order, and what one turn hands a surface. The core's own rules are proved
//! against the core in `platform/kit/core`, not again here.

use std::sync::{Arc, Mutex};

use contract::core_msg::{Action, Event, EventBody, HttpMethod, Target};
use desktop::{
    Answer, Call, Done, Host, Network, NoPlayer, NoReel, Ports, Secrets, Settings, Store,
};

const SERVER: &str = "http://127.0.0.1:1";

/// Every port call, in the order they were made.
#[derive(Clone, Default)]
struct Seen(Arc<Mutex<Vec<String>>>);

impl Seen {
    fn add(&self, line: String) {
        self.0.lock().expect("the list").push(line);
    }

    fn all(&self) -> Vec<String> {
        self.0.lock().expect("the list").clone()
    }
}

/// A network that answers from a short script and records every path. A path with no line in the
/// script is answered 503, which is an answer the core knows how to carry.
struct Scripted {
    seen: Seen,
    lines: Vec<(&'static str, u32, &'static str)>,
}

impl Network for Scripted {
    fn send(&self, call: Call, done: Done) {
        let path = call
            .url
            .strip_prefix(SERVER)
            .unwrap_or(&call.url)
            .to_owned();
        self.seen.add(format!("net {path}"));
        let found = self.lines.iter().find(|(at, _, _)| *at == path);
        let (status, text) = found.map_or((503, "{}"), |(_, status, text)| (*status, *text));
        done(Answer::Reply {
            status,
            etag: None,
            text: String::from(text),
        });
    }

    fn watch(&self, _changed: Box<dyn Fn(bool) + Send + Sync + 'static>) {}
}

/// A network that never answers, which is a dead server rather than a refusing one.
struct Silent;

impl Network for Silent {
    fn send(&self, _call: Call, done: Done) {
        done(Answer::NoReply {
            reason: contract::core_msg::NoAnswerReason::Offline,
        });
    }

    fn watch(&self, _changed: Box<dyn Fn(bool) + Send + Sync + 'static>) {}
}

struct Keys {
    seen: Seen,
    writes: bool,
}

impl Secrets for Keys {
    fn get(&self, name: &str) -> Option<String> {
        self.seen.add(format!("secret get {name}"));
        None
    }

    fn put(&self, name: &str, _value: &str) -> bool {
        self.seen.add(format!("secret put {name}"));
        self.writes
    }

    fn delete(&self, name: &str) {
        self.seen.add(format!("secret delete {name}"));
    }
}

struct Files {
    seen: Seen,
}

impl Store for Files {
    fn get(&self, key: &str) -> Option<String> {
        self.seen.add(format!("store get {key}"));
        None
    }

    fn put(&self, key: &str, _value: &str) -> bool {
        self.seen.add(format!("store put {key}"));
        true
    }

    fn delete(&self, key: &str) {
        self.seen.add(format!("store delete {key}"));
    }

    fn keys(&self, prefix: &str) -> Vec<String> {
        self.seen.add(format!("store list {prefix}"));
        Vec::new()
    }
}

fn settings() -> Settings {
    Settings {
        server: String::from(SERVER),
        data_dir: std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")),
        storage_bytes: 1_000_000,
    }
}

fn host(network: Box<dyn Network>, seen: &Seen, writes: bool) -> Host {
    let ports = Ports {
        network,
        secrets: Box::new(Keys {
            seen: seen.clone(),
            writes,
        }),
        store: Box::new(Files { seen: seen.clone() }),
        player: Box::new(NoPlayer),
        reel: Box::new(NoReel),
    };
    Host::with_ports(&settings(), ports).expect("a host opens")
}

fn scripted(lines: Vec<(&'static str, u32, &'static str)>, seen: &Seen) -> Box<dyn Network> {
    Box::new(Scripted {
        seen: seen.clone(),
        lines,
    })
}

#[test]
fn a_run_begins_with_the_layers_own_files_then_what_was_kept_then_hello() {
    let seen = Seen::default();
    let mut host = host(scripted(Vec::new(), &seen), &seen, true);
    host.step(Vec::new());
    assert_eq!(
        seen.all()[..5],
        [
            // The layer loads before the core runs at all: its waiting commands, then its index.
            "store list q/",
            "store get index",
            // Then the core reads what it kept, and only then does it reach a server.
            "secret get device",
            "secret get session",
            "net /v1/hello",
        ]
    );
}

#[test]
fn a_turn_hands_a_surface_the_views_and_the_targets_the_core_opened() {
    let seen = Seen::default();
    let mut host = host(scripted(Vec::new(), &seen), &seen, true);
    let turn = host.step(Vec::new());
    // An unreachable server counts as accounts required, so a visitor is sent to the start screen.
    assert_eq!(
        turn.opens
            .iter()
            .map(|open| open.target)
            .collect::<Vec<_>>(),
        [Target::Wait, Target::Start]
    );
    assert!(
        turn.views.iter().any(|view| view
            .route
            .as_ref()
            .is_some_and(|route| route.target == Target::Start)),
        "no view carried the start route: {:?}",
        turn.views
    );
    assert!(turn.pages.is_empty());
    assert!(turn.notices.is_empty());
}

#[test]
fn a_request_a_surface_prepared_comes_back_under_the_id_it_gave() {
    let seen = Seen::default();
    let mut host = host(
        scripted(vec![("/v1/screens/home", 200, r#"{"sections":[]}"#)], &seen),
        &seen,
        true,
    );
    host.step(Vec::new());
    host.drain();
    host.step(vec![Event {
        id: Some(String::from("ask-1")),
        body: EventBody::Action {
            action: Action::Request {
                method: HttpMethod::Get,
                path: String::from("/v1/screens/home"),
                body: None,
            },
        },
    }]);
    let turn = host.drain();
    let mine: Vec<_> = turn
        .replies
        .iter()
        .filter(|reply| reply.request == "ask-1")
        .collect();
    assert_eq!(mine.len(), 1, "replies were {:?}", turn.replies);
    assert_eq!(mine[0].status, 200);
    assert_eq!(mine[0].body, r#"{"sections":[]}"#);
    assert!(seen.all().contains(&String::from("net /v1/screens/home")));
}

#[test]
fn a_request_that_got_no_answer_is_told_to_whoever_asked_as_status_0() {
    let seen = Seen::default();
    let mut host = host(Box::new(Silent), &seen, true);
    host.step(Vec::new());
    host.drain();
    host.step(vec![Event {
        id: Some(String::from("ask-7")),
        body: EventBody::Action {
            action: Action::Request {
                method: HttpMethod::Get,
                path: String::from("/v1/screens/home"),
                body: None,
            },
        },
    }]);
    let turn = host.drain();
    let mine: Vec<_> = turn
        .replies
        .iter()
        .filter(|reply| reply.request == "ask-7")
        .collect();
    assert_eq!(mine.len(), 1, "replies were {:?}", turn.replies);
    assert_eq!(mine[0].status, 0);
    assert!(mine[0].body.is_empty());
}

/// A sign in answer the core accepts, so the next thing it does is keep the session.
const SIGNED_IN: &str = r#"{"user_id":"u1","email":"someone@example.test","access_token":"aaaaaaaaaaaaaaaaaaaaaaaa","refresh_token":"rrrrrrrrrrrrrrrrrrrrrrrr","expires_in":900}"#;

fn signs_in(host: &mut Host) {
    host.step(Vec::new());
    host.drain();
    host.step(vec![Event {
        id: None,
        body: EventBody::Action {
            action: Action::SignIn {
                email: String::from("someone@example.test"),
                password: String::from("a password"),
            },
        },
    }]);
    host.drain();
}

#[test]
fn a_secret_store_that_takes_writes_is_handed_the_session_to_keep() {
    let seen = Seen::default();
    let mut host = host(
        scripted(vec![("/v1/auth/password", 200, SIGNED_IN)], &seen),
        &seen,
        true,
    );
    signs_in(&mut host);
    assert!(
        seen.all().contains(&String::from("secret put session")),
        "the session was never kept: {:?}",
        seen.all()
    );
}

#[test]
fn a_refused_write_is_answered_unsupported_and_the_core_asks_that_port_no_more() {
    let seen = Seen::default();
    // This store refuses the device key, the first thing the core writes. On an answer of
    // `Unsupported` the core stops using the port and holds its tokens in memory, so a session
    // that signs in after it is never offered to the store at all.
    let mut host = host(
        scripted(vec![("/v1/auth/password", 200, SIGNED_IN)], &seen),
        &seen,
        false,
    );
    signs_in(&mut host);
    let lines = seen.all();
    assert!(lines.contains(&String::from("secret put device")));
    assert!(
        !lines.contains(&String::from("secret put session")),
        "a refused store was asked again: {lines:?}"
    );
}

#[test]
fn an_unanswerable_server_leaves_a_wake_for_the_surface_to_honour() {
    let seen = Seen::default();
    let mut host = host(scripted(Vec::new(), &seen), &seen, true);
    // The first turn sends hello; the 503 arrives on the next drain and the core backs off.
    host.step(Vec::new());
    let turn = host.drain();
    assert!(
        turn.next_wake_ms.is_some(),
        "no wake came back, so nothing would ever ask again"
    );
}
