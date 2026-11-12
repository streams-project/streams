#if canImport(UIKit)
import Contract
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import UIKit
import XCTest

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
}
#endif
