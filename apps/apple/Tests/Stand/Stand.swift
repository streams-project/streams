#if canImport(UIKit)
import Foundation
import Ports
import UIKit

/// The ``Pictures`` port a test can look into: every ``Want`` it was handed, and whatever it was
/// told to answer with.
///
/// It exists because the port is the whole of what a draw site may do about a picture now, so
/// "did this screen go through the port" is a question with an answer, where before it was a
/// question about what SwiftUI did inside `AsyncImage` and could not be asked at all.
public final class ToldPictures: Pictures, @unchecked Sendable {
    /// What this port does when something asks it for a picture.
    public enum Answer {
        case paint(UIImage)
        case refuse(String)
        /// Never answers, which is a slow network and the state a screen waits in.
        case silence
    }

    private let lock = NSLock()
    private var answer: Answer
    private var held: [Want: UIImage] = [:]
    private var askedFor: [Want] = []
    private var warmedFor: [Want] = []
    private var stops = 0

    public init(_ answer: Answer = .refuse("no test told this port what to answer")) {
        self.answer = answer
    }

    /// Everything a view asked this port to read, in the order it asked.
    public var asked: [Want] {
        lock.lock()
        defer { lock.unlock() }
        return askedFor
    }

    /// Every address a view asked for, for a test that cares about the url alone.
    public var addresses: [String] { asked.map(\.url) }

    public var warmed: [Want] {
        lock.lock()
        defer { lock.unlock() }
        return warmedFor
    }

    /// How many reads were stopped.
    public var stopped: Int {
        lock.lock()
        defer { lock.unlock() }
        return stops
    }

    /// A picture this port already holds, as an adapter holds one it has read before.
    public func hold(_ want: Want, _ picture: UIImage) {
        lock.lock()
        held[want] = picture
        lock.unlock()
    }

    public func fetch(_ want: Want, done: @escaping (Picture) -> Void) -> Stop {
        lock.lock()
        askedFor.append(want)
        let answer = self.answer
        let kept = held[want]
        lock.unlock()
        if let kept {
            done(.painted(kept))
            return Stop {}
        }
        switch answer {
        case .paint(let picture): done(.painted(picture))
        case .refuse(let reason): done(.unpainted(reason: reason))
        case .silence: break
        }
        return Stop { [weak self] in
            guard let self else { return }
            self.lock.lock()
            self.stops += 1
            self.lock.unlock()
        }
    }

    public func warm(_ want: Want, done: @escaping () -> Void) {
        lock.lock()
        warmedFor.append(want)
        lock.unlock()
        done()
    }

    public func kept(_ want: Want) -> Picture? {
        lock.lock()
        defer { lock.unlock() }
        return held[want].map { .painted($0) }
    }
}

/// A picture of one flat colour, as small as a picture can be, for a test that needs a real one.
///
/// Drawn at scale 1 on purpose, so `side` is pixels as well as points. At the screen's own scale a
/// 32 pt picture on this simulator is 96 pixels of PNG, and a test that says 32 and reads 96 back
/// is measuring the renderer rather than what it meant to measure.
@MainActor
public func aPicture(_ side: CGFloat = 4, _ colour: UIColor = .red) -> UIImage {
    let format = UIGraphicsImageRendererFormat.default()
    format.scale = 1
    let renderer = UIGraphicsImageRenderer(size: CGSize(width: side, height: side), format: format)
    return renderer.image { context in
        colour.setFill()
        context.fill(CGRect(x: 0, y: 0, width: side, height: side))
    }
}

/// The same picture as bytes, for a test that reads the adapter's own decode.
@MainActor
public func aPicturesBytes(_ side: CGFloat = 4, _ colour: UIColor = .red) -> Data {
    aPicture(side, colour).pngData()!
}
#endif
