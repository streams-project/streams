//! The real binary as a process, on the sample set, on 127.0.0.1. Nothing outside this machine is
//! called, and the process ends with the test.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("server_binary_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Ends the process however the test ends.
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

fn serve(pairs: &[(&str, String)]) -> Running {
    let mut command = Command::new(env!("CARGO_BIN_EXE_server"));
    command
        .env_clear()
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (name, value) in pairs {
        command.env(name, value);
    }
    Running(command.spawn().expect("the binary starts"))
}

async fn ready(port: u16) -> Value {
    let started = Instant::now();
    loop {
        let address = format!("http://127.0.0.1:{port}/v1/ready");
        if let Ok(answer) = reqwest::get(&address).await {
            return answer.json().await.expect("json");
        }
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "the server did not answer"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn h68_the_process_serves_ready_and_hello_on_the_sample_with_the_build_named() {
    let dir = folder("serve");
    let manifest = server_catalog::sample::write_sample(&dir);
    let port = free_port();
    let _server = serve(&[
        ("SERVER_LISTEN", format!("127.0.0.1:{port}")),
        ("DATA_MANIFEST", manifest.to_string_lossy().into_owned()),
        ("BUILD_ID", String::from("process-test")),
        ("BRAND_NAME", String::from("Sample")),
    ]);
    let answer = ready(port).await;
    assert_eq!(answer["build"], "process-test");
    assert_eq!(answer["index"], "degraded");
    assert!(!answer["data_hash"].as_str().unwrap().is_empty());
    let hello: Value = reqwest::get(format!("http://127.0.0.1:{port}/v1/hello"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(hello["brand"]["name"], "Sample");
    assert_eq!(hello["features"]["accounts_required"], true);
}

#[tokio::test]
async fn s213_a_server_with_no_settings_says_so_and_stops_with_no_address() {
    let output = Command::new(env!("CARGO_BIN_EXE_server"))
        .env_clear()
        .output()
        .expect("the binary runs");
    assert!(!output.status.success());
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(text.contains("SERVER_LISTEN"), "{text}");
}

#[tokio::test]
async fn s151_a_set_refused_at_start_leaves_titles_off_and_the_server_up() {
    let dir = folder("refused");
    let manifest = server_catalog::sample::write_sample(&dir);
    std::fs::write(dir.join("vectors.keys"), [0u8; 4]).unwrap();
    let port = free_port();
    let _server = serve(&[
        ("SERVER_LISTEN", format!("127.0.0.1:{port}")),
        ("DATA_MANIFEST", manifest.to_string_lossy().into_owned()),
    ]);
    let answer = ready(port).await;
    assert_eq!(answer["index"], "down");
    assert_eq!(answer["data_hash"], "");
}

#[test]
fn s138_the_index_builds_from_the_command_and_the_page_check_prints() {
    let dir = folder("index");
    let manifest = server_catalog::sample::write_sample(&dir);
    let index = dir.join("index.bin");
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_server"))
            .env_clear()
            .env("DATA_MANIFEST", &manifest)
            .env("INDEX_PATH", &index)
            .args(args)
            .output()
            .expect("the binary runs")
    };
    assert!(run(&["index", "build"]).status.success());
    assert!(
        server_catalog::index::read_stamp(&index).is_some(),
        "the index has its stamp"
    );
    let check = run(&["check", "pages"]);
    assert!(check.status.success());
    let text = String::from_utf8_lossy(&check.stdout);
    assert!(text.contains("adult movie film.dramas"), "{text}");
    assert!(run(&["nonsense"]).stderr.len() > 10);
}
