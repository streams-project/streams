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
        XCTAssertLessThan(
            heldNow * 6, heldWhole,
            "the boxes this grid asks for save less than five sixths of holding its posters whole"
        )
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
#endif
