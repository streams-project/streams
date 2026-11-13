// How this family reads a picture, and the one file of it that may fetch one.
import Foundation
import ImageIO
import Ports
import UIKit

/// The ``Pictures`` port over URLSession and ImageIO.
///
/// URLSession stays, because the loader was never what was missing: what was missing was a
/// boundary. Before this, `SurfacesShared/Art.swift`, `SurfacesTV/browse/HeroCard.swift`,
/// `SurfacesPhone/profiles/Gate.swift` and `SurfacesPhone/browse/Notifications.swift` each drew
/// through SwiftUI's `AsyncImage`, which fetches the address itself inside the toolkit, and the ring
/// check's ban on the word `URLSession` passed every run because no surface ever typed it
/// (`notes/216`). The port is the permanent part: it outlives the toolkit the surfaces are written
/// in, so a UIKit rewrite has a boundary to aim at rather than this hole to reproduce.
///
/// Two things that lived in a surface live here now, because they were always this side of the line:
///
/// * The cache of pictures already drawn, which was `Pictures.now` / `Pictures.keep` inside
///   `Art.swift`. A picture cache is an adapter's business - what it costs in memory is a fact about
///   the device, not about the screen that drew it.
/// * The disk and memory budget for the fetch, which both entry points set on `URLCache.shared`
///   before anything drew. Shared state set up next to `@main` is the same seam by another name: it
///   is given to this session alone, and a build that does not construct this class now gets no
///   pictures at all rather than pictures through somebody else's cache.
public final class UrlPictures: Pictures, @unchecked Sendable {
    private let session: URLSession
    private let most: Int

    private let lock = NSLock()
    private var pictures: [Want: UIImage] = [:]
    private var order: [Want] = []

    /// - Parameters:
    ///   - memoryBytes: what the fetch keeps in memory, encoded. 64 MB, which is what both entry
    ///     points asked `URLCache.shared` for before this class existed.
    ///   - diskBytes: what it keeps between runs, encoded. 512 MB, the same way.
    ///   - most: how many decoded pictures are held. 400, which is what `Art.swift` held - a phone
    ///     Home of 23 rows and a television page of posters both scroll back over more than a row.
    public init(memoryBytes: Int = 64 << 20, diskBytes: Int = 512 << 20, most: Int = 400) {
        let config = URLSessionConfiguration.default
        config.urlCache = URLCache(memoryCapacity: memoryBytes, diskCapacity: diskBytes, directory: nil)
        config.requestCachePolicy = .returnCacheDataElseLoad
        self.session = URLSession(configuration: config)
        self.most = most
    }

    public func kept(_ want: Want) -> Picture? {
        remembered(want).map { .painted($0) }
    }

    /// The sizes a picture is ever held at.
    ///
    /// Every box a draw site asks for is taken up to the next rung and the picture is held under
    /// that rung, so two tiles eight points apart share one entry instead of holding the same
    /// poster twice. The steps go up by about a half each time, which is the whole of what the
    /// ladder costs: a box one pixel over a rung decodes at the next one, which is at most twice
    /// the pixels it needed. A ladder of doubling steps would have cost four times that.
    ///
    /// Above the top rung there is nothing left to sample away, so the picture is kept whole.
    static let rungs = [128, 180, 256, 360, 512, 720, 1024, 1440, 2048]

    /// The rung a box is held at. 0 is the picture's own size, and it stays its own largest entry.
    static func rung(_ box: Int) -> Int {
        guard box > 0 else { return 0 }
        return rungs.first { box <= $0 } ?? 0
    }

    /// The one want a picture is filed under.
    ///
    /// Two things are folded out of the key here, and both are facts about this adapter rather than
    /// about the port:
    ///
    /// * The box becomes its rung, so near sizes share an entry (above).
    /// * `pixels` goes, because it does not change the bytes this adapter produces: every picture
    ///   it hands over is already backed by pixels a caller may read. Before this, a billboard read
    ///   for its tint (`pixels: true`) and the same billboard drawn beside it (`pixels: false`)
    ///   were two keys, so the card decoded its artwork twice and held both - the opposite of what
    ///   the comment at each of those two call sites claimed.
    ///
    /// The shorter side goes with them: the decode takes the longer side (``decode(_:_:)``), so a
    /// box of 360 by 200 and one of 360 by 300 are one picture, and it is this one.
    static func key(_ want: Want) -> Want {
        let side = rung(max(want.wide, want.high))
        return Want(url: want.url, wide: side, high: side)
    }

    public func fetch(_ want: Want, done: @escaping (Picture) -> Void) -> Stop {
        if let picture = remembered(want) {
            answer(.painted(picture), done)
            return Stop {}
        }
        guard let address = URL(string: want.url), address.scheme != nil else {
            answer(.unpainted(reason: "bad_url"), done)
            return Stop {}
        }
        let once = Once()
        let task = session.dataTask(with: address) { [weak self] data, response, error in
            guard once.raise() else { return }
            let answered: Picture
            if let error {
                answered = .unpainted(reason: (error as? URLError)?.code == .timedOut ? "timeout" : "offline")
            } else if let http = response as? HTTPURLResponse, !(200...299).contains(http.statusCode) {
                answered = .unpainted(reason: "status_\(http.statusCode)")
            } else if let data, let picture = Self.decode(data, Self.key(want)) {
                self?.remember(want, picture)
                answered = .painted(picture)
            } else {
                answered = .unpainted(reason: "not_a_picture")
            }
            DispatchQueue.main.async { done(answered) }
        }
        task.resume()
        return Stop {
            // Closed before the cancel, because cancelling makes the task report an error of its
            // own and a read that was stopped answers neither.
            once.close()
            task.cancel()
        }
    }

    public func warm(_ want: Want, done: @escaping () -> Void) {
        _ = fetch(want) { _ in done() }
    }

    /// Bytes into a picture.
    ///
    /// A box of 0 by 0 is the picture's own size. Given a box, ImageIO decodes the nearest sample
    /// size down to it instead of the whole frame, so a poster drawn in a 120 pt tile does not sit
    /// in memory at 1000 by 1500. The largest side
    /// of the box is what is asked for: the shape is the picture's and the box's shorter side would
    /// decode a picture too small to fill it.
    static func decode(_ data: Data, _ want: Want) -> UIImage? {
        let box = max(want.wide, want.high)
        guard box > 0, let source = CGImageSourceCreateWithData(data as CFData, nil) else {
            return UIImage(data: data)
        }
        let asked: [CFString: Any] = [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true,
            kCGImageSourceThumbnailMaxPixelSize: box,
        ]
        guard let smaller = CGImageSourceCreateThumbnailAtIndex(source, 0, asked as CFDictionary) else {
            return UIImage(data: data)
        }
        return UIImage(cgImage: smaller)
    }

    func remembered(_ want: Want) -> UIImage? {
        lock.lock()
        defer { lock.unlock() }
        return pictures[Self.key(want)]
    }

    /// Keeps a picture, and forgets the one longest unasked for once there are more than `most`.
    func remember(_ want: Want, _ picture: UIImage) {
        let key = Self.key(want)
        lock.lock()
        defer { lock.unlock() }
        if pictures[key] == nil {
            order.append(key)
            if order.count > most { pictures.removeValue(forKey: order.removeFirst()) }
        }
        pictures[key] = picture
    }

    /// Every answer reaches the caller the same way, on the main queue, whether it came from the
    /// network or from memory: a port that answers in place sometimes and later other times makes
    /// the order a surface's own state changes in depend on a cache hit.
    private func answer(_ picture: Picture, _ done: @escaping (Picture) -> Void) {
        DispatchQueue.main.async { done(picture) }
    }
}

/// One answer, once. A stopped read closes it and the task's own cancellation error is dropped.
private final class Once: @unchecked Sendable {
    private let lock = NSLock()
    private var spent = false

    func raise() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        if spent { return false }
        spent = true
        return true
    }

    func close() {
        lock.lock()
        spent = true
        lock.unlock()
    }
}
