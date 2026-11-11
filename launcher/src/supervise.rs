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
/// Failures in a row, none of which ran steadily, before a program is left stopped. A program that
/// cannot take its port never runs steadily, and without this it restarted into a log file for as
/// long as the service was up with nothing on the screen saying so.
const FAILURES_TO_GIVE_UP: u32 = 5;

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

/// Failures in a row with nothing steady between them, and whether that is now enough to leave
/// the program stopped. A program that cannot take its port fails the same way every time, so
/// trying for ever only hides it.
#[must_use]
pub fn give_up(failures: u32, ran_for: Duration) -> (u32, bool) {
    let failures = if ran_for >= STEADY { 0 } else { failures + 1 };
    (failures, failures >= FAILURES_TO_GIVE_UP)
}

/// Run one program for as long as this future lives, or until it has failed to run enough times
/// that something is wrong with it rather than with this moment.
pub async fn keep(program: Program) {
    let mut wait = FIRST_WAIT;
    let mut failures = 0;
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
        let ran_for = began.elapsed();
        let stop;
        (failures, stop) = give_up(failures, ran_for);
        if stop {
            eprintln!(
                "PROBLEM {} stopped {failures} times without staying up, so it is left stopped",
                program.name
            );
            eprintln!("See {}", program.log.display());
            return;
        }
        wait = next_wait(wait, ran_for);
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

    #[test]
    fn a_program_that_never_stays_up_is_given_up_on() {
        let quick = Duration::from_secs(1);
        let mut failures = 0;
        for _ in 1..FAILURES_TO_GIVE_UP {
            let stop;
            (failures, stop) = give_up(failures, quick);
            assert!(!stop, "{failures} is not yet enough");
        }
        assert_eq!(give_up(failures, quick), (FAILURES_TO_GIVE_UP, true));
        // One steady run clears the record, so a program that works is kept for ever.
        assert_eq!(give_up(FAILURES_TO_GIVE_UP - 1, STEADY), (0, false));
    }

    #[tokio::test(start_paused = true)]
    async fn a_program_that_cannot_run_is_left_stopped_rather_than_restarted_for_ever() {
        let folder = std::env::temp_dir().join(format!("keep-{}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("a folder");
        let program = Program {
            name: String::from("missing"),
            path: folder.join("there-is-no-such-program"),
            env: BTreeMap::new(),
            log: folder.join("missing.log"),
        };
        // The future ends. Without the bound it never would, and this test would hang.
        keep(program).await;
        std::fs::remove_dir_all(&folder).expect("cleaned");
    }
}
