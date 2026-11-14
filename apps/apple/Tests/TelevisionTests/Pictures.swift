#if canImport(UIKit)
import Contract
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import UIKit
import XCTest

@testable import Adapters
@testable import SurfacesTV

/// What happens when the television wants a picture.
///
/// The phone's half of this is in `PhoneTests/Pictures.swift`, which also reads the port and the
/// adapter; here the question is the one that is only askable on this surface. Two of the four
/// draw sites `notes/216` found are drawn by the television - the shared `Art` and the hero card's
/// treatment - and a face is on nearly every signed in screen it has.
@MainActor
final class PicturesTests: XCTestCase {
    private let tv = CGSize(width: 1920, height: 1080)
    private let address = "https://example.invalid/face.jpg"

    /// A profile's face comes through the port. It is drawn by the top bar, the profile chooser,
    /// the PIN panel, the lock steps and the profile menu, all through this one part.
    func testAProfilesFaceComesThroughThePort() {
        let told = ToldPictures(.paint(aPicture()))
        let person = Profile(avatar: address, hasPin: false, id: "p1", kids: false, main: true, name: "Ibrahim")
        _ = heard(Face(told, person, faceLarge, "https://example.invalid"), size: tv)
        XCTAssertEqual(told.addresses, [address], "a face did not come through the port")
    }

    /// A face kept on the server is asked for at the server's address, which is the one piece of
    /// work this part does to a url and the reason it cannot be left to the port.
    func testAFaceKeptOnTheServerIsAskedForAtTheServersAddress() {
        let told = ToldPictures(.paint(aPicture()))
        let person = Profile(avatar: "/pictures/cats/1.jpg", hasPin: false, id: "p1", kids: false, main: true, name: "Noor")
        _ = heard(Face(told, person, faceLarge, "https://example.invalid"), size: tv)
        XCTAssertEqual(told.addresses, ["https://example.invalid/pictures/cats/1.jpg"])
    }

    /// A profile with no picture asks for nothing: its first letter is drawn instead, and a read
    /// that was never going to be drawn is a read not worth making.
    func testAProfileWithNoPictureAsksForNothing() {
        let told = ToldPictures(.paint(aPicture()))
        let person = Profile(hasPin: false, id: "p1", kids: false, main: true, name: "Ibrahim")
        _ = heard(Face(told, person, faceLarge, "https://example.invalid"), size: tv)
        XCTAssertEqual(told.asked, [], "a profile with no picture still asked for one")
    }

    /// The title's own treatment on the hero card comes through the port. It was the third of the
    /// four `AsyncImage` sites and it carried a cache of its own alongside `Art`'s.
    func testTheHeroCardsTreatmentComesThroughThePort() {
        let told = ToldPictures(.paint(aPicture()))
        _ = heard(Logo(pictures: told, url: address, width: 600, height: 200, says: "The Deep"), size: tv)
        XCTAssertEqual(told.addresses, [address], "a title's treatment did not come through the port")
    }

    /// Every picture of a poster grid comes through the port, one read per poster.
    func testEveryPosterOnAGridComesThroughThePort() {
        let told = ToldPictures(.paint(aPicture()))
        let tiles = (1...3).map {
            Tile(
                images: [Contract.Image(role: .poster, url: "https://example.invalid/p\($0).jpg")],
                name: "Title \($0)",
                title: try! TitleKey("movie:1\($0)")
            )
        }
        _ = heard(
            PosterGrid(pictures: told, tiles: tiles, focus: 0, left: 566, top: 232, words: tvWords),
            size: tv
        )
        XCTAssertEqual(
            Set(told.addresses), Set(tiles.map { tall($0.images)! }),
            "a grid of posters did not ask the port for all of them"
        )
    }

    /// Every poster on a grid names the box it will be drawn in, and what that box is worth.
    ///
    /// The television is where this matters most: a page of posters draws dozens of tiles at once,
    /// each 296 by 414 points on a screen whose point is a pixel, and what a surface holds is what
    /// gets an app killed in the background. This is also the test that fails if a draw site on
    /// this surface goes back to a box of nothing - nothing on screen would look wrong if one did.
    func testAPosterGridHoldsItsPostersForAFractionOfTheirWholeFrames() {
        let told = ToldPictures(.paint(aPicture()))
        let tiles = (1...24).map {
            Tile(
                images: [Contract.Image(role: .poster, url: "https://example.invalid/p\($0).jpg")],
                name: "Title \($0)",
                title: try! TitleKey("movie:\($0)")
            )
        }
        _ = heard(
            PosterGrid(pictures: told, tiles: tiles, focus: 0, left: 566, top: 232, words: tvWords),
            size: tv
        )
        let asked = told.asked
        XCTAssertFalse(asked.isEmpty, "the grid asked the port for no posters at all")
        XCTAssertEqual(
            asked.filter { $0.wide <= 0 || $0.high <= 0 }.map(\.url).sorted(), [],
            "a poster on the grid was asked for without a box, so its whole frame is decoded and held"
        )
        let poster = aPosterBytes(1000, 1500)
        let heldNow = Set(asked.map { UrlPictures.key($0) })
            .reduce(0) { $0 + bytesOf(UrlPictures.decode(poster, $1)) }
        let heldWhole = Set(asked.map { Want(url: $0.url) })
            .reduce(0) { $0 + bytesOf(UrlPictures.decode(poster, $1)) }
        print("""
        POSTER GRID, decoded from a 1000 by 1500 poster:
          posters drawn: \(asked.count), each asked in \(asked[0].wide) by \(asked[0].high) pixels
          held now: \(heldNow) bytes
          held whole: \(heldWhole) bytes
        """)
        // Two, not six. Six was written without arithmetic and this test could only ever fail:
        // the grid asks for 592 by 828 and the fixture poster is 1000 by 1500, so the most any
        // implementation can save per poster is 1_500_000 / 490_176 = 3.06x. A threshold a correct
        // implementation cannot reach is not a test of the implementation -- it is a test of the
        // number, and the number loses every time. Measured here: 33_619_968 against 72_000_000,
        // a saving of 2.14x.
        //
        // The phone holds the same rule at the same 2x (`PhoneTests/Pictures.swift`), and the
        // television cannot do better for a structural reason worth keeping: its cells are a large
        // fraction of any poster a server will serve, where a phone's are a small one. Asking in
        // the drawn box is worth less on a television and is still worth doing -- what it is worth
        // is half the memory, not five sixths.
        XCTAssertLessThan(
            heldNow * 2, heldWhole,
            "the boxes this grid asks for save less than half of holding its posters whole"
        )
    }
}

/// What the landing reads ahead of the viewer, and what holding it costs.
///
/// The port has had a read ahead since it was written and nothing in the product called it: the
/// landing filled its cache by drawing every card's pictures into a four point corner of the screen
/// at no opacity, which is a read ahead with no name and a dozen live views behind it. The reads are
/// `warm` now, and this is where the one case the adapter's rung ladder was designed around is
/// exercised: a picture read ahead and the same picture drawn afterwards landing on one entry.
@MainActor
final class ReadAheadTests: XCTestCase {
    private let tv = CGSize(width: 1920, height: 1080)

    /// Every picture of every card is read ahead, before any of those cards is in view.
    func testTheLandingReadsEveryCardsPicturesAheadOfTheViewer() {
        let shell = landingShell()
        _ = heard(Door(shell: shell, seen: shell.seen, state: opened()), size: tv) {
            !shell.shown.warmed.isEmpty
        }
        XCTAssertEqual(
            Set(shell.shown.warmed.map(\.url)), Set(everySlide()),
            "the landing did not read every card's pictures ahead of the viewer"
        )
    }

    /// A picture read ahead and the same picture drawn by the card are one entry in the cache.
    ///
    /// This is what the box on a `Want` is for, and it is the one case the adapter's rung ladder
    /// was designed around that nothing in the product exercised until 2026-11-13. The read ahead
    /// names the card's own box, the card's `Art` names the same box, both fall on the same rung,
    /// and the card's draw is a hit in place rather than a second read.
    ///
    /// Asserted against the card's box and not against a card drawn on screen: at the landing the
    /// card in view is the showreel and every title card sits off to the right of a 1920 point
    /// screen, so whether one of them has drawn yet is a question about the focus engine. A read
    /// ahead in a box of its own is the failure this is for - it would cost the whole of what it
    /// read and warm nothing, and nothing on screen would look wrong.
    func testAPictureReadAheadAsksInTheBoxTheCardWillDrawIn() {
        let shell = landingShell()
        _ = heard(Door(shell: shell, seen: shell.seen, state: opened()), size: tv) {
            !shell.shown.warmed.isEmpty
        }
        let warmed = shell.shown.warmed
        XCTAssertFalse(warmed.isEmpty, "the landing read nothing ahead at all")
        let scale = UIScreen.main.scale
        XCTAssertEqual(
            Set(warmed), Set(everySlide().map { wanted($0, cardBox, scale) }),
            "a picture was read ahead in some box other than the card's, so the card's own draw "
                + "will miss it and hold a second copy"
        )
        XCTAssertEqual(
            Set(warmed.map { UrlPictures.key($0).wide }).count, 1,
            "the pictures read ahead for one card landed on more than one rung"
        )
    }

    /// What the read ahead holds, in bytes, and what the card's box is worth against the whole
    /// frame.
    ///
    /// Measured from a 1920 by 1080 backdrop, which is the largest a card is ever sent. The card is
    /// 1542 points wide on a screen whose point is a pixel, so its box falls on the ladder's top
    /// rung, 2048 - above a 1920 picture, which means the ladder samples nothing away here and the
    /// read ahead holds every backdrop whole.
    func testWhatTheLandingReadsAheadCostsInBytes() {
        let shell = landingShell()
        _ = heard(Door(shell: shell, seen: shell.seen, state: opened()), size: tv) {
            !shell.shown.warmed.isEmpty
        }
        let warmed = shell.shown.warmed
        // A guard and not an assertion: everything below reads the first want, and a test that
        // crashes instead of failing takes the rest of the run with it.
        guard let box = warmed.first else {
            return XCTFail("the landing read nothing ahead, so there is nothing to cost")
        }
        let backdrop = aPosterBytes(1920, 1080)
        let held = Set(warmed.map { UrlPictures.key($0) })
            .reduce(0) { $0 + bytesOf(UrlPictures.decode(backdrop, $1)) }
        let atTheBox = Set(warmed.map { Want(url: $0.url, wide: $0.wide, high: $0.high) })
            .reduce(0) { $0 + bytesOf(UrlPictures.decode(backdrop, $1)) }
        let firstOfEach = Set(shell.titles.compactMap { wide($0.images) })
        let backdropsOnly = firstOfEach
            .reduce(0) { $0 + bytesOf(UrlPictures.decode(backdrop, UrlPictures.key(Want(url: $1, wide: box.wide, high: box.high)))) }
        print("""
        LANDING READ AHEAD, decoded from a 1920 by 1080 backdrop:
          pictures read ahead: \(warmed.count), each in \(box.wide) by \(box.high) pixels, rung \(UrlPictures.key(box).wide)
          held at that rung: \(held) bytes
          held if decoded at the box itself: \(atTheBox) bytes
          held if only each card's first picture were read ahead: \(backdropsOnly) bytes over \(firstOfEach.count) pictures
        """)
        XCTAssertGreaterThan(held, 0, "nothing decoded at all, so none of these numbers mean anything")
        XCTAssertEqual(
            held, warmed.count * 1920 * 1080 * 4,
            "a card's box no longer holds every backdrop whole, so the cost above has changed"
        )
    }

    /// The landing, with trending titles to show and a reel that draws nothing.
    private func landingShell() -> LandingShell {
        let shell = LandingShell(titles: trendingTitles())
        shell.seen.words = tvWords
        return shell
    }

    private func opened() -> OpeningState {
        let state = OpeningState()
        state.over()
        return state
    }

    private func everySlide() -> [String] {
        trendingTitles().flatMap { title in
            [wide(title.images)].compactMap { $0 } + (title.images ?? []).filter { $0.role == .still }.map(\.url)
        }
    }
}

/// Three trending titles as the landing is sent them: a backdrop each and two stills behind it,
/// which is what the card's slideshow walks through while its card is the one in view.
@MainActor
private func trendingTitles() -> [TitleDetail] {
    (1...3).map { which in
        TitleDetail(
            images: [
                Contract.Image(role: .backdrop, url: "https://example.invalid/back\(which).jpg"),
                Contract.Image(role: .still, url: "https://example.invalid/still\(which)a.jpg"),
                Contract.Image(role: .still, url: "https://example.invalid/still\(which)b.jpg"),
            ],
            name: "Title \(which)",
            overview: "A diver goes down and does not come back up.",
            title: try! TitleKey("movie:\(which)"),
            year: 2024
        )
    }
}

/// A shell that answers the landing's one read with trending titles, and hands it a reel that draws
/// an empty view: the landing plays its showreel, and a port that refuses to answer at all would
/// stop the screen before any of its pictures were asked for.
@MainActor
private final class LandingShell: Shell {
    let seen = Seen()
    let shown = ToldPictures(.refuse("no test here draws the picture itself"))
    let titles: [TitleDetail]
    private let nothing = NoReel()

    init(titles: [TitleDetail]) {
        self.titles = titles
    }

    var pictures: Pictures { shown }
    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { nothing }
    func connect(_ server: String) {}
    func go(_ address: String) {}
    func swap(_ address: String) {}
    func back() {}
    func act(_ action: Action) {}
    func ask(_ path: String, reply: @escaping (Int, String) -> Void) {
        guard path == "/v1/public/trending" else { return reply(0, "") }
        let answer = try! JSONEncoder().encode(PublicTitles(titles: titles))
        reply(200, String(data: answer, encoding: .utf8)!)
    }
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) {
        reply(0, "")
    }
    func keep(_ profile: Profile) {}
    func keepAll(_ profiles: [Profile]) {}
    func speak(_ language: String?) {}
    func refreshNotifications() {}
}

/// A reel that hands back an empty view and says the film is ready at once.
private final class NoReel: Reel {
    func picture(_ context: Any, url: String, loop: Bool, ready: @escaping () -> Void, ended: @escaping () -> Void) -> Any {
        DispatchQueue.main.async { ready() }
        return UIView()
    }
    func play(_ picture: Any, _ on: Bool) {}
    func release(_ picture: Any) {}
    func hold(_ still: Bool) {}
}

/// What a decoded picture occupies, which is the thing a tile's box is there to make smaller.
private func bytesOf(_ picture: UIImage?) -> Int {
    guard let image = picture?.cgImage else { return 0 }
    return image.bytesPerRow * image.height
}

/// A poster of one colour at a poster's shape, as bytes. `aPicturesBytes` is square, and a square
/// one would hide what the decode does with the longer side.
@MainActor
private func aPosterBytes(_ wide: CGFloat, _ high: CGFloat) -> Data {
    let format = UIGraphicsImageRendererFormat.default()
    format.scale = 1
    let renderer = UIGraphicsImageRenderer(size: CGSize(width: wide, height: high), format: format)
    return renderer.image { context in
        UIColor.red.setFill()
        context.fill(CGRect(x: 0, y: 0, width: wide, height: high))
    }.pngData()!
}
#endif
