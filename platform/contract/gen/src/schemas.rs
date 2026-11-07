//! Every wire type, and the JSON Schema of each.

use std::collections::BTreeMap;

use contract::account::{
    Account, AccountExport, AccountUpdate, AvatarSets, CodeSend, CodeSent, CodeSignIn, Confirm,
    Confirmed, Consent, DeviceCardAnswer, DeviceCardRequest, DeviceCode, DeviceCodeRequest,
    DeviceDecision, DeviceList, DeviceTokenAnswer, DeviceTokenRequest, Done, EmailChange,
    PasswordChange, PasswordSignIn, PhoneChange, PinSet, Profile, ProfileCreate, ProfileEnter,
    ProfileEntered, ProfileLimits, ProfileList, ProfileSettings, ProfileTransfer, ProfileUpdate,
    RecoveryFinish, RecoveryStart, RefreshRequest, SignOut, SignUp, SignedIn,
};
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
    More, Nav, Resume, Screen, Season, Section, Suggestions, Tile, TitleDetail,
};
use contract::version::VersionRange;
use contract::{EpisodeRef, Rating, TitleKey, TitleKind};
use schemars::generate::SchemaSettings;
use serde_json::{Map, Value};

/// Where definitions sit inside openapi.json.
pub const DEFS_PATH: &str = "/components/schemas";

/// Largest integer the wire may carry.
pub const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;

/// All definitions by name, and the names that get a schema file of their own.
#[must_use]
pub fn definitions() -> (BTreeMap<String, Value>, Vec<String>) {
    let mut settings = SchemaSettings::draft2020_12();
    settings.definitions_path = DEFS_PATH.into();
    let mut generator = settings.into_generator();
    let mut roots = Vec::new();
    macro_rules! add {
        ($($name:ident),+ $(,)?) => {
            $(
                let _ = generator.subschema_for::<$name>();
                roots.push(stringify!($name).to_string());
            )+
        };
    }
    add!(
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
        TitleKey,
        TitleKind,
        VersionRange,
        View,
    );
    let defs: Map<String, Value> = generator.take_definitions(true);
    let defs = defs
        .into_iter()
        .map(|(name, mut schema)| {
            tidy(&mut schema);
            (name, schema)
        })
        .collect();
    roots.sort();
    (defs, roots)
}

/// An optional field is left out, never sent as null. Integers are bounded to the range every
/// client reads exactly, and the Rust width names go.
fn tidy(value: &mut Value) {
    match value {
        Value::Object(map) => {
            drop_null(map);
            lift_single_option(map);
            bound_integer(map);
            for inner in map.values_mut() {
                tidy(inner);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(tidy),
        _ => {}
    }
}

fn bound_integer(map: &mut Map<String, Value>) {
    if map.get("type").and_then(Value::as_str) != Some("integer") {
        return;
    }
    let format = map.remove("format");
    let (low, high): (i64, u64) = match format.as_ref().and_then(Value::as_str) {
        Some("uint32") => (0, u64::from(u32::MAX)),
        Some("int32") => (i64::from(i32::MIN), u64::from(i32::MAX.unsigned_abs())),
        Some("uint16") => (0, u64::from(u16::MAX)),
        _ => (0, MAX_SAFE_INTEGER),
    };
    map.insert("minimum".into(), Value::from(low));
    map.insert("maximum".into(), Value::from(high));
}

fn lift_single_option(map: &mut Map<String, Value>) {
    let single = match map.get("oneOf") {
        Some(Value::Array(options)) if options.len() == 1 => options[0].as_object().cloned(),
        _ => None,
    };
    if let Some(only) = single {
        map.remove("oneOf");
        map.extend(only);
    }
}

fn drop_null(map: &mut Map<String, Value>) {
    if let Some(Value::Array(types)) = map.get("type") {
        let kept: Vec<Value> = types.iter().filter(|t| *t != "null").cloned().collect();
        if let [only] = kept.as_slice() {
            map.insert("type".into(), only.clone());
        }
    }
    let Some(Value::Array(options)) = map.get("anyOf") else {
        return;
    };
    let kept: Vec<Value> = options
        .iter()
        .filter(|option| option.get("type").and_then(Value::as_str) != Some("null"))
        .cloned()
        .collect();
    if let ([Value::Object(only)], true) = (kept.as_slice(), kept.len() < options.len()) {
        let only = only.clone();
        map.remove("anyOf");
        map.extend(only);
    }
}
