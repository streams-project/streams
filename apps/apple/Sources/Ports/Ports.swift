// The hexagon of the Apple family. The core sits in the middle, and everything it needs from a
// device goes through one of these. They name the same ports as every other family.
import Contract
import Foundation
import Observation

public struct Call: Sendable {
    public let method: String
    public let url: String
    public let headers: [String: String]
    public let body: String?
    public let deadlineMs: Int64

    public init(
        method: String,
        url: String,
        headers: [String: String],
        body: String?,
        deadlineMs: Int64
    ) {
        self.method = method
        self.url = url
        self.headers = headers
        self.body = body
        self.deadlineMs = deadlineMs
    }
}

public enum Answer: Sendable {
    case reply(status: Int, text: String, etag: String?)
    case noReply(reason: String)
}

public protocol Network: AnyObject, Sendable {
    func send(_ call: Call, done: @escaping @Sendable (Answer) -> Void)
    func watch(_ changed: @escaping @Sendable (Bool) -> Void)
}

public protocol Secrets: AnyObject, Sendable {
    func get(_ name: String) -> String?
    func put(_ name: String, _ value: String)
    func drop(_ name: String)
}

public protocol Store: AnyObject, Sendable {
    func get(_ key: String) -> String?
    func put(_ key: String, _ value: String)
    func delete(_ key: String)
    func keys(_ prefix: String) -> [String]
}

public protocol Player: AnyObject {
    func command(_ given: DriverCommand)
    func listen(_ report: @escaping (Int64, DriverEvent) -> Void)
    func picture(_ context: Any) -> Any
    func now() -> Int64
    func lift(_ share: Float)
    func wordsLook(size: Int, color: String, background: String)
}

public protocol Reel: AnyObject {
    /// A picture of this address. It starts still until `play` says so. `ready` runs once, when it
    /// can show and play; `ended` runs when it finishes or cannot play.
    func picture(_ context: Any, url: String, loop: Bool, ready: @escaping () -> Void, ended: @escaping () -> Void) -> Any
    /// Starts or stops one film.
    func play(_ picture: Any, _ on: Bool)
    func release(_ picture: Any)
    func hold(_ still: Bool)
}

@Observable
@MainActor
public final class Seen {
    public var route: RouteTarget? = nil
    public var device: DeviceSignInView? = nil
    public var profiles: [Profile] = []
    public var play: PlayView? = nil
    public var screen: Screen? = nil
    public var nav: [NavItem] = []
    public var active: String? = nil
    public var words: [String: String] = [:]
    public var language: String = ""
    public var detail: TitleDetail? = nil
    public var notice: String? = nil
    public var brand: String = ""
    public var server: String = ""
    public var positionMs: Int64 = 0
    public var lengthMs: Int64 = 0
    public var audio: [Track] = []
    public var text: [Track] = []

    public init() {}
}

public protocol Shell: AnyObject {
    @MainActor var seen: Seen { get }
    var player: Player { get }
    var reel: Reel { get }
    func connect(_ server: String)
    func go(_ address: String)
    func swap(_ address: String)
    func back()
    func act(_ action: Action)
    func ask(_ path: String, reply: @escaping (Int, String) -> Void)
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void)
    func keep(_ profile: Profile)
    func keepAll(_ profiles: [Profile])
    func speak(_ language: String?)
}

extension Shell {
    public func read(_ path: String) async -> String? {
        await withCheckedContinuation { continuation in
            ask(path) { status, text in
                continuation.resume(returning: (200...299).contains(status) ? text : nil)
            }
        }
    }

    public func write(_ method: String, _ path: String, _ body: String) async -> String? {
        await withCheckedContinuation { continuation in
            send(method, path, body) { status, text in
                continuation.resume(returning: (200...299).contains(status) ? text : nil)
            }
        }
    }
}
