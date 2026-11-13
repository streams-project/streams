//! That no write of playback telemetry can take a person, enforced rather than described.
//!
//! This module used to hold `Learned`, a writer of telemetry about one person that could not be
//! built without their answer about personal rows. It existed because playback health was keyed
//! to `profile_id` in `play_events`, and the rule it carried was that a handler must not be able
//! to skip the consent test.
//!
//! The rows moved. A rebuffer is a fact about a host, a network and a build, so playback health
//! is now folded into counts under (client class, host, day, error class) in `reachability`, a
//! table with no account or profile column, and the write that takes it
//! (`server_play::PlayLog::health`) has no parameter a person could be passed through. A gate you
//! cannot route around is good; a write that has nowhere to put a person is better, because it
//! stays true when someone adds a second caller.
//!
//! What is left here is the check that keeps it true. It reads the two files of the telemetry
//! path as text and fails if either grows a way to name a person. The file name is the one the
//! consent gate had; it is kept so the history of the rule stays findable.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    /// A file of the telemetry path, relative to this crate.
    fn source(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../server-play/src")
            .join(name)
    }

    /// Words that name a person in this tree. `class` is a client class and not one of them.
    const PERSON: &[&str] = &[
        "ProfileId",
        "ProfileScope",
        "AccountId",
        "profile_id",
        "account_id",
        "session_id",
        "SessionId",
    ];

    #[test]
    fn c36_c104_no_writer_of_playback_health_can_be_handed_a_person() {
        for name in ["log.rs", "health.rs"] {
            let text = std::fs::read_to_string(source(name)).expect("a file of the telemetry path");
            let code: String = text
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            for word in PERSON {
                assert!(
                    !code.contains(word),
                    "server-play/src/{name} names {word}: playback health is a fact about a host, \
                     a network and a build, and the way it stays that way is that the code has \
                     nowhere to put a person, not that a setting is read first"
                );
            }
        }
    }
}
