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
    /// A tile's box, which is the box most of the pictures in this app are drawn in.
    private let tileBox = CGSize(width: tileWide, height: tileHigh)
    /// How many pixels a point is here. Read rather than written down: a test that assumed 3 and
    /// ran on a screen at 2 would assert about a box no view ever asked for.
    private var scale: CGFloat { UIScreen.main.scale }

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

    /// Two boxes a few pixels apart are one picture, because a picture is filed under the rung its
    /// box falls on and not under the box itself.
    ///
    /// Without this a row of tiles measured at 357 pixels and a grid of the same posters measured
    /// at 360 would hold every poster twice, and nothing on screen would say so.
    func testTheAdapterHoldsTwoNearBoxesAsOnePicture() {
        let pictures = UrlPictures()
        pictures.remember(Want(url: address, wide: 357, high: 500), aPicture(8))
        XCTAssertNotNil(
            pictures.kept(Want(url: address, wide: 360, high: 504)),
            "two boxes a few pixels apart were held as two pictures"
        )
        XCTAssertNil(
            pictures.kept(Want(url: address)),
            "a picture read in a box answered for a read of the whole frame, which asks for more"
        )
        XCTAssertNil(
            pictures.kept(Want(url: address, wide: 1024, high: 1024)),
            "a small picture answered for a box it cannot fill"
        )
    }

    /// Reading a picture's pixels back is not a second picture.
    ///
    /// `pixels` says what the caller means to do and nothing about the bytes this adapter produces,
    /// so it is not part of what a picture is filed under. Until 2026-11-13 it was, and the two
    /// places that take a tint off their artwork - the gate and the billboard - decoded the same
    /// picture twice and held both, each believing in a comment that it shared one entry with the
    /// drawing beside it.
    func testReadingAPicturesPixelsBackIsNotASecondCopyOfIt() {
        let pictures = UrlPictures()
        pictures.remember(Want(url: address, wide: 320, high: 320), aPicture(8))
        XCTAssertNotNil(
            pictures.kept(Want(url: address, wide: 320, high: 320, pixels: true)),
            "asking to read a picture's pixels back made a second copy of one already in hand"
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
        _ = heard(Art(told, address, box: tileBox), size: phone)
        XCTAssertEqual(told.addresses, [address], "the shared picture view did not go through the port")
    }

    /// A refusal ends the wait. `Art` tells its caller the picture is settled whether one arrived or
    /// not, because the one screen that passes that closure uses it to decide when to draw.
    func testArtSaysThePictureIsSettledEvenWhenThePortRefuses() {
        let told = ToldPictures(.refuse("offline"))
        let box = Settled()
        _ = heard(Art(told, address, box: tileBox) { box.settled = true }, size: phone)
        XCTAssertEqual(told.addresses, [address], "the view did not ask the port at all")
        XCTAssertTrue(box.settled, "a refused picture left the screen waiting for one that is not coming")
    }

    /// A picture the port already holds is drawn without a second read.
    func testArtDrawsAPictureAlreadyInHandWithoutAskingForItAgain() {
        let told = ToldPictures(.silence)
        // Held under the want the view will ask with, box and all. If the two ever came apart the
        // look in the cache would miss and the view would read again, which is what this asserts.
        told.hold(wanted(address, tileBox, scale), aPicture())
        _ = heard(Art(told, address, box: tileBox), size: phone)
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

    /// The more like this grid asks for each poster in the cell it really draws it in, on a page of
    /// any width.
    ///
    /// This is the one grid in this surface whose cell has no size of its own: three `.flexible()`
    /// columns, so a cell is a share of the page. Until 2026-11-13 the box was written down as the
    /// phone's own tile, 119 by 166.6, and on anything wider than a phone the poster was decoded for
    /// a phone and drawn soft (`notes/245`). The page reads its own width now and the grid divides
    /// it.
    ///
    /// Both halves are measured rather than worked out here: the cell comes from the drawn
    /// accessibility frame of the tile itself, and the box from what the port was handed. A test
    /// that computed the share for itself would agree with a draw site that computed the same share
    /// wrongly.
    func testTheMoreLikeGridAsksForItsPostersInTheCellItDrawsThemIn() {
        // The grid alone and not the page around it, because what is read here is where a tile was
        // drawn, and an accessibility tree is only built for what is on the screen: at the foot of
        // the title page the grid is below it. The page's half of this - that it hands its own
        // width down - is the test below.
        for pageWide in [phone.width, 1024] {
            let (shell, seen) = titleSeen()
            let tile = seen.detail!.moreLikeThis!.first!
            let found = heard(
                PhoneMoreLike(shell: shell, data: seen.detail!, pageWide: pageWide),
                size: CGSize(width: pageWide, height: 800)
            )
            guard let drawn = saying(found, tile.name) else {
                return XCTFail("the more like this grid drew no tile for \(tile.name) on a \(pageWide) point page: \(words(found))")
            }
            guard let asked = shell.shown.asked.first(where: { $0.url == tall(tile.images)! }) else {
                return XCTFail("the tile's poster never reached the port on a \(pageWide) point page")
            }
            XCTAssertEqual(
                Double(asked.wide), Double(drawn.frame.width * scale), accuracy: Double(scale),
                "the poster was asked for in a box of \(asked.wide) pixels and drawn in a cell of "
                    + "\(drawn.frame.width * scale) on a \(pageWide) point page"
            )
            XCTAssertEqual(
                Double(asked.high), Double(drawn.frame.height * scale), accuracy: Double(scale),
                "the poster's box is not as tall as the cell it is drawn in on a \(pageWide) point page"
            )
        }
    }

    /// A page wider than a phone asks for a bigger picture than a phone does, and the two are not
    /// the same entry in the cache.
    ///
    /// The test above would pass if every page were somehow one width. This one says the box
    /// follows the page: a tablet's cell is 316 points against a phone's 119, which is rung 1024
    /// against rung 360, and the whole of the open item in `notes/245` was that the tablet asked for
    /// the phone's rung.
    func testAWiderPageAsksForABiggerPictureThanAPhone() {
        func rungAsked(_ pageWide: CGFloat) -> Int {
            let (shell, seen) = titleSeen()
            let tile = seen.detail!.moreLikeThis!.first!
            _ = heard(Title(shell: shell, seen: seen), size: CGSize(width: pageWide, height: 2000), settle: 0.6)
            let asked = shell.shown.asked.first { $0.url == tall(tile.images)! }
            return UrlPictures.key(asked ?? Want(url: "")).wide
        }
        let onAPhone = rungAsked(phone.width)
        let onATablet = rungAsked(1024)
        XCTAssertGreaterThan(onAPhone, 0, "a phone's more like this tile asked for no box at all")
        XCTAssertGreaterThan(
            onATablet, onAPhone,
            "a tablet's cell is wider than a phone's and asked for the same rung, \(onATablet): "
                + "the poster in it is decoded for a phone and drawn soft"
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

// MARK: - What a screen's pictures cost

/// What a whole screen asks the port for, and what holding those answers costs.
///
/// Home is the screen to ask it of: it has the most pictures on it, and it draws every kind this
/// app has - a billboard, rows of posters, a ranked row, a continue row, a face on the bar.
@MainActor
final class PictureCostTests: XCTestCase {
    private let phone = CGSize(width: 393, height: 852)

    /// Every want Home hands the port names the box it will be drawn in.
    ///
    /// This is the test that fails if a draw site goes back to a box of nothing. Nothing on screen
    /// would look wrong if one did - the picture would simply be held at its own size behind a tile
    /// a tenth of it - so this is the only place that failure is visible.
    func testEveryPictureHomeAsksForNamesTheBoxItWillFill() {
        let asked = homesWants()
        XCTAssertFalse(asked.isEmpty, "Home asked the port for no pictures at all")
        let boxless = asked.filter { $0.wide <= 0 || $0.high <= 0 }.map(\.url).sorted()
        XCTAssertEqual(
            boxless, [],
            "a draw site on Home asked for a picture without naming its box, so the whole frame is decoded and held"
        )
    }

    /// What those boxes are worth, in bytes, against holding the same screen's pictures whole.
    ///
    /// Both sides are the same screen's own wants, decoded from the same poster: the left is what
    /// the cache holds now, one entry per box rung; the right is what it held before 2026-11-13,
    /// one entry per address and per `pixels`, each at the poster's full size. The numbers are
    /// printed because a measurement nobody can read is a measurement nobody can check.
    func testHomesPicturesAreHeldForAFractionOfTheirWholeFrames() {
        let asked = homesWants()
        let poster = aPosterBytes(1000, 1500)
        let now = Set(asked.map { UrlPictures.key($0) })
        let before = Set(asked.map { Want(url: $0.url, pixels: $0.pixels) })
        let heldNow = now.reduce(0) { $0 + bytesOf(UrlPictures.decode(poster, $1)) }
        let heldBefore = before.reduce(0) { $0 + bytesOf(UrlPictures.decode(poster, $1)) }
        let each = asked.map { "\($0.url) \($0.wide)x\($0.high) -> rung \(UrlPictures.key($0).wide)" }
        print("""
        HOME PICTURES, decoded from a 1000 by 1500 poster:
          wants asked: \(asked.count)
        \(each.map { "    " + $0 }.joined(separator: "\n"))
          entries now: \(now.count), held: \(heldNow) bytes
          entries before: \(before.count), held: \(heldBefore) bytes
        """)
        XCTAssertGreaterThan(heldBefore, 0, "nothing decoded at all, so neither number means anything")
        XCTAssertLessThan(
            heldNow * 2, heldBefore,
            "the boxes Home asks for save less than half of what holding its pictures whole costs"
        )
    }

    /// Every want a drawn Home handed the port, reads and reads ahead alike.
    private func homesWants() -> [Want] {
        let (shell, seen) = homeSeen()
        _ = heard(Home(shell: shell, seen: seen), size: phone, settle: 0.6)
        return shell.shown.asked + shell.shown.warmed
    }
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
