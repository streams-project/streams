#if canImport(UIKit)
import Contract
import XCTest

@testable import SurfacesTV

/// The episodes page while a season is being read, as this surface answers it.
///
/// None of this was found by a test. It was found by reading, while this screen was ported to Xbox
/// (`apps/xbox/surfaces/remote/title.cpp`), and all of it lives in the gap between what the page
/// holds and what has arrived: the cards on screen belong to the season that answered, which is not
/// the season that is chosen until it does, and a list that has not answered is empty, whose last
/// index is -1.
///
/// What these hold and what they do not. A SwiftUI view's state cannot be pressed from a unit test
/// -- `press` is a function inside `body`, reached through a `UIView` that only a real press
/// reaches -- so each rule is a free function beside the view that uses it, as `gridMove` already
/// is, and these tests hold the rules. The twin on Android is composed and driven key by key
/// (`TitleKeysTest.kt`), which is the behaviour test of the pair; what is checked here is that this
/// surface answers the same way.
final class TitleKeysTests: XCTestCase {
    private func episodes(_ count: Int) -> [Episode] {
        (1...count).map { Episode(name: "Name \($0)", number: $0) }
    }

    /// What OK plays is the card on screen, out of the season that card came from.
    func testAnEpisodeIsPlayedFromTheSeasonItCameFrom() {
        let inHand = Arrived(season: 1, episodes: episodes(7))
        XCTAssertEqual(
            watchAddress("tv/9", inHand, 1, 6),
            "\(Address.watch)tv/9/1/7",
            "the episode in hand came from season 1 and is played from it"
        )
    }

    /// And when the chosen season has answered, it is that season's episode that plays.
    func testTheSeasonThatAnsweredIsTheSeasonThatPlays() {
        XCTAssertEqual(watchAddress("tv/9", Arrived(season: 2, episodes: episodes(4)), 2, 1), "\(Address.watch)tv/9/2/2")
    }

    /// The half that reached every viewer: for the whole of a season's read, the cards on screen
    /// were the season before it, under a column already marking the new one as chosen. A season
    /// being read shows none of them, so there is nothing to walk, nothing lit and nothing to play.
    func testTheCardsOfASeasonBeingReadAreNoneOfThem() {
        let before = Arrived(season: 1, episodes: episodes(7))
        XCTAssertEqual(cardsOf(before, 2), [], "season 2 is being read, so season 1's cards are not its cards")
        XCTAssertEqual(cardsOf(before, 1), before.episodes)
        XCTAssertNil(watchAddress("tv/9", before, 2, 6), "a held Down walked the season before it: nothing plays")
        XCTAssertNil(watchAddress("tv/9", Arrived(), 1, 0), "nothing has arrived at all")
        XCTAssertNil(watchAddress("tv/9", before, 1, 7), "a card past the end of the list that is drawn")
    }

    /// Down before the first season answers. The last index of an empty list is -1, and a card at
    /// -1 is a page drawn a full card low with no card carrying the border that says where the
    /// remote is -- one press on a slow link, on a plain first open of a series nobody has started.
    func testDownBeforeASeasonAnswersLeavesTheCardWhereItIs() {
        XCTAssertEqual(cardBelow(0, 0), 0)
        XCTAssertEqual(
            watchAddress("tv/9", Arrived(season: 1, episodes: episodes(3)), 1, cardBelow(0, 0)),
            "\(Address.watch)tv/9/1/1",
            "the card the answer lands under is the first one"
        )
    }

    /// Down still walks the cards and stops at the last, so the floor is a bound and not a brake.
    func testDownWalksTheCardsAndStopsAtTheLast() {
        XCTAssertEqual(cardBelow(0, 3), 1)
        XCTAssertEqual(cardBelow(1, 3), 2)
        XCTAssertEqual(cardBelow(2, 3), 2)
    }

    /// Whatever a season answers with, the remote is left on a card that list has.
    func testACardIsReadInsideTheListThatArrived() {
        XCTAssertEqual(cardIn(1, 0), 0, "nothing arrived: the first card, not -1")
        XCTAssertEqual(cardIn(6, 4), 3, "a shorter list landed under the card")
        XCTAssertEqual(cardIn(2, 4), 2, "a card the list has is left where it is")
    }
}
#endif
