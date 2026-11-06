//! Keeps each program running. A program that exits is started again after a growing wait, and
//! everything stops together.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::process::{Child, Command};

const FIRST_WAIT: Duration = Duration::from_secs(1);
const LONGEST_WAIT: Duration = Duration::from_secs(30);
/// Running this long counts as healthy, so the next failure waits the shortest time again.
const STEADY: Duration = Duration::from_secs(60);

/// One program to keep running.
#[derive(Clone, Debug)]
pub struct Program {
    pub name: String,
    pub path: PathBuf,
    pub env: BTreeMap<String, String>,
    pub log: PathBuf,
}

impl Program {
    fn spawn(&self) -> Result<Child> {
        if let Some(folder) = self.log.parent() {
            std::fs::create_dir_all(folder)?;
        }
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log)?;
        Command::new(&self.path)
            .env_clear()
            .envs(&self.env)
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log)
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("{} did not start", self.path.display()))
    }
}

/// The next wait, doubling up to the longest.
#[must_use]
pub fn next_wait(wait: Duration, ran_for: Duration) -> Duration {
    if ran_for >= STEADY {
        FIRST_WAIT
    } else {
        (wait * 2).min(LONGEST_WAIT)
    }
}

/// Run one program for as long as this future lives.
pub async fn keep(program: Program) {
    let mut wait = FIRST_WAIT;
    loop {
        let began = Instant::now();
        match program.spawn() {
            Ok(mut child) => {
                let ended = child.wait().await;
                eprintln!(
                    "{} stopped ({:?}), see {}",
                    program.name,
                    ended.ok().and_then(|status| status.code()),
                    program.log.display()
                );
            }
            Err(why) => eprintln!("{why:#}"),
        }
        wait = next_wait(wait, began.elapsed());
        tokio::time::sleep(wait).await;
    }
}

/// True once this address answers.
pub async fn answers(url: &str, within: Duration) -> bool {
    let began = Instant::now();
    while began.elapsed() < within {
        if let Ok(answer) = reqwest::get(url).await
            && answer.status().is_success()
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    false
}

/// A log path inside the home folder.
#[must_use]
pub fn log_path(home: &Path, name: &str) -> PathBuf {
    home.join("logs").join(format!("{name}.log"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wait_doubles_and_stops_growing() {
        let quick = Duration::from_secs(1);
        assert_eq!(next_wait(FIRST_WAIT, quick), Duration::from_secs(2));
        assert_eq!(next_wait(Duration::from_secs(20), quick), LONGEST_WAIT);
        assert_eq!(next_wait(LONGEST_WAIT, quick), LONGEST_WAIT);
    }

    #[test]
    fn a_program_that_ran_a_while_starts_from_the_shortest_wait() {
        assert_eq!(next_wait(LONGEST_WAIT, STEADY), FIRST_WAIT);
    }
}
