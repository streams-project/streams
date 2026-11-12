#if canImport(UIKit)
import Contract
import Foundation
import Ports
import SwiftUI

import Stand

@testable import SurfacesPhone

/// A whole screen, built from the parts the app builds it from, so a test reads Home rather than a
/// handful of pieces that happen to be easy to host. Nothing here is a stand-in for a surface: the
/// views are the real ones and the data is the shape the server sends.
///
/// The stub below answers the `Shell` port and records nothing. A screen reader never presses a
/// control, so no test here needs an answer back; a screen that asks for the rest of its rows gets
/// nothing and draws the rows it was given, which is what the first answer of a real page draws.
@MainActor
final class NoShell: Shell {
    let seen = Seen()
    /// The pictures this screen asked for, which a test may read, and which answer nothing drawn
    /// by default: every address in these fixtures is on `example.invalid`, so a refusal is what a
    /// real read of one would come to anyway.
    let shown = ToldPictures()
    var pictures: Pictures { shown }
    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { fatalError("no screen under test reels") }
    func connect(_ server: String) {}
    func go(_ address: String) {}
    func swap(_ address: String) {}
    func back() {}
    func act(_ action: Action) {}
    // Both answer at once with nothing, rather than never answering. A stub that keeps its reply
    // forever leaves `await shell.read(...)` suspended for the life of the test, and a screen whose
    // `.task` never finishes does not commit its accessibility tree: measured on 2026-11-12, a
    // series' title page reached only the three controls pinned outside its scroll view, while the
    // same page for a film - which asks for nothing - reached all of them. An empty answer fails to
    // decode, which is the "gets nothing" this stub is for.
    func ask(_ path: String, reply: @escaping (Int, String) -> Void) { reply(0, "") }
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) { reply(0, "") }
    func keep(_ profile: Profile) {}
    func keepAll(_ profiles: [Profile]) {}
    func speak(_ language: String?) {}
    func refreshNotifications() {}
}

/// The words these screens are given. They are the real English values out of `strings/en.json`, so
/// a test can assert that what is heard is what the strings hold - and so a label wired to a key
/// that `strings/` does not carry shows up as an empty label instead of passing on a literal.
///
/// Every key here was read from `strings/en.json` on 2026-11-12. A key added to a label and missing
/// from this list reads as "" and the silence tests catch it.
@MainActor
let phoneWords: [String: String] = [
    "common.play": "Play",
    "common.back": "Back",
    "common.done": "Done",
    "common.more_info": "More Info",
    "list.add": "My List",
    "list.added": "In My List",
    "nav.home": "Home",
    "nav.search": "Search",
    "nav.series": "Series",
    "nav.films": "Films",
    "nav.categories": "Categories",
    "nav.mine": "My {brand}",
    "notifications.title": "Notifications",
    "row.continue_watching": "Continue Watching",
    "row.continue.remove": "Remove from row",
    "row.my_list": "My List",
    "badge.new": "New",
    "badge.ranked": "#{rank}",
    "title.kind.series": "Series",
    "title.kind.film": "Film",
    "profiles.choose.phone": "Choose your profile",
    "profiles.edit": "Edit",
    "profiles.manage": "Manage Profiles",
    "profile.add": "Add Profile",
    "profile.badge.kids": "kids",
    "profile.pin.enter": "Enter your PIN to access this profile.",
    "say.cast": "Play on another device",
    "say.chosen": "Selected",
    "say.close": "Close",
    "say.season_pick": "Choose a season",
    "say.show_less": "Tap for less",
    "say.show_more": "Tap for more",
    "say.unread": "Not read yet",
    "player.down": "Not for me",
    "player.love": "Love this!",
    "player.up": "I like this",
    "remind.me": "Remind Me",
    "remind.set": "Reminder Set",
    "title.episodes_more": "Episodes & More",
    "title.more_like": "More Like This",
    "title.season.number": "Season {n}",
    "say.more_options": "More options",
    "say.notifications_unread": "Notifications, {n} new",
    "say.pin": "Enter your PIN",
    "say.profile_locked": "{name}, locked with a PIN",
    "say.ranked": "{title}, no. {rank} today",
    "say.see_all": "See everything in {row}",
    "say.watched": "{title}, {percent}% watched",
]

/// A tile as the catalogue sends one. `TitleKey` refuses anything but `movie:` or `tv:` and a
/// number (`Contract.swift`), so the keys here are real ones and `try!` is the assertion that they
/// are: a key this fixture got wrong fails the test rather than quietly building a different tile.
func tile(_ name: String, _ key: String, badges: [Contract.Badge]? = nil, percent: Int? = nil) -> Tile {
    Tile(
        badges: badges,
        images: [Contract.Image(role: .poster, url: "https://example.invalid/\(name).jpg")],
        name: name,
        percent: percent,
        tags: ["Thrilling", "Dark"],
        title: try! TitleKey(key)
    )
}

func section(_ id: String, _ title: String, _ kind: SectionKind, _ tiles: [Tile]) -> Contract.Section {
    Contract.Section(id: id, kind: kind, tiles: tiles, title: title)
}

/// A title's page as the server sends one: a film, with a synopsis long enough to fold, more genres
/// than the line shows, and three titles like it. No treatment picture, so the name is drawn as text
/// and the page's heading is a word rather than an image.
@MainActor
func titleSeen(series: Bool = false) -> (NoShell, Seen) {
    let (shell, seen) = homeSeen()
    let key = try! TitleKey(series ? "tv:21" : "movie:21")
    let detail = TitleDetail(
        genres: ["Thriller", "Dark", "Mystery", "Crime", "Drama"],
        images: [Contract.Image(role: .poster, url: "https://example.invalid/p.jpg")],
        inList: false,
        moreLikeThis: [tile("North Road", "movie:12"), tile("Glasshouse", "tv:13")],
        name: "The Deep",
        overview: String(repeating: "A diver goes down and does not come back up. ", count: 5),
        rating: nil,
        reminderSet: false,
        runtimeMin: 118,
        seasons: series ? [SeasonSummary(episodeCount: 8, number: 1), SeasonSummary(episodeCount: 6, number: 2)] : nil,
        title: key,
        year: 2024
    )
    seen.detail = detail
    // The page draws itself when the detail in hand is the one the address asks for, so the route
    // carries the key and no screen name: `Target.title` is what the shell sets for a title.
    seen.route = RouteTarget(target: .title, title: key)
    return (shell, seen)
}

/// Home as the server sends it: a hero, a plain row, a ranked row and a continue row.
@MainActor
func homeSeen() -> (NoShell, Seen) {
    let shell = NoShell()
    let seen = shell.seen
    seen.words = phoneWords
    seen.language = "en"
    // Not the product's name. No source file in this project may spell it - it lives in
    // `brand.json` alone and `launcher/tests/one_name.rs` fails the workspace for any tracked file
    // that repeats it, a comment or a fixture included. Nothing here turns on which word it is:
    // the surface fills `{brand}` with whatever this says, and the tests ask whether a control
    // speaks, not what the product is called.
    seen.brand = "Example"
    seen.server = "https://example.invalid"
    seen.active = "p1"
    seen.unreadNotifications = 3
    seen.profiles = [Profile(hasPin: false, id: "p1", kids: false, main: true, name: "Ibrahim")]
    seen.nav = [
        NavItem(id: "home", label: "Home", screen: .home),
        NavItem(id: "series", label: "Series", screen: .series),
        NavItem(id: "films", label: "Films", screen: .films),
        NavItem(id: "search", label: "Search", screen: .search),
        NavItem(id: "mine", label: "My Example", screen: .mine),
    ]
    seen.route = RouteTarget(screen: .home, target: .screen)
    seen.screen = Screen(
        name: .home,
        sections: [
            section("hero", "", .hero, [tile("The Deep", "movie:11")]),
            section("picks", "Picked for You Today", .row, [
                tile("North Road", "movie:12"),
                tile("Glasshouse", "tv:13", badges: [Contract.Badge(kind: .new)]),
            ]),
            section("top", "Top 10", .ranked, [
                tile("Harbour", "movie:14", badges: [Contract.Badge(kind: .ranked, rank: 1)]),
                tile("Low Tide", "movie:15", badges: [Contract.Badge(kind: .ranked, rank: 2)]),
            ]),
            section("going", "Continue Watching", .continue, [
                tile("Signal", "tv:16", percent: 42),
            ]),
        ],
        ttlS: 60
    )
    return (shell, seen)
}
#endif
