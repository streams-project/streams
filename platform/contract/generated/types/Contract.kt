// Generated from the contract. Do not edit.

@file:OptIn(ExperimentalSerializationApi::class)

package contract

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonClassDiscriminator
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.polymorphic

const val WIRE_VERSION = "2026-11-05"
const val ABI_VERSION = 1

/** Ignores unknown fields and reads an unknown tag as the Unknown variant. */
val contractJson = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
    serializersModule = SerializersModule {
        polymorphic(Action::class) { defaultDeserializer { ActionUnknown.serializer() } }
        polymorphic(Change::class) { defaultDeserializer { ChangeUnknown.serializer() } }
        polymorphic(Command::class) { defaultDeserializer { CommandUnknown.serializer() } }
        polymorphic(CommandOp::class) { defaultDeserializer { CommandOpUnknown.serializer() } }
        polymorphic(DriverCommand::class) { defaultDeserializer { DriverCommandUnknown.serializer() } }
        polymorphic(DriverEvent::class) { defaultDeserializer { DriverEventUnknown.serializer() } }
        polymorphic(Effect::class) { defaultDeserializer { EffectUnknown.serializer() } }
        polymorphic(Event::class) { defaultDeserializer { EventUnknown.serializer() } }
    }
}

@Serializable
data class Account(
    @SerialName("email") val email: String,
    @SerialName("first_name") val firstName: String,
    @SerialName("id") val id: String,
    @SerialName("last_name") val lastName: String,
    @SerialName("phone") val phone: String? = null,
)

/** Everything held about an account, in plain JSON. */
@Serializable
data class AccountExport(
    @SerialName("account") val account: Account,
    @SerialName("devices") val devices: List<DeviceSession>,
    @SerialName("profiles") val profiles: List<Profile>,
)

@Serializable
data class AccountUpdate(
    @SerialName("first_name") val firstName: String? = null,
    @SerialName("last_name") val lastName: String? = null,
)

/** What a screen asks for. */
@Serializable
@JsonClassDiscriminator("action")
sealed interface Action

@Serializable
@SerialName("sign_in")
data class ActionSignIn(
    @SerialName("email") val email: String,
    @SerialName("password") val password: String,
) : Action

@Serializable
@SerialName("send_code")
data class ActionSendCode(
    @SerialName("email") val email: String,
) : Action

@Serializable
@SerialName("sign_in_code")
data class ActionSignInCode(
    @SerialName("code") val code: String,
    @SerialName("email") val email: String,
) : Action

@Serializable
@SerialName("sign_out")
data class ActionSignOut(
    @SerialName("everywhere") val everywhere: Boolean,
) : Action

@Serializable
@SerialName("enter_profile")
data class ActionEnterProfile(
    @SerialName("pin") val pin: String? = null,
    @SerialName("profile") val profile: String,
) : Action

@Serializable
@SerialName("play")
data class ActionPlay(
    @SerialName("episode") val episode: EpisodeRef? = null,
    @SerialName("title") val title: TitleKey,
) : Action

@Serializable
@SerialName("pause")
data object ActionPause : Action

@Serializable
@SerialName("unpause")
data object ActionUnpause : Action

@Serializable
@SerialName("seek")
data class ActionSeek(
    @SerialName("to_ms") val toMs: Long,
) : Action

@Serializable
@SerialName("select_audio")
data class ActionSelectAudio(
    @SerialName("track") val track: String,
) : Action

@Serializable
@SerialName("select_text")
data class ActionSelectText(
    @SerialName("track") val track: String? = null,
) : Action

@Serializable
@SerialName("next_subtitle_file")
data object ActionNextSubtitleFile : Action

@Serializable
@SerialName("stop")
data object ActionStop : Action

@Serializable
@SerialName("command")
data class ActionCommand(
    @SerialName("command") val command: Command,
) : Action

/** Send a prepared request with this session's token. The http effect takes the id of this event and its class is always plain, so whoever prepared it gets the answer under that id. Tokens stay in the core. */
@Serializable
@SerialName("request")
data class ActionRequest(
    @SerialName("body") val body: String? = null,
    @SerialName("method") val method: HttpMethod,
    @SerialName("path") val path: String,
) : Action

/** A tag this build does not know. */
@Serializable
data object ActionUnknown : Action

/** Age card shown once at the start of play. */
@Serializable
data class AgeCard(
    @SerialName("label") val label: String,
    @SerialName("reasons") val reasons: List<String>? = null,
)

/** One way to play. Attempts arrive in the order to try, ranked once on the server. `headers` is required, even when empty, and no provider name is carried. */
@Serializable
data class Attempt(
    @SerialName("audio") val audio: List<String>? = null,
    @SerialName("codecs") val codecs: List<String>? = null,
    @SerialName("container") val container: String? = null,
    /** Wall time, ms since 1970, after which the address stops working. */
    @SerialName("expires_at_ms") val expiresAtMs: Long? = null,
    /** Opaque group. After a failure the core skips other attempts of this group. */
    @SerialName("group") val group: String,
    /** Headers the player must send, such as Referer, Origin, User-Agent. */
    @SerialName("headers") val headers: Map<String, String>,
    @SerialName("height") val height: Long? = null,
    @SerialName("id") val id: String,
    @SerialName("kind") val kind: AttemptKind,
    @SerialName("url") val url: String,
    /** True when the finder fetched and read this address. */
    @SerialName("verified") val verified: Boolean,
)

/** How an attempt is played. */
@Serializable
@JvmInline
value class AttemptKind(val value: String) {
    companion object {
        val HLS = AttemptKind("hls")
        val FILE = AttemptKind("file")
        val EMBED = AttemptKind("embed")
    }
}

/** A mark on a tile. */
@Serializable
data class Badge(
    @SerialName("kind") val kind: BadgeKind,
    @SerialName("rank") val rank: Long? = null,
)

/** Marks on a tile. */
@Serializable
@JvmInline
value class BadgeKind(val value: String) {
    companion object {
        val RANKED = BadgeKind("ranked")
        val NEW = BadgeKind("new")
    }
}

/** Where the body of an answer goes. An answer outside 200 to 299 always comes to the core, because the core reads its error code. */
@Serializable
@JvmInline
value class BodyTarget(val value: String) {
    companion object {
        val CORE = BodyTarget("core")
        val SCREEN = BodyTarget("screen")
    }
}

/** Name and contact, filled into text at render time. */
@Serializable
data class BrandValues(
    @SerialName("name") val name: String,
    @SerialName("support_email") val supportEmail: String,
)

/** What a device can do. The server filters attempts and sections by this card. */
@Serializable
data class CapabilityCard(
    /** True when the player switches variants during play. */
    @SerialName("adaptive") val adaptive: Boolean,
    @SerialName("app") val app: String,
    /** Attempt kinds this player opens. An embed appears only when listed here. */
    @SerialName("attempt_kinds") val attemptKinds: List<AttemptKind>,
    @SerialName("class") val `class`: DeviceClass,
    /** Codecs this player opens, such as avc1, hvc1, mp4a. */
    @SerialName("codecs") val codecs: List<String>,
    /** Containers this player opens, such as ts, fmp4, mp4. */
    @SerialName("containers") val containers: List<String>,
    @SerialName("form") val form: Form,
    @SerialName("input") val input: List<InputKind>,
    @SerialName("languages") val languages: List<String>,
    @SerialName("max_height") val maxHeight: Long,
    /** Request headers the player can set on media, such as Referer. */
    @SerialName("media_headers") val mediaHeaders: List<String>,
    @SerialName("os") val os: String,
    @SerialName("os_version") val osVersion: String,
    /** True when the driver reports video width or has_picture. */
    @SerialName("reports_picture") val reportsPicture: Boolean,
    @SerialName("screen_height") val screenHeight: Long,
    @SerialName("screen_width") val screenWidth: Long,
    /** True when the device has a durable secret store. */
    @SerialName("secret_store") val secretStore: Boolean,
    /** Section kinds this build draws. Others fall back to a plain row. */
    @SerialName("section_kinds") val sectionKinds: List<SectionKind>,
    /** True when the player can load a side subtitle file. */
    @SerialName("side_subtitles") val sideSubtitles: Boolean,
    /** Bytes this device gives to kept commands and screens. 0 means none. */
    @SerialName("storage_bytes") val storageBytes: Long,
    @SerialName("version") val version: String,
)

/** One change a device reads. */
@Serializable
@JsonClassDiscriminator("change")
sealed interface Change

@Serializable
@SerialName("title")
data class ChangeTitle(
    @SerialName("list") val list: ListState,
    @SerialName("rating") val rating: RatingState,
    @SerialName("title") val title: TitleKey,
) : Change

@Serializable
@SerialName("progress")
data class ChangeProgress(
    @SerialName("state") val state: ProgressState,
) : Change

/** Hidden as a whole, every episode of a series included. */
@Serializable
@SerialName("hidden")
data class ChangeHidden(
    @SerialName("title") val title: TitleKey,
) : Change

@Serializable
@SerialName("history_cleared")
data class ChangeHistoryCleared(
    @SerialName("changed_at_ms") val changedAtMs: Long,
) : Change

@Serializable
@SerialName("setting")
data class ChangeSetting(
    @SerialName("key") val key: SettingKey,
    @SerialName("value") val value: String,
) : Change

/** A tag this build does not know. */
@Serializable
data object ChangeUnknown : Change

/** `GET /v1/changes`. */
@Serializable
data class Changes(
    @SerialName("changes") val changes: List<Change>,
    @SerialName("cursor") val cursor: Cursor,
    @SerialName("has_more") val hasMore: Boolean,
)

/** One value per client class. */
@Serializable
data class ClassValue(
    @SerialName("class") val `class`: DeviceClass,
    @SerialName("value") val value: Long,
)

/** Lowest app version a class may run. */
@Serializable
data class ClientFloor(
    @SerialName("class") val `class`: DeviceClass,
    @SerialName("version") val version: String,
)

@Serializable
data class CodeSend(
    @SerialName("email") val email: String,
)

/** A code was sent, or would have been. The answer never says whether the account exists. */
@Serializable
data class CodeSent(
    @SerialName("expires_in") val expiresIn: Long,
)

@Serializable
data class CodeSignIn(
    @SerialName("code") val code: String,
    @SerialName("email") val email: String,
)

/** One change to a profile's facts. */
@Serializable
@JsonClassDiscriminator("command")
sealed interface Command

/** Add to or remove from My List. Leaves the rating alone. */
@Serializable
@SerialName("list")
data class CommandList(
    @SerialName("in_list") val inList: Boolean,
    @SerialName("title") val title: TitleKey,
) : Command

/** Set or clear a rating. Leaves the list entry alone. */
@Serializable
@SerialName("rate")
data class CommandRate(
    @SerialName("rating") val rating: Rating? = null,
    @SerialName("title") val title: TitleKey,
) : Command

/** Hide a title from history. For a series this covers every episode. */
@Serializable
@SerialName("hide")
data class CommandHide(
    @SerialName("title") val title: TitleKey,
) : Command

/** Clear all history of this profile, as one operation. */
@Serializable
@SerialName("clear_history")
data object CommandClearHistory : Command

/** Save a position, in whole seconds. */
@Serializable
@SerialName("position")
data class CommandPosition(
    @SerialName("duration_s") val durationS: Long,
    @SerialName("episode") val episode: EpisodeRef? = null,
    @SerialName("position_s") val positionS: Long,
    @SerialName("title") val title: TitleKey,
) : Command

/** Change one profile setting. Booleans as true or false, style as JSON text. */
@Serializable
@SerialName("setting")
data class CommandSetting(
    @SerialName("key") val key: SettingKey,
    @SerialName("value") val value: String,
) : Command

/** A viewer signal. Discarded by the server when consent is off. */
@Serializable
@SerialName("feedback")
data class CommandFeedback(
    @SerialName("event") val event: FeedbackEvent,
    @SerialName("title") val title: TitleKey,
) : Command

/** A trailer failed to play for this title. */
@Serializable
@SerialName("trailer_refused")
data class CommandTrailerRefused(
    @SerialName("title") val title: TitleKey,
) : Command

/** A tag this build does not know. */
@Serializable
data object CommandUnknown : Command

/** Answer for a batch, one entry per op. */
@Serializable
data class CommandAnswer(
    @SerialName("answers") val answers: List<OpAnswer>,
)

/** `POST /v1/commands`. Idempotent by op id. */
@Serializable
data class CommandBatch(
    @SerialName("ops") val ops: List<CommandOp>,
)

/** One queued op. A missing `age_ms` means age unknown, and the server orders by arrival. */
@Serializable
@JsonClassDiscriminator("command")
sealed interface CommandOp {
    val ageMs: Long?
    val opId: String
}

/** Add to or remove from My List. Leaves the rating alone. */
@Serializable
@SerialName("list")
data class CommandOpList(
    @SerialName("age_ms") override val ageMs: Long? = null,
    @SerialName("op_id") override val opId: String,
    @SerialName("in_list") val inList: Boolean,
    @SerialName("title") val title: TitleKey,
) : CommandOp

/** Set or clear a rating. Leaves the list entry alone. */
@Serializable
@SerialName("rate")
data class CommandOpRate(
    @SerialName("age_ms") override val ageMs: Long? = null,
    @SerialName("op_id") override val opId: String,
    @SerialName("rating") val rating: Rating? = null,
    @SerialName("title") val title: TitleKey,
) : CommandOp

/** Hide a title from history. For a series this covers every episode. */
@Serializable
@SerialName("hide")
data class CommandOpHide(
    @SerialName("age_ms") override val ageMs: Long? = null,
    @SerialName("op_id") override val opId: String,
    @SerialName("title") val title: TitleKey,
) : CommandOp

/** Clear all history of this profile, as one operation. */
@Serializable
@SerialName("clear_history")
data class CommandOpClearHistory(
    @SerialName("age_ms") override val ageMs: Long? = null,
    @SerialName("op_id") override val opId: String,
) : CommandOp

/** Save a position, in whole seconds. */
@Serializable
@SerialName("position")
data class CommandOpPosition(
    @SerialName("age_ms") override val ageMs: Long? = null,
    @SerialName("op_id") override val opId: String,
    @SerialName("duration_s") val durationS: Long,
    @SerialName("episode") val episode: EpisodeRef? = null,
    @SerialName("position_s") val positionS: Long,
    @SerialName("title") val title: TitleKey,
) : CommandOp

/** Change one profile setting. Booleans as true or false, style as JSON text. */
@Serializable
@SerialName("setting")
data class CommandOpSetting(
    @SerialName("age_ms") override val ageMs: Long? = null,
    @SerialName("op_id") override val opId: String,
    @SerialName("key") val key: SettingKey,
    @SerialName("value") val value: String,
) : CommandOp

/** A viewer signal. Discarded by the server when consent is off. */
@Serializable
@SerialName("feedback")
data class CommandOpFeedback(
    @SerialName("age_ms") override val ageMs: Long? = null,
    @SerialName("op_id") override val opId: String,
    @SerialName("event") val event: FeedbackEvent,
    @SerialName("title") val title: TitleKey,
) : CommandOp

/** A trailer failed to play for this title. */
@Serializable
@SerialName("trailer_refused")
data class CommandOpTrailerRefused(
    @SerialName("age_ms") override val ageMs: Long? = null,
    @SerialName("op_id") override val opId: String,
    @SerialName("title") val title: TitleKey,
) : CommandOp

/** A tag this build does not know. */
@Serializable
data class CommandOpUnknown(
    @SerialName("age_ms") val rawAgeMs: Long? = null,
    @SerialName("op_id") val rawOpId: String? = null,
) : CommandOp {
    override val ageMs: Long? get() = rawAgeMs
    override val opId: String get() = rawOpId ?: ""
}

/** Step up. Send a password or an emailed code. */
@Serializable
data class Confirm(
    @SerialName("code") val code: String? = null,
    @SerialName("password") val password: String? = null,
)

/** A step up holds for this many seconds and a refresh never renews it. */
@Serializable
data class Confirmed(
    @SerialName("valid_s") val validS: Long,
)

/** Consent for personal rows. */
@Serializable
data class Consent(
    @SerialName("personal") val personal: Boolean,
)

/** Given to `core_open`, once. The seed is the only random input the core ever gets. */
@Serializable
data class CoreConfig(
    @SerialName("app") val app: String,
    /** What this device can do. The core sends its hash after sign in. */
    @SerialName("card") val card: CapabilityCard? = null,
    @SerialName("class") val `class`: DeviceClass,
    @SerialName("contract") val contract: WireDate,
    @SerialName("form") val form: Form,
    @SerialName("mode") val mode: CoreMode,
    @SerialName("os") val os: String,
    /** Hex digits, at least 32. */
    @SerialName("seed") val seed: String,
    @SerialName("type") val type: CoreConfigTag,
    @SerialName("version") val version: String,
)

/** Type tag of `CoreConfig`. */
@Serializable
enum class CoreConfigTag {
    @SerialName("core_config")
    VALUE,
}

/** One step of input. Every input carries wall time and monotonic time. */
@Serializable
data class CoreIn(
    @SerialName("contract") val contract: WireDate,
    @SerialName("events") val events: List<Event>,
    /** Monotonic time, ms. Restarts at boot. */
    @SerialName("mono_ms") val monoMs: Long,
    /** Wall time, ms since 1970. */
    @SerialName("now_ms") val nowMs: Long,
    @SerialName("type") val type: CoreInTag,
)

/** Type tag of `CoreIn`. */
@Serializable
enum class CoreInTag {
    @SerialName("core_in")
    VALUE,
}

/** Where the core runs for this device. */
@Serializable
@JvmInline
value class CoreMode(val value: String) {
    companion object {
        val LOCAL = CoreMode("local")
        val HOSTED = CoreMode("hosted")
    }
}

/** One step of output. */
@Serializable
data class CoreOut(
    @SerialName("contract") val contract: WireDate,
    @SerialName("effects") val effects: List<Effect>,
    /** Monotonic time at which the host steps the core again. Absent means no wake. */
    @SerialName("next_wake_ms") val nextWakeMs: Long? = null,
    @SerialName("type") val type: CoreOutTag,
    /** Ids of in events of a type this core does not know. */
    @SerialName("unsupported") val unsupported: List<String>? = null,
    @SerialName("view") val view: View? = null,
)

/** Type tag of `CoreOut`. */
@Serializable
enum class CoreOutTag {
    @SerialName("core_out")
    VALUE,
}

/** One cue. */
@Serializable
data class Cue(
    @SerialName("end_ms") val endMs: Long,
    @SerialName("start_ms") val startMs: Long,
    @SerialName("text") val text: String,
)

/** An opaque position in a list. Clients hand it back and never read it. */
typealias Cursor = String

/** True when the server wants the whole card. */
@Serializable
data class DeviceCardAnswer(
    @SerialName("send_card") val sendCard: Boolean,
)

/** Sent at each start. The card travels only when the server asks or the app or OS changed. */
@Serializable
data class DeviceCardRequest(
    @SerialName("card") val card: CapabilityCard? = null,
    @SerialName("card_hash") val cardHash: String,
    @SerialName("device_key") val deviceKey: String,
)

/** Client class. Minimum versions, start deadlines and reachability are kept per class. */
@Serializable
@JvmInline
value class DeviceClass(val value: String) {
    companion object {
        val WEB = DeviceClass("web")
        val PHONE = DeviceClass("phone")
        val TABLET = DeviceClass("tablet")
        val DESKTOP = DeviceClass("desktop")
        val TV = DeviceClass("tv")
        val BOX = DeviceClass("box")
        val CAR = DeviceClass("car")
    }
}

/** Device sign in. The screen shows `user_code` even when it shows a QR code. */
@Serializable
data class DeviceCode(
    @SerialName("device_code") val deviceCode: String,
    @SerialName("expires_in") val expiresIn: Long,
    @SerialName("interval") val interval: Long,
    @SerialName("user_code") val userCode: String,
    @SerialName("verification_uri") val verificationUri: String,
    @SerialName("verification_uri_complete") val verificationUriComplete: String,
)

@Serializable
data class DeviceCodeRequest(
    @SerialName("device_name") val deviceName: String,
)

@Serializable
data class DeviceDecision(
    @SerialName("user_code") val userCode: String,
)

@Serializable
data class DeviceList(
    @SerialName("devices") val devices: List<DeviceSession>,
)

/** Answers while a device waits for approval. */
@Serializable
@JvmInline
value class DevicePoll(val value: String) {
    companion object {
        val AUTHORIZATION_PENDING = DevicePoll("authorization_pending")
        val SLOW_DOWN = DevicePoll("slow_down")
        val ACCESS_DENIED = DevicePoll("access_denied")
        val EXPIRED_TOKEN = DevicePoll("expired_token")
    }
}

/** One signed in device. */
@Serializable
data class DeviceSession(
    @SerialName("class") val `class`: DeviceClass,
    @SerialName("current") val current: Boolean,
    @SerialName("id") val id: String,
    @SerialName("last_seen_ms") val lastSeenMs: Long,
    @SerialName("name") val name: String,
)

/** Either tokens or a word saying to wait or stop. */
@Serializable
data class DeviceTokenAnswer(
    @SerialName("poll") val poll: DevicePoll? = null,
    @SerialName("signed_in") val signedIn: SignedIn? = null,
)

@Serializable
data class DeviceTokenRequest(
    @SerialName("device_code") val deviceCode: String,
)

/** Answer with no data. */
@Serializable
data class Done(
    @SerialName("done") val done: Boolean,
)

/** A command to the player. Attempts carry their headers here. */
@Serializable
@JsonClassDiscriminator("op")
sealed interface DriverCommand

@Serializable
@SerialName("open")
data class DriverCommandOpen(
    @SerialName("generation") val generation: Long,
    @SerialName("headers") val headers: Map<String, String>,
    @SerialName("max_bitrate") val maxBitrate: Long? = null,
    @SerialName("max_height") val maxHeight: Long? = null,
    @SerialName("start_at_ms") val startAtMs: Long,
    @SerialName("url") val url: String,
) : DriverCommand

@Serializable
@SerialName("play")
data object DriverCommandPlay : DriverCommand

@Serializable
@SerialName("pause")
data object DriverCommandPause : DriverCommand

@Serializable
@SerialName("seek")
data class DriverCommandSeek(
    @SerialName("to_ms") val toMs: Long,
) : DriverCommand

@Serializable
@SerialName("rate")
data class DriverCommandRate(
    @SerialName("percent") val percent: Long,
) : DriverCommand

@Serializable
@SerialName("volume")
data class DriverCommandVolume(
    @SerialName("percent") val percent: Long,
) : DriverCommand

@Serializable
@SerialName("select_audio")
data class DriverCommandSelectAudio(
    @SerialName("track") val track: String,
) : DriverCommand

@Serializable
@SerialName("select_text")
data class DriverCommandSelectText(
    @SerialName("track") val track: String? = null,
) : DriverCommand

@Serializable
@SerialName("close")
data object DriverCommandClose : DriverCommand

/** A tag this build does not know. */
@Serializable
data object DriverCommandUnknown : DriverCommand

/** What the driver reports. No driver decides failover or no picture by itself. */
@Serializable
@JsonClassDiscriminator("kind")
sealed interface DriverEvent

@Serializable
@SerialName("first_frame")
data object DriverEventFirstFrame : DriverEvent

@Serializable
@SerialName("position")
data class DriverEventPosition(
    /** 0 or absent when not known. */
    @SerialName("duration_ms") val durationMs: Long? = null,
    /** Absent means unknown, and the no picture rule stays off. */
    @SerialName("has_picture") val hasPicture: Boolean? = null,
    @SerialName("time_ms") val timeMs: Long,
    @SerialName("video_width") val videoWidth: Long? = null,
) : DriverEvent

@Serializable
@SerialName("stalled")
data object DriverEventStalled : DriverEvent

@Serializable
@SerialName("ended")
data object DriverEventEnded : DriverEvent

@Serializable
@SerialName("tracks")
data class DriverEventTracks(
    @SerialName("audio") val audio: List<Track>,
    @SerialName("text") val text: List<Track>,
) : DriverEvent

@Serializable
@SerialName("error")
data class DriverEventError(
    @SerialName("class") val `class`: ErrorClass,
) : DriverEvent

/** A tag this build does not know. */
@Serializable
data object DriverEventUnknown : DriverEvent

/** One effect. The result comes back as an event with this id. */
@Serializable
@JsonClassDiscriminator("kind")
sealed interface Effect {
    val id: String
}

/** A path, never an address. The host holds the base address. */
@Serializable
@SerialName("http")
data class EffectHttp(
    @SerialName("id") override val id: String,
    @SerialName("body") val body: String? = null,
    @SerialName("class") val `class`: HttpClass,
    @SerialName("deadline_ms") val deadlineMs: Long,
    @SerialName("headers") val headers: Map<String, String>? = null,
    @SerialName("method") val method: HttpMethod,
    @SerialName("path") val path: String,
    /** Opaque scope of an account and profile, for kept copies. */
    @SerialName("scope") val scope: String? = null,
    @SerialName("to") val to: BodyTarget,
) : Effect

@Serializable
@SerialName("store")
data class EffectStore(
    @SerialName("id") override val id: String,
    @SerialName("key") val key: String? = null,
    @SerialName("op") val op: StoreOp,
    @SerialName("value") val value: String? = null,
) : Effect

@Serializable
@SerialName("secret")
data class EffectSecret(
    @SerialName("id") override val id: String,
    @SerialName("name") val name: String,
    @SerialName("op") val op: SecretOp,
    @SerialName("value") val value: String? = null,
) : Effect

@Serializable
@SerialName("timer")
data class EffectTimer(
    @SerialName("id") override val id: String,
    @SerialName("after_ms") val afterMs: Long? = null,
    @SerialName("op") val op: TimerOp,
) : Effect

@Serializable
@SerialName("player")
data class EffectPlayer(
    @SerialName("id") override val id: String,
    @SerialName("command") val command: DriverCommand,
) : Effect

/** A one time notice or a kept page. Lasting state travels in `view`. */
@Serializable
@SerialName("view")
data class EffectView(
    @SerialName("id") override val id: String,
    @SerialName("body") val body: ScreenBody? = null,
    @SerialName("notice") val notice: Notice? = null,
) : Effect

@Serializable
@SerialName("open")
data class EffectOpen(
    @SerialName("id") override val id: String,
    @SerialName("target") val target: RouteTarget,
) : Effect

@Serializable
@SerialName("log")
data class EffectLog(
    @SerialName("id") override val id: String,
    @SerialName("code") val code: String,
    @SerialName("detail") val detail: String? = null,
    @SerialName("level") val level: LogLevel,
) : Effect

/** A tag this build does not know. */
@Serializable
data class EffectUnknown(
    @SerialName("id") val rawId: String? = null,
) : Effect {
    override val id: String get() = rawId ?: ""
}

@Serializable
data class EmailChange(
    @SerialName("email") val email: String,
)

/** Why a session ended. */
@Serializable
@JvmInline
value class EndReason(val value: String) {
    companion object {
        val SIGNED_OUT = EndReason("signed_out")
        val EVERYWHERE = EndReason("everywhere")
        val REFUSED = EndReason("refused")
        val ELSEWHERE = EndReason("elsewhere")
    }
}

/** One episode. */
@Serializable
data class Episode(
    @SerialName("images") val images: List<Image>? = null,
    @SerialName("name") val name: String,
    @SerialName("number") val number: Long,
    @SerialName("overview") val overview: String? = null,
    /** Whole percent watched. */
    @SerialName("percent") val percent: Long? = null,
    @SerialName("runtime_min") val runtimeMin: Long? = null,
)

/** One episode of a series. A film has none. */
@Serializable
data class EpisodeRef(
    @SerialName("episode") val episode: Long,
    @SerialName("season") val season: Long,
)

/** Body of every error answer. */
@Serializable
data class ErrorBody(
    /** The build that answered. */
    @SerialName("build") val build: String,
    @SerialName("code") val code: ErrorCode,
    /** Label key for this error. */
    @SerialName("label") val label: String,
    @SerialName("notice") val notice: Notice? = null,
    @SerialName("retry_after_s") val retryAfterS: Long? = null,
)

/** Failure classes a driver reports. The core picks the next attempt from these. */
@Serializable
@JvmInline
value class ErrorClass(val value: String) {
    companion object {
        val NETWORK = ErrorClass("network")
        val MANIFEST = ErrorClass("manifest")
        val SEGMENT = ErrorClass("segment")
        val DECODE = ErrorClass("decode")
        val KEY = ErrorClass("key")
        val FORMAT = ErrorClass("format")
        val REFUSED = ErrorClass("refused")
    }
}

/** Stable error code. A code this build does not know reads as `Unknown`. */
@Serializable
@JvmInline
value class ErrorCode(val value: String) {
    companion object {
        val BAD_REQUEST = ErrorCode("bad_request")
        val UNAUTHORIZED = ErrorCode("unauthorized")
        val SESSION_ENDED = ErrorCode("session_ended")
        val PROFILE_REQUIRED = ErrorCode("profile_required")
        val PIN_REQUIRED = ErrorCode("pin_required")
        val STEP_UP_REQUIRED = ErrorCode("step_up_required")
        val REFUSED = ErrorCode("refused")
        val NOT_FOUND = ErrorCode("not_found")
        val NO_LONGER_AVAILABLE = ErrorCode("no_longer_available")
        val CONFLICT = ErrorCode("conflict")
        val RATE_LIMITED = ErrorCode("rate_limited")
        val OVERLOADED = ErrorCode("overloaded")
        val DEADLINE_EXCEEDED = ErrorCode("deadline_exceeded")
        val UPGRADE_REQUIRED = ErrorCode("upgrade_required")
        val UNSUPPORTED = ErrorCode("unsupported")
        val NO_SOURCE = ErrorCode("no_source")
        val CLOCK_WRONG = ErrorCode("clock_wrong")
        val INTERNAL = ErrorCode("internal")
    }
}

/** One event. `id` is the id of the effect it answers, when it answers one. */
@Serializable
@JsonClassDiscriminator("type")
sealed interface Event {
    val id: String?
}

/** An http effect got an answer. Every status is data. */
@Serializable
@SerialName("http_done")
data class EventHttpDone(
    @SerialName("id") override val id: String? = null,
    /** Absent when the body went to the screen. */
    @SerialName("body") val body: String? = null,
    @SerialName("headers") val headers: Map<String, String>? = null,
    @SerialName("status") val status: Long,
) : Event

/** An http effect got no answer at all. */
@Serializable
@SerialName("http_failed")
data class EventHttpFailed(
    @SerialName("id") override val id: String? = null,
    @SerialName("reason") val reason: NoAnswerReason,
) : Event

@Serializable
@SerialName("store_done")
data class EventStoreDone(
    @SerialName("id") override val id: String? = null,
    @SerialName("keys") val keys: List<String>? = null,
    @SerialName("value") val value: String? = null,
) : Event

@Serializable
@SerialName("secret_done")
data class EventSecretDone(
    @SerialName("id") override val id: String? = null,
    @SerialName("value") val value: String? = null,
) : Event

@Serializable
@SerialName("timer_fired")
data class EventTimerFired(
    @SerialName("id") override val id: String? = null,
) : Event

/** The host cannot perform this effect. */
@Serializable
@SerialName("unsupported")
data class EventUnsupported(
    @SerialName("id") override val id: String? = null,
) : Event

/** The effect with this id was dropped. */
@Serializable
@SerialName("cancelled")
data class EventCancelled(
    @SerialName("id") override val id: String? = null,
) : Event

@Serializable
@SerialName("suspend")
data class EventSuspend(
    @SerialName("id") override val id: String? = null,
) : Event

@Serializable
@SerialName("resume")
data class EventResume(
    @SerialName("id") override val id: String? = null,
) : Event

@Serializable
@SerialName("visibility")
data class EventVisibility(
    @SerialName("id") override val id: String? = null,
    @SerialName("visible") val visible: Boolean,
) : Event

@Serializable
@SerialName("network")
data class EventNetwork(
    @SerialName("id") override val id: String? = null,
    @SerialName("online") val online: Boolean,
) : Event

/** Touch, key, wheel or playing video in view. */
@Serializable
@SerialName("activity")
data class EventActivity(
    @SerialName("id") override val id: String? = null,
) : Event

/** An address or deep link to open. */
@Serializable
@SerialName("address")
data class EventAddress(
    @SerialName("id") override val id: String? = null,
    @SerialName("address") val address: String,
) : Event

@Serializable
@SerialName("action")
data class EventAction(
    @SerialName("id") override val id: String? = null,
    @SerialName("action") val action: Action,
) : Event

/** A driver event, stamped with the generation of the open that made it. */
@Serializable
@SerialName("driver")
data class EventDriver(
    @SerialName("id") override val id: String? = null,
    @SerialName("event") val event: DriverEvent,
    @SerialName("generation") val generation: Long,
) : Event

/** A tag this build does not know. */
@Serializable
data class EventUnknown(
    @SerialName("id") val rawId: String? = null,
) : Event {
    override val id: String? get() = rawId
}

/** Feature values. With no answer from the server, accounts are required. */
@Serializable
data class Features(
    @SerialName("accounts_required") val accountsRequired: Boolean? = null,
    @SerialName("clips") val clips: Boolean? = null,
    @SerialName("games") val games: Boolean? = null,
    @SerialName("original_audio") val originalAudio: Boolean? = null,
    /** Tier for visitors. Present only when accounts are not required. */
    @SerialName("visitor_tier") val visitorTier: String? = null,
)

/** Signals a viewer gives. */
@Serializable
@JvmInline
value class FeedbackEvent(val value: String) {
    companion object {
        val CLICK = FeedbackEvent("click")
        val START = FeedbackEvent("start")
        val COMPLETE = FeedbackEvent("complete")
        val LIKE = FeedbackEvent("like")
        val SKIP = FeedbackEvent("skip")
    }
}

/** Answer, possibly partial when the budget ran out. */
@Serializable
data class FindAnswer(
    /** Read the rest through `/v1/find/{job}`. */
    @SerialName("job") val job: String? = null,
    @SerialName("outcome") val outcome: FindOutcome,
    @SerialName("sources") val sources: List<FoundSource>,
)

/** How a search for sources ended. */
@Serializable
@JvmInline
value class FindOutcome(val value: String) {
    companion object {
        val FOUND = FindOutcome("found")
        val PARTIAL = FindOutcome("partial")
        val NOT_FOUND = FindOutcome("not_found")
        val PENDING = FindOutcome("pending")
    }
}

/** Title facts come from the server, so a finder never looks them up. */
@Serializable
data class FindRequest(
    /** Total time this search may take. */
    @SerialName("budget_ms") val budgetMs: Long,
    @SerialName("episode") val episode: EpisodeRef? = null,
    @SerialName("name") val name: String,
    @SerialName("title") val title: TitleKey,
    @SerialName("year") val year: Long? = null,
)

/** `/v1/health` of a finder. */
@Serializable
data class FinderHealth(
    @SerialName("build") val build: String,
    @SerialName("state") val state: ServiceState,
)

/** The form a client draws in. A browser on a phone is class web, form phone. */
@Serializable
@JvmInline
value class Form(val value: String) {
    companion object {
        val PHONE = Form("phone")
        val TABLET = Form("tablet")
        val DESKTOP = Form("desktop")
        val TV = Form("tv")
    }
}

/** One source as facts. The server turns these into ranked attempts. */
@Serializable
data class FoundSource(
    @SerialName("audio") val audio: List<String>? = null,
    @SerialName("container") val container: String? = null,
    @SerialName("group") val group: String,
    @SerialName("headers") val headers: Map<String, String>,
    @SerialName("height") val height: Long? = null,
    @SerialName("kind") val kind: AttemptKind,
    @SerialName("url") val url: String,
    @SerialName("verified") val verified: Boolean,
)

/** True while the who is watching prompt should show. */
@Serializable
data class GateView(
    @SerialName("ask") val ask: Boolean,
)

/** First answer a client reads. */
@Serializable
data class Hello(
    @SerialName("brand") val brand: BrandValues,
    @SerialName("client_min") val clientMin: List<ClientFloor>,
    /** Wire dates this server answers. */
    @SerialName("contract") val contract: VersionRange,
    @SerialName("features") val features: Features,
    /** Builds refused by name. */
    @SerialName("killed_builds") val killedBuilds: List<String>? = null,
    @SerialName("limits") val limits: Limits,
    @SerialName("moved") val moved: List<MovedAddress>? = null,
    /** False stops playback everywhere with a notice. */
    @SerialName("playback") val playback: Boolean,
    @SerialName("rollouts") val rollouts: List<Rollout>? = null,
    @SerialName("start_deadline_ms") val startDeadlineMs: List<ClassValue>,
    /** Seconds this answer stays good. */
    @SerialName("ttl_s") val ttlS: Long,
)

/** `POST /v1/core/step`. `config` is sent on the first step of a hosted session. */
@Serializable
data class HostedStep(
    @SerialName("config") val config: CoreConfig? = null,
    @SerialName("input") val input: CoreIn,
)

/** How an http effect may be handled. */
@Serializable
@JvmInline
value class HttpClass(val value: String) {
    companion object {
        val COMMAND = HttpClass("command")
        val CACHEABLE = HttpClass("cacheable")
        val PLAIN = HttpClass("plain")
    }
}

/** Methods of an http effect. */
@Serializable
@JvmInline
value class HttpMethod(val value: String) {
    companion object {
        val GET = HttpMethod("GET")
        val POST = HttpMethod("POST")
        val PUT = HttpMethod("PUT")
        val PATCH = HttpMethod("PATCH")
        val DELETE = HttpMethod("DELETE")
    }
}

/** One image address. Images go straight from the host to the viewer. */
@Serializable
data class Image(
    @SerialName("role") val role: ImageRole,
    @SerialName("url") val url: String,
    @SerialName("width") val width: Long? = null,
)

/** What an image is for. */
@Serializable
@JvmInline
value class ImageRole(val value: String) {
    companion object {
        val POSTER = ImageRole("poster")
        val BACKDROP = ImageRole("backdrop")
        val LOGO = ImageRole("logo")
        val STILL = ImageRole("still")
        val AVATAR = ImageRole("avatar")
    }
}

/** Input a device has. */
@Serializable
@JvmInline
value class InputKind(val value: String) {
    companion object {
        val TOUCH = InputKind("touch")
        val POINTER = InputKind("pointer")
        val KEYBOARD = InputKind("keyboard")
        val REMOTE = InputKind("remote")
        val GAMEPAD = InputKind("gamepad")
        val ROTARY = InputKind("rotary")
    }
}

/** Limits served from one place, so no client holds a copy. */
@Serializable
data class Limits(
    @SerialName("blocked_title_max") val blockedTitleMax: Long,
    @SerialName("blocked_titles_max") val blockedTitlesMax: Long,
    @SerialName("code_digits") val codeDigits: Long,
    @SerialName("code_valid_s") val codeValidS: Long,
    @SerialName("command_batch_max") val commandBatchMax: Long,
    @SerialName("handle_max") val handleMax: Long,
    @SerialName("handle_min") val handleMin: Long,
    @SerialName("name_max") val nameMax: Long,
    @SerialName("password_min") val passwordMin: Long,
    @SerialName("phone_digits_max") val phoneDigitsMax: Long,
    @SerialName("phone_digits_min") val phoneDigitsMin: Long,
    @SerialName("pin_digits") val pinDigits: Long,
    @SerialName("profile_name_max") val profileNameMax: Long,
    @SerialName("profiles_max") val profilesMax: Long,
    @SerialName("step_up_s") val stepUpS: Long,
)

/** List state of a title, versioned apart from its rating. */
@Serializable
data class ListState(
    @SerialName("changed_at_ms") val changedAtMs: Long,
    @SerialName("in_list") val inList: Boolean,
    @SerialName("version") val version: Long,
)

/** Log levels. */
@Serializable
@JvmInline
value class LogLevel(val value: String) {
    companion object {
        val DEBUG = LogLevel("debug")
        val INFO = LogLevel("info")
        val WARN = LogLevel("warn")
        val ERROR = LogLevel("error")
    }
}

/** Skip marks. */
@Serializable
data class Marks(
    @SerialName("windows") val windows: List<TimeWindow>,
)

/** Ask for skip marks, after first frame, series only. */
@Serializable
data class MarksRequest(
    @SerialName("attempt") val attempt: String,
    @SerialName("delay_ms") val delayMs: Long? = null,
    @SerialName("duration_ms") val durationMs: Long,
    @SerialName("file") val file: String? = null,
)

/** The next part of a screen or a section. */
@Serializable
data class More(
    @SerialName("cursor") val cursor: Cursor? = null,
    @SerialName("sections") val sections: List<Section>? = null,
    @SerialName("tiles") val tiles: List<Tile>? = null,
)

/** Late attempts. */
@Serializable
data class MoreAttempts(
    @SerialName("attempts") val attempts: List<Attempt>,
    @SerialName("more") val more: Boolean,
)

/** One row of the moved address table. */
@Serializable
data class MovedAddress(
    @SerialName("from") val from: String,
    @SerialName("to") val to: String,
)

/** The menu for this class and profile. */
@Serializable
data class Nav(
    @SerialName("items") val items: List<NavItem>,
    @SerialName("ttl_s") val ttlS: Long,
)

/** One entry of the menu. */
@Serializable
data class NavItem(
    @SerialName("genre") val genre: String? = null,
    @SerialName("id") val id: String,
    @SerialName("label") val label: String,
    @SerialName("screen") val screen: ScreenName,
)

/** Why an http effect got no answer. Any status at all is an answer. */
@Serializable
@JvmInline
value class NoAnswerReason(val value: String) {
    companion object {
        val OFFLINE = NoAnswerReason("offline")
        val TIMEOUT = NoAnswerReason("timeout")
        val TLS = NoAnswerReason("tls")
        val CLOCK_WRONG = NoAnswerReason("clock_wrong")
        val CANCELLED = NoAnswerReason("cancelled")
    }
}

/** Text a client can draw with no other knowledge, including a client below the floor. */
@Serializable
data class Notice(
    @SerialName("action_address") val actionAddress: String? = null,
    @SerialName("action_label") val actionLabel: String? = null,
    /** Label key, for clients with the strings. */
    @SerialName("label") val label: String,
    /** Plain text in the viewer's language. */
    @SerialName("text") val text: String,
)

/** Answer for one op. */
@Serializable
data class OpAnswer(
    @SerialName("op_id") val opId: String,
    @SerialName("outcome") val outcome: Outcome,
)

/** Answer per op. */
@Serializable
@JvmInline
value class Outcome(val value: String) {
    companion object {
        val APPLIED = Outcome("applied")
        val SUPERSEDED = Outcome("superseded")
        val REJECTED = Outcome("rejected")
    }
}

@Serializable
data class PasswordChange(
    @SerialName("new_password") val newPassword: String,
)

@Serializable
data class PasswordSignIn(
    @SerialName("email") val email: String,
    @SerialName("password") val password: String,
)

@Serializable
data class PhoneChange(
    @SerialName("phone") val phone: String,
)

/** Absent clears the PIN. */
@Serializable
data class PinSet(
    @SerialName("pin") val pin: String? = null,
)

/** Answer to a play request. */
@Serializable
data class PlayAnswer(
    @SerialName("age_card") val ageCard: AgeCard? = null,
    @SerialName("attempts") val attempts: List<Attempt>,
    @SerialName("audio_default") val audioDefault: String? = null,
    /** Attempts removed by the capability card. */
    @SerialName("dropped") val dropped: Long,
    /** True when more attempts may arrive through `/v1/play/{id}/more`. */
    @SerialName("more") val more: Boolean,
    @SerialName("next_episode") val nextEpisode: EpisodeRef? = null,
    @SerialName("play_id") val playId: String,
    /** Where to start. */
    @SerialName("start_at_ms") val startAtMs: Long,
    @SerialName("subtitle_default") val subtitleDefault: String? = null,
)

/** One thing that happened during play. */
@Serializable
data class PlayEvent(
    @SerialName("at_ms") val atMs: Long,
    @SerialName("attempt") val attempt: String? = null,
    @SerialName("failure") val failure: ErrorClass? = null,
    @SerialName("first_frame_ms") val firstFrameMs: Long? = null,
    @SerialName("host") val host: String? = null,
    @SerialName("name") val name: PlayEventName,
    @SerialName("rung") val rung: CoreMode? = null,
)

/** What a play report says happened. */
@Serializable
@JvmInline
value class PlayEventName(val value: String) {
    companion object {
        val FIRST_FRAME = PlayEventName("first_frame")
        val REBUFFER = PlayEventName("rebuffer")
        val FAILURE = PlayEventName("failure")
        val ENDED = PlayEventName("ended")
    }
}

/** `/v1/play/{id}/events`. */
@Serializable
data class PlayEvents(
    @SerialName("events") val events: List<PlayEvent>,
)

/** Ask to play a title. */
@Serializable
data class PlayRequest(
    @SerialName("episode") val episode: EpisodeRef? = null,
    @SerialName("title") val title: TitleKey,
)

/** Where the play session stands. */
@Serializable
@JvmInline
value class PlayState(val value: String) {
    companion object {
        val RESOLVING = PlayState("resolving")
        val STARTING = PlayState("starting")
        val PLAYING = PlayState("playing")
        val PAUSED = PlayState("paused")
        val STALLED = PlayState("stalled")
        val SWITCHING = PlayState("switching")
        val ENDED = PlayState("ended")
        val FAILED = PlayState("failed")
    }
}

/** Play session state for screens. */
@Serializable
data class PlayView(
    @SerialName("attempt") val attempt: String? = null,
    @SerialName("episode") val episode: EpisodeRef? = null,
    @SerialName("state") val state: PlayState,
    @SerialName("title") val title: TitleKey,
    @SerialName("windows") val windows: List<TimeWindow>? = null,
)

/** One profile. */
@Serializable
data class Profile(
    @SerialName("avatar") val avatar: String? = null,
    @SerialName("has_pin") val hasPin: Boolean,
    @SerialName("id") val id: String,
    @SerialName("kids") val kids: Boolean,
    @SerialName("main") val main: Boolean,
    @SerialName("name") val name: String,
)

@Serializable
data class ProfileCreate(
    @SerialName("kids") val kids: Boolean? = null,
    @SerialName("name") val name: String,
)

@Serializable
data class ProfileEnter(
    @SerialName("pin") val pin: String? = null,
)

/** The profile answer names its home, so the core can map it to a target. */
@Serializable
data class ProfileEntered(
    @SerialName("home") val home: ScreenName,
    @SerialName("profile") val profile: Profile,
)

/** Needs a step up. Never reachable through a queued command. */
@Serializable
data class ProfileLimits(
    @SerialName("age_rating") val ageRating: String,
    @SerialName("blocked_titles") val blockedTitles: List<String>,
)

@Serializable
data class ProfileList(
    @SerialName("profiles") val profiles: List<Profile>,
)

@Serializable
data class ProfileSettings(
    @SerialName("audio_language") val audioLanguage: String? = null,
    @SerialName("autoplay_next") val autoplayNext: Boolean,
    @SerialName("autoplay_previews") val autoplayPreviews: Boolean,
    @SerialName("gender") val gender: String? = null,
    @SerialName("handle") val handle: String? = null,
    @SerialName("subtitle_language") val subtitleLanguage: String? = null,
    @SerialName("subtitle_style") val subtitleStyle: Map<String, String>? = null,
    @SerialName("ui_language") val uiLanguage: String? = null,
)

@Serializable
data class ProfileTransfer(
    @SerialName("to_account_email") val toAccountEmail: String,
)

@Serializable
data class ProfileUpdate(
    @SerialName("avatar") val avatar: String? = null,
    @SerialName("kids") val kids: Boolean? = null,
    @SerialName("name") val name: String? = null,
)

/** A position, key by title and episode. */
@Serializable
data class ProgressState(
    @SerialName("changed_at_ms") val changedAtMs: Long,
    @SerialName("completed") val completed: Boolean,
    @SerialName("duration_s") val durationS: Long,
    @SerialName("episode") val episode: EpisodeRef? = null,
    @SerialName("hidden") val hidden: Boolean,
    @SerialName("position_s") val positionS: Long,
    @SerialName("title") val title: TitleKey,
    @SerialName("version") val version: Long,
)

/** Rating words on the wire. Screens use the words like, love and dislike. */
@Serializable
@JvmInline
value class Rating(val value: String) {
    companion object {
        val DOWN = Rating("down")
        val UP = Rating("up")
        val LOVE = Rating("love")
    }
}

/** Rating state of a title, versioned apart from its list state. */
@Serializable
data class RatingState(
    @SerialName("changed_at_ms") val changedAtMs: Long,
    /** Absent when no rating is set. */
    @SerialName("rating") val rating: Rating? = null,
    @SerialName("version") val version: Long,
)

/** `/v1/ready`, for outside checks. */
@Serializable
data class Ready(
    @SerialName("build") val build: String,
    @SerialName("data_age_s") val dataAgeS: Long,
    @SerialName("data_hash") val dataHash: String,
    @SerialName("finder") val finder: ServiceState,
    @SerialName("index") val index: ServiceState,
)

@Serializable
data class RecoveryFinish(
    @SerialName("code") val code: String,
    @SerialName("email") val email: String,
    @SerialName("new_password") val newPassword: String,
)

@Serializable
data class RecoveryStart(
    @SerialName("email") val email: String,
)

@Serializable
data class RefreshRequest(
    @SerialName("refresh_token") val refreshToken: String,
)

/** Events after which a screen should be read again. */
@Serializable
@JvmInline
value class RefreshTrigger(val value: String) {
    companion object {
        val FOCUS = RefreshTrigger("focus")
        val SIGN_IN = RefreshTrigger("sign_in")
        val PROFILE_CHANGE = RefreshTrigger("profile_change")
        val LIST_CHANGE = RefreshTrigger("list_change")
        val HISTORY_CHANGE = RefreshTrigger("history_change")
    }
}

/** `/v1/reports`. */
@Serializable
data class Report(
    @SerialName("build") val build: String,
    @SerialName("code") val code: String,
    @SerialName("detail") val detail: String? = null,
)

/** A staged rollout. The bucket comes from the household, so a home moves together. */
@Serializable
data class Rollout(
    @SerialName("name") val name: String,
    @SerialName("percent") val percent: Long,
)

/** What an address opens. */
@Serializable
data class RouteTarget(
    @SerialName("episode") val episode: EpisodeRef? = null,
    /** The genre, for the genre screen. */
    @SerialName("genre") val genre: String? = null,
    /** A safe address to return to after sign in. */
    @SerialName("next") val next: String? = null,
    @SerialName("screen") val screen: ScreenName? = null,
    @SerialName("target") val target: Target,
    @SerialName("title") val title: TitleKey? = null,
)

/** A built page. */
@Serializable
data class Screen(
    /** Reads more sections through `/v1/more/{cursor}`. */
    @SerialName("cursor") val cursor: Cursor? = null,
    @SerialName("name") val name: ScreenName,
    @SerialName("refresh_on") val refreshOn: List<RefreshTrigger>? = null,
    @SerialName("sections") val sections: List<Section>,
    /** Seconds this answer stays good. */
    @SerialName("ttl_s") val ttlS: Long,
)

/** A page body handed to a screen. It stands in for the answer to an http effect whose body was going to the screen. */
@Serializable
data class ScreenBody(
    @SerialName("body") val body: String,
    /** Id of the http effect this answers. */
    @SerialName("request") val request: String,
    /** True when this copy is past its ttl and was kept for when the network is gone. */
    @SerialName("stale") val stale: Boolean? = null,
)

/** The screens the server builds. */
@Serializable
@JvmInline
value class ScreenName(val value: String) {
    companion object {
        val HOME = ScreenName("home")
        val FILMS = ScreenName("films")
        val SERIES = ScreenName("series")
        val GENRE = ScreenName("genre")
        val NEW = ScreenName("new")
        val MY_LIST = ScreenName("my-list")
        val KIDS = ScreenName("kids")
        val SEARCH = ScreenName("search")
        val NOTIFICATIONS = ScreenName("notifications")
        val ACTIVITY = ScreenName("activity")
        val GAMES = ScreenName("games")
        val CLIPS = ScreenName("clips")
        val LANGUAGE = ScreenName("language")
    }
}

/** One season with its episodes. */
@Serializable
data class Season(
    @SerialName("episodes") val episodes: List<Episode>,
    @SerialName("name") val name: String? = null,
    @SerialName("number") val number: Long,
)

/** A season named on a title page. */
@Serializable
data class SeasonSummary(
    @SerialName("episode_count") val episodeCount: Long,
    @SerialName("number") val number: Long,
)

/** Secret operations. */
@Serializable
@JvmInline
value class SecretOp(val value: String) {
    companion object {
        val GET = SecretOp("get")
        val PUT = SecretOp("put")
        val DELETE = SecretOp("delete")
    }
}

/** One row, hero, grid or notice of a screen. */
@Serializable
data class Section(
    /** Reads more of this section through `/v1/more/{cursor}`. */
    @SerialName("cursor") val cursor: Cursor? = null,
    @SerialName("id") val id: String,
    @SerialName("kind") val kind: SectionKind,
    @SerialName("notice") val notice: Notice? = null,
    @SerialName("tiles") val tiles: List<Tile>? = null,
    /** Translated title. */
    @SerialName("title") val title: String? = null,
)

/** Section kinds. A kind this build does not draw falls back to a plain row. */
@Serializable
@JvmInline
value class SectionKind(val value: String) {
    companion object {
        val HERO = SectionKind("hero")
        val ROW = SectionKind("row")
        val RANKED = SectionKind("ranked")
        val CONTINUE = SectionKind("continue")
        val GRID = SectionKind("grid")
        val NOTICE = SectionKind("notice")
    }
}

/** State of a part of the service. */
@Serializable
@JvmInline
value class ServiceState(val value: String) {
    companion object {
        val READY = ServiceState("ready")
        val DEGRADED = ServiceState("degraded")
        val DOWN = ServiceState("down")
    }
}

/** Where sign in stands. */
@Serializable
@JvmInline
value class SessionState(val value: String) {
    companion object {
        val STARTING = SessionState("starting")
        val SIGNED_OUT = SessionState("signed_out")
        val SIGNED_IN = SessionState("signed_in")
    }
}

/** Sign in state for screens. Tokens never appear here. */
@Serializable
data class SessionView(
    @SerialName("email") val email: String? = null,
    @SerialName("ended") val ended: EndReason? = null,
    @SerialName("expires_at_ms") val expiresAtMs: Long? = null,
    @SerialName("profile") val profile: String? = null,
    /** Opaque scope of this account and profile, the same value http effects carry. */
    @SerialName("scope") val scope: String? = null,
    @SerialName("state") val state: SessionState,
    @SerialName("user_id") val userId: String? = null,
)

/** Settings a queued command may change. Age rating and blocked titles are absent on purpose: they need a step up, so a queued write can never touch them. */
@Serializable
@JvmInline
value class SettingKey(val value: String) {
    companion object {
        val AUTOPLAY_NEXT = SettingKey("autoplay_next")
        val AUTOPLAY_PREVIEWS = SettingKey("autoplay_previews")
        val AUDIO_LANGUAGE = SettingKey("audio_language")
        val SUBTITLE_LANGUAGE = SettingKey("subtitle_language")
        val SUBTITLE_STYLE = SettingKey("subtitle_style")
        val UI_LANGUAGE = SettingKey("ui_language")
        val HANDLE = SettingKey("handle")
        val GENDER = SettingKey("gender")
    }
}

@Serializable
data class SignOut(
    /** True ends every session of this account. */
    @SerialName("everywhere") val everywhere: Boolean,
)

@Serializable
data class SignUp(
    @SerialName("email") val email: String,
    @SerialName("first_name") val firstName: String,
    @SerialName("last_name") val lastName: String,
    @SerialName("password") val password: String,
)

/** Opaque tokens. `expires_in` is seconds, so no device clock decides anything. */
@Serializable
data class SignedIn(
    @SerialName("access_token") val accessToken: String,
    @SerialName("email") val email: String,
    @SerialName("expires_in") val expiresIn: Long,
    @SerialName("refresh_token") val refreshToken: String,
    @SerialName("user_id") val userId: String,
)

/** Store operations. Only the offline layer uses store. */
@Serializable
@JvmInline
value class StoreOp(val value: String) {
    companion object {
        val GET = StoreOp("get")
        val PUT = StoreOp("put")
        val DELETE = StoreOp("delete")
        val LIST = StoreOp("list")
    }
}

/** `/v1/strings/{lang}`. Text holds the placeholders below. */
@Serializable
data class Strings(
    @SerialName("lang") val lang: String,
    @SerialName("texts") val texts: Map<String, String>,
)

/** `/v1/subtitles/{file}`. The core checks `last_end_ms` against the driver's duration. */
@Serializable
data class SubtitleBody(
    @SerialName("cues") val cues: List<Cue>,
    @SerialName("last_end_ms") val lastEndMs: Long,
)

/** One subtitle file of a language, in the order to try. */
@Serializable
data class SubtitleFile(
    @SerialName("hearing_impaired") val hearingImpaired: Boolean? = null,
    @SerialName("id") val id: String,
)

/** One language. */
@Serializable
data class SubtitleLanguage(
    @SerialName("files") val files: List<SubtitleFile>,
    @SerialName("label") val label: String,
    @SerialName("lang") val lang: String,
)

/** `/v1/subtitles`. */
@Serializable
data class SubtitleList(
    @SerialName("languages") val languages: List<SubtitleLanguage>,
)

/** Search suggestions. */
@Serializable
data class Suggestions(
    @SerialName("items") val items: List<String>,
)

/** A semantic target. Each device maps it to one of its own screens. */
@Serializable
@JvmInline
value class Target(val value: String) {
    companion object {
        val START = Target("start")
        val SIGN_IN = Target("sign_in")
        val PROFILES = Target("profiles")
        val HOME = Target("home")
        val KIDS_HOME = Target("kids_home")
        val SCREEN = Target("screen")
        val TITLE = Target("title")
        val WATCH = Target("watch")
        val NOT_FOUND = Target("not_found")
        val WAIT = Target("wait")
        val UPGRADE = Target("upgrade")
    }
}

/** One title in a section. */
@Serializable
data class Tile(
    @SerialName("badges") val badges: List<Badge>? = null,
    @SerialName("episode") val episode: EpisodeRef? = null,
    @SerialName("images") val images: List<Image>? = null,
    @SerialName("name") val name: String,
    /** Whole percent watched. */
    @SerialName("percent") val percent: Long? = null,
    @SerialName("tags") val tags: List<String>? = null,
    @SerialName("title") val title: TitleKey,
)

/** A span of a video the host compares to player time. Nothing ticks across the boundary. */
@Serializable
data class TimeWindow(
    @SerialName("end_ms") val endMs: Long,
    @SerialName("kind") val kind: WindowKind,
    /** Where a skip button moves to. */
    @SerialName("seek_to_ms") val seekToMs: Long? = null,
    @SerialName("start_ms") val startMs: Long,
    @SerialName("text") val text: String? = null,
)

/** Timer operations. A clear uses the id of the timer it clears. */
@Serializable
@JvmInline
value class TimerOp(val value: String) {
    companion object {
        val SET = TimerOp("set")
        val CLEAR = TimerOp("clear")
    }
}

/** Facts of one title. */
@Serializable
data class TitleDetail(
    @SerialName("genres") val genres: List<String>? = null,
    @SerialName("images") val images: List<Image>? = null,
    /** Hidden when empty. */
    @SerialName("more_like_this") val moreLikeThis: List<Tile>? = null,
    @SerialName("name") val name: String,
    @SerialName("overview") val overview: String? = null,
    @SerialName("runtime_min") val runtimeMin: Long? = null,
    @SerialName("seasons") val seasons: List<SeasonSummary>? = null,
    @SerialName("title") val title: TitleKey,
    @SerialName("year") val year: Long? = null,
)

/** movie:<id> or tv:<id>. A bare id does not parse. */
@Serializable
@JvmInline
value class TitleKey(val value: String) {
    init {
        require(Regex("^(movie|tv):[1-9][0-9]{0,9}\$").matches(value)) { "not a TitleKey" }
    }
}

/** The two kinds of title. Closed on purpose: a new kind reaches old clients only through a dated transform, so a key always names a kind this build can open. */
@Serializable
@JvmInline
value class TitleKind(val value: String) {
    companion object {
        val MOVIE = TitleKind("movie")
        val TV = TitleKind("tv")
    }
}

/** One audio or text track, with the tag the manifest gave it. */
@Serializable
data class Track(
    @SerialName("id") val id: String,
    @SerialName("label") val label: String? = null,
    @SerialName("lang") val lang: String? = null,
)

/** The wire dates a server answers. Never compared for equality. */
@Serializable
data class VersionRange(
    @SerialName("max") val max: WireDate,
    @SerialName("min") val min: WireDate,
)

/** Changed screen state. A part that is absent did not change. */
@Serializable
data class View(
    @SerialName("gate") val gate: GateView? = null,
    @SerialName("play") val play: PlayView? = null,
    /** The account's profiles, for the picker. No PIN is ever held here. */
    @SerialName("profiles") val profiles: ProfileList? = null,
    @SerialName("route") val route: RouteTarget? = null,
    @SerialName("session") val session: SessionView? = null,
)

/** Windows of time inside a video. */
@Serializable
@JvmInline
value class WindowKind(val value: String) {
    companion object {
        val RECAP = WindowKind("recap")
        val INTRO = WindowKind("intro")
        val CREDITS = WindowKind("credits")
        val AGE_CARD = WindowKind("age_card")
        val CUE = WindowKind("cue")
    }
}

/** A wire date, YYYY-MM-DD. */
@Serializable
@JvmInline
value class WireDate(val value: String) {
    init {
        require(Regex("^[0-9]{4}-[0-9]{2}-[0-9]{2}\$").matches(value)) { "not a WireDate" }
    }
}
