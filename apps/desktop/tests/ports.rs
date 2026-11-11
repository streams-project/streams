//! The three ports a desktop really fills: the files behind secrets and the store, and a real
//! request over a real socket. Nothing here is mocked at the socket.

use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use contract::core_msg::{HttpMethod, NoAnswerReason};
use contract::driver::DriverCommand;
use desktop::{Answer, Call, FileSecrets, FileStore, Network, NoPlayer, Player};
use desktop::{HttpNetwork, Secrets, Store};

/// A folder of this test's own, under the target folder so a run leaves nothing behind elsewhere.
fn folder(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a folder for this test");
    path
}

#[test]
fn a_secret_goes_in_comes_back_and_goes() {
    let secrets = FileSecrets::new(folder("secrets-round-trip"));
    assert_eq!(secrets.get("session"), None);
    assert!(secrets.put("session", r#"{"kept":true}"#));
    assert_eq!(secrets.get("session").as_deref(), Some(r#"{"kept":true}"#));
    assert!(secrets.put("session", "second"));
    assert_eq!(secrets.get("session").as_deref(), Some("second"));
    secrets.delete("session");
    assert_eq!(secrets.get("session"), None);
    // Deleting what is not there is not a fault: the core drops a secret it never wrote.
    secrets.delete("session");
}

#[cfg(unix)]
#[test]
fn a_secret_file_is_the_owners_alone() {
    use std::os::unix::fs::PermissionsExt as _;

    let at = folder("secrets-rights");
    let secrets = FileSecrets::new(&at);
    assert!(secrets.put("device", "key"));
    let only = std::fs::read_dir(&at)
        .expect("the folder")
        .filter_map(Result::ok)
        .next()
        .expect("one file");
    let mode = only.metadata().expect("its rights").permissions().mode();
    assert_eq!(mode & 0o777, 0o600, "mode was {:o}", mode & 0o777);
}

#[test]
fn a_refused_folder_is_said_to_be_refused_rather_than_pretended() {
    // A file where the folder should be: nothing can be written under it on any system.
    let at = folder("secrets-refused");
    let blocked = at.join("wall");
    std::fs::write(&blocked, "").expect("a file in the way");
    let secrets = FileSecrets::new(blocked.join("secrets"));
    assert!(!secrets.put("session", "token"));
    assert_eq!(secrets.get("session"), None);
}

#[test]
fn the_store_lists_the_layers_own_keys_in_order_and_nothing_else() {
    let at = folder("store-keys");
    let store = FileStore::new(&at);
    for key in [
        "q/0000000000000002",
        "q/0000000000000001",
        "index",
        "c/0000000000000001",
    ] {
        assert!(store.put(key, key), "{key} was refused");
    }
    // A file this host did not write, which must never come back as a key.
    std::fs::write(at.join("notes.txt"), "stray").expect("a stray file");
    assert_eq!(
        store.keys("q/"),
        vec![
            String::from("q/0000000000000001"),
            String::from("q/0000000000000002")
        ]
    );
    assert_eq!(store.keys("index"), vec![String::from("index")]);
    assert_eq!(store.get("index").as_deref(), Some("index"));
    store.delete("index");
    assert_eq!(store.get("index"), None);
    assert!(store.keys("").contains(&String::from("c/0000000000000001")));
    assert!(!store.keys("").contains(&String::from("notes.txt")));
}

#[test]
fn a_store_with_no_folder_lists_nothing_and_reads_nothing() {
    let store = FileStore::new(folder("store-empty").join("never-made"));
    assert_eq!(store.keys("q/"), Vec::<String>::new());
    assert_eq!(store.get("index"), None);
}

/// A server of this test's own: it answers the first caller with these bytes and stops.
fn once(reply: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let at = listener.local_addr().expect("its address");
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut heard = [0_u8; 1024];
            let _ = stream.read(&mut heard);
            let _ = stream.write_all(reply.as_bytes());
            let _ = stream.flush();
        }
    });
    format!("http://{at}")
}

fn network() -> (tokio::runtime::Runtime, HttpNetwork) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("a runtime");
    let port = HttpNetwork::new(runtime.handle().clone()).expect("tls on this machine");
    (runtime, port)
}

fn ask(port: &HttpNetwork, url: String, deadline_ms: u32) -> Answer {
    let (back, answers) = channel();
    port.send(
        Call {
            method: HttpMethod::Get,
            url,
            headers: BTreeMap::from([(String::from("Api-Version"), String::from("2026-11-05"))]),
            body: None,
            deadline_ms,
        },
        Box::new(move |answer| {
            let _ = back.send(answer);
        }),
    );
    answers
        .recv_timeout(Duration::from_secs(10))
        .expect("an answer within ten seconds")
}

#[test]
fn a_real_answer_brings_back_its_status_its_etag_and_its_body() {
    let (_runtime, port) = network();
    let at = once("HTTP/1.1 200 OK\r\nContent-Length: 13\r\nETag: \"w-7\"\r\n\r\n{\"screen\":1}\n");
    match ask(&port, format!("{at}/v1/screens/home"), 5_000) {
        Answer::Reply { status, etag, text } => {
            assert_eq!(status, 200);
            assert_eq!(etag.as_deref(), Some("\"w-7\""));
            assert_eq!(text, "{\"screen\":1}\n");
        }
        Answer::NoReply { reason } => panic!("no answer came: {reason:?}"),
    }
}

#[test]
fn an_error_status_is_an_answer_too() {
    let (_runtime, port) = network();
    let at = once("HTTP/1.1 503 Service Unavailable\r\nContent-Length: 2\r\n\r\n{}");
    match ask(&port, format!("{at}/v1/hello"), 5_000) {
        Answer::Reply { status, text, .. } => {
            assert_eq!(status, 503);
            assert_eq!(text, "{}");
        }
        Answer::NoReply { reason } => panic!("no answer came: {reason:?}"),
    }
}

#[test]
fn a_server_that_never_answers_ends_at_the_deadline() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let at = listener.local_addr().expect("its address");
    std::thread::spawn(move || {
        // Held open and silent, which is the fault the deadline is for.
        let held = listener.accept();
        std::thread::sleep(Duration::from_secs(5));
        drop(held);
    });
    let (_runtime, port) = network();
    match ask(&port, format!("http://{at}/v1/hello"), 300) {
        Answer::NoReply { reason } => assert_eq!(reason, NoAnswerReason::Timeout),
        Answer::Reply { status, .. } => panic!("a silent server answered {status}"),
    }
}

#[test]
fn nothing_listening_is_no_answer_at_all() {
    // A port bound and dropped: the address is real and nothing is behind it.
    let taken = TcpListener::bind("127.0.0.1:0").expect("a port");
    let at = taken.local_addr().expect("its address");
    drop(taken);
    let (_runtime, port) = network();
    match ask(&port, format!("http://{at}/v1/hello"), 2_000) {
        Answer::NoReply { reason } => assert_eq!(reason, NoAnswerReason::Offline),
        Answer::Reply { status, .. } => panic!("a closed port answered {status}"),
    }
}

#[test]
fn the_connection_watch_says_nothing_yet_and_says_so_by_doing_nothing() {
    let (_runtime, port) = network();
    let (back, heard) = channel();
    port.watch(Box::new(move |online| {
        let _ = back.send(online);
    }));
    assert!(heard.recv_timeout(Duration::from_millis(200)).is_err());
}

#[test]
fn no_film_plays_and_a_command_to_a_host_with_no_driver_goes_nowhere_quietly() {
    // The port hands nothing back, so what is proved here is that every command is taken and none
    // of them reaches anything: a dropped command is the whole behaviour, and the play is rescued
    // by the core's own start deadline, which `platform/kit/core` proves and this crate does not.
    let mut player = NoPlayer;
    for command in [
        DriverCommand::Play,
        DriverCommand::Pause,
        DriverCommand::Seek { to_ms: 1_000 },
    ] {
        player.command(command);
    }
}
