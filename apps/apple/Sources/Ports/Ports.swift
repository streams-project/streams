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
    /// Says when the device gains or loses its connection.
    func watch(_ changed: @escaping @Sendable (Bool) -> Void)
}

public protocol Secrets: AnyObject, Sendable {
    func get(_ name: String) -> String?
    /// False when the store refused the write. The caller must say so rather than pretend: a
    /// session token that was never written signs the viewer out at the next launch.
    @discardableResult
    func put(_ name: String, _ value: String) -> Bool
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

/// One picture wanted.
///
/// The box it will fill is asked for here and not left to the layout for two reasons: the read
/// starts before the first measure instead of a frame after it, and a picture read ahead of the
/// viewer lands under the same key as the one that is drawn, so reading ahead is worth something.
/// The nearest sample size is taken, so a w342 poster is not re-encoded to fit a box exactly.
public struct Want: Sendable, Hashable {
    public let url: String
    /// The box it will fill, in pixels, or 0 for the picture's own size.
    public let wide: Int
    public let high: Int
    /// True when the caller will read the pixels back, which is what taking a tint off a billboard
    /// or the picture behind a profile gate does. It is the same word the android family's port
    /// uses; this family's adapter needs nothing of it, because every picture it hands over is
    /// already backed by pixels a caller may read (`Adapters/Pictures.swift`). It is kept so the
    /// two families name one thing one way, and so a call site says what it means to do.
    public let pixels: Bool

    public init(url: String, wide: Int = 0, high: Int = 0, pixels: Bool = false) {
        self.url = url
        self.wide = wide
        self.high = high
        self.pixels = pixels
    }
}

/// A picture, or why there is none.
public enum Picture {
    /// A picture, as an object only the surface's own toolkit understands - the bargain
    /// ``Player/picture(_:)`` already makes, and for the same reason: this file may not name a type
    /// that belongs to one device. The family's adapter documents what it hands over.
    case painted(Any)
    /// Why there is no picture. The caller draws something else; it never waits for one that is
    /// not coming.
    case unpainted(reason: String)
}

/// The right to stop one read. Letting it go stops nothing.
public struct Stop: Sendable {
    private let act: @Sendable () -> Void

    public init(_ act: @escaping @Sendable () -> Void) {
        self.act = act
    }

    public func stop() { act() }
}

/// Pictures from the network: artwork, logos, treatments and the faces of profiles.
///
/// This was the one device capability of this family with no port. The network, the secrets, the
/// store, the player and the reel are all above; pictures went from a surface straight into
/// SwiftUI's `AsyncImage`, which does its own networking, keeps its own cache and never writes the
/// word `URLSession` - so the ring check's ban on that word passed on every run while four surface
/// sites fetched from the network themselves (`notes/216`).
///
/// What is here is what the surfaces' own call sites ask for and nothing more: a url, the box it
/// will fill, whether the caller means to read the pixels back, a way to stop, a read ahead of the
/// viewer, and a look in the cache that answers in place. Drawing is not here, because drawing is
/// what a surface is for.
public protocol Pictures: AnyObject, Sendable {
    /// Reads one picture. `done` runs once, on the main queue, with a ``Picture/painted(_:)`` or a
    /// ``Picture/unpainted(reason:)``. A read that was stopped answers neither.
    func fetch(_ want: Want, done: @escaping (Picture) -> Void) -> Stop

    /// Reads a picture into the cache before anything draws it, at the size the box will ask for,
    /// so a row is already painted when the viewer reaches it.
    ///
    /// `done` runs whether the picture arrived or not, because a failed read has to stop a wait as
    /// surely as a good one.
    func warm(_ want: Want, done: @escaping () -> Void)

    /// The picture already in memory, or nothing, answered in place.
    ///
    /// Not in the android family's port, and it is here for a difference in the toolkits rather
    /// than a difference of opinion: a Compose picture reads its loader's memory cache inside the
    /// draw, while SwiftUI runs a view's `task` only after that view's first frame is on screen.
    /// Without this a row scrolled back would draw one blank frame per tile for pictures that are
    /// already in hand, which is exactly what the cache inside `Art.swift` was written to avoid.
    func kept(_ want: Want) -> Picture?
}

extension Pictures {
    /// A read ahead of the viewer that nobody is waiting on.
    public func warm(_ want: Want) { warm(want) {} }

    /// One picture, awaited. Cancelling the task that asked stops the read, and the answer is then
    /// an ``Picture/unpainted(reason:)``: a Swift continuation has to be resumed exactly once, so
    /// "answers neither" is the port's contract with ``fetch(_:done:)`` and not with this.
    public func picture(_ want: Want) async -> Picture {
        let read = PictureRead()
        return await withTaskCancellationHandler {
            await withCheckedContinuation { (continuation: CheckedContinuation<Picture, Never>) in
                read.start(continuation) { done in fetch(want, done: done) }
            }
        } onCancel: {
            read.cancel()
        }
    }
}

/// One awaited read, so it answers exactly once however the race falls: a picture that arrives
/// after the task was cancelled is dropped, and a cancellation that lands before the read began
/// stops it as soon as there is something to stop.
final class PictureRead: @unchecked Sendable {
    private let lock = NSLock()
    private var waiting: CheckedContinuation<Picture, Never>?
    private var stop: Stop?
    private var cancelled = false

    func start(
        _ continuation: CheckedContinuation<Picture, Never>,
        _ begin: (@escaping (Picture) -> Void) -> Stop
    ) {
        lock.lock()
        waiting = continuation
        lock.unlock()
        let stop = begin { [weak self] picture in self?.landed(picture) }
        lock.lock()
        let late = cancelled
        self.stop = stop
        lock.unlock()
        if late {
            stop.stop()
            landed(.unpainted(reason: "stopped"))
        }
    }

    func cancel() {
        lock.lock()
        cancelled = true
        let stop = self.stop
        lock.unlock()
        stop?.stop()
        landed(.unpainted(reason: "stopped"))
    }

    private func landed(_ picture: Picture) {
        lock.lock()
        let continuation = waiting
        waiting = nil
        lock.unlock()
        continuation?.resume(returning: picture)
    }
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
    /// True when that language runs right to left: the interface mirrors, the player never does.
    public var rtl: Bool = false
    public var detail: TitleDetail? = nil
    public var notice: String? = nil
    /// How many notices have been said in this run. The slot is app wide and a notice is one time,
    /// so a screen that acts records this count first and reads the slot as its own answer only
    /// once the count has moved. Without it a screen shows words another screen left behind.
    public var notices: Int = 0
    public var brand: String = ""
    public var server: String = ""
    /// True when that server is reached over plain http. The screen that shows the address marks
    /// it, so nobody runs on an unencrypted connection without being told.
    public var serverPlain: Bool = false
    public var positionMs: Int64 = 0
    public var lengthMs: Int64 = 0
    public var audio: [Track] = []
    public var text: [Track] = []
    public var unreadNotifications: Int = 0

    public init() {}
}

public protocol Shell: AnyObject {
    @MainActor var seen: Seen { get }
    var player: Player { get }
    var reel: Reel { get }
    /// Where a surface's pictures come from.
    ///
    /// No default, and no stand-in for one. A port that is defaulted is a seam that rots: a surface
    /// compiles against it and gets refusals at run time instead of pictures. `App/Main.swift` and
    /// `PhoneApp/Main.swift` build `UrlPictures` and `CoreHost/Host.swift` carries it, so a family
    /// that forgets to plug one in fails to compile, which is where that mistake belongs. The draw
    /// sites take it by value for the same reason (`SurfacesShared/Art.swift`).
    var pictures: Pictures { get }
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
    func refreshNotifications()
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
