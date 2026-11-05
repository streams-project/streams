// Generated from the contract. Do not edit.

import Foundation

public let wireVersion = "2026-11-05"
public let abiVersion = 1

/// A value the contract refuses, named by its type.
public enum ContractError: Error { case refused(String) }

public struct Account: Codable, Equatable, Sendable {
    public var email: String
    public var firstName: String
    public var id: String
    public var lastName: String
    public var phone: String?

    enum CodingKeys: String, CodingKey {
        case email = "email"
        case firstName = "first_name"
        case id = "id"
        case lastName = "last_name"
        case phone = "phone"
    }

    public init(email: String, firstName: String, id: String, lastName: String, phone: String? = nil) {
        self.email = email
        self.firstName = firstName
        self.id = id
        self.lastName = lastName
        self.phone = phone
    }
}

/// Everything held about an account, in plain JSON.
public struct AccountExport: Codable, Equatable, Sendable {
    public var account: Account
    public var devices: [DeviceSession]
    public var profiles: [Profile]

    enum CodingKeys: String, CodingKey {
        case account = "account"
        case devices = "devices"
        case profiles = "profiles"
    }

    public init(account: Account, devices: [DeviceSession], profiles: [Profile]) {
        self.account = account
        self.devices = devices
        self.profiles = profiles
    }
}

public struct AccountUpdate: Codable, Equatable, Sendable {
    public var firstName: String?
    public var lastName: String?

    enum CodingKeys: String, CodingKey {
        case firstName = "first_name"
        case lastName = "last_name"
    }

    public init(firstName: String? = nil, lastName: String? = nil) {
        self.firstName = firstName
        self.lastName = lastName
    }
}

/// What a screen asks for.
public struct ActionSignIn: Codable, Equatable, Sendable {
    public var email: String
    public var password: String

    enum CodingKeys: String, CodingKey {
        case email = "email"
        case password = "password"
    }

    public init(email: String, password: String) {
        self.email = email
        self.password = password
    }
}

public struct ActionSendCode: Codable, Equatable, Sendable {
    public var email: String

    enum CodingKeys: String, CodingKey {
        case email = "email"
    }

    public init(email: String) {
        self.email = email
    }
}

public struct ActionSignInCode: Codable, Equatable, Sendable {
    public var code: String
    public var email: String

    enum CodingKeys: String, CodingKey {
        case code = "code"
        case email = "email"
    }

    public init(code: String, email: String) {
        self.code = code
        self.email = email
    }
}

public struct ActionSignOut: Codable, Equatable, Sendable {
    public var everywhere: Bool

    enum CodingKeys: String, CodingKey {
        case everywhere = "everywhere"
    }

    public init(everywhere: Bool) {
        self.everywhere = everywhere
    }
}

public struct ActionEnterProfile: Codable, Equatable, Sendable {
    public var pin: String?
    public var profile: String

    enum CodingKeys: String, CodingKey {
        case pin = "pin"
        case profile = "profile"
    }

    public init(pin: String? = nil, profile: String) {
        self.pin = pin
        self.profile = profile
    }
}

public struct ActionPlay: Codable, Equatable, Sendable {
    public var episode: EpisodeRef?
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case episode = "episode"
        case title = "title"
    }

    public init(episode: EpisodeRef? = nil, title: TitleKey) {
        self.episode = episode
        self.title = title
    }
}

public struct ActionPause: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct ActionUnpause: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct ActionSeek: Codable, Equatable, Sendable {
    public var toMs: Int

    enum CodingKeys: String, CodingKey {
        case toMs = "to_ms"
    }

    public init(toMs: Int) {
        self.toMs = toMs
    }
}

public struct ActionSelectAudio: Codable, Equatable, Sendable {
    public var track: String

    enum CodingKeys: String, CodingKey {
        case track = "track"
    }

    public init(track: String) {
        self.track = track
    }
}

public struct ActionSelectText: Codable, Equatable, Sendable {
    public var track: String?

    enum CodingKeys: String, CodingKey {
        case track = "track"
    }

    public init(track: String? = nil) {
        self.track = track
    }
}

public struct ActionNextSubtitleFile: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct ActionStop: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct ActionCommand: Codable, Equatable, Sendable {
    public var command: Command

    enum CodingKeys: String, CodingKey {
        case command = "command"
    }

    public init(command: Command) {
        self.command = command
    }
}

/// Send a prepared request with this session's token. The http effect takes the id of this event and its class is always plain, so whoever prepared it gets the answer under that id. Tokens stay in the core.
public struct ActionRequest: Codable, Equatable, Sendable {
    public var body: String?
    public var method: HttpMethod
    public var path: String

    enum CodingKeys: String, CodingKey {
        case body = "body"
        case method = "method"
        case path = "path"
    }

    public init(body: String? = nil, method: HttpMethod, path: String) {
        self.body = body
        self.method = method
        self.path = path
    }
}

public enum Action: Codable, Equatable, Sendable {
    case signIn(ActionSignIn)
    case sendCode(ActionSendCode)
    case signInCode(ActionSignInCode)
    case signOut(ActionSignOut)
    case enterProfile(ActionEnterProfile)
    case play(ActionPlay)
    case pause(ActionPause)
    case unpause(ActionUnpause)
    case seek(ActionSeek)
    case selectAudio(ActionSelectAudio)
    case selectText(ActionSelectText)
    case nextSubtitleFile(ActionNextSubtitleFile)
    case stop(ActionStop)
    case command(ActionCommand)
    case request(ActionRequest)
    /// A tag this build does not know.
    case unknown(String)

    private enum TagKey: String, CodingKey { case tag = "action" }

    public init(from decoder: Decoder) throws {
        let tagged = try decoder.container(keyedBy: TagKey.self)
        let tag = try tagged.decode(String.self, forKey: .tag)
        switch tag {
        case "sign_in": self = .signIn(try ActionSignIn(from: decoder))
        case "send_code": self = .sendCode(try ActionSendCode(from: decoder))
        case "sign_in_code": self = .signInCode(try ActionSignInCode(from: decoder))
        case "sign_out": self = .signOut(try ActionSignOut(from: decoder))
        case "enter_profile": self = .enterProfile(try ActionEnterProfile(from: decoder))
        case "play": self = .play(try ActionPlay(from: decoder))
        case "pause": self = .pause(try ActionPause(from: decoder))
        case "unpause": self = .unpause(try ActionUnpause(from: decoder))
        case "seek": self = .seek(try ActionSeek(from: decoder))
        case "select_audio": self = .selectAudio(try ActionSelectAudio(from: decoder))
        case "select_text": self = .selectText(try ActionSelectText(from: decoder))
        case "next_subtitle_file": self = .nextSubtitleFile(try ActionNextSubtitleFile(from: decoder))
        case "stop": self = .stop(try ActionStop(from: decoder))
        case "command": self = .command(try ActionCommand(from: decoder))
        case "request": self = .request(try ActionRequest(from: decoder))
        default: self = .unknown(tag)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var tagged = encoder.container(keyedBy: TagKey.self)
        switch self {
        case .signIn(let value):
            try tagged.encode("sign_in", forKey: .tag)
            try value.encode(to: encoder)
        case .sendCode(let value):
            try tagged.encode("send_code", forKey: .tag)
            try value.encode(to: encoder)
        case .signInCode(let value):
            try tagged.encode("sign_in_code", forKey: .tag)
            try value.encode(to: encoder)
        case .signOut(let value):
            try tagged.encode("sign_out", forKey: .tag)
            try value.encode(to: encoder)
        case .enterProfile(let value):
            try tagged.encode("enter_profile", forKey: .tag)
            try value.encode(to: encoder)
        case .play(let value):
            try tagged.encode("play", forKey: .tag)
            try value.encode(to: encoder)
        case .pause(let value):
            try tagged.encode("pause", forKey: .tag)
            try value.encode(to: encoder)
        case .unpause(let value):
            try tagged.encode("unpause", forKey: .tag)
            try value.encode(to: encoder)
        case .seek(let value):
            try tagged.encode("seek", forKey: .tag)
            try value.encode(to: encoder)
        case .selectAudio(let value):
            try tagged.encode("select_audio", forKey: .tag)
            try value.encode(to: encoder)
        case .selectText(let value):
            try tagged.encode("select_text", forKey: .tag)
            try value.encode(to: encoder)
        case .nextSubtitleFile(let value):
            try tagged.encode("next_subtitle_file", forKey: .tag)
            try value.encode(to: encoder)
        case .stop(let value):
            try tagged.encode("stop", forKey: .tag)
            try value.encode(to: encoder)
        case .command(let value):
            try tagged.encode("command", forKey: .tag)
            try value.encode(to: encoder)
        case .request(let value):
            try tagged.encode("request", forKey: .tag)
            try value.encode(to: encoder)
        case .unknown(let tag):
            try tagged.encode(tag, forKey: .tag)
        }
    }
}

/// Age card shown once at the start of play.
public struct AgeCard: Codable, Equatable, Sendable {
    public var label: String
    public var reasons: [String]?

    enum CodingKeys: String, CodingKey {
        case label = "label"
        case reasons = "reasons"
    }

    public init(label: String, reasons: [String]? = nil) {
        self.label = label
        self.reasons = reasons
    }
}

/// One way to play. Attempts arrive in the order to try, ranked once on the server. `headers` is required, even when empty, and no provider name is carried.
public struct Attempt: Codable, Equatable, Sendable {
    public var audio: [String]?
    public var codecs: [String]?
    public var container: String?
    /// Wall time, ms since 1970, after which the address stops working.
    public var expiresAtMs: Int?
    /// Opaque group. After a failure the core skips other attempts of this group.
    public var group: String
    /// Headers the player must send, such as Referer, Origin, User-Agent.
    public var headers: [String: String]
    public var height: Int?
    public var id: String
    public var kind: AttemptKind
    public var url: String
    /// True when the finder fetched and read this address.
    public var verified: Bool

    enum CodingKeys: String, CodingKey {
        case audio = "audio"
        case codecs = "codecs"
        case container = "container"
        case expiresAtMs = "expires_at_ms"
        case group = "group"
        case headers = "headers"
        case height = "height"
        case id = "id"
        case kind = "kind"
        case url = "url"
        case verified = "verified"
    }

    public init(audio: [String]? = nil, codecs: [String]? = nil, container: String? = nil, expiresAtMs: Int? = nil, group: String, headers: [String: String], height: Int? = nil, id: String, kind: AttemptKind, url: String, verified: Bool) {
        self.audio = audio
        self.codecs = codecs
        self.container = container
        self.expiresAtMs = expiresAtMs
        self.group = group
        self.headers = headers
        self.height = height
        self.id = id
        self.kind = kind
        self.url = url
        self.verified = verified
    }
}

/// How an attempt is played.
public struct AttemptKind: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let hls = AttemptKind(rawValue: "hls")
    public static let file = AttemptKind(rawValue: "file")
    public static let embed = AttemptKind(rawValue: "embed")
}

/// A mark on a tile.
public struct Badge: Codable, Equatable, Sendable {
    public var kind: BadgeKind
    public var rank: Int?

    enum CodingKeys: String, CodingKey {
        case kind = "kind"
        case rank = "rank"
    }

    public init(kind: BadgeKind, rank: Int? = nil) {
        self.kind = kind
        self.rank = rank
    }
}

/// Marks on a tile.
public struct BadgeKind: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let ranked = BadgeKind(rawValue: "ranked")
    public static let new = BadgeKind(rawValue: "new")
}

/// Where the body of an answer goes. An answer outside 200 to 299 always comes to the core, because the core reads its error code.
public struct BodyTarget: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let core = BodyTarget(rawValue: "core")
    public static let screen = BodyTarget(rawValue: "screen")
}

/// Name and contact, filled into text at render time.
public struct BrandValues: Codable, Equatable, Sendable {
    public var name: String
    public var supportEmail: String

    enum CodingKeys: String, CodingKey {
        case name = "name"
        case supportEmail = "support_email"
    }

    public init(name: String, supportEmail: String) {
        self.name = name
        self.supportEmail = supportEmail
    }
}

/// What a device can do. The server filters attempts and sections by this card.
public struct CapabilityCard: Codable, Equatable, Sendable {
    /// True when the player switches variants during play.
    public var adaptive: Bool
    public var app: String
    /// Attempt kinds this player opens. An embed appears only when listed here.
    public var attemptKinds: [AttemptKind]
    public var `class`: DeviceClass
    /// Codecs this player opens, such as avc1, hvc1, mp4a.
    public var codecs: [String]
    /// Containers this player opens, such as ts, fmp4, mp4.
    public var containers: [String]
    public var form: Form
    public var input: [InputKind]
    public var languages: [String]
    public var maxHeight: Int
    /// Request headers the player can set on media, such as Referer.
    public var mediaHeaders: [String]
    public var os: String
    public var osVersion: String
    /// True when the driver reports video width or has_picture.
    public var reportsPicture: Bool
    public var screenHeight: Int
    public var screenWidth: Int
    /// True when the device has a durable secret store.
    public var secretStore: Bool
    /// Section kinds this build draws. Others fall back to a plain row.
    public var sectionKinds: [SectionKind]
    /// True when the player can load a side subtitle file.
    public var sideSubtitles: Bool
    /// Bytes this device gives to kept commands and screens. 0 means none.
    public var storageBytes: Int
    public var version: String

    enum CodingKeys: String, CodingKey {
        case adaptive = "adaptive"
        case app = "app"
        case attemptKinds = "attempt_kinds"
        case `class` = "class"
        case codecs = "codecs"
        case containers = "containers"
        case form = "form"
        case input = "input"
        case languages = "languages"
        case maxHeight = "max_height"
        case mediaHeaders = "media_headers"
        case os = "os"
        case osVersion = "os_version"
        case reportsPicture = "reports_picture"
        case screenHeight = "screen_height"
        case screenWidth = "screen_width"
        case secretStore = "secret_store"
        case sectionKinds = "section_kinds"
        case sideSubtitles = "side_subtitles"
        case storageBytes = "storage_bytes"
        case version = "version"
    }

    public init(adaptive: Bool, app: String, attemptKinds: [AttemptKind], `class`: DeviceClass, codecs: [String], containers: [String], form: Form, input: [InputKind], languages: [String], maxHeight: Int, mediaHeaders: [String], os: String, osVersion: String, reportsPicture: Bool, screenHeight: Int, screenWidth: Int, secretStore: Bool, sectionKinds: [SectionKind], sideSubtitles: Bool, storageBytes: Int, version: String) {
        self.adaptive = adaptive
        self.app = app
        self.attemptKinds = attemptKinds
        self.`class` = `class`
        self.codecs = codecs
        self.containers = containers
        self.form = form
        self.input = input
        self.languages = languages
        self.maxHeight = maxHeight
        self.mediaHeaders = mediaHeaders
        self.os = os
        self.osVersion = osVersion
        self.reportsPicture = reportsPicture
        self.screenHeight = screenHeight
        self.screenWidth = screenWidth
        self.secretStore = secretStore
        self.sectionKinds = sectionKinds
        self.sideSubtitles = sideSubtitles
        self.storageBytes = storageBytes
        self.version = version
    }
}

/// One change a device reads.
public struct ChangeTitle: Codable, Equatable, Sendable {
    public var list: ListState
    public var rating: RatingState
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case list = "list"
        case rating = "rating"
        case title = "title"
    }

    public init(list: ListState, rating: RatingState, title: TitleKey) {
        self.list = list
        self.rating = rating
        self.title = title
    }
}

public struct ChangeProgress: Codable, Equatable, Sendable {
    public var state: ProgressState

    enum CodingKeys: String, CodingKey {
        case state = "state"
    }

    public init(state: ProgressState) {
        self.state = state
    }
}

/// Hidden as a whole, every episode of a series included.
public struct ChangeHidden: Codable, Equatable, Sendable {
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case title = "title"
    }

    public init(title: TitleKey) {
        self.title = title
    }
}

public struct ChangeHistoryCleared: Codable, Equatable, Sendable {
    public var changedAtMs: Int

    enum CodingKeys: String, CodingKey {
        case changedAtMs = "changed_at_ms"
    }

    public init(changedAtMs: Int) {
        self.changedAtMs = changedAtMs
    }
}

public struct ChangeSetting: Codable, Equatable, Sendable {
    public var key: SettingKey
    public var value: String

    enum CodingKeys: String, CodingKey {
        case key = "key"
        case value = "value"
    }

    public init(key: SettingKey, value: String) {
        self.key = key
        self.value = value
    }
}

public enum Change: Codable, Equatable, Sendable {
    case title(ChangeTitle)
    case progress(ChangeProgress)
    case hidden(ChangeHidden)
    case historyCleared(ChangeHistoryCleared)
    case setting(ChangeSetting)
    /// A tag this build does not know.
    case unknown(String)

    private enum TagKey: String, CodingKey { case tag = "change" }

    public init(from decoder: Decoder) throws {
        let tagged = try decoder.container(keyedBy: TagKey.self)
        let tag = try tagged.decode(String.self, forKey: .tag)
        switch tag {
        case "title": self = .title(try ChangeTitle(from: decoder))
        case "progress": self = .progress(try ChangeProgress(from: decoder))
        case "hidden": self = .hidden(try ChangeHidden(from: decoder))
        case "history_cleared": self = .historyCleared(try ChangeHistoryCleared(from: decoder))
        case "setting": self = .setting(try ChangeSetting(from: decoder))
        default: self = .unknown(tag)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var tagged = encoder.container(keyedBy: TagKey.self)
        switch self {
        case .title(let value):
            try tagged.encode("title", forKey: .tag)
            try value.encode(to: encoder)
        case .progress(let value):
            try tagged.encode("progress", forKey: .tag)
            try value.encode(to: encoder)
        case .hidden(let value):
            try tagged.encode("hidden", forKey: .tag)
            try value.encode(to: encoder)
        case .historyCleared(let value):
            try tagged.encode("history_cleared", forKey: .tag)
            try value.encode(to: encoder)
        case .setting(let value):
            try tagged.encode("setting", forKey: .tag)
            try value.encode(to: encoder)
        case .unknown(let tag):
            try tagged.encode(tag, forKey: .tag)
        }
    }
}

/// `GET /v1/changes`.
public struct Changes: Codable, Equatable, Sendable {
    public var changes: [Change]
    public var cursor: Cursor
    public var hasMore: Bool

    enum CodingKeys: String, CodingKey {
        case changes = "changes"
        case cursor = "cursor"
        case hasMore = "has_more"
    }

    public init(changes: [Change], cursor: Cursor, hasMore: Bool) {
        self.changes = changes
        self.cursor = cursor
        self.hasMore = hasMore
    }
}

/// One value per client class.
public struct ClassValue: Codable, Equatable, Sendable {
    public var `class`: DeviceClass
    public var value: Int

    enum CodingKeys: String, CodingKey {
        case `class` = "class"
        case value = "value"
    }

    public init(`class`: DeviceClass, value: Int) {
        self.`class` = `class`
        self.value = value
    }
}

/// Lowest app version a class may run.
public struct ClientFloor: Codable, Equatable, Sendable {
    public var `class`: DeviceClass
    public var version: String

    enum CodingKeys: String, CodingKey {
        case `class` = "class"
        case version = "version"
    }

    public init(`class`: DeviceClass, version: String) {
        self.`class` = `class`
        self.version = version
    }
}

public struct CodeSend: Codable, Equatable, Sendable {
    public var email: String

    enum CodingKeys: String, CodingKey {
        case email = "email"
    }

    public init(email: String) {
        self.email = email
    }
}

/// A code was sent, or would have been. The answer never says whether the account exists.
public struct CodeSent: Codable, Equatable, Sendable {
    public var expiresIn: Int

    enum CodingKeys: String, CodingKey {
        case expiresIn = "expires_in"
    }

    public init(expiresIn: Int) {
        self.expiresIn = expiresIn
    }
}

public struct CodeSignIn: Codable, Equatable, Sendable {
    public var code: String
    public var email: String

    enum CodingKeys: String, CodingKey {
        case code = "code"
        case email = "email"
    }

    public init(code: String, email: String) {
        self.code = code
        self.email = email
    }
}

/// One change to a profile's facts.
/// Add to or remove from My List. Leaves the rating alone.
public struct CommandList: Codable, Equatable, Sendable {
    public var inList: Bool
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case inList = "in_list"
        case title = "title"
    }

    public init(inList: Bool, title: TitleKey) {
        self.inList = inList
        self.title = title
    }
}

/// Set or clear a rating. Leaves the list entry alone.
public struct CommandRate: Codable, Equatable, Sendable {
    public var rating: Rating?
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case rating = "rating"
        case title = "title"
    }

    public init(rating: Rating? = nil, title: TitleKey) {
        self.rating = rating
        self.title = title
    }
}

/// Hide a title from history. For a series this covers every episode.
public struct CommandHide: Codable, Equatable, Sendable {
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case title = "title"
    }

    public init(title: TitleKey) {
        self.title = title
    }
}

/// Clear all history of this profile, as one operation.
public struct CommandClearHistory: Codable, Equatable, Sendable {

    public init() {
    }
}

/// Save a position, in whole seconds.
public struct CommandPosition: Codable, Equatable, Sendable {
    public var durationS: Int
    public var episode: EpisodeRef?
    public var positionS: Int
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case durationS = "duration_s"
        case episode = "episode"
        case positionS = "position_s"
        case title = "title"
    }

    public init(durationS: Int, episode: EpisodeRef? = nil, positionS: Int, title: TitleKey) {
        self.durationS = durationS
        self.episode = episode
        self.positionS = positionS
        self.title = title
    }
}

/// Change one profile setting. Booleans as true or false, style as JSON text.
public struct CommandSetting: Codable, Equatable, Sendable {
    public var key: SettingKey
    public var value: String

    enum CodingKeys: String, CodingKey {
        case key = "key"
        case value = "value"
    }

    public init(key: SettingKey, value: String) {
        self.key = key
        self.value = value
    }
}

/// A viewer signal. Discarded by the server when consent is off.
public struct CommandFeedback: Codable, Equatable, Sendable {
    public var event: FeedbackEvent
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case event = "event"
        case title = "title"
    }

    public init(event: FeedbackEvent, title: TitleKey) {
        self.event = event
        self.title = title
    }
}

/// A trailer failed to play for this title.
public struct CommandTrailerRefused: Codable, Equatable, Sendable {
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case title = "title"
    }

    public init(title: TitleKey) {
        self.title = title
    }
}

public enum Command: Codable, Equatable, Sendable {
    case list(CommandList)
    case rate(CommandRate)
    case hide(CommandHide)
    case clearHistory(CommandClearHistory)
    case position(CommandPosition)
    case setting(CommandSetting)
    case feedback(CommandFeedback)
    case trailerRefused(CommandTrailerRefused)
    /// A tag this build does not know.
    case unknown(String)

    private enum TagKey: String, CodingKey { case tag = "command" }

    public init(from decoder: Decoder) throws {
        let tagged = try decoder.container(keyedBy: TagKey.self)
        let tag = try tagged.decode(String.self, forKey: .tag)
        switch tag {
        case "list": self = .list(try CommandList(from: decoder))
        case "rate": self = .rate(try CommandRate(from: decoder))
        case "hide": self = .hide(try CommandHide(from: decoder))
        case "clear_history": self = .clearHistory(try CommandClearHistory(from: decoder))
        case "position": self = .position(try CommandPosition(from: decoder))
        case "setting": self = .setting(try CommandSetting(from: decoder))
        case "feedback": self = .feedback(try CommandFeedback(from: decoder))
        case "trailer_refused": self = .trailerRefused(try CommandTrailerRefused(from: decoder))
        default: self = .unknown(tag)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var tagged = encoder.container(keyedBy: TagKey.self)
        switch self {
        case .list(let value):
            try tagged.encode("list", forKey: .tag)
            try value.encode(to: encoder)
        case .rate(let value):
            try tagged.encode("rate", forKey: .tag)
            try value.encode(to: encoder)
        case .hide(let value):
            try tagged.encode("hide", forKey: .tag)
            try value.encode(to: encoder)
        case .clearHistory(let value):
            try tagged.encode("clear_history", forKey: .tag)
            try value.encode(to: encoder)
        case .position(let value):
            try tagged.encode("position", forKey: .tag)
            try value.encode(to: encoder)
        case .setting(let value):
            try tagged.encode("setting", forKey: .tag)
            try value.encode(to: encoder)
        case .feedback(let value):
            try tagged.encode("feedback", forKey: .tag)
            try value.encode(to: encoder)
        case .trailerRefused(let value):
            try tagged.encode("trailer_refused", forKey: .tag)
            try value.encode(to: encoder)
        case .unknown(let tag):
            try tagged.encode(tag, forKey: .tag)
        }
    }
}

/// Answer for a batch, one entry per op.
public struct CommandAnswer: Codable, Equatable, Sendable {
    public var answers: [OpAnswer]

    enum CodingKeys: String, CodingKey {
        case answers = "answers"
    }

    public init(answers: [OpAnswer]) {
        self.answers = answers
    }
}

/// `POST /v1/commands`. Idempotent by op id.
public struct CommandBatch: Codable, Equatable, Sendable {
    public var ops: [CommandOp]

    enum CodingKeys: String, CodingKey {
        case ops = "ops"
    }

    public init(ops: [CommandOp]) {
        self.ops = ops
    }
}

/// One queued op. A missing `age_ms` means age unknown, and the server orders by arrival.
public struct CommandOp: Codable, Equatable, Sendable {
    public var ageMs: Int?
    public var opId: String
    public var body: CommandOpBody

    private enum CodingKeys: String, CodingKey {
        case ageMs = "age_ms"
        case opId = "op_id"
    }

    public init(ageMs: Int? = nil, opId: String, body: CommandOpBody) {
        self.ageMs = ageMs
        self.opId = opId
        self.body = body
    }

    public init(from decoder: Decoder) throws {
        let keyed = try decoder.container(keyedBy: CodingKeys.self)
        self.ageMs = try keyed.decodeIfPresent(Int.self, forKey: .ageMs)
        self.opId = try keyed.decode(String.self, forKey: .opId)
        self.body = try CommandOpBody(from: decoder)
    }

    public func encode(to encoder: Encoder) throws {
        var keyed = encoder.container(keyedBy: CodingKeys.self)
        try keyed.encodeIfPresent(ageMs, forKey: .ageMs)
        try keyed.encode(opId, forKey: .opId)
        try body.encode(to: encoder)
    }
}

/// Add to or remove from My List. Leaves the rating alone.
public struct CommandOpList: Codable, Equatable, Sendable {
    public var inList: Bool
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case inList = "in_list"
        case title = "title"
    }

    public init(inList: Bool, title: TitleKey) {
        self.inList = inList
        self.title = title
    }
}

/// Set or clear a rating. Leaves the list entry alone.
public struct CommandOpRate: Codable, Equatable, Sendable {
    public var rating: Rating?
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case rating = "rating"
        case title = "title"
    }

    public init(rating: Rating? = nil, title: TitleKey) {
        self.rating = rating
        self.title = title
    }
}

/// Hide a title from history. For a series this covers every episode.
public struct CommandOpHide: Codable, Equatable, Sendable {
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case title = "title"
    }

    public init(title: TitleKey) {
        self.title = title
    }
}

/// Clear all history of this profile, as one operation.
public struct CommandOpClearHistory: Codable, Equatable, Sendable {

    public init() {
    }
}

/// Save a position, in whole seconds.
public struct CommandOpPosition: Codable, Equatable, Sendable {
    public var durationS: Int
    public var episode: EpisodeRef?
    public var positionS: Int
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case durationS = "duration_s"
        case episode = "episode"
        case positionS = "position_s"
        case title = "title"
    }

    public init(durationS: Int, episode: EpisodeRef? = nil, positionS: Int, title: TitleKey) {
        self.durationS = durationS
        self.episode = episode
        self.positionS = positionS
        self.title = title
    }
}

/// Change one profile setting. Booleans as true or false, style as JSON text.
public struct CommandOpSetting: Codable, Equatable, Sendable {
    public var key: SettingKey
    public var value: String

    enum CodingKeys: String, CodingKey {
        case key = "key"
        case value = "value"
    }

    public init(key: SettingKey, value: String) {
        self.key = key
        self.value = value
    }
}

/// A viewer signal. Discarded by the server when consent is off.
public struct CommandOpFeedback: Codable, Equatable, Sendable {
    public var event: FeedbackEvent
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case event = "event"
        case title = "title"
    }

    public init(event: FeedbackEvent, title: TitleKey) {
        self.event = event
        self.title = title
    }
}

/// A trailer failed to play for this title.
public struct CommandOpTrailerRefused: Codable, Equatable, Sendable {
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case title = "title"
    }

    public init(title: TitleKey) {
        self.title = title
    }
}

public enum CommandOpBody: Codable, Equatable, Sendable {
    case list(CommandOpList)
    case rate(CommandOpRate)
    case hide(CommandOpHide)
    case clearHistory(CommandOpClearHistory)
    case position(CommandOpPosition)
    case setting(CommandOpSetting)
    case feedback(CommandOpFeedback)
    case trailerRefused(CommandOpTrailerRefused)
    /// A tag this build does not know.
    case unknown(String)

    private enum TagKey: String, CodingKey { case tag = "command" }

    public init(from decoder: Decoder) throws {
        let tagged = try decoder.container(keyedBy: TagKey.self)
        let tag = try tagged.decode(String.self, forKey: .tag)
        switch tag {
        case "list": self = .list(try CommandOpList(from: decoder))
        case "rate": self = .rate(try CommandOpRate(from: decoder))
        case "hide": self = .hide(try CommandOpHide(from: decoder))
        case "clear_history": self = .clearHistory(try CommandOpClearHistory(from: decoder))
        case "position": self = .position(try CommandOpPosition(from: decoder))
        case "setting": self = .setting(try CommandOpSetting(from: decoder))
        case "feedback": self = .feedback(try CommandOpFeedback(from: decoder))
        case "trailer_refused": self = .trailerRefused(try CommandOpTrailerRefused(from: decoder))
        default: self = .unknown(tag)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var tagged = encoder.container(keyedBy: TagKey.self)
        switch self {
        case .list(let value):
            try tagged.encode("list", forKey: .tag)
            try value.encode(to: encoder)
        case .rate(let value):
            try tagged.encode("rate", forKey: .tag)
            try value.encode(to: encoder)
        case .hide(let value):
            try tagged.encode("hide", forKey: .tag)
            try value.encode(to: encoder)
        case .clearHistory(let value):
            try tagged.encode("clear_history", forKey: .tag)
            try value.encode(to: encoder)
        case .position(let value):
            try tagged.encode("position", forKey: .tag)
            try value.encode(to: encoder)
        case .setting(let value):
            try tagged.encode("setting", forKey: .tag)
            try value.encode(to: encoder)
        case .feedback(let value):
            try tagged.encode("feedback", forKey: .tag)
            try value.encode(to: encoder)
        case .trailerRefused(let value):
            try tagged.encode("trailer_refused", forKey: .tag)
            try value.encode(to: encoder)
        case .unknown(let tag):
            try tagged.encode(tag, forKey: .tag)
        }
    }
}

/// Step up. Send a password or an emailed code.
public struct Confirm: Codable, Equatable, Sendable {
    public var code: String?
    public var password: String?

    enum CodingKeys: String, CodingKey {
        case code = "code"
        case password = "password"
    }

    public init(code: String? = nil, password: String? = nil) {
        self.code = code
        self.password = password
    }
}

/// A step up holds for this many seconds and a refresh never renews it.
public struct Confirmed: Codable, Equatable, Sendable {
    public var validS: Int

    enum CodingKeys: String, CodingKey {
        case validS = "valid_s"
    }

    public init(validS: Int) {
        self.validS = validS
    }
}

/// Consent for personal rows.
public struct Consent: Codable, Equatable, Sendable {
    public var personal: Bool

    enum CodingKeys: String, CodingKey {
        case personal = "personal"
    }

    public init(personal: Bool) {
        self.personal = personal
    }
}

/// Given to `core_open`, once. The seed is the only random input the core ever gets.
public struct CoreConfig: Codable, Equatable, Sendable {
    public var app: String
    /// What this device can do. The core sends its hash after sign in.
    public var card: CapabilityCard?
    public var `class`: DeviceClass
    public var contract: WireDate
    public var form: Form
    public var mode: CoreMode
    public var os: String
    /// Hex digits, at least 32.
    public var seed: String
    public var type: CoreConfigTag
    public var version: String

    enum CodingKeys: String, CodingKey {
        case app = "app"
        case card = "card"
        case `class` = "class"
        case contract = "contract"
        case form = "form"
        case mode = "mode"
        case os = "os"
        case seed = "seed"
        case type = "type"
        case version = "version"
    }

    public init(app: String, card: CapabilityCard? = nil, `class`: DeviceClass, contract: WireDate, form: Form, mode: CoreMode, os: String, seed: String, type: CoreConfigTag, version: String) {
        self.app = app
        self.card = card
        self.`class` = `class`
        self.contract = contract
        self.form = form
        self.mode = mode
        self.os = os
        self.seed = seed
        self.type = type
        self.version = version
    }
}

/// Type tag of `CoreConfig`.
public enum CoreConfigTag: String, Codable, Sendable { case value = "core_config" }

/// One step of input. Every input carries wall time and monotonic time.
public struct CoreIn: Codable, Equatable, Sendable {
    public var contract: WireDate
    public var events: [Event]
    /// Monotonic time, ms. Restarts at boot.
    public var monoMs: Int
    /// Wall time, ms since 1970.
    public var nowMs: Int
    public var type: CoreInTag

    enum CodingKeys: String, CodingKey {
        case contract = "contract"
        case events = "events"
        case monoMs = "mono_ms"
        case nowMs = "now_ms"
        case type = "type"
    }

    public init(contract: WireDate, events: [Event], monoMs: Int, nowMs: Int, type: CoreInTag) {
        self.contract = contract
        self.events = events
        self.monoMs = monoMs
        self.nowMs = nowMs
        self.type = type
    }
}

/// Type tag of `CoreIn`.
public enum CoreInTag: String, Codable, Sendable { case value = "core_in" }

/// Where the core runs for this device.
public struct CoreMode: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let local = CoreMode(rawValue: "local")
    public static let hosted = CoreMode(rawValue: "hosted")
}

/// One step of output.
public struct CoreOut: Codable, Equatable, Sendable {
    public var contract: WireDate
    public var effects: [Effect]
    /// Monotonic time at which the host steps the core again. Absent means no wake.
    public var nextWakeMs: Int?
    public var type: CoreOutTag
    /// Ids of in events of a type this core does not know.
    public var unsupported: [String]?
    public var view: View?

    enum CodingKeys: String, CodingKey {
        case contract = "contract"
        case effects = "effects"
        case nextWakeMs = "next_wake_ms"
        case type = "type"
        case unsupported = "unsupported"
        case view = "view"
    }

    public init(contract: WireDate, effects: [Effect], nextWakeMs: Int? = nil, type: CoreOutTag, unsupported: [String]? = nil, view: View? = nil) {
        self.contract = contract
        self.effects = effects
        self.nextWakeMs = nextWakeMs
        self.type = type
        self.unsupported = unsupported
        self.view = view
    }
}

/// Type tag of `CoreOut`.
public enum CoreOutTag: String, Codable, Sendable { case value = "core_out" }

/// One cue.
public struct Cue: Codable, Equatable, Sendable {
    public var endMs: Int
    public var startMs: Int
    public var text: String

    enum CodingKeys: String, CodingKey {
        case endMs = "end_ms"
        case startMs = "start_ms"
        case text = "text"
    }

    public init(endMs: Int, startMs: Int, text: String) {
        self.endMs = endMs
        self.startMs = startMs
        self.text = text
    }
}

/// An opaque position in a list. Clients hand it back and never read it.
public typealias Cursor = String

/// True when the server wants the whole card.
public struct DeviceCardAnswer: Codable, Equatable, Sendable {
    public var sendCard: Bool

    enum CodingKeys: String, CodingKey {
        case sendCard = "send_card"
    }

    public init(sendCard: Bool) {
        self.sendCard = sendCard
    }
}

/// Sent at each start. The card travels only when the server asks or the app or OS changed.
public struct DeviceCardRequest: Codable, Equatable, Sendable {
    public var card: CapabilityCard?
    public var cardHash: String
    public var deviceKey: String

    enum CodingKeys: String, CodingKey {
        case card = "card"
        case cardHash = "card_hash"
        case deviceKey = "device_key"
    }

    public init(card: CapabilityCard? = nil, cardHash: String, deviceKey: String) {
        self.card = card
        self.cardHash = cardHash
        self.deviceKey = deviceKey
    }
}

/// Client class. Minimum versions, start deadlines and reachability are kept per class.
public struct DeviceClass: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let web = DeviceClass(rawValue: "web")
    public static let phone = DeviceClass(rawValue: "phone")
    public static let tablet = DeviceClass(rawValue: "tablet")
    public static let desktop = DeviceClass(rawValue: "desktop")
    public static let tv = DeviceClass(rawValue: "tv")
    public static let box = DeviceClass(rawValue: "box")
    public static let car = DeviceClass(rawValue: "car")
}

/// Device sign in. The screen shows `user_code` even when it shows a QR code.
public struct DeviceCode: Codable, Equatable, Sendable {
    public var deviceCode: String
    public var expiresIn: Int
    public var interval: Int
    public var userCode: String
    public var verificationUri: String
    public var verificationUriComplete: String

    enum CodingKeys: String, CodingKey {
        case deviceCode = "device_code"
        case expiresIn = "expires_in"
        case interval = "interval"
        case userCode = "user_code"
        case verificationUri = "verification_uri"
        case verificationUriComplete = "verification_uri_complete"
    }

    public init(deviceCode: String, expiresIn: Int, interval: Int, userCode: String, verificationUri: String, verificationUriComplete: String) {
        self.deviceCode = deviceCode
        self.expiresIn = expiresIn
        self.interval = interval
        self.userCode = userCode
        self.verificationUri = verificationUri
        self.verificationUriComplete = verificationUriComplete
    }
}

public struct DeviceCodeRequest: Codable, Equatable, Sendable {
    public var deviceName: String

    enum CodingKeys: String, CodingKey {
        case deviceName = "device_name"
    }

    public init(deviceName: String) {
        self.deviceName = deviceName
    }
}

public struct DeviceDecision: Codable, Equatable, Sendable {
    public var userCode: String

    enum CodingKeys: String, CodingKey {
        case userCode = "user_code"
    }

    public init(userCode: String) {
        self.userCode = userCode
    }
}

public struct DeviceList: Codable, Equatable, Sendable {
    public var devices: [DeviceSession]

    enum CodingKeys: String, CodingKey {
        case devices = "devices"
    }

    public init(devices: [DeviceSession]) {
        self.devices = devices
    }
}

/// Answers while a device waits for approval.
public struct DevicePoll: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let authorizationPending = DevicePoll(rawValue: "authorization_pending")
    public static let slowDown = DevicePoll(rawValue: "slow_down")
    public static let accessDenied = DevicePoll(rawValue: "access_denied")
    public static let expiredToken = DevicePoll(rawValue: "expired_token")
}

/// One signed in device.
public struct DeviceSession: Codable, Equatable, Sendable {
    public var `class`: DeviceClass
    public var current: Bool
    public var id: String
    public var lastSeenMs: Int
    public var name: String

    enum CodingKeys: String, CodingKey {
        case `class` = "class"
        case current = "current"
        case id = "id"
        case lastSeenMs = "last_seen_ms"
        case name = "name"
    }

    public init(`class`: DeviceClass, current: Bool, id: String, lastSeenMs: Int, name: String) {
        self.`class` = `class`
        self.current = current
        self.id = id
        self.lastSeenMs = lastSeenMs
        self.name = name
    }
}

/// Either tokens or a word saying to wait or stop.
public struct DeviceTokenAnswer: Codable, Equatable, Sendable {
    public var poll: DevicePoll?
    public var signedIn: SignedIn?

    enum CodingKeys: String, CodingKey {
        case poll = "poll"
        case signedIn = "signed_in"
    }

    public init(poll: DevicePoll? = nil, signedIn: SignedIn? = nil) {
        self.poll = poll
        self.signedIn = signedIn
    }
}

public struct DeviceTokenRequest: Codable, Equatable, Sendable {
    public var deviceCode: String

    enum CodingKeys: String, CodingKey {
        case deviceCode = "device_code"
    }

    public init(deviceCode: String) {
        self.deviceCode = deviceCode
    }
}

/// Answer with no data.
public struct Done: Codable, Equatable, Sendable {
    public var done: Bool

    enum CodingKeys: String, CodingKey {
        case done = "done"
    }

    public init(done: Bool) {
        self.done = done
    }
}

/// A command to the player. Attempts carry their headers here.
public struct DriverCommandOpen: Codable, Equatable, Sendable {
    public var generation: Int
    public var headers: [String: String]
    public var maxBitrate: Int?
    public var maxHeight: Int?
    public var startAtMs: Int
    public var url: String

    enum CodingKeys: String, CodingKey {
        case generation = "generation"
        case headers = "headers"
        case maxBitrate = "max_bitrate"
        case maxHeight = "max_height"
        case startAtMs = "start_at_ms"
        case url = "url"
    }

    public init(generation: Int, headers: [String: String], maxBitrate: Int? = nil, maxHeight: Int? = nil, startAtMs: Int, url: String) {
        self.generation = generation
        self.headers = headers
        self.maxBitrate = maxBitrate
        self.maxHeight = maxHeight
        self.startAtMs = startAtMs
        self.url = url
    }
}

public struct DriverCommandPlay: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct DriverCommandPause: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct DriverCommandSeek: Codable, Equatable, Sendable {
    public var toMs: Int

    enum CodingKeys: String, CodingKey {
        case toMs = "to_ms"
    }

    public init(toMs: Int) {
        self.toMs = toMs
    }
}

public struct DriverCommandRate: Codable, Equatable, Sendable {
    public var percent: Int

    enum CodingKeys: String, CodingKey {
        case percent = "percent"
    }

    public init(percent: Int) {
        self.percent = percent
    }
}

public struct DriverCommandVolume: Codable, Equatable, Sendable {
    public var percent: Int

    enum CodingKeys: String, CodingKey {
        case percent = "percent"
    }

    public init(percent: Int) {
        self.percent = percent
    }
}

public struct DriverCommandSelectAudio: Codable, Equatable, Sendable {
    public var track: String

    enum CodingKeys: String, CodingKey {
        case track = "track"
    }

    public init(track: String) {
        self.track = track
    }
}

public struct DriverCommandSelectText: Codable, Equatable, Sendable {
    public var track: String?

    enum CodingKeys: String, CodingKey {
        case track = "track"
    }

    public init(track: String? = nil) {
        self.track = track
    }
}

public struct DriverCommandClose: Codable, Equatable, Sendable {

    public init() {
    }
}

public enum DriverCommand: Codable, Equatable, Sendable {
    case open(DriverCommandOpen)
    case play(DriverCommandPlay)
    case pause(DriverCommandPause)
    case seek(DriverCommandSeek)
    case rate(DriverCommandRate)
    case volume(DriverCommandVolume)
    case selectAudio(DriverCommandSelectAudio)
    case selectText(DriverCommandSelectText)
    case close(DriverCommandClose)
    /// A tag this build does not know.
    case unknown(String)

    private enum TagKey: String, CodingKey { case tag = "op" }

    public init(from decoder: Decoder) throws {
        let tagged = try decoder.container(keyedBy: TagKey.self)
        let tag = try tagged.decode(String.self, forKey: .tag)
        switch tag {
        case "open": self = .open(try DriverCommandOpen(from: decoder))
        case "play": self = .play(try DriverCommandPlay(from: decoder))
        case "pause": self = .pause(try DriverCommandPause(from: decoder))
        case "seek": self = .seek(try DriverCommandSeek(from: decoder))
        case "rate": self = .rate(try DriverCommandRate(from: decoder))
        case "volume": self = .volume(try DriverCommandVolume(from: decoder))
        case "select_audio": self = .selectAudio(try DriverCommandSelectAudio(from: decoder))
        case "select_text": self = .selectText(try DriverCommandSelectText(from: decoder))
        case "close": self = .close(try DriverCommandClose(from: decoder))
        default: self = .unknown(tag)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var tagged = encoder.container(keyedBy: TagKey.self)
        switch self {
        case .open(let value):
            try tagged.encode("open", forKey: .tag)
            try value.encode(to: encoder)
        case .play(let value):
            try tagged.encode("play", forKey: .tag)
            try value.encode(to: encoder)
        case .pause(let value):
            try tagged.encode("pause", forKey: .tag)
            try value.encode(to: encoder)
        case .seek(let value):
            try tagged.encode("seek", forKey: .tag)
            try value.encode(to: encoder)
        case .rate(let value):
            try tagged.encode("rate", forKey: .tag)
            try value.encode(to: encoder)
        case .volume(let value):
            try tagged.encode("volume", forKey: .tag)
            try value.encode(to: encoder)
        case .selectAudio(let value):
            try tagged.encode("select_audio", forKey: .tag)
            try value.encode(to: encoder)
        case .selectText(let value):
            try tagged.encode("select_text", forKey: .tag)
            try value.encode(to: encoder)
        case .close(let value):
            try tagged.encode("close", forKey: .tag)
            try value.encode(to: encoder)
        case .unknown(let tag):
            try tagged.encode(tag, forKey: .tag)
        }
    }
}

/// What the driver reports. No driver decides failover or no picture by itself.
public struct DriverEventFirstFrame: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct DriverEventPosition: Codable, Equatable, Sendable {
    /// 0 or absent when not known.
    public var durationMs: Int?
    /// Absent means unknown, and the no picture rule stays off.
    public var hasPicture: Bool?
    public var timeMs: Int
    public var videoWidth: Int?

    enum CodingKeys: String, CodingKey {
        case durationMs = "duration_ms"
        case hasPicture = "has_picture"
        case timeMs = "time_ms"
        case videoWidth = "video_width"
    }

    public init(durationMs: Int? = nil, hasPicture: Bool? = nil, timeMs: Int, videoWidth: Int? = nil) {
        self.durationMs = durationMs
        self.hasPicture = hasPicture
        self.timeMs = timeMs
        self.videoWidth = videoWidth
    }
}

public struct DriverEventStalled: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct DriverEventEnded: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct DriverEventTracks: Codable, Equatable, Sendable {
    public var audio: [Track]
    public var text: [Track]

    enum CodingKeys: String, CodingKey {
        case audio = "audio"
        case text = "text"
    }

    public init(audio: [Track], text: [Track]) {
        self.audio = audio
        self.text = text
    }
}

public struct DriverEventError: Codable, Equatable, Sendable {
    public var `class`: ErrorClass

    enum CodingKeys: String, CodingKey {
        case `class` = "class"
    }

    public init(`class`: ErrorClass) {
        self.`class` = `class`
    }
}

public enum DriverEvent: Codable, Equatable, Sendable {
    case firstFrame(DriverEventFirstFrame)
    case position(DriverEventPosition)
    case stalled(DriverEventStalled)
    case ended(DriverEventEnded)
    case tracks(DriverEventTracks)
    case error(DriverEventError)
    /// A tag this build does not know.
    case unknown(String)

    private enum TagKey: String, CodingKey { case tag = "kind" }

    public init(from decoder: Decoder) throws {
        let tagged = try decoder.container(keyedBy: TagKey.self)
        let tag = try tagged.decode(String.self, forKey: .tag)
        switch tag {
        case "first_frame": self = .firstFrame(try DriverEventFirstFrame(from: decoder))
        case "position": self = .position(try DriverEventPosition(from: decoder))
        case "stalled": self = .stalled(try DriverEventStalled(from: decoder))
        case "ended": self = .ended(try DriverEventEnded(from: decoder))
        case "tracks": self = .tracks(try DriverEventTracks(from: decoder))
        case "error": self = .error(try DriverEventError(from: decoder))
        default: self = .unknown(tag)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var tagged = encoder.container(keyedBy: TagKey.self)
        switch self {
        case .firstFrame(let value):
            try tagged.encode("first_frame", forKey: .tag)
            try value.encode(to: encoder)
        case .position(let value):
            try tagged.encode("position", forKey: .tag)
            try value.encode(to: encoder)
        case .stalled(let value):
            try tagged.encode("stalled", forKey: .tag)
            try value.encode(to: encoder)
        case .ended(let value):
            try tagged.encode("ended", forKey: .tag)
            try value.encode(to: encoder)
        case .tracks(let value):
            try tagged.encode("tracks", forKey: .tag)
            try value.encode(to: encoder)
        case .error(let value):
            try tagged.encode("error", forKey: .tag)
            try value.encode(to: encoder)
        case .unknown(let tag):
            try tagged.encode(tag, forKey: .tag)
        }
    }
}

/// One effect. The result comes back as an event with this id.
public struct Effect: Codable, Equatable, Sendable {
    public var id: String
    public var body: EffectBody

    private enum CodingKeys: String, CodingKey {
        case id = "id"
    }

    public init(id: String, body: EffectBody) {
        self.id = id
        self.body = body
    }

    public init(from decoder: Decoder) throws {
        let keyed = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try keyed.decode(String.self, forKey: .id)
        self.body = try EffectBody(from: decoder)
    }

    public func encode(to encoder: Encoder) throws {
        var keyed = encoder.container(keyedBy: CodingKeys.self)
        try keyed.encode(id, forKey: .id)
        try body.encode(to: encoder)
    }
}

/// A path, never an address. The host holds the base address.
public struct EffectHttp: Codable, Equatable, Sendable {
    public var body: String?
    public var `class`: HttpClass
    public var deadlineMs: Int
    public var headers: [String: String]?
    public var method: HttpMethod
    public var path: String
    /// Opaque scope of an account and profile, for kept copies.
    public var scope: String?
    public var to: BodyTarget

    enum CodingKeys: String, CodingKey {
        case body = "body"
        case `class` = "class"
        case deadlineMs = "deadline_ms"
        case headers = "headers"
        case method = "method"
        case path = "path"
        case scope = "scope"
        case to = "to"
    }

    public init(body: String? = nil, `class`: HttpClass, deadlineMs: Int, headers: [String: String]? = nil, method: HttpMethod, path: String, scope: String? = nil, to: BodyTarget) {
        self.body = body
        self.`class` = `class`
        self.deadlineMs = deadlineMs
        self.headers = headers
        self.method = method
        self.path = path
        self.scope = scope
        self.to = to
    }
}

public struct EffectStore: Codable, Equatable, Sendable {
    public var key: String?
    public var op: StoreOp
    public var value: String?

    enum CodingKeys: String, CodingKey {
        case key = "key"
        case op = "op"
        case value = "value"
    }

    public init(key: String? = nil, op: StoreOp, value: String? = nil) {
        self.key = key
        self.op = op
        self.value = value
    }
}

public struct EffectSecret: Codable, Equatable, Sendable {
    public var name: String
    public var op: SecretOp
    public var value: String?

    enum CodingKeys: String, CodingKey {
        case name = "name"
        case op = "op"
        case value = "value"
    }

    public init(name: String, op: SecretOp, value: String? = nil) {
        self.name = name
        self.op = op
        self.value = value
    }
}

public struct EffectTimer: Codable, Equatable, Sendable {
    public var afterMs: Int?
    public var op: TimerOp

    enum CodingKeys: String, CodingKey {
        case afterMs = "after_ms"
        case op = "op"
    }

    public init(afterMs: Int? = nil, op: TimerOp) {
        self.afterMs = afterMs
        self.op = op
    }
}

public struct EffectPlayer: Codable, Equatable, Sendable {
    public var command: DriverCommand

    enum CodingKeys: String, CodingKey {
        case command = "command"
    }

    public init(command: DriverCommand) {
        self.command = command
    }
}

/// A one time notice or a kept page. Lasting state travels in `view`.
public struct EffectView: Codable, Equatable, Sendable {
    public var body: ScreenBody?
    public var notice: Notice?

    enum CodingKeys: String, CodingKey {
        case body = "body"
        case notice = "notice"
    }

    public init(body: ScreenBody? = nil, notice: Notice? = nil) {
        self.body = body
        self.notice = notice
    }
}

public struct EffectOpen: Codable, Equatable, Sendable {
    public var target: RouteTarget

    enum CodingKeys: String, CodingKey {
        case target = "target"
    }

    public init(target: RouteTarget) {
        self.target = target
    }
}

public struct EffectLog: Codable, Equatable, Sendable {
    public var code: String
    public var detail: String?
    public var level: LogLevel

    enum CodingKeys: String, CodingKey {
        case code = "code"
        case detail = "detail"
        case level = "level"
    }

    public init(code: String, detail: String? = nil, level: LogLevel) {
        self.code = code
        self.detail = detail
        self.level = level
    }
}

public enum EffectBody: Codable, Equatable, Sendable {
    case http(EffectHttp)
    case store(EffectStore)
    case secret(EffectSecret)
    case timer(EffectTimer)
    case player(EffectPlayer)
    case view(EffectView)
    case open(EffectOpen)
    case log(EffectLog)
    /// A tag this build does not know.
    case unknown(String)

    private enum TagKey: String, CodingKey { case tag = "kind" }

    public init(from decoder: Decoder) throws {
        let tagged = try decoder.container(keyedBy: TagKey.self)
        let tag = try tagged.decode(String.self, forKey: .tag)
        switch tag {
        case "http": self = .http(try EffectHttp(from: decoder))
        case "store": self = .store(try EffectStore(from: decoder))
        case "secret": self = .secret(try EffectSecret(from: decoder))
        case "timer": self = .timer(try EffectTimer(from: decoder))
        case "player": self = .player(try EffectPlayer(from: decoder))
        case "view": self = .view(try EffectView(from: decoder))
        case "open": self = .open(try EffectOpen(from: decoder))
        case "log": self = .log(try EffectLog(from: decoder))
        default: self = .unknown(tag)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var tagged = encoder.container(keyedBy: TagKey.self)
        switch self {
        case .http(let value):
            try tagged.encode("http", forKey: .tag)
            try value.encode(to: encoder)
        case .store(let value):
            try tagged.encode("store", forKey: .tag)
            try value.encode(to: encoder)
        case .secret(let value):
            try tagged.encode("secret", forKey: .tag)
            try value.encode(to: encoder)
        case .timer(let value):
            try tagged.encode("timer", forKey: .tag)
            try value.encode(to: encoder)
        case .player(let value):
            try tagged.encode("player", forKey: .tag)
            try value.encode(to: encoder)
        case .view(let value):
            try tagged.encode("view", forKey: .tag)
            try value.encode(to: encoder)
        case .open(let value):
            try tagged.encode("open", forKey: .tag)
            try value.encode(to: encoder)
        case .log(let value):
            try tagged.encode("log", forKey: .tag)
            try value.encode(to: encoder)
        case .unknown(let tag):
            try tagged.encode(tag, forKey: .tag)
        }
    }
}

public struct EmailChange: Codable, Equatable, Sendable {
    public var email: String

    enum CodingKeys: String, CodingKey {
        case email = "email"
    }

    public init(email: String) {
        self.email = email
    }
}

/// Why a session ended.
public struct EndReason: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let signedOut = EndReason(rawValue: "signed_out")
    public static let everywhere = EndReason(rawValue: "everywhere")
    public static let refused = EndReason(rawValue: "refused")
    public static let elsewhere = EndReason(rawValue: "elsewhere")
}

/// One episode.
public struct Episode: Codable, Equatable, Sendable {
    public var images: [Image]?
    public var name: String
    public var number: Int
    public var overview: String?
    /// Whole percent watched.
    public var percent: Int?
    public var runtimeMin: Int?

    enum CodingKeys: String, CodingKey {
        case images = "images"
        case name = "name"
        case number = "number"
        case overview = "overview"
        case percent = "percent"
        case runtimeMin = "runtime_min"
    }

    public init(images: [Image]? = nil, name: String, number: Int, overview: String? = nil, percent: Int? = nil, runtimeMin: Int? = nil) {
        self.images = images
        self.name = name
        self.number = number
        self.overview = overview
        self.percent = percent
        self.runtimeMin = runtimeMin
    }
}

/// One episode of a series. A film has none.
public struct EpisodeRef: Codable, Equatable, Sendable {
    public var episode: Int
    public var season: Int

    enum CodingKeys: String, CodingKey {
        case episode = "episode"
        case season = "season"
    }

    public init(episode: Int, season: Int) {
        self.episode = episode
        self.season = season
    }
}

/// Body of every error answer.
public struct ErrorBody: Codable, Equatable, Sendable {
    /// The build that answered.
    public var build: String
    public var code: ErrorCode
    /// Label key for this error.
    public var label: String
    public var notice: Notice?
    public var retryAfterS: Int?

    enum CodingKeys: String, CodingKey {
        case build = "build"
        case code = "code"
        case label = "label"
        case notice = "notice"
        case retryAfterS = "retry_after_s"
    }

    public init(build: String, code: ErrorCode, label: String, notice: Notice? = nil, retryAfterS: Int? = nil) {
        self.build = build
        self.code = code
        self.label = label
        self.notice = notice
        self.retryAfterS = retryAfterS
    }
}

/// Failure classes a driver reports. The core picks the next attempt from these.
public struct ErrorClass: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let network = ErrorClass(rawValue: "network")
    public static let manifest = ErrorClass(rawValue: "manifest")
    public static let segment = ErrorClass(rawValue: "segment")
    public static let decode = ErrorClass(rawValue: "decode")
    public static let key = ErrorClass(rawValue: "key")
    public static let format = ErrorClass(rawValue: "format")
    public static let refused = ErrorClass(rawValue: "refused")
}

/// Stable error code. A code this build does not know reads as `Unknown`.
public struct ErrorCode: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let badRequest = ErrorCode(rawValue: "bad_request")
    public static let unauthorized = ErrorCode(rawValue: "unauthorized")
    public static let sessionEnded = ErrorCode(rawValue: "session_ended")
    public static let profileRequired = ErrorCode(rawValue: "profile_required")
    public static let pinRequired = ErrorCode(rawValue: "pin_required")
    public static let stepUpRequired = ErrorCode(rawValue: "step_up_required")
    public static let refused = ErrorCode(rawValue: "refused")
    public static let notFound = ErrorCode(rawValue: "not_found")
    public static let noLongerAvailable = ErrorCode(rawValue: "no_longer_available")
    public static let conflict = ErrorCode(rawValue: "conflict")
    public static let rateLimited = ErrorCode(rawValue: "rate_limited")
    public static let overloaded = ErrorCode(rawValue: "overloaded")
    public static let deadlineExceeded = ErrorCode(rawValue: "deadline_exceeded")
    public static let upgradeRequired = ErrorCode(rawValue: "upgrade_required")
    public static let unsupported = ErrorCode(rawValue: "unsupported")
    public static let noSource = ErrorCode(rawValue: "no_source")
    public static let clockWrong = ErrorCode(rawValue: "clock_wrong")
    public static let `internal` = ErrorCode(rawValue: "internal")
}

/// One event. `id` is the id of the effect it answers, when it answers one.
public struct Event: Codable, Equatable, Sendable {
    public var id: String?
    public var body: EventBody

    private enum CodingKeys: String, CodingKey {
        case id = "id"
    }

    public init(id: String? = nil, body: EventBody) {
        self.id = id
        self.body = body
    }

    public init(from decoder: Decoder) throws {
        let keyed = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try keyed.decodeIfPresent(String.self, forKey: .id)
        self.body = try EventBody(from: decoder)
    }

    public func encode(to encoder: Encoder) throws {
        var keyed = encoder.container(keyedBy: CodingKeys.self)
        try keyed.encodeIfPresent(id, forKey: .id)
        try body.encode(to: encoder)
    }
}

/// An http effect got an answer. Every status is data.
public struct EventHttpDone: Codable, Equatable, Sendable {
    /// Absent when the body went to the screen.
    public var body: String?
    public var headers: [String: String]?
    public var status: Int

    enum CodingKeys: String, CodingKey {
        case body = "body"
        case headers = "headers"
        case status = "status"
    }

    public init(body: String? = nil, headers: [String: String]? = nil, status: Int) {
        self.body = body
        self.headers = headers
        self.status = status
    }
}

/// An http effect got no answer at all.
public struct EventHttpFailed: Codable, Equatable, Sendable {
    public var reason: NoAnswerReason

    enum CodingKeys: String, CodingKey {
        case reason = "reason"
    }

    public init(reason: NoAnswerReason) {
        self.reason = reason
    }
}

public struct EventStoreDone: Codable, Equatable, Sendable {
    public var keys: [String]?
    public var value: String?

    enum CodingKeys: String, CodingKey {
        case keys = "keys"
        case value = "value"
    }

    public init(keys: [String]? = nil, value: String? = nil) {
        self.keys = keys
        self.value = value
    }
}

public struct EventSecretDone: Codable, Equatable, Sendable {
    public var value: String?

    enum CodingKeys: String, CodingKey {
        case value = "value"
    }

    public init(value: String? = nil) {
        self.value = value
    }
}

public struct EventTimerFired: Codable, Equatable, Sendable {

    public init() {
    }
}

/// The host cannot perform this effect.
public struct EventUnsupported: Codable, Equatable, Sendable {

    public init() {
    }
}

/// The effect with this id was dropped.
public struct EventCancelled: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct EventSuspend: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct EventResume: Codable, Equatable, Sendable {

    public init() {
    }
}

public struct EventVisibility: Codable, Equatable, Sendable {
    public var visible: Bool

    enum CodingKeys: String, CodingKey {
        case visible = "visible"
    }

    public init(visible: Bool) {
        self.visible = visible
    }
}

public struct EventNetwork: Codable, Equatable, Sendable {
    public var online: Bool

    enum CodingKeys: String, CodingKey {
        case online = "online"
    }

    public init(online: Bool) {
        self.online = online
    }
}

/// Touch, key, wheel or playing video in view.
public struct EventActivity: Codable, Equatable, Sendable {

    public init() {
    }
}

/// Another tab of this browser ended the session. The server has heard already.
public struct EventEndedElsewhere: Codable, Equatable, Sendable {

    public init() {
    }
}

/// An address or deep link to open.
public struct EventAddress: Codable, Equatable, Sendable {
    public var address: String

    enum CodingKeys: String, CodingKey {
        case address = "address"
    }

    public init(address: String) {
        self.address = address
    }
}

public struct EventAction: Codable, Equatable, Sendable {
    public var action: Action

    enum CodingKeys: String, CodingKey {
        case action = "action"
    }

    public init(action: Action) {
        self.action = action
    }
}

/// A driver event, stamped with the generation of the open that made it.
public struct EventDriver: Codable, Equatable, Sendable {
    public var event: DriverEvent
    public var generation: Int

    enum CodingKeys: String, CodingKey {
        case event = "event"
        case generation = "generation"
    }

    public init(event: DriverEvent, generation: Int) {
        self.event = event
        self.generation = generation
    }
}

public enum EventBody: Codable, Equatable, Sendable {
    case httpDone(EventHttpDone)
    case httpFailed(EventHttpFailed)
    case storeDone(EventStoreDone)
    case secretDone(EventSecretDone)
    case timerFired(EventTimerFired)
    case unsupported(EventUnsupported)
    case cancelled(EventCancelled)
    case suspend(EventSuspend)
    case resume(EventResume)
    case visibility(EventVisibility)
    case network(EventNetwork)
    case activity(EventActivity)
    case endedElsewhere(EventEndedElsewhere)
    case address(EventAddress)
    case action(EventAction)
    case driver(EventDriver)
    /// A tag this build does not know.
    case unknown(String)

    private enum TagKey: String, CodingKey { case tag = "type" }

    public init(from decoder: Decoder) throws {
        let tagged = try decoder.container(keyedBy: TagKey.self)
        let tag = try tagged.decode(String.self, forKey: .tag)
        switch tag {
        case "http_done": self = .httpDone(try EventHttpDone(from: decoder))
        case "http_failed": self = .httpFailed(try EventHttpFailed(from: decoder))
        case "store_done": self = .storeDone(try EventStoreDone(from: decoder))
        case "secret_done": self = .secretDone(try EventSecretDone(from: decoder))
        case "timer_fired": self = .timerFired(try EventTimerFired(from: decoder))
        case "unsupported": self = .unsupported(try EventUnsupported(from: decoder))
        case "cancelled": self = .cancelled(try EventCancelled(from: decoder))
        case "suspend": self = .suspend(try EventSuspend(from: decoder))
        case "resume": self = .resume(try EventResume(from: decoder))
        case "visibility": self = .visibility(try EventVisibility(from: decoder))
        case "network": self = .network(try EventNetwork(from: decoder))
        case "activity": self = .activity(try EventActivity(from: decoder))
        case "ended_elsewhere": self = .endedElsewhere(try EventEndedElsewhere(from: decoder))
        case "address": self = .address(try EventAddress(from: decoder))
        case "action": self = .action(try EventAction(from: decoder))
        case "driver": self = .driver(try EventDriver(from: decoder))
        default: self = .unknown(tag)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var tagged = encoder.container(keyedBy: TagKey.self)
        switch self {
        case .httpDone(let value):
            try tagged.encode("http_done", forKey: .tag)
            try value.encode(to: encoder)
        case .httpFailed(let value):
            try tagged.encode("http_failed", forKey: .tag)
            try value.encode(to: encoder)
        case .storeDone(let value):
            try tagged.encode("store_done", forKey: .tag)
            try value.encode(to: encoder)
        case .secretDone(let value):
            try tagged.encode("secret_done", forKey: .tag)
            try value.encode(to: encoder)
        case .timerFired(let value):
            try tagged.encode("timer_fired", forKey: .tag)
            try value.encode(to: encoder)
        case .unsupported(let value):
            try tagged.encode("unsupported", forKey: .tag)
            try value.encode(to: encoder)
        case .cancelled(let value):
            try tagged.encode("cancelled", forKey: .tag)
            try value.encode(to: encoder)
        case .suspend(let value):
            try tagged.encode("suspend", forKey: .tag)
            try value.encode(to: encoder)
        case .resume(let value):
            try tagged.encode("resume", forKey: .tag)
            try value.encode(to: encoder)
        case .visibility(let value):
            try tagged.encode("visibility", forKey: .tag)
            try value.encode(to: encoder)
        case .network(let value):
            try tagged.encode("network", forKey: .tag)
            try value.encode(to: encoder)
        case .activity(let value):
            try tagged.encode("activity", forKey: .tag)
            try value.encode(to: encoder)
        case .endedElsewhere(let value):
            try tagged.encode("ended_elsewhere", forKey: .tag)
            try value.encode(to: encoder)
        case .address(let value):
            try tagged.encode("address", forKey: .tag)
            try value.encode(to: encoder)
        case .action(let value):
            try tagged.encode("action", forKey: .tag)
            try value.encode(to: encoder)
        case .driver(let value):
            try tagged.encode("driver", forKey: .tag)
            try value.encode(to: encoder)
        case .unknown(let tag):
            try tagged.encode(tag, forKey: .tag)
        }
    }
}

/// Feature values. With no answer from the server, accounts are required.
public struct Features: Codable, Equatable, Sendable {
    public var accountsRequired: Bool?
    public var clips: Bool?
    public var games: Bool?
    public var originalAudio: Bool?
    /// Tier for visitors. Present only when accounts are not required.
    public var visitorTier: String?

    enum CodingKeys: String, CodingKey {
        case accountsRequired = "accounts_required"
        case clips = "clips"
        case games = "games"
        case originalAudio = "original_audio"
        case visitorTier = "visitor_tier"
    }

    public init(accountsRequired: Bool? = nil, clips: Bool? = nil, games: Bool? = nil, originalAudio: Bool? = nil, visitorTier: String? = nil) {
        self.accountsRequired = accountsRequired
        self.clips = clips
        self.games = games
        self.originalAudio = originalAudio
        self.visitorTier = visitorTier
    }
}

/// Signals a viewer gives.
public struct FeedbackEvent: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let click = FeedbackEvent(rawValue: "click")
    public static let start = FeedbackEvent(rawValue: "start")
    public static let complete = FeedbackEvent(rawValue: "complete")
    public static let like = FeedbackEvent(rawValue: "like")
    public static let skip = FeedbackEvent(rawValue: "skip")
}

/// Answer, possibly partial when the budget ran out.
public struct FindAnswer: Codable, Equatable, Sendable {
    /// Read the rest through `/v1/find/{job}`.
    public var job: String?
    public var outcome: FindOutcome
    public var sources: [FoundSource]

    enum CodingKeys: String, CodingKey {
        case job = "job"
        case outcome = "outcome"
        case sources = "sources"
    }

    public init(job: String? = nil, outcome: FindOutcome, sources: [FoundSource]) {
        self.job = job
        self.outcome = outcome
        self.sources = sources
    }
}

/// How a search for sources ended.
public struct FindOutcome: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let found = FindOutcome(rawValue: "found")
    public static let partial = FindOutcome(rawValue: "partial")
    public static let notFound = FindOutcome(rawValue: "not_found")
    public static let pending = FindOutcome(rawValue: "pending")
}

/// Title facts come from the server, so a finder never looks them up.
public struct FindRequest: Codable, Equatable, Sendable {
    /// Total time this search may take.
    public var budgetMs: Int
    public var episode: EpisodeRef?
    public var name: String
    public var title: TitleKey
    public var year: Int?

    enum CodingKeys: String, CodingKey {
        case budgetMs = "budget_ms"
        case episode = "episode"
        case name = "name"
        case title = "title"
        case year = "year"
    }

    public init(budgetMs: Int, episode: EpisodeRef? = nil, name: String, title: TitleKey, year: Int? = nil) {
        self.budgetMs = budgetMs
        self.episode = episode
        self.name = name
        self.title = title
        self.year = year
    }
}

/// `/v1/health` of a finder.
public struct FinderHealth: Codable, Equatable, Sendable {
    public var build: String
    public var state: ServiceState

    enum CodingKeys: String, CodingKey {
        case build = "build"
        case state = "state"
    }

    public init(build: String, state: ServiceState) {
        self.build = build
        self.state = state
    }
}

/// The form a client draws in. A browser on a phone is class web, form phone.
public struct Form: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let phone = Form(rawValue: "phone")
    public static let tablet = Form(rawValue: "tablet")
    public static let desktop = Form(rawValue: "desktop")
    public static let tv = Form(rawValue: "tv")
}

/// One source as facts. The server turns these into ranked attempts.
public struct FoundSource: Codable, Equatable, Sendable {
    public var audio: [String]?
    public var container: String?
    public var group: String
    public var headers: [String: String]
    public var height: Int?
    public var kind: AttemptKind
    public var url: String
    public var verified: Bool

    enum CodingKeys: String, CodingKey {
        case audio = "audio"
        case container = "container"
        case group = "group"
        case headers = "headers"
        case height = "height"
        case kind = "kind"
        case url = "url"
        case verified = "verified"
    }

    public init(audio: [String]? = nil, container: String? = nil, group: String, headers: [String: String], height: Int? = nil, kind: AttemptKind, url: String, verified: Bool) {
        self.audio = audio
        self.container = container
        self.group = group
        self.headers = headers
        self.height = height
        self.kind = kind
        self.url = url
        self.verified = verified
    }
}

/// True while the who is watching prompt should show.
public struct GateView: Codable, Equatable, Sendable {
    public var ask: Bool

    enum CodingKeys: String, CodingKey {
        case ask = "ask"
    }

    public init(ask: Bool) {
        self.ask = ask
    }
}

/// First answer a client reads.
public struct Hello: Codable, Equatable, Sendable {
    public var brand: BrandValues
    public var clientMin: [ClientFloor]
    /// Wire dates this server answers.
    public var contract: VersionRange
    public var features: Features
    /// Builds refused by name.
    public var killedBuilds: [String]?
    public var limits: Limits
    public var moved: [MovedAddress]?
    /// False stops playback everywhere with a notice.
    public var playback: Bool
    public var rollouts: [Rollout]?
    public var startDeadlineMs: [ClassValue]
    /// Seconds this answer stays good.
    public var ttlS: Int

    enum CodingKeys: String, CodingKey {
        case brand = "brand"
        case clientMin = "client_min"
        case contract = "contract"
        case features = "features"
        case killedBuilds = "killed_builds"
        case limits = "limits"
        case moved = "moved"
        case playback = "playback"
        case rollouts = "rollouts"
        case startDeadlineMs = "start_deadline_ms"
        case ttlS = "ttl_s"
    }

    public init(brand: BrandValues, clientMin: [ClientFloor], contract: VersionRange, features: Features, killedBuilds: [String]? = nil, limits: Limits, moved: [MovedAddress]? = nil, playback: Bool, rollouts: [Rollout]? = nil, startDeadlineMs: [ClassValue], ttlS: Int) {
        self.brand = brand
        self.clientMin = clientMin
        self.contract = contract
        self.features = features
        self.killedBuilds = killedBuilds
        self.limits = limits
        self.moved = moved
        self.playback = playback
        self.rollouts = rollouts
        self.startDeadlineMs = startDeadlineMs
        self.ttlS = ttlS
    }
}

/// `POST /v1/core/step`. `config` is sent on the first step of a hosted session.
public struct HostedStep: Codable, Equatable, Sendable {
    public var config: CoreConfig?
    public var input: CoreIn

    enum CodingKeys: String, CodingKey {
        case config = "config"
        case input = "input"
    }

    public init(config: CoreConfig? = nil, input: CoreIn) {
        self.config = config
        self.input = input
    }
}

/// How an http effect may be handled.
public struct HttpClass: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let command = HttpClass(rawValue: "command")
    public static let cacheable = HttpClass(rawValue: "cacheable")
    public static let plain = HttpClass(rawValue: "plain")
}

/// Methods of an http effect.
public struct HttpMethod: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let gET = HttpMethod(rawValue: "GET")
    public static let pOST = HttpMethod(rawValue: "POST")
    public static let pUT = HttpMethod(rawValue: "PUT")
    public static let pATCH = HttpMethod(rawValue: "PATCH")
    public static let dELETE = HttpMethod(rawValue: "DELETE")
}

/// One image address. Images go straight from the host to the viewer.
public struct Image: Codable, Equatable, Sendable {
    public var role: ImageRole
    public var url: String
    public var width: Int?

    enum CodingKeys: String, CodingKey {
        case role = "role"
        case url = "url"
        case width = "width"
    }

    public init(role: ImageRole, url: String, width: Int? = nil) {
        self.role = role
        self.url = url
        self.width = width
    }
}

/// What an image is for.
public struct ImageRole: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let poster = ImageRole(rawValue: "poster")
    public static let backdrop = ImageRole(rawValue: "backdrop")
    public static let logo = ImageRole(rawValue: "logo")
    public static let still = ImageRole(rawValue: "still")
    public static let avatar = ImageRole(rawValue: "avatar")
}

/// Input a device has.
public struct InputKind: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let touch = InputKind(rawValue: "touch")
    public static let pointer = InputKind(rawValue: "pointer")
    public static let keyboard = InputKind(rawValue: "keyboard")
    public static let remote = InputKind(rawValue: "remote")
    public static let gamepad = InputKind(rawValue: "gamepad")
    public static let rotary = InputKind(rawValue: "rotary")
}

/// Limits served from one place, so no client holds a copy.
public struct Limits: Codable, Equatable, Sendable {
    public var blockedTitleMax: Int
    public var blockedTitlesMax: Int
    public var codeDigits: Int
    public var codeValidS: Int
    public var commandBatchMax: Int
    public var handleMax: Int
    public var handleMin: Int
    public var nameMax: Int
    public var passwordMin: Int
    public var phoneDigitsMax: Int
    public var phoneDigitsMin: Int
    public var pinDigits: Int
    public var profileNameMax: Int
    public var profilesMax: Int
    public var stepUpS: Int

    enum CodingKeys: String, CodingKey {
        case blockedTitleMax = "blocked_title_max"
        case blockedTitlesMax = "blocked_titles_max"
        case codeDigits = "code_digits"
        case codeValidS = "code_valid_s"
        case commandBatchMax = "command_batch_max"
        case handleMax = "handle_max"
        case handleMin = "handle_min"
        case nameMax = "name_max"
        case passwordMin = "password_min"
        case phoneDigitsMax = "phone_digits_max"
        case phoneDigitsMin = "phone_digits_min"
        case pinDigits = "pin_digits"
        case profileNameMax = "profile_name_max"
        case profilesMax = "profiles_max"
        case stepUpS = "step_up_s"
    }

    public init(blockedTitleMax: Int, blockedTitlesMax: Int, codeDigits: Int, codeValidS: Int, commandBatchMax: Int, handleMax: Int, handleMin: Int, nameMax: Int, passwordMin: Int, phoneDigitsMax: Int, phoneDigitsMin: Int, pinDigits: Int, profileNameMax: Int, profilesMax: Int, stepUpS: Int) {
        self.blockedTitleMax = blockedTitleMax
        self.blockedTitlesMax = blockedTitlesMax
        self.codeDigits = codeDigits
        self.codeValidS = codeValidS
        self.commandBatchMax = commandBatchMax
        self.handleMax = handleMax
        self.handleMin = handleMin
        self.nameMax = nameMax
        self.passwordMin = passwordMin
        self.phoneDigitsMax = phoneDigitsMax
        self.phoneDigitsMin = phoneDigitsMin
        self.pinDigits = pinDigits
        self.profileNameMax = profileNameMax
        self.profilesMax = profilesMax
        self.stepUpS = stepUpS
    }
}

/// List state of a title, versioned apart from its rating.
public struct ListState: Codable, Equatable, Sendable {
    public var changedAtMs: Int
    public var inList: Bool
    public var version: Int

    enum CodingKeys: String, CodingKey {
        case changedAtMs = "changed_at_ms"
        case inList = "in_list"
        case version = "version"
    }

    public init(changedAtMs: Int, inList: Bool, version: Int) {
        self.changedAtMs = changedAtMs
        self.inList = inList
        self.version = version
    }
}

/// Log levels.
public struct LogLevel: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let debug = LogLevel(rawValue: "debug")
    public static let info = LogLevel(rawValue: "info")
    public static let warn = LogLevel(rawValue: "warn")
    public static let error = LogLevel(rawValue: "error")
}

/// Skip marks.
public struct Marks: Codable, Equatable, Sendable {
    public var windows: [TimeWindow]

    enum CodingKeys: String, CodingKey {
        case windows = "windows"
    }

    public init(windows: [TimeWindow]) {
        self.windows = windows
    }
}

/// Ask for skip marks, after first frame, series only.
public struct MarksRequest: Codable, Equatable, Sendable {
    public var attempt: String
    public var delayMs: Int?
    public var durationMs: Int
    public var file: String?

    enum CodingKeys: String, CodingKey {
        case attempt = "attempt"
        case delayMs = "delay_ms"
        case durationMs = "duration_ms"
        case file = "file"
    }

    public init(attempt: String, delayMs: Int? = nil, durationMs: Int, file: String? = nil) {
        self.attempt = attempt
        self.delayMs = delayMs
        self.durationMs = durationMs
        self.file = file
    }
}

/// The next part of a screen or a section.
public struct More: Codable, Equatable, Sendable {
    public var cursor: Cursor?
    public var sections: [Section]?
    public var tiles: [Tile]?

    enum CodingKeys: String, CodingKey {
        case cursor = "cursor"
        case sections = "sections"
        case tiles = "tiles"
    }

    public init(cursor: Cursor? = nil, sections: [Section]? = nil, tiles: [Tile]? = nil) {
        self.cursor = cursor
        self.sections = sections
        self.tiles = tiles
    }
}

/// Late attempts.
public struct MoreAttempts: Codable, Equatable, Sendable {
    public var attempts: [Attempt]
    public var more: Bool

    enum CodingKeys: String, CodingKey {
        case attempts = "attempts"
        case more = "more"
    }

    public init(attempts: [Attempt], more: Bool) {
        self.attempts = attempts
        self.more = more
    }
}

/// One row of the moved address table.
public struct MovedAddress: Codable, Equatable, Sendable {
    public var from: String
    public var to: String

    enum CodingKeys: String, CodingKey {
        case from = "from"
        case to = "to"
    }

    public init(from: String, to: String) {
        self.from = from
        self.to = to
    }
}

/// The menu for this class and profile.
public struct Nav: Codable, Equatable, Sendable {
    public var items: [NavItem]
    public var ttlS: Int

    enum CodingKeys: String, CodingKey {
        case items = "items"
        case ttlS = "ttl_s"
    }

    public init(items: [NavItem], ttlS: Int) {
        self.items = items
        self.ttlS = ttlS
    }
}

/// One entry of the menu.
public struct NavItem: Codable, Equatable, Sendable {
    public var genre: String?
    public var id: String
    public var label: String
    public var screen: ScreenName

    enum CodingKeys: String, CodingKey {
        case genre = "genre"
        case id = "id"
        case label = "label"
        case screen = "screen"
    }

    public init(genre: String? = nil, id: String, label: String, screen: ScreenName) {
        self.genre = genre
        self.id = id
        self.label = label
        self.screen = screen
    }
}

/// Why an http effect got no answer. Any status at all is an answer.
public struct NoAnswerReason: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let offline = NoAnswerReason(rawValue: "offline")
    public static let timeout = NoAnswerReason(rawValue: "timeout")
    public static let tls = NoAnswerReason(rawValue: "tls")
    public static let clockWrong = NoAnswerReason(rawValue: "clock_wrong")
    public static let cancelled = NoAnswerReason(rawValue: "cancelled")
}

/// Text a client can draw with no other knowledge, including a client below the floor.
public struct Notice: Codable, Equatable, Sendable {
    public var actionAddress: String?
    public var actionLabel: String?
    /// Label key, for clients with the strings.
    public var label: String
    /// Plain text in the viewer's language.
    public var text: String

    enum CodingKeys: String, CodingKey {
        case actionAddress = "action_address"
        case actionLabel = "action_label"
        case label = "label"
        case text = "text"
    }

    public init(actionAddress: String? = nil, actionLabel: String? = nil, label: String, text: String) {
        self.actionAddress = actionAddress
        self.actionLabel = actionLabel
        self.label = label
        self.text = text
    }
}

/// Answer for one op.
public struct OpAnswer: Codable, Equatable, Sendable {
    public var opId: String
    public var outcome: Outcome

    enum CodingKeys: String, CodingKey {
        case opId = "op_id"
        case outcome = "outcome"
    }

    public init(opId: String, outcome: Outcome) {
        self.opId = opId
        self.outcome = outcome
    }
}

/// Answer per op.
public struct Outcome: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let applied = Outcome(rawValue: "applied")
    public static let superseded = Outcome(rawValue: "superseded")
    public static let rejected = Outcome(rawValue: "rejected")
}

public struct PasswordChange: Codable, Equatable, Sendable {
    public var newPassword: String

    enum CodingKeys: String, CodingKey {
        case newPassword = "new_password"
    }

    public init(newPassword: String) {
        self.newPassword = newPassword
    }
}

public struct PasswordSignIn: Codable, Equatable, Sendable {
    public var email: String
    public var password: String

    enum CodingKeys: String, CodingKey {
        case email = "email"
        case password = "password"
    }

    public init(email: String, password: String) {
        self.email = email
        self.password = password
    }
}

public struct PhoneChange: Codable, Equatable, Sendable {
    public var phone: String

    enum CodingKeys: String, CodingKey {
        case phone = "phone"
    }

    public init(phone: String) {
        self.phone = phone
    }
}

/// Absent clears the PIN.
public struct PinSet: Codable, Equatable, Sendable {
    public var pin: String?

    enum CodingKeys: String, CodingKey {
        case pin = "pin"
    }

    public init(pin: String? = nil) {
        self.pin = pin
    }
}

/// Answer to a play request.
public struct PlayAnswer: Codable, Equatable, Sendable {
    public var ageCard: AgeCard?
    public var attempts: [Attempt]
    public var audioDefault: String?
    /// This profile's setting: the next episode starts by itself at the end of play.
    public var autoplayNext: Bool?
    /// Attempts removed by the capability card.
    public var dropped: Int
    /// True when more attempts may arrive through `/v1/play/{id}/more`.
    public var more: Bool
    /// The next episode in the server's order, across seasons. Absent on the last one.
    public var nextEpisode: EpisodeRef?
    public var playId: String
    /// Where to start.
    public var startAtMs: Int
    public var subtitleDefault: String?

    enum CodingKeys: String, CodingKey {
        case ageCard = "age_card"
        case attempts = "attempts"
        case audioDefault = "audio_default"
        case autoplayNext = "autoplay_next"
        case dropped = "dropped"
        case more = "more"
        case nextEpisode = "next_episode"
        case playId = "play_id"
        case startAtMs = "start_at_ms"
        case subtitleDefault = "subtitle_default"
    }

    public init(ageCard: AgeCard? = nil, attempts: [Attempt], audioDefault: String? = nil, autoplayNext: Bool? = nil, dropped: Int, more: Bool, nextEpisode: EpisodeRef? = nil, playId: String, startAtMs: Int, subtitleDefault: String? = nil) {
        self.ageCard = ageCard
        self.attempts = attempts
        self.audioDefault = audioDefault
        self.autoplayNext = autoplayNext
        self.dropped = dropped
        self.more = more
        self.nextEpisode = nextEpisode
        self.playId = playId
        self.startAtMs = startAtMs
        self.subtitleDefault = subtitleDefault
    }
}

/// One thing that happened during play.
public struct PlayEvent: Codable, Equatable, Sendable {
    public var atMs: Int
    public var attempt: String?
    public var failure: ErrorClass?
    public var firstFrameMs: Int?
    public var host: String?
    public var name: PlayEventName
    public var rung: CoreMode?

    enum CodingKeys: String, CodingKey {
        case atMs = "at_ms"
        case attempt = "attempt"
        case failure = "failure"
        case firstFrameMs = "first_frame_ms"
        case host = "host"
        case name = "name"
        case rung = "rung"
    }

    public init(atMs: Int, attempt: String? = nil, failure: ErrorClass? = nil, firstFrameMs: Int? = nil, host: String? = nil, name: PlayEventName, rung: CoreMode? = nil) {
        self.atMs = atMs
        self.attempt = attempt
        self.failure = failure
        self.firstFrameMs = firstFrameMs
        self.host = host
        self.name = name
        self.rung = rung
    }
}

/// What a play report says happened.
public struct PlayEventName: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let firstFrame = PlayEventName(rawValue: "first_frame")
    public static let rebuffer = PlayEventName(rawValue: "rebuffer")
    public static let failure = PlayEventName(rawValue: "failure")
    public static let ended = PlayEventName(rawValue: "ended")
}

/// `/v1/play/{id}/events`.
public struct PlayEvents: Codable, Equatable, Sendable {
    public var events: [PlayEvent]

    enum CodingKeys: String, CodingKey {
        case events = "events"
    }

    public init(events: [PlayEvent]) {
        self.events = events
    }
}

/// Ask to play a title.
public struct PlayRequest: Codable, Equatable, Sendable {
    public var episode: EpisodeRef?
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case episode = "episode"
        case title = "title"
    }

    public init(episode: EpisodeRef? = nil, title: TitleKey) {
        self.episode = episode
        self.title = title
    }
}

/// Where the play session stands.
public struct PlayState: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let resolving = PlayState(rawValue: "resolving")
    public static let starting = PlayState(rawValue: "starting")
    public static let playing = PlayState(rawValue: "playing")
    public static let paused = PlayState(rawValue: "paused")
    public static let stalled = PlayState(rawValue: "stalled")
    public static let switching = PlayState(rawValue: "switching")
    public static let ended = PlayState(rawValue: "ended")
    public static let failed = PlayState(rawValue: "failed")
}

/// Play session state for screens.
public struct PlayView: Codable, Equatable, Sendable {
    public var attempt: String?
    public var episode: EpisodeRef?
    /// Absent on the last episode, so no screen offers one that does not exist.
    public var nextEpisode: EpisodeRef?
    public var state: PlayState
    public var title: TitleKey
    public var windows: [TimeWindow]?

    enum CodingKeys: String, CodingKey {
        case attempt = "attempt"
        case episode = "episode"
        case nextEpisode = "next_episode"
        case state = "state"
        case title = "title"
        case windows = "windows"
    }

    public init(attempt: String? = nil, episode: EpisodeRef? = nil, nextEpisode: EpisodeRef? = nil, state: PlayState, title: TitleKey, windows: [TimeWindow]? = nil) {
        self.attempt = attempt
        self.episode = episode
        self.nextEpisode = nextEpisode
        self.state = state
        self.title = title
        self.windows = windows
    }
}

/// One profile.
public struct Profile: Codable, Equatable, Sendable {
    public var avatar: String?
    public var hasPin: Bool
    public var id: String
    public var kids: Bool
    public var main: Bool
    public var name: String

    enum CodingKeys: String, CodingKey {
        case avatar = "avatar"
        case hasPin = "has_pin"
        case id = "id"
        case kids = "kids"
        case main = "main"
        case name = "name"
    }

    public init(avatar: String? = nil, hasPin: Bool, id: String, kids: Bool, main: Bool, name: String) {
        self.avatar = avatar
        self.hasPin = hasPin
        self.id = id
        self.kids = kids
        self.main = main
        self.name = name
    }
}

public struct ProfileCreate: Codable, Equatable, Sendable {
    public var kids: Bool?
    public var name: String

    enum CodingKeys: String, CodingKey {
        case kids = "kids"
        case name = "name"
    }

    public init(kids: Bool? = nil, name: String) {
        self.kids = kids
        self.name = name
    }
}

public struct ProfileEnter: Codable, Equatable, Sendable {
    public var pin: String?

    enum CodingKeys: String, CodingKey {
        case pin = "pin"
    }

    public init(pin: String? = nil) {
        self.pin = pin
    }
}

/// The profile answer names its home, so the core can map it to a target.
public struct ProfileEntered: Codable, Equatable, Sendable {
    public var home: ScreenName
    public var profile: Profile

    enum CodingKeys: String, CodingKey {
        case home = "home"
        case profile = "profile"
    }

    public init(home: ScreenName, profile: Profile) {
        self.home = home
        self.profile = profile
    }
}

/// Needs a step up. Never reachable through a queued command.
public struct ProfileLimits: Codable, Equatable, Sendable {
    public var ageRating: String
    public var blockedTitles: [String]

    enum CodingKeys: String, CodingKey {
        case ageRating = "age_rating"
        case blockedTitles = "blocked_titles"
    }

    public init(ageRating: String, blockedTitles: [String]) {
        self.ageRating = ageRating
        self.blockedTitles = blockedTitles
    }
}

public struct ProfileList: Codable, Equatable, Sendable {
    public var profiles: [Profile]

    enum CodingKeys: String, CodingKey {
        case profiles = "profiles"
    }

    public init(profiles: [Profile]) {
        self.profiles = profiles
    }
}

public struct ProfileSettings: Codable, Equatable, Sendable {
    public var audioLanguage: String?
    public var autoplayNext: Bool
    public var autoplayPreviews: Bool
    public var gender: String?
    public var handle: String?
    public var subtitleLanguage: String?
    public var subtitleStyle: [String: String]?
    public var uiLanguage: String?

    enum CodingKeys: String, CodingKey {
        case audioLanguage = "audio_language"
        case autoplayNext = "autoplay_next"
        case autoplayPreviews = "autoplay_previews"
        case gender = "gender"
        case handle = "handle"
        case subtitleLanguage = "subtitle_language"
        case subtitleStyle = "subtitle_style"
        case uiLanguage = "ui_language"
    }

    public init(audioLanguage: String? = nil, autoplayNext: Bool, autoplayPreviews: Bool, gender: String? = nil, handle: String? = nil, subtitleLanguage: String? = nil, subtitleStyle: [String: String]? = nil, uiLanguage: String? = nil) {
        self.audioLanguage = audioLanguage
        self.autoplayNext = autoplayNext
        self.autoplayPreviews = autoplayPreviews
        self.gender = gender
        self.handle = handle
        self.subtitleLanguage = subtitleLanguage
        self.subtitleStyle = subtitleStyle
        self.uiLanguage = uiLanguage
    }
}

public struct ProfileTransfer: Codable, Equatable, Sendable {
    public var toAccountEmail: String

    enum CodingKeys: String, CodingKey {
        case toAccountEmail = "to_account_email"
    }

    public init(toAccountEmail: String) {
        self.toAccountEmail = toAccountEmail
    }
}

public struct ProfileUpdate: Codable, Equatable, Sendable {
    public var avatar: String?
    public var kids: Bool?
    public var name: String?

    enum CodingKeys: String, CodingKey {
        case avatar = "avatar"
        case kids = "kids"
        case name = "name"
    }

    public init(avatar: String? = nil, kids: Bool? = nil, name: String? = nil) {
        self.avatar = avatar
        self.kids = kids
        self.name = name
    }
}

/// A position, key by title and episode.
public struct ProgressState: Codable, Equatable, Sendable {
    public var changedAtMs: Int
    public var completed: Bool
    public var durationS: Int
    public var episode: EpisodeRef?
    public var hidden: Bool
    public var positionS: Int
    public var title: TitleKey
    public var version: Int

    enum CodingKeys: String, CodingKey {
        case changedAtMs = "changed_at_ms"
        case completed = "completed"
        case durationS = "duration_s"
        case episode = "episode"
        case hidden = "hidden"
        case positionS = "position_s"
        case title = "title"
        case version = "version"
    }

    public init(changedAtMs: Int, completed: Bool, durationS: Int, episode: EpisodeRef? = nil, hidden: Bool, positionS: Int, title: TitleKey, version: Int) {
        self.changedAtMs = changedAtMs
        self.completed = completed
        self.durationS = durationS
        self.episode = episode
        self.hidden = hidden
        self.positionS = positionS
        self.title = title
        self.version = version
    }
}

/// Rating words on the wire. Screens use the words like, love and dislike.
public struct Rating: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let down = Rating(rawValue: "down")
    public static let up = Rating(rawValue: "up")
    public static let love = Rating(rawValue: "love")
}

/// Rating state of a title, versioned apart from its list state.
public struct RatingState: Codable, Equatable, Sendable {
    public var changedAtMs: Int
    /// Absent when no rating is set.
    public var rating: Rating?
    public var version: Int

    enum CodingKeys: String, CodingKey {
        case changedAtMs = "changed_at_ms"
        case rating = "rating"
        case version = "version"
    }

    public init(changedAtMs: Int, rating: Rating? = nil, version: Int) {
        self.changedAtMs = changedAtMs
        self.rating = rating
        self.version = version
    }
}

/// `/v1/ready`, for outside checks.
public struct Ready: Codable, Equatable, Sendable {
    public var build: String
    public var dataAgeS: Int
    public var dataHash: String
    public var finder: ServiceState
    public var index: ServiceState

    enum CodingKeys: String, CodingKey {
        case build = "build"
        case dataAgeS = "data_age_s"
        case dataHash = "data_hash"
        case finder = "finder"
        case index = "index"
    }

    public init(build: String, dataAgeS: Int, dataHash: String, finder: ServiceState, index: ServiceState) {
        self.build = build
        self.dataAgeS = dataAgeS
        self.dataHash = dataHash
        self.finder = finder
        self.index = index
    }
}

public struct RecoveryFinish: Codable, Equatable, Sendable {
    public var code: String
    public var email: String
    public var newPassword: String

    enum CodingKeys: String, CodingKey {
        case code = "code"
        case email = "email"
        case newPassword = "new_password"
    }

    public init(code: String, email: String, newPassword: String) {
        self.code = code
        self.email = email
        self.newPassword = newPassword
    }
}

public struct RecoveryStart: Codable, Equatable, Sendable {
    public var email: String

    enum CodingKeys: String, CodingKey {
        case email = "email"
    }

    public init(email: String) {
        self.email = email
    }
}

public struct RefreshRequest: Codable, Equatable, Sendable {
    public var refreshToken: String

    enum CodingKeys: String, CodingKey {
        case refreshToken = "refresh_token"
    }

    public init(refreshToken: String) {
        self.refreshToken = refreshToken
    }
}

/// Events after which a screen should be read again.
public struct RefreshTrigger: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let focus = RefreshTrigger(rawValue: "focus")
    public static let signIn = RefreshTrigger(rawValue: "sign_in")
    public static let profileChange = RefreshTrigger(rawValue: "profile_change")
    public static let listChange = RefreshTrigger(rawValue: "list_change")
    public static let historyChange = RefreshTrigger(rawValue: "history_change")
}

/// `/v1/reports`.
public struct Report: Codable, Equatable, Sendable {
    public var build: String
    public var code: String
    public var detail: String?

    enum CodingKeys: String, CodingKey {
        case build = "build"
        case code = "code"
        case detail = "detail"
    }

    public init(build: String, code: String, detail: String? = nil) {
        self.build = build
        self.code = code
        self.detail = detail
    }
}

/// A staged rollout. The bucket comes from the household, so a home moves together.
public struct Rollout: Codable, Equatable, Sendable {
    public var name: String
    public var percent: Int

    enum CodingKeys: String, CodingKey {
        case name = "name"
        case percent = "percent"
    }

    public init(name: String, percent: Int) {
        self.name = name
        self.percent = percent
    }
}

/// What an address opens.
public struct RouteTarget: Codable, Equatable, Sendable {
    public var episode: EpisodeRef?
    /// The genre, for the genre screen.
    public var genre: String?
    /// A safe address to return to after sign in.
    public var next: String?
    public var screen: ScreenName?
    public var target: Target
    public var title: TitleKey?

    enum CodingKeys: String, CodingKey {
        case episode = "episode"
        case genre = "genre"
        case next = "next"
        case screen = "screen"
        case target = "target"
        case title = "title"
    }

    public init(episode: EpisodeRef? = nil, genre: String? = nil, next: String? = nil, screen: ScreenName? = nil, target: Target, title: TitleKey? = nil) {
        self.episode = episode
        self.genre = genre
        self.next = next
        self.screen = screen
        self.target = target
        self.title = title
    }
}

/// A built page.
public struct Screen: Codable, Equatable, Sendable {
    /// Reads more sections through `/v1/more/{cursor}`.
    public var cursor: Cursor?
    public var name: ScreenName
    public var refreshOn: [RefreshTrigger]?
    public var sections: [Section]
    /// Seconds this answer stays good.
    public var ttlS: Int

    enum CodingKeys: String, CodingKey {
        case cursor = "cursor"
        case name = "name"
        case refreshOn = "refresh_on"
        case sections = "sections"
        case ttlS = "ttl_s"
    }

    public init(cursor: Cursor? = nil, name: ScreenName, refreshOn: [RefreshTrigger]? = nil, sections: [Section], ttlS: Int) {
        self.cursor = cursor
        self.name = name
        self.refreshOn = refreshOn
        self.sections = sections
        self.ttlS = ttlS
    }
}

/// A page body handed to a screen. It stands in for the answer to an http effect whose body was going to the screen.
public struct ScreenBody: Codable, Equatable, Sendable {
    public var body: String
    /// Id of the http effect this answers.
    public var request: String
    /// True when this copy is past its ttl and was kept for when the network is gone.
    public var stale: Bool?

    enum CodingKeys: String, CodingKey {
        case body = "body"
        case request = "request"
        case stale = "stale"
    }

    public init(body: String, request: String, stale: Bool? = nil) {
        self.body = body
        self.request = request
        self.stale = stale
    }
}

/// The screens the server builds.
public struct ScreenName: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let home = ScreenName(rawValue: "home")
    public static let films = ScreenName(rawValue: "films")
    public static let series = ScreenName(rawValue: "series")
    public static let genre = ScreenName(rawValue: "genre")
    public static let new = ScreenName(rawValue: "new")
    public static let myList = ScreenName(rawValue: "my-list")
    public static let kids = ScreenName(rawValue: "kids")
    public static let search = ScreenName(rawValue: "search")
    public static let notifications = ScreenName(rawValue: "notifications")
    public static let activity = ScreenName(rawValue: "activity")
    public static let games = ScreenName(rawValue: "games")
    public static let clips = ScreenName(rawValue: "clips")
    public static let language = ScreenName(rawValue: "language")
}

/// One season with its episodes.
public struct Season: Codable, Equatable, Sendable {
    public var episodes: [Episode]
    public var name: String?
    public var number: Int

    enum CodingKeys: String, CodingKey {
        case episodes = "episodes"
        case name = "name"
        case number = "number"
    }

    public init(episodes: [Episode], name: String? = nil, number: Int) {
        self.episodes = episodes
        self.name = name
        self.number = number
    }
}

/// A season named on a title page.
public struct SeasonSummary: Codable, Equatable, Sendable {
    public var episodeCount: Int
    public var number: Int

    enum CodingKeys: String, CodingKey {
        case episodeCount = "episode_count"
        case number = "number"
    }

    public init(episodeCount: Int, number: Int) {
        self.episodeCount = episodeCount
        self.number = number
    }
}

/// Secret operations.
public struct SecretOp: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let get = SecretOp(rawValue: "get")
    public static let put = SecretOp(rawValue: "put")
    public static let delete = SecretOp(rawValue: "delete")
}

/// One row, hero, grid or notice of a screen.
public struct Section: Codable, Equatable, Sendable {
    /// Reads more of this section through `/v1/more/{cursor}`.
    public var cursor: Cursor?
    public var id: String
    public var kind: SectionKind
    public var notice: Notice?
    public var tiles: [Tile]?
    /// Translated title.
    public var title: String?

    enum CodingKeys: String, CodingKey {
        case cursor = "cursor"
        case id = "id"
        case kind = "kind"
        case notice = "notice"
        case tiles = "tiles"
        case title = "title"
    }

    public init(cursor: Cursor? = nil, id: String, kind: SectionKind, notice: Notice? = nil, tiles: [Tile]? = nil, title: String? = nil) {
        self.cursor = cursor
        self.id = id
        self.kind = kind
        self.notice = notice
        self.tiles = tiles
        self.title = title
    }
}

/// Section kinds. A kind this build does not draw falls back to a plain row.
public struct SectionKind: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let hero = SectionKind(rawValue: "hero")
    public static let row = SectionKind(rawValue: "row")
    public static let ranked = SectionKind(rawValue: "ranked")
    public static let `continue` = SectionKind(rawValue: "continue")
    public static let grid = SectionKind(rawValue: "grid")
    public static let notice = SectionKind(rawValue: "notice")
}

/// State of a part of the service.
public struct ServiceState: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let ready = ServiceState(rawValue: "ready")
    public static let degraded = ServiceState(rawValue: "degraded")
    public static let down = ServiceState(rawValue: "down")
}

/// Where sign in stands.
public struct SessionState: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let starting = SessionState(rawValue: "starting")
    public static let signedOut = SessionState(rawValue: "signed_out")
    public static let signedIn = SessionState(rawValue: "signed_in")
}

/// Sign in state for screens. Tokens never appear here.
public struct SessionView: Codable, Equatable, Sendable {
    public var email: String?
    public var ended: EndReason?
    public var expiresAtMs: Int?
    public var profile: String?
    /// Opaque scope of this account and profile, the same value http effects carry.
    public var scope: String?
    public var state: SessionState
    public var userId: String?

    enum CodingKeys: String, CodingKey {
        case email = "email"
        case ended = "ended"
        case expiresAtMs = "expires_at_ms"
        case profile = "profile"
        case scope = "scope"
        case state = "state"
        case userId = "user_id"
    }

    public init(email: String? = nil, ended: EndReason? = nil, expiresAtMs: Int? = nil, profile: String? = nil, scope: String? = nil, state: SessionState, userId: String? = nil) {
        self.email = email
        self.ended = ended
        self.expiresAtMs = expiresAtMs
        self.profile = profile
        self.scope = scope
        self.state = state
        self.userId = userId
    }
}

/// Settings a queued command may change. Age rating and blocked titles are absent on purpose: they need a step up, so a queued write can never touch them.
public struct SettingKey: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let autoplayNext = SettingKey(rawValue: "autoplay_next")
    public static let autoplayPreviews = SettingKey(rawValue: "autoplay_previews")
    public static let audioLanguage = SettingKey(rawValue: "audio_language")
    public static let subtitleLanguage = SettingKey(rawValue: "subtitle_language")
    public static let subtitleStyle = SettingKey(rawValue: "subtitle_style")
    public static let uiLanguage = SettingKey(rawValue: "ui_language")
    public static let handle = SettingKey(rawValue: "handle")
    public static let gender = SettingKey(rawValue: "gender")
}

public struct SignOut: Codable, Equatable, Sendable {
    /// True ends every session of this account.
    public var everywhere: Bool

    enum CodingKeys: String, CodingKey {
        case everywhere = "everywhere"
    }

    public init(everywhere: Bool) {
        self.everywhere = everywhere
    }
}

public struct SignUp: Codable, Equatable, Sendable {
    public var email: String
    public var firstName: String
    public var lastName: String
    public var password: String

    enum CodingKeys: String, CodingKey {
        case email = "email"
        case firstName = "first_name"
        case lastName = "last_name"
        case password = "password"
    }

    public init(email: String, firstName: String, lastName: String, password: String) {
        self.email = email
        self.firstName = firstName
        self.lastName = lastName
        self.password = password
    }
}

/// Opaque tokens. `expires_in` is seconds, so no device clock decides anything.
public struct SignedIn: Codable, Equatable, Sendable {
    public var accessToken: String
    public var email: String
    public var expiresIn: Int
    public var refreshToken: String
    public var userId: String

    enum CodingKeys: String, CodingKey {
        case accessToken = "access_token"
        case email = "email"
        case expiresIn = "expires_in"
        case refreshToken = "refresh_token"
        case userId = "user_id"
    }

    public init(accessToken: String, email: String, expiresIn: Int, refreshToken: String, userId: String) {
        self.accessToken = accessToken
        self.email = email
        self.expiresIn = expiresIn
        self.refreshToken = refreshToken
        self.userId = userId
    }
}

/// Store operations. Only the offline layer uses store.
public struct StoreOp: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let get = StoreOp(rawValue: "get")
    public static let put = StoreOp(rawValue: "put")
    public static let delete = StoreOp(rawValue: "delete")
    public static let list = StoreOp(rawValue: "list")
}

/// `/v1/strings/{lang}`. Text holds the placeholders below.
public struct Strings: Codable, Equatable, Sendable {
    public var lang: String
    public var texts: [String: String]

    enum CodingKeys: String, CodingKey {
        case lang = "lang"
        case texts = "texts"
    }

    public init(lang: String, texts: [String: String]) {
        self.lang = lang
        self.texts = texts
    }
}

/// `/v1/subtitles/{file}`. The core checks `last_end_ms` against the driver's duration.
public struct SubtitleBody: Codable, Equatable, Sendable {
    public var cues: [Cue]
    public var lastEndMs: Int

    enum CodingKeys: String, CodingKey {
        case cues = "cues"
        case lastEndMs = "last_end_ms"
    }

    public init(cues: [Cue], lastEndMs: Int) {
        self.cues = cues
        self.lastEndMs = lastEndMs
    }
}

/// One subtitle file of a language, in the order to try.
public struct SubtitleFile: Codable, Equatable, Sendable {
    public var hearingImpaired: Bool?
    public var id: String

    enum CodingKeys: String, CodingKey {
        case hearingImpaired = "hearing_impaired"
        case id = "id"
    }

    public init(hearingImpaired: Bool? = nil, id: String) {
        self.hearingImpaired = hearingImpaired
        self.id = id
    }
}

/// One language.
public struct SubtitleLanguage: Codable, Equatable, Sendable {
    public var files: [SubtitleFile]
    public var label: String
    public var lang: String

    enum CodingKeys: String, CodingKey {
        case files = "files"
        case label = "label"
        case lang = "lang"
    }

    public init(files: [SubtitleFile], label: String, lang: String) {
        self.files = files
        self.label = label
        self.lang = lang
    }
}

/// `/v1/subtitles`.
public struct SubtitleList: Codable, Equatable, Sendable {
    public var languages: [SubtitleLanguage]

    enum CodingKeys: String, CodingKey {
        case languages = "languages"
    }

    public init(languages: [SubtitleLanguage]) {
        self.languages = languages
    }
}

/// Search suggestions.
public struct Suggestions: Codable, Equatable, Sendable {
    public var items: [String]

    enum CodingKeys: String, CodingKey {
        case items = "items"
    }

    public init(items: [String]) {
        self.items = items
    }
}

/// A semantic target. Each device maps it to one of its own screens.
public struct Target: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let start = Target(rawValue: "start")
    public static let signIn = Target(rawValue: "sign_in")
    public static let profiles = Target(rawValue: "profiles")
    public static let home = Target(rawValue: "home")
    public static let kidsHome = Target(rawValue: "kids_home")
    public static let screen = Target(rawValue: "screen")
    public static let title = Target(rawValue: "title")
    public static let watch = Target(rawValue: "watch")
    public static let notFound = Target(rawValue: "not_found")
    public static let wait = Target(rawValue: "wait")
    public static let upgrade = Target(rawValue: "upgrade")
}

/// One title in a section.
public struct Tile: Codable, Equatable, Sendable {
    public var badges: [Badge]?
    public var episode: EpisodeRef?
    public var images: [Image]?
    public var name: String
    /// Whole percent watched.
    public var percent: Int?
    public var tags: [String]?
    public var title: TitleKey

    enum CodingKeys: String, CodingKey {
        case badges = "badges"
        case episode = "episode"
        case images = "images"
        case name = "name"
        case percent = "percent"
        case tags = "tags"
        case title = "title"
    }

    public init(badges: [Badge]? = nil, episode: EpisodeRef? = nil, images: [Image]? = nil, name: String, percent: Int? = nil, tags: [String]? = nil, title: TitleKey) {
        self.badges = badges
        self.episode = episode
        self.images = images
        self.name = name
        self.percent = percent
        self.tags = tags
        self.title = title
    }
}

/// A span of a video the host compares to player time. Nothing ticks across the boundary.
public struct TimeWindow: Codable, Equatable, Sendable {
    public var endMs: Int
    public var kind: WindowKind
    /// Where a skip button moves to.
    public var seekToMs: Int?
    public var startMs: Int
    public var text: String?

    enum CodingKeys: String, CodingKey {
        case endMs = "end_ms"
        case kind = "kind"
        case seekToMs = "seek_to_ms"
        case startMs = "start_ms"
        case text = "text"
    }

    public init(endMs: Int, kind: WindowKind, seekToMs: Int? = nil, startMs: Int, text: String? = nil) {
        self.endMs = endMs
        self.kind = kind
        self.seekToMs = seekToMs
        self.startMs = startMs
        self.text = text
    }
}

/// Timer operations. A clear uses the id of the timer it clears.
public struct TimerOp: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let set = TimerOp(rawValue: "set")
    public static let clear = TimerOp(rawValue: "clear")
}

/// Facts of one title.
public struct TitleDetail: Codable, Equatable, Sendable {
    public var genres: [String]?
    public var images: [Image]?
    /// Hidden when empty.
    public var moreLikeThis: [Tile]?
    public var name: String
    public var overview: String?
    public var runtimeMin: Int?
    public var seasons: [SeasonSummary]?
    public var title: TitleKey
    public var year: Int?

    enum CodingKeys: String, CodingKey {
        case genres = "genres"
        case images = "images"
        case moreLikeThis = "more_like_this"
        case name = "name"
        case overview = "overview"
        case runtimeMin = "runtime_min"
        case seasons = "seasons"
        case title = "title"
        case year = "year"
    }

    public init(genres: [String]? = nil, images: [Image]? = nil, moreLikeThis: [Tile]? = nil, name: String, overview: String? = nil, runtimeMin: Int? = nil, seasons: [SeasonSummary]? = nil, title: TitleKey, year: Int? = nil) {
        self.genres = genres
        self.images = images
        self.moreLikeThis = moreLikeThis
        self.name = name
        self.overview = overview
        self.runtimeMin = runtimeMin
        self.seasons = seasons
        self.title = title
        self.year = year
    }
}

/// movie:<id> or tv:<id>. A bare id does not parse.
public struct TitleKey: Codable, Hashable, Sendable {
    public let rawValue: String

    public init(_ rawValue: String) throws {
        guard rawValue.range(of: "^(movie|tv):[1-9][0-9]{0,9}$", options: .regularExpression) != nil else {
            throw ContractError.refused("TitleKey")
        }
        self.rawValue = rawValue
    }

    public init(from decoder: Decoder) throws {
        try self.init(try decoder.singleValueContainer().decode(String.self))
    }

    public func encode(to encoder: Encoder) throws {
        var single = encoder.singleValueContainer()
        try single.encode(rawValue)
    }
}

/// The two kinds of title. Closed on purpose: a new kind reaches old clients only through a dated transform, so a key always names a kind this build can open.
public struct TitleKind: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let movie = TitleKind(rawValue: "movie")
    public static let tv = TitleKind(rawValue: "tv")
}

/// One audio or text track, with the tag the manifest gave it.
public struct Track: Codable, Equatable, Sendable {
    public var id: String
    public var label: String?
    public var lang: String?

    enum CodingKeys: String, CodingKey {
        case id = "id"
        case label = "label"
        case lang = "lang"
    }

    public init(id: String, label: String? = nil, lang: String? = nil) {
        self.id = id
        self.label = label
        self.lang = lang
    }
}

/// The wire dates a server answers. Never compared for equality.
public struct VersionRange: Codable, Equatable, Sendable {
    public var max: WireDate
    public var min: WireDate

    enum CodingKeys: String, CodingKey {
        case max = "max"
        case min = "min"
    }

    public init(max: WireDate, min: WireDate) {
        self.max = max
        self.min = min
    }
}

/// Changed screen state. A part that is absent did not change.
public struct View: Codable, Equatable, Sendable {
    public var gate: GateView?
    /// The last hello read, so a screen takes limits and features from one place.
    public var hello: Hello?
    public var play: PlayView?
    /// The account's profiles, for the picker. No PIN is ever held here.
    public var profiles: ProfileList?
    public var route: RouteTarget?
    public var session: SessionView?

    enum CodingKeys: String, CodingKey {
        case gate = "gate"
        case hello = "hello"
        case play = "play"
        case profiles = "profiles"
        case route = "route"
        case session = "session"
    }

    public init(gate: GateView? = nil, hello: Hello? = nil, play: PlayView? = nil, profiles: ProfileList? = nil, route: RouteTarget? = nil, session: SessionView? = nil) {
        self.gate = gate
        self.hello = hello
        self.play = play
        self.profiles = profiles
        self.route = route
        self.session = session
    }
}

/// Windows of time inside a video.
public struct WindowKind: RawRepresentable, Codable, Hashable, Sendable {
    public var rawValue: String
    public init(rawValue: String) { self.rawValue = rawValue }
    public static let recap = WindowKind(rawValue: "recap")
    public static let intro = WindowKind(rawValue: "intro")
    public static let credits = WindowKind(rawValue: "credits")
    public static let ageCard = WindowKind(rawValue: "age_card")
    public static let cue = WindowKind(rawValue: "cue")
}

/// A wire date, YYYY-MM-DD.
public struct WireDate: Codable, Hashable, Sendable {
    public let rawValue: String

    public init(_ rawValue: String) throws {
        guard rawValue.range(of: "^[0-9]{4}-[0-9]{2}-[0-9]{2}$", options: .regularExpression) != nil else {
            throw ContractError.refused("WireDate")
        }
        self.rawValue = rawValue
    }

    public init(from decoder: Decoder) throws {
        try self.init(try decoder.singleValueContainer().decode(String.self))
    }

    public func encode(to encoder: Encoder) throws {
        var single = encoder.singleValueContainer()
        try single.encode(rawValue)
    }
}
