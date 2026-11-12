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
@testable import SurfacesPhone

/// What happens when this family wants a picture.
///
/// Every test here is about the one thing `notes/216` found: four draw sites fetched their own
/// pictures through SwiftUI's `AsyncImage`, which meant the app had two picture paths and no way to
/// ask which one a screen used. There is one path now, so each question below has an answer:
/// a draw site asks the port, the port is the only thing that fetches, and the cache and the decode
/// live behind it.
///
/// The first group reads the port, the second the adapter, and the third holds real parts of the
/// phone surface in a window and reads what they asked for.
@MainActor
final class PicturesTests: XCTestCase {
    private let phone = CGSize(width: 393, height: 852)
    private let address = "https://example.invalid/poster.jpg"

    // MARK: - The port

    /// An awaited read carries back what the port painted, and the picture is the one it was given.
    func testAnAwaitedReadCarriesBackWhatThePortPainted() async {
        let drawn = aPicture(8)
        let told = ToldPictures(.paint(drawn))
        guard case .painted(let held) = await told.picture(Want(url: address)) else {
            return XCTFail("the port painted a picture and the read did not carry it back")
        }
        XCTAssertEqual((held as? UIImage)?.size, drawn.size, "the picture that came back is not the one painted")
        XCTAssertEqual(told.addresses, [address], "the read did not ask for the address it was given")
    }

    /// Cancelling the task that asked stops the read at the port and answers rather than hanging.
    ///
    /// Both halves matter. A port that is not told to stop goes on fetching for a row the viewer has
    /// scrolled past, and a Swift continuation that is never resumed leaves the screen that awaited
    /// it suspended for the life of the app.
    func testCancellingAReadStopsItAtThePortAndStillAnswers() async {
        let told = ToldPictures(.silence)
        let want = Want(url: address)
        let reading = Task { () -> String in
            switch await told.picture(want) {
            case .painted: return "painted"
            case .unpainted(let reason): return reason
            }
        }
        var turns = 0
        while told.asked.isEmpty && turns < 2000 {
            await Task.yield()
            turns += 1
        }
        XCTAssertEqual(told.asked, [want], "the read never reached the port")
        reading.cancel()
        let answer = await reading.value
        XCTAssertEqual(answer, "stopped", "a cancelled read did not answer that it was stopped")
        XCTAssertEqual(told.stopped, 1, "the port was never told to stop the read")
    }

    /// A read ahead of the viewer answers whether the picture arrived or not, because the one kind
    /// of caller that waits on it has to stop waiting either way.
    func testAReadAheadAnswersEvenWhenThePictureDoesNotArrive() {
        let told = ToldPictures(.refuse("nothing here"))
        let want = Want(url: address, wide: 240, high: 360)
        var answered = false
        told.warm(want) { answered = true }
        XCTAssertTrue(answered, "a read ahead of a picture that never arrives never finished")
        XCTAssertEqual(told.warmed, [want], "the read ahead did not reach the port, or lost its box")
    }

    // MARK: - The adapter

    /// Bytes become a picture, at the picture's own size when no box was asked for.
    func testTheAdapterDecodesBytesIntoAPicture() {
        let picture = UrlPictures.decode(aPicturesBytes(32), Want(url: address))
        XCTAssertEqual(picture?.size, CGSize(width: 32, height: 32), "the bytes did not decode at their own size")
    }

    /// Given a box, the adapter decodes down to it rather than holding the whole frame.
    ///
    /// This is the one thing the port's `wide` and `high` are for, and it is worth a test of its own
    /// because nothing on screen would look wrong if it were ignored: a poster drawn in a 120 pt
    /// tile would simply sit in memory at its full size, which is a cost nobody sees.
    func testTheAdapterDecodesDownToTheBoxItWasAskedFor() {
        let want = Want(url: address, wide: 16, high: 16)
        guard let picture = UrlPictures.decode(aPicturesBytes(64), want) else {
            return XCTFail("a picture asked for in a box did not decode at all")
        }
        XCTAssertEqual(
            max(picture.size.width, picture.size.height), 16,
            "the adapter decoded the whole frame instead of the box it was asked for"
        )
    }

    /// A picture the adapter has read before is answered in place, which is what lets a row that
    /// scrolls back draw in the same frame instead of one frame later.
    func testTheAdapterAnswersInPlaceForAPictureItHasAlreadyRead() {
        let pictures = UrlPictures()
        let want = Want(url: address)
        XCTAssertNil(pictures.kept(want), "the adapter answered a picture it has never read")
        pictures.remember(want, aPicture(8))
        guard case .painted(let held) = pictures.kept(want) else {
            return XCTFail("the adapter forgot a picture it had just read")
        }
        XCTAssertEqual((held as? UIImage)?.size, CGSize(width: 8, height: 8))
        XCTAssertNil(
            pictures.kept(Want(url: address, wide: 16, high: 16)),
            "a picture asked for in a box is a different picture and may not answer for another box"
        )
    }

    /// The cache has a limit and the picture longest unasked for is the one that goes.
    func testTheAdapterForgetsTheOldestPictureOverItsLimit() {
        let pictures = UrlPictures(most: 2)
        let wants = (0..<3).map { Want(url: "https://example.invalid/\($0).jpg") }
        for want in wants { pictures.remember(want, aPicture(4)) }
        XCTAssertNil(pictures.kept(wants[0]), "the cache held more pictures than its limit")
        XCTAssertNotNil(pictures.kept(wants[1]))
        XCTAssertNotNil(pictures.kept(wants[2]))
    }

    /// An address that is not one is refused at once and said out loud, rather than left as a read
    /// that never answers.
    func testTheAdapterRefusesAnAddressThatIsNotOne() async {
        let answer = await UrlPictures().picture(Want(url: "poster.jpg"))
        guard case .unpainted(let reason) = answer else {
            return XCTFail("a picture was painted from something that is not an address")
        }
        XCTAssertEqual(reason, "bad_url")
    }

    // MARK: - The surface

    /// The shared view every screen draws its pictures through asks the port for the address it was
    /// given, and nothing else fetches it.
    ///
    /// This is the test `notes/216` could not have been written against: with `AsyncImage` in here
    /// the fetch happened inside SwiftUI and no stand-in could see it.
    func testArtAsksThePortForTheAddressItWasGiven() {
        let told = ToldPictures(.paint(aPicture()))
        _ = heard(Art(told, address), size: phone)
        XCTAssertEqual(told.addresses, [address], "the shared picture view did not go through the port")
    }

    /// A refusal ends the wait. `Art` tells its caller the picture is settled whether one arrived or
    /// not, because the one screen that passes that closure uses it to decide when to draw.
    func testArtSaysThePictureIsSettledEvenWhenThePortRefuses() {
        let told = ToldPictures(.refuse("offline"))
        let box = Settled()
        _ = heard(Art(told, address) { box.settled = true }, size: phone)
        XCTAssertEqual(told.addresses, [address], "the view did not ask the port at all")
        XCTAssertTrue(box.settled, "a refused picture left the screen waiting for one that is not coming")
    }

    /// A picture the port already holds is drawn without a second read.
    func testArtDrawsAPictureAlreadyInHandWithoutAskingForItAgain() {
        let told = ToldPictures(.silence)
        told.hold(Want(url: address), aPicture())
        _ = heard(Art(told, address), size: phone)
        XCTAssertEqual(told.asked, [], "a picture already in hand was read a second time")
    }

    /// A real part of the phone surface, held in a window: the poster on a tile comes through the
    /// port, at the address the catalogue gave for it.
    func testATilesPosterComesThroughThePort() {
        let (_, seen) = homeSeen()
        let tile = (seen.screen?.sections ?? []).first { $0.kind == .row }!.tiles!.first!
        let told = ToldPictures(.paint(aPicture()))
        _ = heard(
            TileCard(
                pictures: told, tile: tile, wide: tileWide, high: tileHigh, rank: nil, percent: nil,
                words: seen.words, open: {}
            ),
            size: phone
        )
        XCTAssertEqual(
            told.addresses, [tall(tile.images)!],
            "a tile's poster did not come through the port, or came through it at another address"
        )
    }

    /// A notification's poster takes the same path, which is the whole point of there being one
    /// path: this row fetched its own picture through `AsyncImage` before this, and it is the
    /// fourth of the four sites `notes/216` found.
    func testANotificationsPosterComesThroughThePort() {
        let poster = "https://example.invalid/notice.jpg"
        let item = NoticeItem(
            date: try! CalendarDay("2026-11-12"),
            id: "n1",
            kind: .newArrival,
            tile: Tile(
                images: [Contract.Image(role: .poster, url: poster)],
                name: "The Deep",
                title: try! TitleKey("movie:11")
            ),
            unread: true
        )
        let feed = Notifications(cursor: nil, hasMore: false, items: [item])
        let shell = FeedShell(feed: String(data: try! JSONEncoder().encode(feed), encoding: .utf8)!)
        shell.seen.words = phoneWords
        // The feed is read in a `.task`, then its rows draw, so this one needs more than a turn of
        // the loop: 0.6 s is what `Heard` says a page of rows takes on iPhone 17.
        _ = heard(NotificationsView(shell: shell, seen: shell.seen), size: phone, settle: 0.6)
        XCTAssertEqual(
            shell.shown.addresses, [poster],
            "the poster on a notification did not come through the port at the address it was given"
        )
    }
}

/// Whether a picture settled, held by reference so a view's closure can say so.
private final class Settled {
    var settled = false
}

/// A shell that answers one path with one payload and nothing else, for a screen that has to have
/// read its answer before there is anything on it to draw.
@MainActor
private final class FeedShell: Shell {
    let seen = Seen()
    let shown = ToldPictures(.refuse("no test here draws the picture itself"))
    var pictures: Pictures { shown }
    private let feed: String

    init(feed: String) {
        self.feed = feed
    }

    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { fatalError("no screen under test reels") }
    func connect(_ server: String) {}
    func go(_ address: String) {}
    func swap(_ address: String) {}
    func back() {}
    func act(_ action: Action) {}
    func ask(_ path: String, reply: @escaping (Int, String) -> Void) {
        reply(200, path.hasPrefix("/v1/notifications") ? feed : "")
    }
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) {
        reply(0, "")
    }
    func keep(_ profile: Profile) {}
    func keepAll(_ profiles: [Profile]) {}
    func speak(_ language: String?) {}
    func refreshNotifications() {}
}
#endif
