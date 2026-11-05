//! Messages in and out of the core. UTF-8 JSON with a contract date and a type tag.
//!
//! In: `{now_ms, mono_ms, events}`. Out: `{effects, view, next_wake_ms}`. The host performs each
//! effect and returns its result as a later event with the same id.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::account::ProfileList;
use crate::client::{CapabilityCard, CoreMode, DeviceClass, Form};
use crate::command::Command;
use crate::driver::{DriverCommand, DriverEvent};
use crate::error::Notice;
use crate::play::TimeWindow;
use crate::screen::ScreenName;
use crate::title::{EpisodeRef, TitleKey};
use crate::version::WireDate;

macro_rules! message_tag {
    ($(#[$meta:meta])* $name:ident, $word:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            #[default]
            $word,
        }
    };
}

message_tag!(
    /// Type tag of `CoreConfig`.
    CoreConfigTag,
    CoreConfig
);
message_tag!(
    /// Type tag of `CoreIn`.
    CoreInTag,
    CoreIn
);
message_tag!(
    /// Type tag of `CoreOut`.
    CoreOutTag,
    CoreOut
);

words! {
    /// Why an http effect got no answer. Any status at all is an answer.
    pub enum NoAnswerReason {
        Offline,
        Timeout,
        Tls,
        ClockWrong,
        Cancelled,
    }
}

words! {
    /// Methods of an http effect.
    pub enum HttpMethod as "UPPERCASE" {
        Get,
        Post,
        Put,
        Patch,
        Delete,
    }
}

words! {
    /// How an http effect may be handled.
    pub enum HttpClass {
        /// Must reach the server. The offline layer keeps it when one is present.
        Command,
        /// May be answered from a kept copy.
        Cacheable,
        /// The host performs it as it arrives.
        Plain,
    }
}

words! {
    /// Where the body of an answer goes. An answer outside 200 to 299 always comes to the core,
    /// because the core reads its error code.
    pub enum BodyTarget {
        Core,
        Screen,
    }
}

words! {
    /// Store operations. Only the offline layer uses store.
    pub enum StoreOp {
        Get,
        Put,
        Delete,
        List,
    }
}

words! {
    /// Secret operations.
    pub enum SecretOp {
        Get,
        Put,
        Delete,
    }
}

words! {
    /// Timer operations. A clear uses the id of the timer it clears.
    pub enum TimerOp {
        Set,
        Clear,
    }
}

words! {
    /// Log levels.
    pub enum LogLevel {
        Debug,
        Info,
        Warn,
        Error,
    }
}

words! {
    /// Where sign in stands.
    pub enum SessionState {
        Starting,
        SignedOut,
        SignedIn,
    }
}

words! {
    /// Why a session ended.
    pub enum EndReason {
        SignedOut,
        Everywhere,
        Refused,
        Elsewhere,
    }
}

words! {
    /// A semantic target. Each device maps it to one of its own screens.
    pub enum Target {
        Start,
        SignIn,
        Profiles,
        Home,
        KidsHome,
        Screen,
        Title,
        Watch,
        NotFound,
        Wait,
        Upgrade,
    }
}

words! {
    /// Where the play session stands.
    pub enum PlayState {
        Resolving,
        Starting,
        Playing,
        Paused,
        Stalled,
        Switching,
        Ended,
        Failed,
    }
}

wire! {
    /// Given to `core_open`, once. The seed is the only random input the core ever gets.
    pub struct CoreConfig {
        pub contract: WireDate,
        #[serde(rename = "type")]
        pub tag: CoreConfigTag,
        pub class: DeviceClass,
        pub form: Form,
        pub mode: CoreMode,
        pub app: String,
        pub version: String,
        pub os: String,
        /// Hex digits, at least 32.
        pub seed: String,
        /// What this device can do. The core sends its hash after sign in.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub card: Option<CapabilityCard>,
    }

    /// What a screen asks for.
    #[serde(tag = "action", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum Action {
        SignIn { email: String, password: String },
        SendCode { email: String },
        SignInCode { email: String, code: String },
        SignOut { everywhere: bool },
        EnterProfile {
            profile: String,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pin: Option<String>,
        },
        Play {
            title: TitleKey,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            episode: Option<EpisodeRef>,
        },
        Pause,
        Unpause,
        Seek { to_ms: u64 },
        SelectAudio { track: String },
        SelectText {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            track: Option<String>,
        },
        NextSubtitleFile,
        Stop,
        Command { command: Command },
        /// Send a prepared request with this session's token. The http effect takes the id of
        /// this event and its class is always plain, so whoever prepared it gets the answer
        /// under that id. Tokens stay in the core.
        Request {
            method: HttpMethod,
            path: String,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            body: Option<String>,
        },
        #[serde(other)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        Unknown,
    }

    /// Body of an event, tagged by `type`.
    #[serde(tag = "type", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum EventBody {
        /// An http effect got an answer. Every status is data.
        HttpDone {
            status: u32,
            #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
            headers: BTreeMap<String, String>,
            /// Absent when the body went to the screen.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            body: Option<String>,
        },
        /// An http effect got no answer at all.
        HttpFailed { reason: NoAnswerReason },
        StoreDone {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            value: Option<String>,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            keys: Vec<String>,
        },
        SecretDone {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            value: Option<String>,
        },
        TimerFired,
        /// The host cannot perform this effect.
        Unsupported,
        /// The effect with this id was dropped.
        Cancelled,
        Suspend,
        Resume,
        Visibility { visible: bool },
        Network { online: bool },
        /// Touch, key, wheel or playing video in view.
        Activity,
        /// An address or deep link to open.
        Address { address: String },
        Action { action: Action },
        /// A driver event, stamped with the generation of the open that made it.
        Driver { generation: u32, event: DriverEvent },
        #[serde(other)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        Unknown,
    }

    /// One event. `id` is the id of the effect it answers, when it answers one.
    pub struct Event {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        #[serde(flatten)]
        pub body: EventBody,
    }

    /// One step of input. Every input carries wall time and monotonic time.
    pub struct CoreIn {
        pub contract: WireDate,
        #[serde(rename = "type")]
        pub tag: CoreInTag,
        /// Wall time, ms since 1970.
        pub now_ms: u64,
        /// Monotonic time, ms. Restarts at boot.
        pub mono_ms: u64,
        pub events: Vec<Event>,
    }

    /// Body of an effect, tagged by `kind`. Exactly eight kinds. Any may come back unsupported.
    #[serde(tag = "kind", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum EffectBody {
        /// A path, never an address. The host holds the base address.
        Http {
            method: HttpMethod,
            path: String,
            #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
            headers: BTreeMap<String, String>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            body: Option<String>,
            deadline_ms: u32,
            class: HttpClass,
            to: BodyTarget,
            /// Opaque scope of an account and profile, for kept copies.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            scope: Option<String>,
        },
        Store {
            op: StoreOp,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            key: Option<String>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            value: Option<String>,
        },
        Secret {
            op: SecretOp,
            name: String,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            value: Option<String>,
        },
        Timer {
            op: TimerOp,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            after_ms: Option<u64>,
        },
        Player { command: DriverCommand },
        /// A one time notice or a kept page. Lasting state travels in `view`.
        View {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            notice: Option<Notice>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            body: Option<ScreenBody>,
        },
        Open { target: RouteTarget },
        Log {
            level: LogLevel,
            code: String,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            detail: Option<String>,
        },
        #[serde(other)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        Unknown,
    }

    /// A page body handed to a screen. It stands in for the answer to an http effect whose
    /// body was going to the screen.
    pub struct ScreenBody {
        /// Id of the http effect this answers.
        pub request: String,
        pub body: String,
        /// True when this copy is past its ttl and was kept for when the network is gone.
        #[serde(default)]
        pub stale: bool,
    }

    /// One effect. The result comes back as an event with this id.
    pub struct Effect {
        pub id: String,
        #[serde(flatten)]
        pub body: EffectBody,
    }

    /// What an address opens.
    pub struct RouteTarget {
        pub target: Target,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub screen: Option<ScreenName>,
        /// The genre, for the genre screen.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub genre: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub title: Option<TitleKey>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub episode: Option<EpisodeRef>,
        /// A safe address to return to after sign in.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub next: Option<String>,
    }

    /// Sign in state for screens. Tokens never appear here.
    pub struct SessionView {
        pub state: SessionState,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub user_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub email: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub expires_at_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub profile: Option<String>,
        /// Opaque scope of this account and profile, the same value http effects carry.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub scope: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub ended: Option<EndReason>,
    }

    /// True while the who is watching prompt should show.
    pub struct GateView {
        pub ask: bool,
    }

    /// Play session state for screens.
    pub struct PlayView {
        pub state: PlayState,
        pub title: TitleKey,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub episode: Option<EpisodeRef>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub attempt: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub windows: Vec<TimeWindow>,
    }

    /// Changed screen state. A part that is absent did not change.
    #[derive(Default)]
    pub struct View {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub session: Option<SessionView>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub route: Option<RouteTarget>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub gate: Option<GateView>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub play: Option<PlayView>,
        /// The account's profiles, for the picker. No PIN is ever held here.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub profiles: Option<ProfileList>,
    }

    /// One step of output.
    pub struct CoreOut {
        pub contract: WireDate,
        #[serde(rename = "type")]
        pub tag: CoreOutTag,
        pub effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub view: Option<View>,
        /// Monotonic time at which the host steps the core again. Absent means no wake.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub next_wake_ms: Option<u64>,
        /// Ids of in events of a type this core does not know.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub unsupported: Vec<String>,
    }

    /// `POST /v1/core/step`. `config` is sent on the first step of a hosted session.
    pub struct HostedStep {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub config: Option<CoreConfig>,
        pub input: CoreIn,
    }
}
