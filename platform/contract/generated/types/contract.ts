// Generated from the contract. Do not edit.

export const WIRE_VERSION = "2026-11-05";
export const ABI_VERSION = 1;

export interface Account {
  email: string;
  first_name: string;
  id: string;
  last_name: string;
  phone?: string;
}

/** Everything held about an account, in plain JSON. */
export interface AccountExport {
  account: Account;
  devices: DeviceSession[];
  profiles: Profile[];
}

export interface AccountUpdate {
  first_name?: string;
  last_name?: string;
}

/** What a screen asks for. */
export type Action =
  | {
      action: "sign_in";
      email: string;
      password: string;
    }
  | {
      action: "send_code";
      email: string;
    }
  | {
      action: "sign_in_code";
      code: string;
      email: string;
    }
  | {
      /** Start device sign in. The core asks for a code, polls and renews it, and the screen draws `View.device_sign_in`. While a code is live this changes nothing. */
      action: "device_sign_in";
      device_name: string;
    }
  | {
      action: "device_sign_in_stop";
    }
  | {
      action: "sign_out";
      everywhere: boolean;
    }
  | {
      action: "enter_profile";
      pin?: string;
      profile: string;
    }
  | {
      action: "play";
      episode?: EpisodeRef;
      title: TitleKey;
    }
  | {
      action: "pause";
    }
  | {
      action: "unpause";
    }
  | {
      action: "seek";
      to_ms: number;
    }
  | {
      action: "select_audio";
      track: string;
    }
  | {
      action: "select_text";
      track?: string;
    }
  | {
      action: "next_subtitle_file";
    }
  | {
      action: "stop";
    }
  | {
      action: "command";
      command: Command;
    }
  | {
      /** Send a prepared request with this session's token. The http effect takes the id of this event and its class is always plain, so whoever prepared it gets the answer under that id. Tokens stay in the core. */
      action: "request";
      body?: string;
      method: HttpMethod;
      path: string;
    }
  ;

/** Age card shown once at the start of play. */
export interface AgeCard {
  label: string;
  reasons?: string[];
}

/** An age mark, as this viewer reads it. Each screen picks its shape. */
export interface AgeMark {
  label: string;
  tone: AgeTone;
}

/** The colour family of an age mark. The server picks it and each app paints it. */
export type AgeTone = "green" | "yellow" | "orange" | "pink" | "red" | "dark_red" | "blue" | "grey" | (string & {});

/** One way to play. Attempts arrive in the order to try, ranked once on the server. `headers` is required, even when empty, and no provider name is carried. */
export interface Attempt {
  audio?: string[];
  codecs?: string[];
  container?: string;
  /** Wall time, ms since 1970, after which the address stops working. */
  expires_at_ms?: number;
  /** Opaque group. After a failure the core skips other attempts of this group. */
  group: string;
  /** Headers the player must send, such as Referer, Origin, User-Agent. */
  headers: Record<string, string>;
  height?: number;
  id: string;
  kind: AttemptKind;
  url: string;
  /** True when the finder fetched and read this address. */
  verified: boolean;
}

/** How an attempt is played. */
export type AttemptKind = "hls" | "file" | "embed" | (string & {});

/** One picture a profile can wear. */
export interface AvatarPicture {
  name: string;
  /** Where the picture sits on this server. */
  path: string;
}

/** A row of pictures on one theme. `kids` says a Kids profile may wear them. */
export interface AvatarSet {
  kids: boolean;
  name: string;
  pictures: AvatarPicture[];
}

export interface AvatarSets {
  sets: AvatarSet[];
}

/** A mark on a tile. */
export interface Badge {
  kind: BadgeKind;
  rank?: number;
}

/** Marks on a tile. */
export type BadgeKind = "ranked" | "new" | (string & {});

/** Where the body of an answer goes. An answer outside 200 to 299 always comes to the core, because the core reads its error code. */
export type BodyTarget = "core" | "screen" | (string & {});

/** Name and contact, filled into text at render time. */
export interface BrandValues {
  name: string;
  support_email: string;
}

/** What a device can do. The server filters attempts and sections by this card. */
export interface CapabilityCard {
  /** True when the player switches variants during play. */
  adaptive: boolean;
  app: string;
  /** Attempt kinds this player opens. An embed appears only when listed here. */
  attempt_kinds: AttemptKind[];
  class: DeviceClass;
  /** Codecs this player opens, such as avc1, hvc1, mp4a. */
  codecs: string[];
  /** Containers this player opens, such as ts, fmp4, mp4. */
  containers: string[];
  form: Form;
  input: InputKind[];
  languages: string[];
  max_height: number;
  /** Request headers the player can set on media, such as Referer. */
  media_headers: string[];
  os: string;
  os_version: string;
  /** True when the driver reports video width or has_picture. */
  reports_picture: boolean;
  screen_height: number;
  screen_width: number;
  /** True when the device has a durable secret store. */
  secret_store: boolean;
  /** Section kinds this build draws. Others fall back to a plain row. */
  section_kinds: SectionKind[];
  /** True when the player can load a side subtitle file. */
  side_subtitles: boolean;
  /** Bytes this device gives to kept commands and screens. 0 means none. */
  storage_bytes: number;
  version: string;
}

/** One change a device reads. */
export type Change =
  | {
      change: "title";
      list: ListState;
      rating: RatingState;
      title: TitleKey;
    }
  | {
      change: "progress";
      state: ProgressState;
    }
  | {
      /** Hidden as a whole, every episode of a series included. */
      change: "hidden";
      title: TitleKey;
    }
  | {
      change: "history_cleared";
      changed_at_ms: number;
    }
  | {
      change: "setting";
      key: SettingKey;
      value: string;
    }
  ;

/** `GET /v1/changes`. */
export interface Changes {
  changes: Change[];
  cursor: Cursor;
  has_more: boolean;
}

/** One value per client class. */
export interface ClassValue {
  class: DeviceClass;
  value: number;
}

/** Lowest app version a class may run. */
export interface ClientFloor {
  class: DeviceClass;
  version: string;
}

export interface CodeSend {
  email: string;
}

/** A code was sent, or would have been. The answer never says whether the account exists. */
export interface CodeSent {
  expires_in: number;
}

export interface CodeSignIn {
  code: string;
  email: string;
}

/** One change to a profile's facts. */
export type Command =
  | {
      /** Add to or remove from the saved list. Leaves the rating alone. */
      command: "list";
      in_list: boolean;
      title: TitleKey;
    }
  | {
      /** Set or clear a rating. Leaves the list entry alone. */
      command: "rate";
      rating?: Rating;
      title: TitleKey;
    }
  | {
      /** Hide a title from history. For a series this covers every episode. */
      command: "hide";
      title: TitleKey;
    }
  | {
      /** Clear all history of this profile, as one operation. */
      command: "clear_history";
    }
  | {
      /** Save a position, in whole seconds. */
      command: "position";
      duration_s: number;
      episode?: EpisodeRef;
      position_s: number;
      title: TitleKey;
    }
  | {
      /** Change one profile setting. Booleans as true or false, style as JSON text. */
      command: "setting";
      key: SettingKey;
      value: string;
    }
  | {
      /** A viewer signal. Discarded by the server when consent is off. */
      command: "feedback";
      event: FeedbackEvent;
      title: TitleKey;
    }
  | {
      /** A trailer failed to play for this title. */
      command: "trailer_refused";
      title: TitleKey;
    }
  ;

/** Answer for a batch, one entry per op. */
export interface CommandAnswer {
  answers: OpAnswer[];
}

/** `POST /v1/commands`. Idempotent by op id. */
export interface CommandBatch {
  ops: CommandOp[];
}

/** One queued op. A missing `age_ms` means age unknown, and the server orders by arrival. */
export type CommandOp =
  | {
      /** Add to or remove from the saved list. Leaves the rating alone. */
      command: "list";
      age_ms?: number;
      op_id: string;
      in_list: boolean;
      title: TitleKey;
    }
  | {
      /** Set or clear a rating. Leaves the list entry alone. */
      command: "rate";
      age_ms?: number;
      op_id: string;
      rating?: Rating;
      title: TitleKey;
    }
  | {
      /** Hide a title from history. For a series this covers every episode. */
      command: "hide";
      age_ms?: number;
      op_id: string;
      title: TitleKey;
    }
  | {
      /** Clear all history of this profile, as one operation. */
      command: "clear_history";
      age_ms?: number;
      op_id: string;
    }
  | {
      /** Save a position, in whole seconds. */
      command: "position";
      age_ms?: number;
      op_id: string;
      duration_s: number;
      episode?: EpisodeRef;
      position_s: number;
      title: TitleKey;
    }
  | {
      /** Change one profile setting. Booleans as true or false, style as JSON text. */
      command: "setting";
      age_ms?: number;
      op_id: string;
      key: SettingKey;
      value: string;
    }
  | {
      /** A viewer signal. Discarded by the server when consent is off. */
      command: "feedback";
      age_ms?: number;
      op_id: string;
      event: FeedbackEvent;
      title: TitleKey;
    }
  | {
      /** A trailer failed to play for this title. */
      command: "trailer_refused";
      age_ms?: number;
      op_id: string;
      title: TitleKey;
    }
  ;

/** Step up. Send a password or an emailed code. */
export interface Confirm {
  code?: string;
  password?: string;
}

/** A step up holds for this many seconds and a refresh never renews it. */
export interface Confirmed {
  valid_s: number;
}

/** Consent for personal rows. */
export interface Consent {
  personal: boolean;
}

/** Given to `core_open`, once. The seed is the only random input the core ever gets. */
export interface CoreConfig {
  app: string;
  /** What this device can do. The core sends its hash after sign in. */
  card?: CapabilityCard;
  class: DeviceClass;
  contract: WireDate;
  form: Form;
  mode: CoreMode;
  os: string;
  /** Hex digits, at least 32. */
  seed: string;
  type: CoreConfigTag;
  version: string;
}

/** Type tag of `CoreConfig`. */
export type CoreConfigTag = "core_config";

/** One step of input. Every input carries wall time and monotonic time. */
export interface CoreIn {
  contract: WireDate;
  events: Event[];
  /** Monotonic time, ms. Restarts at boot. */
  mono_ms: number;
  /** Wall time, ms since 1970. */
  now_ms: number;
  type: CoreInTag;
}

/** Type tag of `CoreIn`. */
export type CoreInTag = "core_in";

/** Where the core runs for this device. */
export type CoreMode = "local" | "hosted" | (string & {});

/** One step of output. */
export interface CoreOut {
  contract: WireDate;
  effects: Effect[];
  /** Monotonic time at which the host steps the core again. Absent means no wake. */
  next_wake_ms?: number;
  type: CoreOutTag;
  /** Ids of in events of a type this core does not know. */
  unsupported?: string[];
  view?: View;
}

/** Type tag of `CoreOut`. */
export type CoreOutTag = "core_out";

/** One cue. */
export interface Cue {
  end_ms: number;
  start_ms: number;
  text: string;
}

/** An opaque position in a list. Clients hand it back and never read it. */
export type Cursor = string;

/** True when the server wants the whole card. */
export interface DeviceCardAnswer {
  send_card: boolean;
}

/** Sent at each start. The card travels only when the server asks or the app or OS changed. */
export interface DeviceCardRequest {
  card?: CapabilityCard;
  card_hash: string;
  device_key: string;
}

/** Client class. Minimum versions, start deadlines and reachability are kept per class. */
export type DeviceClass = "web" | "phone" | "tablet" | "desktop" | "tv" | "box" | "car" | (string & {});

/** Device sign in. The screen shows `user_code` even when it shows a QR code. */
export interface DeviceCode {
  device_code: string;
  expires_in: number;
  interval: number;
  user_code: string;
  verification_uri: string;
  verification_uri_complete: string;
}

export interface DeviceCodeRequest {
  device_name: string;
}

export interface DeviceDecision {
  user_code: string;
}

export interface DeviceList {
  devices: DeviceSession[];
}

/** Answers while a device waits for approval. */
export type DevicePoll = "authorization_pending" | "slow_down" | "access_denied" | "expired_token" | (string & {});

/** One signed in device. */
export interface DeviceSession {
  class: DeviceClass;
  current: boolean;
  id: string;
  last_seen_ms: number;
  name: string;
}

/** Where a device sign in stands. */
export type DeviceSignInState = "idle" | "waiting" | "denied" | "expired" | "off" | (string & {});

/** Device sign in state for screens. A dead code is left out, so no screen can show one, and the device code has no field here. */
export interface DeviceSignInView {
  /** Seconds this code lives, counted from when it reached this view. */
  expires_in?: number;
  state: DeviceSignInState;
  /** The screen shows this even when it shows a QR code. */
  user_code?: string;
  verification_uri?: string;
  /** The same address with the code in it, for a QR code. */
  verification_uri_complete?: string;
}

/** Either tokens or a word saying to wait or stop. */
export interface DeviceTokenAnswer {
  poll?: DevicePoll;
  signed_in?: SignedIn;
}

export interface DeviceTokenRequest {
  device_code: string;
}

/** Answer with no data. */
export interface Done {
  done: boolean;
}

/** A command to the player. Attempts carry their headers here. */
export type DriverCommand =
  | {
      op: "open";
      generation: number;
      headers: Record<string, string>;
      max_bitrate?: number;
      max_height?: number;
      start_at_ms: number;
      url: string;
    }
  | {
      op: "play";
    }
  | {
      op: "pause";
    }
  | {
      op: "seek";
      to_ms: number;
    }
  | {
      op: "rate";
      percent: number;
    }
  | {
      op: "volume";
      percent: number;
    }
  | {
      op: "select_audio";
      track: string;
    }
  | {
      op: "select_text";
      track?: string;
    }
  | {
      op: "close";
    }
  ;

/** What the driver reports. No driver decides failover or no picture by itself. */
export type DriverEvent =
  | {
      kind: "first_frame";
    }
  | {
      kind: "position";
      /** 0 or absent when not known. */
      duration_ms?: number;
      /** Absent means unknown, and the no picture rule stays off. */
      has_picture?: boolean;
      time_ms: number;
      video_width?: number;
    }
  | {
      kind: "stalled";
    }
  | {
      kind: "ended";
    }
  | {
      kind: "tracks";
      audio: Track[];
      text: Track[];
    }
  | {
      kind: "error";
      class: ErrorClass;
    }
  ;

/** One effect. The result comes back as an event with this id. */
export type Effect =
  | {
      /** A path, never an address. The host holds the base address. */
      kind: "http";
      id: string;
      body?: string;
      class: HttpClass;
      deadline_ms: number;
      headers?: Record<string, string>;
      method: HttpMethod;
      path: string;
      /** Opaque scope of an account and profile, for kept copies. */
      scope?: string;
      to: BodyTarget;
    }
  | {
      kind: "store";
      id: string;
      key?: string;
      op: StoreOp;
      value?: string;
    }
  | {
      kind: "secret";
      id: string;
      name: string;
      op: SecretOp;
      value?: string;
    }
  | {
      kind: "timer";
      id: string;
      after_ms?: number;
      op: TimerOp;
    }
  | {
      kind: "player";
      id: string;
      command: DriverCommand;
    }
  | {
      /** A one time notice or a kept page. Lasting state travels in `view`. */
      kind: "view";
      id: string;
      body?: ScreenBody;
      notice?: Notice;
    }
  | {
      kind: "open";
      id: string;
      target: RouteTarget;
    }
  | {
      kind: "log";
      id: string;
      code: string;
      detail?: string;
      level: LogLevel;
    }
  ;

export interface EmailChange {
  email: string;
}

/** Why a session ended. */
export type EndReason = "signed_out" | "everywhere" | "refused" | "elsewhere" | (string & {});

/** One episode. */
export interface Episode {
  images?: Image[];
  name: string;
  number: number;
  overview?: string;
  /** Whole percent watched. */
  percent?: number;
  runtime_min?: number;
}

/** One episode of a series. A film has none. */
export interface EpisodeRef {
  episode: number;
  season: number;
}

/** Body of every error answer. */
export interface ErrorBody {
  /** The build that answered. */
  build: string;
  code: ErrorCode;
  /** Label key for this error. */
  label: string;
  notice?: Notice;
  retry_after_s?: number;
}

/** Failure classes a driver reports. The core picks the next attempt from these. */
export type ErrorClass = "network" | "manifest" | "segment" | "decode" | "key" | "format" | "refused" | (string & {});

/** Stable error code. A code this build does not know reads as `Unknown`. */
export type ErrorCode = "bad_request" | "unauthorized" | "session_ended" | "profile_required" | "pin_required" | "step_up_required" | "refused" | "not_found" | "no_longer_available" | "conflict" | "rate_limited" | "overloaded" | "deadline_exceeded" | "upgrade_required" | "unsupported" | "no_source" | "clock_wrong" | "internal" | (string & {});

/** One event. `id` is the id of the effect it answers, when it answers one. */
export type Event =
  | {
      /** An http effect got an answer. Every status is data. */
      type: "http_done";
      id?: string;
      /** Absent when the body went to the screen. */
      body?: string;
      headers?: Record<string, string>;
      status: number;
    }
  | {
      /** An http effect got no answer at all. */
      type: "http_failed";
      id?: string;
      reason: NoAnswerReason;
    }
  | {
      type: "store_done";
      id?: string;
      keys?: string[];
      value?: string;
    }
  | {
      type: "secret_done";
      id?: string;
      value?: string;
    }
  | {
      type: "timer_fired";
      id?: string;
    }
  | {
      /** The host cannot perform this effect. */
      type: "unsupported";
      id?: string;
    }
  | {
      /** The effect with this id was dropped. */
      type: "cancelled";
      id?: string;
    }
  | {
      type: "suspend";
      id?: string;
    }
  | {
      type: "resume";
      id?: string;
    }
  | {
      type: "visibility";
      id?: string;
      visible: boolean;
    }
  | {
      type: "network";
      id?: string;
      online: boolean;
    }
  | {
      /** Touch, key, wheel or playing video in view. */
      type: "activity";
      id?: string;
    }
  | {
      /** Another tab of this browser ended the session. The server has heard already. */
      type: "ended_elsewhere";
      id?: string;
    }
  | {
      /** An address or deep link to open. */
      type: "address";
      id?: string;
      address: string;
    }
  | {
      type: "action";
      id?: string;
      action: Action;
    }
  | {
      /** A driver event, stamped with the generation of the open that made it. */
      type: "driver";
      id?: string;
      event: DriverEvent;
      generation: number;
    }
  ;

/** Feature values. With no answer from the server, accounts are required. */
export interface Features {
  accounts_required?: boolean;
  clips?: boolean;
  games?: boolean;
  original_audio?: boolean;
  /** Tier for visitors. Present only when accounts are not required. */
  visitor_tier?: string;
}

/** Signals a viewer gives. */
export type FeedbackEvent = "click" | "start" | "complete" | "like" | "skip" | (string & {});

/** Answer, possibly partial when the budget ran out. */
export interface FindAnswer {
  /** Read the rest through `/v1/find/{job}`. */
  job?: string;
  outcome: FindOutcome;
  sources: FoundSource[];
}

/** How a search for sources ended. */
export type FindOutcome = "found" | "partial" | "not_found" | "pending" | (string & {});

/** Title facts come from the server, so a finder never looks them up. */
export interface FindRequest {
  /** Total time this search may take. */
  budget_ms: number;
  episode?: EpisodeRef;
  name: string;
  title: TitleKey;
  year?: number;
}

/** `/v1/health` of a finder. */
export interface FinderHealth {
  build: string;
  state: ServiceState;
}

/** The form a client draws in. A browser on a phone is class web, form phone. */
export type Form = "phone" | "tablet" | "desktop" | "tv" | (string & {});

/** One source as facts. The server turns these into ranked attempts. */
export interface FoundSource {
  audio?: string[];
  container?: string;
  group: string;
  headers: Record<string, string>;
  height?: number;
  kind: AttemptKind;
  url: string;
  verified: boolean;
}

/** True while the who is watching prompt should show. */
export interface GateView {
  ask: boolean;
}

/** First answer a client reads. */
export interface Hello {
  brand: BrandValues;
  client_min: ClientFloor[];
  /** Wire dates this server answers. */
  contract: VersionRange;
  features: Features;
  /** Builds refused by name. */
  killed_builds?: string[];
  limits: Limits;
  moved?: MovedAddress[];
  /** False stops playback everywhere with a notice. */
  playback: boolean;
  rollouts?: Rollout[];
  start_deadline_ms: ClassValue[];
  /** Seconds this answer stays good. */
  ttl_s: number;
}

/** `/v1/help/articles/{slug}`. */
export interface HelpArticle {
  related?: HelpLink[];
  sections: HelpSection[];
  slug: string;
  title: string;
}

export interface HelpBlock {
  kind: HelpBlockKind;
  lines?: HelpLine[];
  /** The action of a tool block. */
  tool?: string;
}

/** What a block of an article is. */
export type HelpBlockKind = "paragraph" | "heading" | "steps" | "list" | "note" | "tool" | (string & {});

export interface HelpGroup {
  articles: HelpLink[];
  id: string;
  title: string;
}

/** `/v1/help`. */
export interface HelpHome {
  policies: HelpPolicyLink[];
  popular: HelpLink[];
  topics: HelpTopic[];
}

/** One line of a block. A paragraph has one, a list has one per item. */
export interface HelpLine {
  runs: HelpRun[];
}

/** An article as a list shows it. */
export interface HelpLink {
  slug: string;
  summary?: string;
  title: string;
}

/** `/v1/help/policies/{id}`. */
export interface HelpPolicy {
  id: string;
  sections: HelpSection[];
  title: string;
  updated?: string;
}

export interface HelpPolicyLink {
  id: string;
  title: string;
}

/** `/v1/help/search`. */
export interface HelpResults {
  articles: HelpLink[];
}

/** Some words of a line, with how they are marked. */
export interface HelpRun {
  bold?: boolean;
  /** Where the words lead: an address in the app, a web page or a mail address. */
  link?: string;
  text: string;
}

/** A part of a page. A folded one starts closed where the app can open and close it. */
export interface HelpSection {
  blocks: HelpBlock[];
  folded?: boolean;
  title?: string;
}

export interface HelpTopic {
  groups: HelpGroup[];
  id: string;
  title: string;
}

/** `POST /v1/core/step`. `config` is sent on the first step of a hosted session. */
export interface HostedStep {
  config?: CoreConfig;
  input: CoreIn;
}

/** How an http effect may be handled. */
export type HttpClass = "command" | "cacheable" | "plain" | (string & {});

/** Methods of an http effect. */
export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | (string & {});

/** One image address. Images go straight from the host to the viewer. */
export interface Image {
  role: ImageRole;
  url: string;
  width?: number;
}

/** What an image is for. */
export type ImageRole = "poster" | "backdrop" | "logo" | "still" | "avatar" | (string & {});

/** Input a device has. */
export type InputKind = "touch" | "pointer" | "keyboard" | "remote" | "gamepad" | "rotary" | (string & {});

/** A language this server has text for, named in itself. */
export interface Language {
  code: string;
  name: string;
}

export interface LanguageList {
  languages: Language[];
}

/** Asks `core_open` for the offline layer. A host sets it as the `layer` key of the text it gives `core_open`, next to the `CoreConfig` keys. With no `layer` key the core runs alone, and the core itself never reads this key. */
export interface LayerConfig {
  /** Names this boot. It changes when monotonic time restarts. */
  boot_id: string;
  /** The byte cap from the capability card. 0 means nothing is kept across a restart. */
  storage_bytes: number;
}

/** Limits served from one place, so no client holds a copy. */
export interface Limits {
  blocked_title_max: number;
  blocked_titles_max: number;
  code_digits: number;
  code_valid_s: number;
  command_batch_max: number;
  handle_max: number;
  handle_min: number;
  name_max: number;
  password_min: number;
  phone_digits_max: number;
  phone_digits_min: number;
  pin_digits: number;
  profile_name_max: number;
  profiles_max: number;
  step_up_s: number;
}

/** List state of a title, versioned apart from its rating. */
export interface ListState {
  changed_at_ms: number;
  in_list: boolean;
  version: number;
}

/** Log levels. */
export type LogLevel = "debug" | "info" | "warn" | "error" | (string & {});

/** Skip marks. */
export interface Marks {
  windows: TimeWindow[];
}

/** Ask for skip marks, after first frame, series only. */
export interface MarksRequest {
  attempt: string;
  delay_ms?: number;
  duration_ms: number;
  file?: string;
}

/** The next part of a screen or a section. */
export interface More {
  cursor?: Cursor;
  sections?: Section[];
  tiles?: Tile[];
}

/** Late attempts. */
export interface MoreAttempts {
  attempts: Attempt[];
  more: boolean;
}

/** One row of the moved address table. */
export interface MovedAddress {
  from: string;
  to: string;
}

/** The menu for this class and profile. */
export interface Nav {
  items: NavItem[];
  ttl_s: number;
}

/** One entry of the menu. */
export interface NavItem {
  genre?: string;
  id: string;
  label: string;
  screen: ScreenName;
}

/** Why an http effect got no answer. Any status at all is an answer. */
export type NoAnswerReason = "offline" | "timeout" | "tls" | "clock_wrong" | "cancelled" | (string & {});

/** Text a client can draw with no other knowledge, including a client below the floor. */
export interface Notice {
  action_address?: string;
  action_label?: string;
  /** Label key, for clients with the strings. */
  label: string;
  /** Plain text in the viewer's language. */
  text: string;
}

/** Answer for one op. */
export interface OpAnswer {
  op_id: string;
  outcome: Outcome;
}

/** Answer per op. */
export type Outcome = "applied" | "superseded" | "rejected" | (string & {});

export interface PasswordChange {
  new_password: string;
}

export interface PasswordSignIn {
  email: string;
  password: string;
}

export interface PhoneChange {
  phone: string;
}

/** Absent clears the PIN. */
export interface PinSet {
  pin?: string;
}

/** Answer to a play request. */
export interface PlayAnswer {
  age_card?: AgeCard;
  attempts: Attempt[];
  audio_default?: string;
  /** This profile's setting: the next episode starts by itself at the end of play. */
  autoplay_next?: boolean;
  /** Attempts removed by the capability card. */
  dropped: number;
  /** True when more attempts may arrive through `/v1/play/{id}/more`. */
  more: boolean;
  /** The next episode in the server's order, across seasons. Absent on the last one. */
  next_episode?: EpisodeRef;
  play_id: string;
  /** Where to start. */
  start_at_ms: number;
  subtitle_default?: string;
}

/** One thing that happened during play. */
export interface PlayEvent {
  at_ms: number;
  attempt?: string;
  failure?: ErrorClass;
  first_frame_ms?: number;
  host?: string;
  name: PlayEventName;
  rung?: CoreMode;
}

/** What a play report says happened. */
export type PlayEventName = "first_frame" | "rebuffer" | "failure" | "ended" | (string & {});

/** `/v1/play/{id}/events`. */
export interface PlayEvents {
  events: PlayEvent[];
}

/** Ask to play a title. */
export interface PlayRequest {
  episode?: EpisodeRef;
  title: TitleKey;
}

/** Where the play session stands. */
export type PlayState = "resolving" | "starting" | "playing" | "paused" | "stalled" | "switching" | "ended" | "failed" | (string & {});

/** Play session state for screens. */
export interface PlayView {
  attempt?: string;
  episode?: EpisodeRef;
  /** Absent on the last episode, so no screen offers one that does not exist. */
  next_episode?: EpisodeRef;
  state: PlayState;
  title: TitleKey;
  windows?: TimeWindow[];
}

/** One profile. */
export interface Profile {
  avatar?: string;
  has_pin: boolean;
  id: string;
  kids: boolean;
  /** A language code from `/v1/languages`, absent until one is chosen. */
  language?: string;
  main: boolean;
  name: string;
}

export interface ProfileCreate {
  kids?: boolean;
  name: string;
}

export interface ProfileEnter {
  pin?: string;
}

/** The profile answer names its home, so the core can map it to a target. */
export interface ProfileEntered {
  home: ScreenName;
  profile: Profile;
}

/** Needs a step up. Never reachable through a queued command. */
export interface ProfileLimits {
  age_rating: string;
  blocked_titles: string[];
}

export interface ProfileList {
  profiles: Profile[];
}

export interface ProfileSettings {
  audio_language?: string;
  autoplay_next: boolean;
  autoplay_previews: boolean;
  gender?: string;
  handle?: string;
  subtitle_language?: string;
  subtitle_style?: Record<string, string>;
  ui_language?: string;
}

export interface ProfileTransfer {
  to_account_email: string;
}

export interface ProfileUpdate {
  avatar?: string;
  kids?: boolean;
  language?: string;
  name?: string;
}

/** A position, key by title and episode. */
export interface ProgressState {
  changed_at_ms: number;
  completed: boolean;
  duration_s: number;
  episode?: EpisodeRef;
  hidden: boolean;
  position_s: number;
  title: TitleKey;
  version: number;
}

/** Rating words on the wire. Screens use the words like, love and dislike. */
export type Rating = "down" | "up" | "love" | (string & {});

/** Rating state of a title, versioned apart from its list state. */
export interface RatingState {
  changed_at_ms: number;
  /** Absent when no rating is set. */
  rating?: Rating;
  version: number;
}

/** `/v1/ready`, for outside checks. */
export interface Ready {
  build: string;
  data_age_s: number;
  data_hash: string;
  finder: ServiceState;
  index: ServiceState;
}

export interface RecoveryFinish {
  code: string;
  email: string;
  new_password: string;
}

export interface RecoveryStart {
  email: string;
}

export interface RefreshRequest {
  refresh_token: string;
}

/** Events after which a screen should be read again. */
export type RefreshTrigger = "focus" | "sign_in" | "profile_change" | "list_change" | "history_change" | (string & {});

/** `/v1/reports`. */
export interface Report {
  build: string;
  code: string;
  detail?: string;
}

/** A staged rollout. The bucket comes from the household, so a home moves together. */
export interface Rollout {
  name: string;
  percent: number;
}

/** What an address opens. */
export interface RouteTarget {
  /** The code an approve address carried, as it stood in the address. */
  code?: string;
  episode?: EpisodeRef;
  /** The genre, for the genre screen. */
  genre?: string;
  /** A safe address to return to after sign in. */
  next?: string;
  screen?: ScreenName;
  target: Target;
  title?: TitleKey;
}

/** A built page. */
export interface Screen {
  /** Reads more sections through `/v1/more/{cursor}`. */
  cursor?: Cursor;
  name: ScreenName;
  refresh_on?: RefreshTrigger[];
  sections: Section[];
  /** Seconds this answer stays good. */
  ttl_s: number;
}

/** A page body handed to a screen. It stands in for the answer to an http effect whose body was going to the screen. */
export interface ScreenBody {
  body: string;
  /** Id of the http effect this answers. */
  request: string;
  /** True when this copy is past its ttl and was kept for when the network is gone. */
  stale?: boolean;
}

/** The screens the server builds. */
export type ScreenName = "home" | "films" | "series" | "genre" | "new" | "my-list" | "kids" | "search" | "notifications" | "activity" | "games" | "clips" | "language" | "mine" | "help" | "settings" | (string & {});

/** One season with its episodes. */
export interface Season {
  episodes: Episode[];
  name?: string;
  number: number;
}

/** A season named on a title page. */
export interface SeasonSummary {
  episode_count: number;
  number: number;
}

/** Secret operations. */
export type SecretOp = "get" | "put" | "delete" | (string & {});

/** One row, hero, grid or notice of a screen. */
export interface Section {
  /** Reads more of this section through `/v1/more/{cursor}`. */
  cursor?: Cursor;
  id: string;
  kind: SectionKind;
  notice?: Notice;
  tiles?: Tile[];
  /** Translated title. */
  title?: string;
}

/** Section kinds. A kind this build does not draw falls back to a plain row. */
export type SectionKind = "hero" | "row" | "ranked" | "continue" | "grid" | "notice" | (string & {});

/** State of a part of the service. */
export type ServiceState = "ready" | "degraded" | "down" | (string & {});

/** Where sign in stands. */
export type SessionState = "starting" | "signed_out" | "signed_in" | (string & {});

/** Sign in state for screens. Tokens never appear here. */
export interface SessionView {
  email?: string;
  ended?: EndReason;
  expires_at_ms?: number;
  profile?: string;
  /** Opaque scope of this account and profile, the same value http effects carry. */
  scope?: string;
  state: SessionState;
  user_id?: string;
}

/** Settings a queued command may change. Age rating and blocked titles are absent on purpose: they need a step up, so a queued write can never touch them. */
export type SettingKey = "autoplay_next" | "autoplay_previews" | "audio_language" | "subtitle_language" | "subtitle_style" | "ui_language" | "handle" | "gender" | (string & {});

export interface SignOut {
  /** True ends every session of this account. */
  everywhere: boolean;
}

export interface SignUp {
  email: string;
  first_name: string;
  last_name: string;
  password: string;
}

/** Opaque tokens. `expires_in` is seconds, so no device clock decides anything. */
export interface SignedIn {
  access_token: string;
  email: string;
  expires_in: number;
  refresh_token: string;
  user_id: string;
}

/** Store operations. Only the offline layer uses store. */
export type StoreOp = "get" | "put" | "delete" | "list" | (string & {});

/** `/v1/strings/{lang}`. Text holds the placeholders below. */
export interface Strings {
  lang: string;
  texts: Record<string, string>;
}

/** `/v1/subtitles/{file}`. The core checks `last_end_ms` against the driver's duration. */
export interface SubtitleBody {
  cues: Cue[];
  last_end_ms: number;
}

/** One subtitle file of a language, in the order to try. */
export interface SubtitleFile {
  hearing_impaired?: boolean;
  id: string;
}

/** One language. */
export interface SubtitleLanguage {
  files: SubtitleFile[];
  label: string;
  lang: string;
}

/** `/v1/subtitles`. */
export interface SubtitleList {
  languages: SubtitleLanguage[];
}

/** Search suggestions. */
export interface Suggestions {
  items: string[];
}

/** A semantic target. Each device maps it to one of its own screens. */
export type Target = "start" | "sign_in" | "profiles" | "home" | "kids_home" | "screen" | "title" | "watch" | "not_found" | "wait" | "upgrade" | "activate" | (string & {});

/** One title in a section. */
export interface Tile {
  badges?: Badge[];
  episode?: EpisodeRef;
  images?: Image[];
  name: string;
  /** Whole percent watched. */
  percent?: number;
  tags?: string[];
  title: TitleKey;
}

/** A span of a video the host compares to player time. Nothing ticks across the boundary. */
export interface TimeWindow {
  end_ms: number;
  kind: WindowKind;
  /** Where a skip button moves to. */
  seek_to_ms?: number;
  start_ms: number;
  text?: string;
}

/** Timer operations. A clear uses the id of the timer it clears. */
export type TimerOp = "set" | "clear" | (string & {});

/** Facts of one title. */
export interface TitleDetail {
  /** Absent when the title has no rating in either system. */
  age?: AgeMark;
  genres?: string[];
  images?: Image[];
  /** True when this profile has saved the title. */
  in_list?: boolean;
  /** Hidden when empty. */
  more_like_this?: Tile[];
  name: string;
  overview?: string;
  runtime_min?: number;
  seasons?: SeasonSummary[];
  title: TitleKey;
  year?: number;
}

/** movie:<id> or tv:<id>. A bare id does not parse. */
export type TitleKey = `movie:${number}` | `tv:${number}`;

/** The two kinds of title. Closed on purpose: a new kind reaches old clients only through a dated transform, so a key always names a kind this build can open. */
export type TitleKind = "movie" | "tv" | (string & {});

/** One audio or text track, with the tag the manifest gave it. */
export interface Track {
  id: string;
  label?: string;
  lang?: string;
  /** True for the track playing now. */
  selected?: boolean;
}

/** The wire dates a server answers. Never compared for equality. */
export interface VersionRange {
  max: WireDate;
  min: WireDate;
}

/** Changed screen state. A part that is absent did not change. */
export interface View {
  device_sign_in?: DeviceSignInView;
  gate?: GateView;
  /** The last hello read, so a screen takes limits and features from one place. */
  hello?: Hello;
  play?: PlayView;
  /** The account's profiles, for the picker. No PIN is ever held here. */
  profiles?: ProfileList;
  route?: RouteTarget;
  session?: SessionView;
}

/** Windows of time inside a video. */
export type WindowKind = "recap" | "intro" | "credits" | "age_card" | "cue" | (string & {});

/** A wire date, YYYY-MM-DD. */
export type WireDate = `${number}-${number}-${number}`;
