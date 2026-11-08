//! Fixtures are plain JSON and every language reads the same files. Covers C11, H57, H91, C16,
//! C112, and the rule that unknown fields are ignored.

use std::fs;
use std::path::{Path, PathBuf};

use contract::account::{
    Account, AccountExport, AccountUpdate, AvatarSets, CodeSend, CodeSent, CodeSignIn, Confirm,
    Confirmed, Consent, DeviceCardAnswer, DeviceCardRequest, DeviceCode, DeviceCodeRequest,
    DeviceDecision, DeviceList, DeviceTokenAnswer, DeviceTokenRequest, Done, EmailChange,
    PasswordChange, PasswordSignIn, PhoneChange, PinSet, Profile, ProfileCreate, ProfileEnter,
    ProfileEntered, ProfileLimits, ProfileList, ProfileSettings, ProfileTransfer, ProfileUpdate,
    RecoveryFinish, RecoveryStart, RefreshRequest, SignOut, SignUp, SignedIn,
};
use contract::api::ROUTES;
use contract::client::CapabilityCard;
use contract::command::{Change, Changes, Command, CommandAnswer, CommandBatch};
use contract::core_msg::{
    Action, CoreConfig, CoreIn, CoreOut, Effect, Event, HostedStep, LayerConfig, View,
};
use contract::driver::{DriverCommand, DriverEvent};
use contract::error::{ErrorBody, ErrorCode, Notice};
use contract::finder::{FindAnswer, FindRequest, FinderHealth};
use contract::hello::{Hello, LanguageList, Ready, Strings};
use contract::help::{HelpArticle, HelpHome, HelpPolicy, HelpResults};
use contract::play::{
    Attempt, Marks, MarksRequest, MoreAttempts, PlayAnswer, PlayEvents, PlayRequest, Report,
    SubtitleBody, SubtitleList,
};
use contract::screen::{
    More, Nav, PublicTitles, Resume, Screen, Season, Section, Suggestions, Tile, TitleDetail,
};
use contract::version::{VersionRange, WIRE_VERSION};
use contract::{EpisodeRef, Rating, TitleKey, TitleKind};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

const MAP_FIELDS: &[&str] = &["headers", "texts", "subtitle_style"];

fn folder(kind: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(WIRE_VERSION)
        .join(kind)
}

fn files(kind: &str) -> Vec<PathBuf> {
    let mut list: Vec<PathBuf> = fs::read_dir(folder(kind))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    list.sort();
    list
}

fn type_of(path: &Path) -> String {
    let name = path.file_name().unwrap().to_string_lossy();
    name.split('.').next().unwrap().to_string()
}

fn read<T: DeserializeOwned + Serialize>(value: &Value) -> Result<Value, String> {
    let parsed: T = serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
    serde_json::to_value(parsed).map_err(|error| error.to_string())
}

/// Parse `value` as the named type and write it back.
fn read_as(type_name: &str, value: &Value) -> Result<Value, String> {
    macro_rules! table {
        ($($name:ident),+ $(,)?) => {
            match type_name {
                $(stringify!($name) => read::<$name>(value),)+
                other => panic!("no such type {other}"),
            }
        };
    }
    table!(
        Account,
        AccountExport,
        AccountUpdate,
        AvatarSets,
        Action,
        Attempt,
        CapabilityCard,
        Change,
        Changes,
        CodeSend,
        CodeSent,
        CodeSignIn,
        Command,
        CommandAnswer,
        CommandBatch,
        Confirm,
        Confirmed,
        Consent,
        CoreConfig,
        CoreIn,
        CoreOut,
        DeviceCardAnswer,
        DeviceCardRequest,
        DeviceCode,
        DeviceCodeRequest,
        DeviceDecision,
        DeviceList,
        DeviceTokenAnswer,
        DeviceTokenRequest,
        Done,
        DriverCommand,
        DriverEvent,
        Effect,
        EmailChange,
        EpisodeRef,
        ErrorBody,
        ErrorCode,
        Event,
        FindAnswer,
        FindRequest,
        FinderHealth,
        Hello,
        HostedStep,
        LayerConfig,
        Marks,
        MarksRequest,
        More,
        MoreAttempts,
        Nav,
        Notice,
        PasswordChange,
        PasswordSignIn,
        HelpArticle,
        HelpHome,
        HelpPolicy,
        HelpResults,
        LanguageList,
        PhoneChange,
        PinSet,
        PlayAnswer,
        PlayEvents,
        PlayRequest,
        Profile,
        ProfileCreate,
        ProfileEnter,
        ProfileEntered,
        ProfileLimits,
        ProfileList,
        ProfileSettings,
        ProfileTransfer,
        ProfileUpdate,
        Rating,
        Ready,
        RecoveryFinish,
        RecoveryStart,
        RefreshRequest,
        Report,
        Screen,
        Resume,
        Season,
        Section,
        SignOut,
        SignUp,
        SignedIn,
        Strings,
        SubtitleBody,
        SubtitleList,
        Suggestions,
        Tile,
        TitleDetail,
        PublicTitles,
        TitleKey,
        TitleKind,
        VersionRange,
        View,
    )
}

fn load(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap();
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path:?}: {error}"))
}

fn valid_names() -> Vec<String> {
    files("valid")
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn c11_every_valid_fixture_reads_and_writes_back_the_same() {
    let list = files("valid");
    assert!(!list.is_empty());
    for path in list {
        let value = load(&path);
        let back = read_as(&type_of(&path), &value).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        assert_eq!(back, value, "{path:?}");
    }
}

#[test]
fn c11_every_type_on_a_route_has_a_fixture() {
    let names = valid_names();
    for route in ROUTES {
        let mut wanted = vec![route.response, "ErrorBody"];
        wanted.extend(route.request);
        for type_name in wanted {
            let prefix = format!("{type_name}.");
            assert!(
                names.iter().any(|name| name.starts_with(&prefix)),
                "{} has no fixture for {type_name}",
                route.name
            );
        }
    }
}

#[test]
fn h57_h91_every_refused_fixture_fails_to_parse() {
    let list = files("refused");
    let bare: Vec<&PathBuf> = list
        .iter()
        .filter(|path| path.to_string_lossy().contains("bare_id"))
        .collect();
    assert!(bare.len() >= 5, "bare id fixtures: {}", bare.len());
    for path in &list {
        let value = load(path);
        assert!(read_as(&type_of(path), &value).is_err(), "{path:?} parsed");
    }
}

fn add_unknown_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map.iter_mut() {
                if !MAP_FIELDS.contains(&key.as_str()) {
                    add_unknown_fields(inner);
                }
            }
            map.insert("field_from_the_future".into(), json!({"deep": [1, "two"]}));
        }
        Value::Array(items) => items.iter_mut().for_each(add_unknown_fields),
        _ => {}
    }
}

#[test]
fn unknown_fields_are_ignored_at_every_level_of_every_fixture() {
    for path in files("valid") {
        let clean = load(&path);
        let mut noisy = clean.clone();
        add_unknown_fields(&mut noisy);
        let back = read_as(&type_of(&path), &noisy).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        assert_eq!(back, clean, "{path:?}");
    }
}

#[test]
fn tolerated_fixtures_read_and_match_their_valid_sibling() {
    let list = files("tolerated");
    assert!(!list.is_empty());
    for path in list {
        let value = load(&path);
        let type_name = type_of(&path);
        let back = read_as(&type_name, &value).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() == 4 {
            let sibling = folder("valid").join(format!("{}.{}.json", parts[0], parts[1]));
            assert_eq!(back, load(&sibling), "{path:?}");
        }
    }
}

fn check_numbers(value: &Value, path: &Path) {
    match value {
        Value::Number(number) => {
            assert!(!number.is_f64(), "{path:?}: float {number}");
            let high = (1_i64 << 53) - 1;
            assert!(
                number.as_i64().is_some_and(|n| n.abs() <= high),
                "{path:?}: {number}"
            );
        }
        Value::Object(map) => map.values().for_each(|inner| check_numbers(inner, path)),
        Value::Array(items) => items.iter().for_each(|inner| check_numbers(inner, path)),
        _ => {}
    }
}

#[test]
fn integers_stay_under_two_to_the_53_and_no_float_appears() {
    for path in files("valid") {
        check_numbers(&load(&path), &path);
    }
}

fn kinds_of(type_name: &str, tag: &str) -> Vec<String> {
    let prefix = format!("{type_name}.");
    let mut kinds: Vec<String> = valid_names()
        .iter()
        .filter(|name| name.starts_with(&prefix))
        .map(|name| {
            load(&folder("valid").join(name))[tag]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    kinds.sort();
    kinds.dedup();
    kinds
}

#[test]
fn effect_fixtures_show_exactly_the_eight_kinds() {
    let wanted = [
        "http", "log", "open", "player", "secret", "store", "timer", "view",
    ];
    assert_eq!(kinds_of("Effect", "kind"), wanted);
}

#[test]
fn driver_fixtures_show_all_six_events_and_nine_commands() {
    let events = [
        "ended",
        "error",
        "first_frame",
        "position",
        "stalled",
        "tracks",
    ];
    assert_eq!(kinds_of("DriverEvent", "kind"), events);
    let commands = [
        "close",
        "open",
        "pause",
        "play",
        "rate",
        "seek",
        "select_audio",
        "select_text",
        "volume",
    ];
    assert_eq!(kinds_of("DriverCommand", "op"), commands);
}

#[test]
fn command_fixtures_show_every_command() {
    let wanted = [
        "clear_history",
        "feedback",
        "hide",
        "list",
        "position",
        "rate",
        "setting",
        "trailer_refused",
    ];
    assert_eq!(kinds_of("Command", "command"), wanted);
}

#[test]
fn c112_no_screen_state_fixture_holds_a_token() {
    for path in files("valid") {
        let type_name = type_of(&path);
        if type_name == "View" || type_name == "CoreOut" {
            let value = load(&path);
            let view = if type_name == "View" {
                &value
            } else {
                &value["view"]
            };
            let text = view.to_string();
            assert!(!text.contains("token"), "{path:?}");
        }
    }
}

#[test]
fn s36_s175_u8_every_attempt_address_is_an_outside_host() {
    for path in files("valid") {
        let text = fs::read_to_string(&path).unwrap();
        for line in text.lines().filter(|line| line.contains("\"url\"")) {
            assert!(line.contains("https://"), "{path:?}: {line}");
            assert!(line.contains(".example/"), "{path:?}: {line}");
        }
    }
}
