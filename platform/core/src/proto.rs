//! The step boundary on the binary wire, off by default.
//!
//! What this is: note 204's P3 reduced to the one question that decides it — what
//! the proto codec costs in the real core, rather than in a matched spike. The
//! JSON path is untouched and stays the default; with `--features proto` this
//! module adds a second entry point, `step_proto`, which reads a
//! `contract.v1.CoreIn` from bytes and writes a `contract.v1.CoreOut` back.
//!
//! What this is NOT: the swap. `serde_json` is still here, still reads and writes
//! every http body and every kept value, and still builds the view. Nothing in the
//! core is cheaper for turning this on; the module is strictly an addition, so the
//! number it produces is the cost of carrying both codecs, which is the number
//! note 204 projected at about 179,000 bytes and never measured in the core.
//!
//! Where the conversions live. Note 204 puts the `TryFrom` pair in
//! `platform/contract/src/convert.rs` (its P2). They are here instead because this
//! is a measurement behind an off-by-default feature and `platform/contract` is
//! another lead's file. If the swap happens they move; nothing in this file knows
//! where it lives.
//!
//! What this path cannot carry, stated once rather than discovered twice:
//!
//!   * `Core::open` still reads its `CoreConfig` as JSON. The schema has the
//!     message; the conversion is not written here, because the config carries a
//!     `CapabilityCard` and nothing in this measurement turns on it.
//!   * A word out of an open enum that this build does not know has no number on
//!     the wire, so a `CoreOut` carrying one is refused rather than quietly
//!     written as the zero value. The JSON path writes the word `unknown` and
//!     keeps going. The only values that can reach it are ones the core read from
//!     a server: `Hello.client_min[].class` and `Hello.start_deadline_ms[].class`.
//!   * Every http body and every kept value inside the core is still JSON. This
//!     path changes the step boundary and nothing else.
//!
//! The wire structs never become the core's domain types. Every value that has a
//! fallible constructor is built through it — `TitleKey::new`, `WireDate::parse` —
//! so the refusals the JSON path makes are the refusals this path makes.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use contract_proto::contract as p;
use contract_proto::prost::Message as _;

use contract::account::{Profile, ProfileList};
use contract::client::DeviceClass;
use contract::command::{Command, FeedbackEvent, SettingKey};
use contract::core_msg::{
    Action, BodyTarget, CoreIn, CoreInTag, CoreOut, DeviceSignInState, DeviceSignInView, Effect,
    EffectBody, EndReason, Event, EventBody, GateView, HttpClass, HttpMethod, LogLevel,
    NoAnswerReason, PlayState, PlayView, RouteTarget, ScreenBody, SecretOp, SessionState,
    SessionView, StoreOp, Target, TimerOp, View,
};
use contract::driver::{DriverCommand, DriverEvent, ErrorClass, Track};
use contract::error::Notice;
use contract::hello::{
    BrandValues, ClassValue, ClientFloor, Features, Hello, Limits, MovedAddress, Rollout,
};
use contract::play::{TimeWindow, WindowKind};
use contract::screen::ScreenName;
use contract::version::{VersionRange, WireDate};
use contract::{EpisodeRef, Rating, TitleKey, TitleKind};

use crate::{BadMessage, Step};

/// Step with contract message bytes in and contract message bytes out.
///
/// The same decisions as [`crate::step_json`], over the schema in
/// `platform/contract-proto` instead of over JSON.
///
/// # Errors
/// Bytes that are not a `contract.v1.CoreIn`, a message missing a field the
/// contract requires, or an out message carrying a word the schema has no number
/// for. State is unchanged in every case.
pub fn step_proto<S: Step>(stepper: &mut S, input: &[u8]) -> Result<Vec<u8>, BadMessage> {
    let wire = p::CoreIn::decode(input).map_err(|_| BadMessage)?;
    let read = core_in(wire)?;
    Ok(core_out(stepper.step(read))?.encode_to_vec())
}

/// The whole generated validator, run over the in message before it is read.
///
/// Not called by [`step_proto`]: the conversions below refuse every missing field
/// themselves, on the way past, and running both would state each presence rule
/// twice. It is here so the byte cost of the generated validator in the real core
/// can be measured rather than guessed — see note 238.
///
/// # Errors
/// Any rule the schema declares.
pub fn check_in(input: &[u8]) -> Result<(), BadMessage> {
    let wire = p::CoreIn::decode(input).map_err(|_| BadMessage)?;
    contract_proto::validate::core_in(&wire).map_err(|_| BadMessage)
}

fn need<T>(value: Option<T>) -> Result<T, BadMessage> {
    value.ok_or(BadMessage)
}

// ---------------------------------------------------------------------------
// Scalars with a fallible constructor.
// ---------------------------------------------------------------------------

/// `20261105` to `2026-11-05`, through the contract's own parser.
fn date_in(wire: Option<p::ContractDate>) -> Result<WireDate, BadMessage> {
    let ymd = need(need(wire)?.ymd)?;
    let text = format!(
        "{:04}-{:02}-{:02}",
        ymd / 10_000,
        ymd / 100 % 100,
        ymd % 100
    );
    WireDate::parse(&text).map_err(|_| BadMessage)
}

fn date_out(date: &WireDate) -> Result<p::ContractDate, BadMessage> {
    let digits: String = date.as_str().chars().filter(char::is_ascii_digit).collect();
    let ymd = digits.parse::<u64>().map_err(|_| BadMessage)?;
    Ok(p::ContractDate { ymd: Some(ymd) })
}

/// The id is `uint64` on the wire and `u32` in the domain, and the kind is closed:
/// both bounds are stated here a second time because this is the one place a wire
/// value becomes a domain value (contract-proto/README.md says the same).
fn title_in(wire: Option<p::TitleKey>) -> Result<TitleKey, BadMessage> {
    let wire = need(wire)?;
    let kind = match p::TitleKind::try_from(wire.kind) {
        Ok(p::TitleKind::Movie) => TitleKind::Movie,
        Ok(p::TitleKind::Tv) => TitleKind::Tv,
        _ => return Err(BadMessage),
    };
    let id = u32::try_from(wire.id).map_err(|_| BadMessage)?;
    TitleKey::new(kind, id).map_err(|_| BadMessage)
}

fn title_out(key: TitleKey) -> p::TitleKey {
    let kind = match key.kind() {
        TitleKind::Movie => p::TitleKind::Movie,
        TitleKind::Tv => p::TitleKind::Tv,
        _ => p::TitleKind::Unspecified,
    };
    p::TitleKey {
        kind: kind as i32,
        id: u64::from(key.id()),
    }
}

fn episode_in(wire: Option<p::EpisodeRef>) -> Result<EpisodeRef, BadMessage> {
    let wire = need(wire)?;
    Ok(EpisodeRef {
        season: need(wire.season)?,
        episode: need(wire.episode)?,
    })
}

fn episode_out(episode: &EpisodeRef) -> p::EpisodeRef {
    p::EpisodeRef {
        season: Some(episode.season),
        episode: Some(episode.episode),
    }
}

// ---------------------------------------------------------------------------
// Words. A number the schema does not declare reads as `Unknown`, which is the
// contract's forward compatibility rule (macros.rs:21-33) kept over numbers
// instead of over words. The zero value means the field never arrived, so it is a
// refusal and not an unknown word.
// ---------------------------------------------------------------------------

macro_rules! words_in {
    ($name:ident, $wire:ident, $domain:ident, $($variant:ident),+ $(,)?) => {
        fn $name(value: Option<i32>) -> Result<$domain, BadMessage> {
            Ok(match p::$wire::try_from(need(value)?) {
                $(Ok(p::$wire::$variant) => $domain::$variant,)+
                Ok(p::$wire::Unspecified) => return Err(BadMessage),
                Err(_) => $domain::Unknown,
            })
        }
    };
}

macro_rules! words_out {
    ($name:ident, $wire:ident, $domain:ident, $($variant:ident),+ $(,)?) => {
        fn $name(value: $domain) -> Result<i32, BadMessage> {
            Ok(match value {
                $($domain::$variant => p::$wire::$variant as i32,)+
                _ => return Err(BadMessage),
            })
        }
    };
}

words_in!(
    method_in, HttpMethod, HttpMethod, Get, Post, Put, Patch, Delete
);
words_in!(
    reason_in,
    NoAnswerReason,
    NoAnswerReason,
    Offline,
    Timeout,
    Tls,
    ClockWrong,
    Cancelled
);
words_in!(rating_in, Rating, Rating, Down, Up, Love);
words_in!(
    setting_in,
    SettingKey,
    SettingKey,
    AutoplayNext,
    AutoplayPreviews,
    AudioLanguage,
    SubtitleLanguage,
    SubtitleStyle,
    UiLanguage,
    Handle,
    Gender
);
words_in!(
    feedback_in,
    FeedbackEvent,
    FeedbackEvent,
    Click,
    Start,
    Complete,
    Like,
    Skip
);
words_in!(
    class_in, ErrorClass, ErrorClass, Network, Manifest, Segment, Decode, Key, Format, Refused
);

words_out!(
    method_out, HttpMethod, HttpMethod, Get, Post, Put, Patch, Delete
);
words_out!(
    http_class_out,
    HttpClass,
    HttpClass,
    Command,
    Cacheable,
    Plain
);
words_out!(to_out, BodyTarget, BodyTarget, Core, Screen);
words_out!(store_op_out, StoreOp, StoreOp, Get, Put, Delete, List);
words_out!(secret_op_out, SecretOp, SecretOp, Get, Put, Delete);
words_out!(timer_op_out, TimerOp, TimerOp, Set, Clear);
words_out!(level_out, LogLevel, LogLevel, Debug, Info, Warn, Error);
words_out!(
    session_state_out,
    SessionState,
    SessionState,
    Starting,
    SignedOut,
    SignedIn
);
words_out!(
    device_state_out,
    DeviceSignInState,
    DeviceSignInState,
    Idle,
    Waiting,
    Denied,
    Expired,
    Off
);
words_out!(
    ended_out, EndReason, EndReason, SignedOut, Everywhere, Refused, Elsewhere
);
words_out!(
    target_out, Target, Target, Start, SignIn, Profiles, Home, KidsHome, Screen, Title, Watch,
    NotFound, Wait, Upgrade, Activate
);
words_out!(
    screen_out,
    ScreenName,
    ScreenName,
    Home,
    Films,
    Series,
    Genre,
    New,
    MyList,
    Mine,
    Kids,
    Search,
    Notifications,
    Activity,
    Games,
    Clips,
    Language,
    Help,
    Settings
);
words_out!(
    play_state_out,
    PlayState,
    PlayState,
    Resolving,
    Starting,
    Playing,
    Paused,
    Stalled,
    Switching,
    Ended,
    Failed
);
words_out!(
    window_out, WindowKind, WindowKind, Recap, Intro, Credits, AgeCard, Cue
);
words_out!(
    device_class_out,
    DeviceClass,
    DeviceClass,
    Web,
    Phone,
    Tablet,
    Desktop,
    Tv,
    Box,
    Car
);

// ---------------------------------------------------------------------------
// In: wire to domain.
// ---------------------------------------------------------------------------

fn core_in(wire: p::CoreIn) -> Result<CoreIn, BadMessage> {
    Ok(CoreIn {
        contract: date_in(wire.contract)?,
        tag: CoreInTag::CoreIn,
        now_ms: need(wire.now_ms)?,
        mono_ms: need(wire.mono_ms)?,
        events: wire
            .events
            .into_iter()
            .map(event_in)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn event_in(wire: p::Event) -> Result<Event, BadMessage> {
    use p::event::Body;
    // A oneof arm this build does not know arrives as no arm at all, which is the
    // same answer the JSON path gives an unknown `type`: the core reports the id
    // in `unsupported` and changes nothing.
    let body = match wire.body {
        None => EventBody::Unknown,
        Some(Body::HttpDone(done)) => EventBody::HttpDone {
            status: need(done.status)?,
            headers: done.headers,
            body: done.body,
        },
        Some(Body::HttpFailed(failed)) => EventBody::HttpFailed {
            reason: reason_in(failed.reason)?,
        },
        Some(Body::StoreDone(done)) => EventBody::StoreDone {
            value: done.value,
            keys: done.keys,
        },
        Some(Body::SecretDone(done)) => EventBody::SecretDone { value: done.value },
        Some(Body::TimerFired(_)) => EventBody::TimerFired,
        Some(Body::Unsupported(_)) => EventBody::Unsupported,
        Some(Body::Cancelled(_)) => EventBody::Cancelled,
        Some(Body::Suspend(_)) => EventBody::Suspend,
        Some(Body::Resume(_)) => EventBody::Resume,
        Some(Body::Visibility(seen)) => EventBody::Visibility {
            visible: need(seen.visible)?,
        },
        Some(Body::Network(net)) => EventBody::Network {
            online: need(net.online)?,
        },
        Some(Body::Activity(_)) => EventBody::Activity,
        Some(Body::EndedElsewhere(_)) => EventBody::EndedElsewhere,
        Some(Body::Address(address)) => EventBody::Address {
            address: need(address.address)?,
        },
        Some(Body::Action(action)) => EventBody::Action {
            action: action_in(need(action.action)?)?,
        },
        Some(Body::Driver(driver)) => EventBody::Driver {
            generation: need(driver.generation)?,
            event: driver_event_in(need(driver.event)?)?,
        },
    };
    Ok(Event { id: wire.id, body })
}

fn action_in(wire: p::Action) -> Result<Action, BadMessage> {
    use p::action::Action as A;
    Ok(match wire.action {
        None => Action::Unknown,
        Some(A::SignIn(it)) => Action::SignIn {
            email: need(it.email)?,
            password: need(it.password)?,
        },
        Some(A::SendCode(it)) => Action::SendCode {
            email: need(it.email)?,
        },
        Some(A::SignInCode(it)) => Action::SignInCode {
            email: need(it.email)?,
            code: need(it.code)?,
        },
        Some(A::DeviceSignIn(it)) => Action::DeviceSignIn {
            device_name: need(it.device_name)?,
        },
        Some(A::DeviceSignInStop(_)) => Action::DeviceSignInStop,
        Some(A::SignOut(it)) => Action::SignOut {
            everywhere: need(it.everywhere)?,
        },
        Some(A::EnterProfile(it)) => Action::EnterProfile {
            profile: need(it.profile)?,
            pin: it.pin,
        },
        Some(A::Play(it)) => Action::Play {
            title: title_in(it.title)?,
            episode: it.episode.map(|e| episode_in(Some(e))).transpose()?,
        },
        Some(A::Pause(_)) => Action::Pause,
        Some(A::Unpause(_)) => Action::Unpause,
        Some(A::Seek(it)) => Action::Seek {
            to_ms: need(it.to_ms)?,
        },
        Some(A::SelectAudio(it)) => Action::SelectAudio {
            track: need(it.track)?,
        },
        Some(A::SelectText(it)) => Action::SelectText { track: it.track },
        Some(A::NextSubtitleFile(_)) => Action::NextSubtitleFile,
        Some(A::Stop(_)) => Action::Stop,
        Some(A::Command(it)) => Action::Command {
            command: command_in(need(it.command)?)?,
        },
        Some(A::Request(it)) => Action::Request {
            method: method_in(it.method)?,
            path: need(it.path)?,
            body: it.body,
        },
    })
}

fn command_in(wire: p::Command) -> Result<Command, BadMessage> {
    use p::command::Command as C;
    Ok(match wire.command {
        None => Command::Unknown,
        Some(C::List(it)) => Command::List {
            title: title_in(it.title)?,
            in_list: need(it.in_list)?,
        },
        Some(C::Rate(it)) => Command::Rate {
            title: title_in(it.title)?,
            rating: it.rating.map(|r| rating_in(Some(r))).transpose()?,
        },
        Some(C::Hide(it)) => Command::Hide {
            title: title_in(it.title)?,
        },
        Some(C::ClearHistory(_)) => Command::ClearHistory,
        Some(C::Position(it)) => Command::Position {
            title: title_in(it.title)?,
            episode: it.episode.map(|e| episode_in(Some(e))).transpose()?,
            position_s: need(it.position_s)?,
            duration_s: need(it.duration_s)?,
        },
        Some(C::Setting(it)) => Command::Setting {
            key: setting_in(it.key)?,
            value: need(it.value)?,
        },
        Some(C::Feedback(it)) => Command::Feedback {
            title: title_in(it.title)?,
            event: feedback_in(it.event)?,
        },
        Some(C::TrailerRefused(it)) => Command::TrailerRefused {
            title: title_in(it.title)?,
        },
        Some(C::Remind(it)) => Command::Remind {
            title: title_in(it.title)?,
            on: need(it.on)?,
        },
    })
}

fn driver_event_in(wire: p::DriverEvent) -> Result<DriverEvent, BadMessage> {
    use p::driver_event::Kind;
    Ok(match wire.kind {
        None => DriverEvent::Unknown,
        Some(Kind::FirstFrame(_)) => DriverEvent::FirstFrame,
        Some(Kind::Position(it)) => DriverEvent::Position {
            time_ms: need(it.time_ms)?,
            duration_ms: it.duration_ms,
            video_width: it.video_width,
            has_picture: it.has_picture,
        },
        Some(Kind::Stalled(_)) => DriverEvent::Stalled,
        Some(Kind::Ended(_)) => DriverEvent::Ended,
        Some(Kind::Tracks(it)) => DriverEvent::Tracks {
            audio: it
                .audio
                .into_iter()
                .map(track_in)
                .collect::<Result<Vec<_>, _>>()?,
            text: it
                .text
                .into_iter()
                .map(track_in)
                .collect::<Result<Vec<_>, _>>()?,
        },
        Some(Kind::Error(it)) => DriverEvent::Error {
            class: class_in(it.class)?,
        },
    })
}

fn track_in(wire: p::Track) -> Result<Track, BadMessage> {
    Ok(Track {
        id: need(wire.id)?,
        lang: wire.lang,
        label: wire.label,
        selected: wire.selected,
    })
}

// ---------------------------------------------------------------------------
// Out: domain to wire.
// ---------------------------------------------------------------------------

fn core_out(out: CoreOut) -> Result<p::CoreOut, BadMessage> {
    let CoreOut {
        contract,
        effects,
        view,
        next_wake_ms,
        unsupported,
        ..
    } = out;
    Ok(p::CoreOut {
        contract: Some(date_out(&contract)?),
        effects: effects
            .into_iter()
            .map(effect_out)
            .collect::<Result<Vec<_>, _>>()?,
        view: view.map(view_out).transpose()?,
        next_wake_ms,
        unsupported,
    })
}

fn effect_out(effect: Effect) -> Result<p::Effect, BadMessage> {
    use p::effect::{Body, Http, Log, Open, Player, Secret, Store, Timer, ViewEffect};
    let body = match effect.body {
        EffectBody::Http {
            method,
            path,
            headers,
            body,
            deadline_ms,
            class,
            to,
            scope,
        } => Body::Http(Http {
            method: Some(method_out(method)?),
            path: Some(path),
            headers,
            body,
            deadline_ms: Some(deadline_ms),
            class: Some(http_class_out(class)?),
            to: Some(to_out(to)?),
            scope,
        }),
        EffectBody::Store { op, key, value } => Body::Store(Store {
            op: Some(store_op_out(op)?),
            key,
            value,
        }),
        EffectBody::Secret { op, name, value } => Body::Secret(Secret {
            op: Some(secret_op_out(op)?),
            name: Some(name),
            value,
        }),
        EffectBody::Timer { op, after_ms } => Body::Timer(Timer {
            op: Some(timer_op_out(op)?),
            after_ms,
        }),
        EffectBody::Player { command } => Body::Player(Player {
            command: Some(driver_command_out(command)?),
        }),
        EffectBody::View { notice, body } => Body::View(ViewEffect {
            notice: notice.map(notice_out),
            body: body.map(screen_body_out),
        }),
        EffectBody::Open { target } => Body::Open(Open {
            target: Some(route_out(target)?),
        }),
        EffectBody::Log {
            level,
            code,
            detail,
        } => Body::Log(Log {
            level: Some(level_out(level)?),
            code: Some(code),
            detail,
        }),
        // `Unknown` is what an unreadable `kind` becomes on the way in. The core
        // never builds one, and the schema has no arm for it.
        _ => return Err(BadMessage),
    };
    Ok(p::Effect {
        id: Some(effect.id),
        body: Some(body),
    })
}

fn driver_command_out(command: DriverCommand) -> Result<p::DriverCommand, BadMessage> {
    use p::driver_command::{
        Close, Op, Open, Pause, Play, Rate, Seek, SelectAudio, SelectText, Volume,
    };
    let op = match command {
        DriverCommand::Open {
            url,
            headers,
            start_at_ms,
            generation,
            max_height,
            max_bitrate,
        } => Op::Open(Open {
            url: Some(url),
            headers: Some(p::Headers { values: headers }),
            start_at_ms: Some(start_at_ms),
            generation: Some(generation),
            max_height,
            max_bitrate,
        }),
        DriverCommand::Play => Op::Play(Play {}),
        DriverCommand::Pause => Op::Pause(Pause {}),
        DriverCommand::Seek { to_ms } => Op::Seek(Seek { to_ms: Some(to_ms) }),
        DriverCommand::Rate { percent } => Op::Rate(Rate {
            percent: Some(percent),
        }),
        DriverCommand::Volume { percent } => Op::Volume(Volume {
            percent: Some(percent),
        }),
        DriverCommand::SelectAudio { track } => Op::SelectAudio(SelectAudio { track: Some(track) }),
        DriverCommand::SelectText { track } => Op::SelectText(SelectText { track }),
        DriverCommand::Close => Op::Close(Close {}),
        _ => return Err(BadMessage),
    };
    Ok(p::DriverCommand { op: Some(op) })
}

fn notice_out(notice: Notice) -> p::Notice {
    p::Notice {
        label: Some(notice.label),
        text: Some(notice.text),
        action_label: notice.action_label,
        action_address: notice.action_address,
    }
}

fn screen_body_out(body: ScreenBody) -> p::ScreenBody {
    p::ScreenBody {
        request: Some(body.request),
        body: Some(body.body),
        stale: body.stale,
    }
}

fn route_out(route: RouteTarget) -> Result<p::RouteTarget, BadMessage> {
    Ok(p::RouteTarget {
        target: Some(target_out(route.target)?),
        screen: route.screen.map(screen_out).transpose()?,
        genre: route.genre,
        title: route.title.map(title_out),
        episode: route.episode.as_ref().map(episode_out),
        next: route.next,
        code: route.code,
    })
}

fn view_out(view: View) -> Result<p::View, BadMessage> {
    Ok(p::View {
        session: view.session.map(session_out).transpose()?,
        device_sign_in: view.device_sign_in.map(device_out).transpose()?,
        route: view.route.map(route_out).transpose()?,
        gate: view
            .gate
            .map(|GateView { ask }| p::GateView { ask: Some(ask) }),
        play: view.play.map(play_out).transpose()?,
        profiles: view.profiles.map(profiles_out),
        hello: view.hello.map(hello_out).transpose()?,
    })
}

fn session_out(session: SessionView) -> Result<p::SessionView, BadMessage> {
    Ok(p::SessionView {
        state: Some(session_state_out(session.state)?),
        user_id: session.user_id,
        email: session.email,
        expires_at_ms: session.expires_at_ms,
        profile: session.profile,
        scope: session.scope,
        ended: session.ended.map(ended_out).transpose()?,
    })
}

fn device_out(device: DeviceSignInView) -> Result<p::DeviceSignInView, BadMessage> {
    Ok(p::DeviceSignInView {
        state: Some(device_state_out(device.state)?),
        user_code: device.user_code,
        verification_uri: device.verification_uri,
        verification_uri_complete: device.verification_uri_complete,
        expires_in: device.expires_in,
    })
}

fn play_out(play: PlayView) -> Result<p::PlayView, BadMessage> {
    Ok(p::PlayView {
        state: Some(play_state_out(play.state)?),
        title: Some(title_out(play.title)),
        episode: play.episode.as_ref().map(episode_out),
        attempt: play.attempt,
        windows: play
            .windows
            .into_iter()
            .map(window_kind_out)
            .collect::<Result<Vec<_>, _>>()?,
        next_episode: play.next_episode.as_ref().map(episode_out),
        still_asking: play.still_asking,
    })
}

fn window_kind_out(window: TimeWindow) -> Result<p::TimeWindow, BadMessage> {
    Ok(p::TimeWindow {
        kind: Some(window_out(window.kind)?),
        start_ms: Some(window.start_ms),
        end_ms: Some(window.end_ms),
        seek_to_ms: window.seek_to_ms,
        text: window.text,
    })
}

fn profiles_out(list: ProfileList) -> p::ProfileList {
    p::ProfileList {
        profiles: list.profiles.into_iter().map(profile_out).collect(),
    }
}

fn profile_out(profile: Profile) -> p::Profile {
    p::Profile {
        id: Some(profile.id),
        name: Some(profile.name),
        kids: Some(profile.kids),
        main: Some(profile.main),
        has_pin: Some(profile.has_pin),
        avatar: profile.avatar,
        language: profile.language,
    }
}

fn hello_out(hello: Hello) -> Result<p::Hello, BadMessage> {
    let Hello {
        contract,
        client_min,
        features,
        limits,
        playback,
        killed_builds,
        rollouts,
        moved,
        start_deadline_ms,
        brand,
        ttl_s,
    } = hello;
    Ok(p::Hello {
        contract: Some(range_out(&contract)?),
        client_min: client_min
            .into_iter()
            .map(floor_out)
            .collect::<Result<Vec<_>, _>>()?,
        features: Some(features_out(features)),
        limits: Some(limits_out(&limits)),
        playback: Some(playback),
        killed_builds,
        rollouts: rollouts
            .into_iter()
            .map(|Rollout { name, percent }| p::Rollout {
                name: Some(name),
                percent: Some(percent),
            })
            .collect(),
        moved: moved
            .into_iter()
            .map(|MovedAddress { from, to }| p::MovedAddress {
                from: Some(from),
                to: Some(to),
            })
            .collect(),
        start_deadline_ms: start_deadline_ms
            .into_iter()
            .map(|ClassValue { class, value }| {
                Ok(p::ClassValue {
                    class: Some(device_class_out(class)?),
                    value: Some(value),
                })
            })
            .collect::<Result<Vec<_>, BadMessage>>()?,
        brand: Some(brand_out(brand)),
        ttl_s: Some(ttl_s),
    })
}

fn range_out(range: &VersionRange) -> Result<p::VersionRange, BadMessage> {
    Ok(p::VersionRange {
        min: Some(date_out(&range.min)?),
        max: Some(date_out(&range.max)?),
    })
}

fn floor_out(floor: ClientFloor) -> Result<p::ClientFloor, BadMessage> {
    Ok(p::ClientFloor {
        class: Some(device_class_out(floor.class)?),
        version: Some(floor.version),
    })
}

fn features_out(features: Features) -> p::Features {
    p::Features {
        accounts_required: Some(features.accounts_required),
        games: features.games,
        clips: features.clips,
        original_audio: features.original_audio,
        visitor_tier: features.visitor_tier,
    }
}

fn limits_out(limits: &Limits) -> p::Limits {
    p::Limits {
        profiles_max: Some(limits.profiles_max),
        profile_name_max: Some(limits.profile_name_max),
        pin_digits: Some(limits.pin_digits),
        password_min: Some(limits.password_min),
        step_up_s: Some(limits.step_up_s),
        phone_digits_min: Some(limits.phone_digits_min),
        phone_digits_max: Some(limits.phone_digits_max),
        blocked_title_max: Some(limits.blocked_title_max),
        blocked_titles_max: Some(limits.blocked_titles_max),
        name_max: Some(limits.name_max),
        handle_min: Some(limits.handle_min),
        handle_max: Some(limits.handle_max),
        code_digits: Some(limits.code_digits),
        code_valid_s: Some(limits.code_valid_s),
        command_batch_max: Some(limits.command_batch_max),
    }
}

fn brand_out(brand: BrandValues) -> p::BrandValues {
    p::BrandValues {
        name: Some(brand.name),
        support_email: Some(brand.support_email),
    }
}

/// Write an out message as `contract.v1.CoreOut` bytes, for a test or a host that
/// holds both paths.
///
/// # Errors
/// An out message carrying a word the schema has no number for.
pub fn out_to_bytes(out: CoreOut) -> Result<Vec<u8>, BadMessage> {
    Ok(core_out(out)?.encode_to_vec())
}

/// Read back a `contract.v1.CoreOut`, for a test or a host that holds both paths.
///
/// # Errors
/// Bytes that are not a `contract.v1.CoreOut`.
pub fn out_from_bytes(bytes: &[u8]) -> Result<p::CoreOut, BadMessage> {
    p::CoreOut::decode(bytes).map_err(|_| BadMessage)
}

/// Write a `contract.v1.CoreIn` for a test or a host that holds both paths.
#[must_use]
pub fn in_to_bytes(contract: &str, now_ms: u64, mono_ms: u64, events: Vec<p::Event>) -> Vec<u8> {
    let digits: String = contract.chars().filter(char::is_ascii_digit).collect();
    p::CoreIn {
        contract: Some(p::ContractDate {
            ymd: digits.parse::<u64>().ok(),
        }),
        now_ms: Some(now_ms),
        mono_ms: Some(mono_ms),
        events,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_date_goes_out_and_comes_back() {
        let date = WireDate::parse("2026-11-05").unwrap();
        let wire = date_out(&date).unwrap();
        assert_eq!(wire.ymd, Some(20_261_105));
        assert_eq!(date_in(Some(wire)).unwrap(), date);
    }

    #[test]
    fn a_title_id_that_does_not_fit_a_u32_is_refused() {
        let wire = p::TitleKey {
            kind: p::TitleKind::Tv as i32,
            id: 9_999_999_999,
        };
        assert_eq!(title_in(Some(wire)), Err(BadMessage));
    }

    #[test]
    fn a_kind_the_schema_does_not_declare_is_refused() {
        for kind in [0, 7] {
            let wire = p::TitleKey { kind, id: 1 };
            assert_eq!(title_in(Some(wire)), Err(BadMessage));
        }
    }

    #[test]
    fn a_number_no_word_is_declared_for_reads_as_unknown() {
        assert_eq!(reason_in(Some(99)).unwrap(), NoAnswerReason::Unknown);
        assert_eq!(reason_in(Some(0)), Err(BadMessage));
        assert_eq!(reason_in(None), Err(BadMessage));
    }

    #[test]
    fn a_word_with_no_number_cannot_go_out() {
        assert_eq!(device_class_out(DeviceClass::Unknown), Err(BadMessage));
        assert_eq!(device_class_out(DeviceClass::Tv).unwrap(), 5);
    }
}
