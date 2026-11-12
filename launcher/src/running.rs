//! Whether this stack is already running, and the handover when it is.
//!
//! `./up` says of itself "Run it again to pick up changes.", and before this file a second run
//! did not pick anything up. It started a second copy of everything on the same home folder, the
//! second server could not bind the port the first one's server held, and it restarted into
//! `logs/server.log` with nothing on the screen saying so. Two launchers were found on 2026-11-11
//! running 3h18m and 3h03m, the second one failing the whole time.
//!
//! So a second run takes over, and it does it by asking rather than killing: the first launcher
//! is sent the stop it already handles, which is the one that closes its children and its
//! database together. Only once the first has gone does the second start. If it has not gone
//! within [`HANDOVER`] the second says what is in the way, names the process, and stops. It never
//! runs beside the first.
//!
//! The claim to the home folder is an exclusive lock on one file, not a bare process number. A
//! number on its own cannot tell a launcher that is still here from a number the system has since
//! given to something else, and stopping the wrong process is worse than refusing to start. While
//! the lock is held the number inside the file belongs to whoever holds it.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

/// The file a running launcher holds in the home folder.
const LOCK: &str = "up.lock";

/// How long a handover waits for the launcher that is already here to finish stopping. Its own
/// stop closes the children and the database, and the database is given up to 60 s of that by
/// `pg_ctl`, so this is that with room to spare rather than a guess.
const HANDOVER: Duration = Duration::from_secs(90);

/// How often the handover looks again. Short enough that a quick stop is not waited out.
const LOOK_AGAIN: Duration = Duration::from_millis(200);

/// This run's claim to the home folder. Dropping it, or the process ending for any reason,
/// gives the claim up.
#[derive(Debug)]
pub struct Claim {
    /// Held open, because the lock lives on the open file and not on its name.
    _file: File,
    path: PathBuf,
}

impl Drop for Claim {
    fn drop(&mut self) {
        // The number inside is this process's and means nothing once it has gone. The file stays:
        // removing it would race a launcher that has already opened it to take over.
        let _ = std::fs::write(&self.path, b"");
    }
}

/// What stands between this run and the home folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Nothing else is here.
    Free,
    /// A launcher holds it, with its process number.
    Held(u32),
}

/// Tries before a refused lock is believed, and the wait between them.
///
/// One refusal proves nothing. Starting any child duplicates every descriptor this process has
/// open, lock and all, and the duplicate only goes when the child reaches its own program, so for
/// that moment a lock this process has already let go of is still held by the copy. The launcher
/// starts children constantly, and the test of a dropped claim read it as held about three times
/// in five until this loop was here. The window is a fork away from an exec, so a few tries over
/// a tenth of a second settle it, and a launcher that really is here fails all of them.
const CONFIRM_TRIES: u8 = 5;
const CONFIRM_WAIT: Duration = Duration::from_millis(20);

/// Which launcher, if any, holds `home`. A refused lock is confirmed first, which costs up to
/// [`CONFIRM_TRIES`] times [`CONFIRM_WAIT`] and only when something does hold it.
///
/// # Errors
/// The home folder or the lock file cannot be opened.
pub fn who_holds(home: &Path) -> Result<(Standing, File)> {
    std::fs::create_dir_all(home)?;
    let path = home.join(LOCK);
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("{} could not be opened", path.display()))?;
    for attempt in 1..=CONFIRM_TRIES {
        match file.try_lock() {
            Ok(()) => {
                // Taken and given straight back: this call answers a question and claims nothing,
                // so the caller that wants the claim takes it itself.
                let _ = file.unlock();
                return Ok((Standing::Free, file));
            }
            Err(TryLockError::WouldBlock) if attempt < CONFIRM_TRIES => {
                std::thread::sleep(CONFIRM_WAIT);
            }
            Err(TryLockError::WouldBlock) => {
                let mut text = String::new();
                file.read_to_string(&mut text)?;
                // Held with no readable number: a launcher that has the file open and has not
                // written itself into it yet. It is still in the way, and 0 is never a process a
                // stop is sent to, so it reads as held by nobody nameable.
                return Ok((Standing::Held(text.trim().parse().unwrap_or(0)), file));
            }
            Err(TryLockError::Error(why)) => {
                return Err(why).with_context(|| format!("{} could not be locked", path.display()));
            }
        }
    }
    unreachable!("the last attempt either takes the lock or answers that it is held")
}

/// Take the claim, which must be free.
fn take(home: &Path, mut file: File) -> Result<Claim> {
    file.try_lock()
        .map_err(|_| anyhow::anyhow!("another launcher took the home folder first"))?;
    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;
    write!(file, "{}", std::process::id())?;
    file.flush()?;
    Ok(Claim {
        _file: file,
        path: home.join(LOCK),
    })
}

/// Ask a process to stop, the way Ctrl-C does. `false` when there is no such process to ask,
/// which is as good as it having stopped.
fn ask_to_stop(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// The claim on `home` for this run, taking over from a launcher that is already here.
///
/// A launcher already holding it is asked to stop and waited for, up to [`HANDOVER`]. `say` is
/// given each line meant for the person, so this is quiet in a test and spoken on a terminal.
///
/// # Errors
/// The launcher that is here did not stop inside [`HANDOVER`], or the lock file cannot be used.
/// Either way nothing has been started.
pub fn claim(home: &Path, say: &mut dyn FnMut(&str)) -> Result<Claim> {
    claim_within(home, HANDOVER, say)
}

/// [`claim`] with the wait named, so a test does not sit through the real one.
///
/// # Errors
/// As [`claim`].
pub fn claim_within(home: &Path, within: Duration, say: &mut dyn FnMut(&str)) -> Result<Claim> {
    let (standing, file) = who_holds(home)?;
    let Standing::Held(pid) = standing else {
        return take(home, file);
    };
    drop(file);
    if pid == 0 {
        bail!(
            "another copy of this program is starting in {}. Wait for it, or stop it and run \
             this again.",
            home.display()
        );
    }
    // The one number a stop is never sent to. A lock cannot be taken twice even inside one
    // process, so a caller that already holds this home would otherwise ask itself to stop and
    // be the only casualty of its own handover. Found by the test below, which stopped its own
    // test runner before this check existed.
    if pid == std::process::id() {
        bail!(
            "this program already holds {} in this very process, so there is nothing to hand \
             over to. Nothing was started.",
            home.display()
        );
    }
    say(&format!(
        "This is already running here (process {pid}). Stopping it to take over..."
    ));
    ask_to_stop(pid);
    let began = Instant::now();
    loop {
        let (standing, file) = who_holds(home)?;
        if standing == Standing::Free {
            say("The one that was running has stopped.");
            return take(home, file);
        }
        drop(file);
        if began.elapsed() >= within {
            bail!(
                "this is already running here as process {pid} and has not stopped in {} \
                 seconds. Stop it yourself, then run this again. Nothing was started.",
                within.as_secs()
            );
        }
        std::thread::sleep(LOOK_AGAIN);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("running-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).expect("a home");
        folder
    }

    fn quiet() -> impl FnMut(&str) {
        |_| {}
    }

    #[test]
    fn an_empty_home_is_free_and_the_claim_writes_this_process() {
        let home = home("free");
        assert_eq!(who_holds(&home).expect("read").0, Standing::Free);
        let claim = claim(&home, &mut quiet()).expect("the claim");
        assert_eq!(
            who_holds(&home).expect("read").0,
            Standing::Held(std::process::id()),
            "a held home must name who holds it, so a second run has someone to ask"
        );
        drop(claim);
        assert_eq!(
            who_holds(&home).expect("read").0,
            Standing::Free,
            "the claim outlived the run that took it"
        );
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn a_handover_to_a_launcher_that_will_not_stop_is_bounded_and_starts_nothing() {
        let home = home("stuck");
        // A holder that is alive, is asked to stop, and does not: a shell that ignores the stop
        // signal. The lock is held here and the file names that child, which is exactly what a
        // second launcher meets. Before this file there was no bound at all and the second
        // launcher kept trying for three hours, so what is proved is that this call comes back.
        let _held = claim(&home, &mut quiet()).expect("the first claim");
        let mut deaf = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg("trap '' TERM; sleep 5")
            .spawn()
            .expect("a child that ignores the stop");
        std::fs::write(home.join(LOCK), deaf.id().to_string()).expect("the holder's number");
        let began = Instant::now();
        let why = claim_within(&home, Duration::from_millis(400), &mut quiet())
            .expect_err("a second launcher must not run beside the first")
            .to_string();
        let waited = began.elapsed();
        let _ = deaf.kill();
        let _ = deaf.wait();
        assert!(why.contains("already running"), "{why}");
        assert!(why.contains("Nothing was started"), "{why}");
        assert!(
            waited >= Duration::from_millis(400),
            "it gave up before the wait was over: {waited:?}"
        );
        assert!(
            waited < Duration::from_secs(5),
            "the wait is not bounded: {waited:?}"
        );
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn a_run_that_already_holds_the_home_is_never_asked_to_stop_itself() {
        let home = home("itself");
        let _held = claim(&home, &mut quiet()).expect("the first claim");
        let why = claim(&home, &mut quiet())
            .expect_err("asking itself to stop would end the only launcher there is")
            .to_string();
        assert!(why.contains("this very process"), "{why}");
        std::fs::remove_dir_all(&home).expect("cleaned");
    }

    #[test]
    fn a_lock_file_left_behind_by_a_run_that_is_gone_does_not_hold_the_home() {
        let home = home("stale");
        // Exactly what a launcher killed outright leaves: the file with its number still in it
        // and no lock, because the lock went with the process. A number alone must not stand in
        // the way, or a machine that was shut down hard would need a file deleted by hand.
        std::fs::write(home.join(LOCK), "999999").expect("a stale file");
        assert_eq!(who_holds(&home).expect("read").0, Standing::Free);
        claim(&home, &mut quiet()).expect("a stale file must not refuse a start");
        std::fs::remove_dir_all(&home).expect("cleaned");
    }
}
