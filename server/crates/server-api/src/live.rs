//! The live settings file: flags, floors, switches, row blocks and the moved table. Read again
//! when it changes. A bad file keeps the last good value, and a missing file means defaults, so a
//! deployment without the file works.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use contract::client::DeviceClass;
use contract::hello::{ClassValue, ClientFloor, MovedAddress, Rollout};
use serde::Deserialize;
use tracing::warn;

fn yes() -> bool {
    true
}

/// What the operator can change without a restart.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LiveValues {
    pub client_min: Vec<ClientFloor>,
    pub games: bool,
    pub clips: bool,
    pub original_audio: bool,
    #[serde(default = "yes")]
    pub playback: bool,
    pub killed_builds: Vec<String>,
    pub rollouts: Vec<Rollout>,
    pub moved: Vec<MovedAddress>,
    pub start_deadline_ms: Vec<ClassValue>,
    /// Row titles the operator took off every page, lower case.
    pub blocked_rows: Vec<String>,
    /// The operator allows embeds. Off by default and in store builds.
    pub embeds: bool,
}

impl Default for LiveValues {
    fn default() -> Self {
        Self {
            client_min: Vec::new(),
            games: false,
            clips: false,
            original_audio: false,
            playback: true,
            killed_builds: Vec::new(),
            rollouts: Vec::new(),
            moved: Vec::new(),
            start_deadline_ms: Vec::new(),
            blocked_rows: Vec::new(),
            embeds: false,
        }
    }
}

impl LiveValues {
    /// The lowest version a class may run.
    #[must_use]
    pub fn floor_of(&self, class: DeviceClass) -> Option<&str> {
        self.client_min
            .iter()
            .find(|floor| floor.class == class)
            .map(|floor| floor.version.as_str())
    }
}

/// One file with its last good value.
#[derive(Debug)]
pub struct LiveFile {
    path: Option<PathBuf>,
    held: Mutex<(Option<SystemTime>, LiveValues)>,
}

impl LiveFile {
    /// A file at this path, or defaults with no path.
    #[must_use]
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            held: Mutex::new((None, LiveValues::default())),
        }
    }

    /// The values now. Costs one stat per call, and reads only when the time changed.
    #[must_use]
    pub fn values(&self) -> LiveValues {
        let mut held = self
            .held
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(path) = &self.path else {
            return held.1.clone();
        };
        let Ok(changed) = std::fs::metadata(path).and_then(|meta| meta.modified()) else {
            return held.1.clone();
        };
        if held.0 != Some(changed) {
            held.0 = Some(changed);
            let read = std::fs::read(path)
                .map_err(|error| error.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<LiveValues>(&bytes).map_err(|error| error.to_string())
                });
            match read {
                Ok(values) => held.1 = values,
                Err(error) => warn!("Live settings kept, the file did not read: {error}"),
            }
        }
        held.1.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn s152_a_bad_file_keeps_the_last_good_value() {
        let dir = std::env::temp_dir().join(format!("live_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("live.json");
        let live = LiveFile::new(Some(path.clone()));
        assert!(
            live.values().playback,
            "S153: a missing file means defaults"
        );
        std::fs::write(&path, r#"{"playback":false,"games":true}"#).unwrap();
        let first = live.values();
        assert!(!first.playback && first.games);
        std::thread::sleep(Duration::from_millis(20));
        std::fs::write(&path, "not json").unwrap();
        let file = std::fs::File::options().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::now() + Duration::from_secs(5))
            .unwrap();
        assert_eq!(live.values(), first);
    }

    #[test]
    fn s199_an_unknown_switch_is_off() {
        let live: LiveValues = serde_json::from_str(r#"{"future_switch":true}"#).unwrap();
        assert_eq!(live, LiveValues::default());
    }
}
