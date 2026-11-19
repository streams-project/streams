#if canImport(UIKit)
import Contract
import Foundation
import Heard
import Ports
import SwiftUI

import Stand

@testable import SurfacesVision

/// Where these tests run, said once so no test below has to pretend.
///
/// There is no visionOS simulator on this machine. Measured on 2026-11-14: `xcrun simctl list
/// runtimes` answers iOS 26.5 and tvOS 26.5 and nothing else, and `xcrun simctl list devicetypes`
/// has no visionOS device at all. The SDKs are installed -- `swift build --triple
/// arm64-apple-xros1.0-simulator` compiles `SurfacesVision` on this machine -- so the module is
/// *built* for visionOS; it is never *run* there, here or anywhere, by anything.
///
/// So `check_apple.py` resolves this target to the iPhone simulator, and that is where every
/// number any test here prints was taken. `SurfacesVision` is written to compile on both platforms
/// for exactly this reason: a module behind `#if os(visionOS)` would be an empty module on the one
/// simulator that exists, and a test target that ran nothing would be a green run that proved
/// nothing, which is the failure this project refuses above all others.
///
/// What running here does prove: the window rule's arithmetic, that every control carries words,
/// and how large each control is actually laid out -- none of which is a fact about visionOS, and
/// all of which is a fact about this code.
///
/// What it cannot prove, and what only a device can: that the system's gaze highlight lands on the
/// shape each control hands it, that 60 pt is in fact large enough to look at, that the window's
/// glass leaves the text readable against a real room, and that a bar drawn inline is the right
/// answer where an ornament exists.

/// Answers the `Shell` port and records nothing. A screen reader never presses a control, so no
/// test here needs an answer back; the two calls that would reply answer at once with nothing,
/// because a stub that keeps its reply forever leaves a screen's `.task` suspended, and a screen
/// whose task never finishes does not commit its accessibility tree.
@MainActor
final class NoShell: Shell {
    let seen = Seen()
    /// The pictures a screen asked for, which a test may read, and which answer nothing drawn by
    /// default: every address in these fixtures is on `example.invalid`, so a refusal is what a
    /// real read of one would come to anyway.
    let shown = ToldPictures()
    var pictures: Pictures { shown }
    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { fatalError("no screen under test reels") }
    func retry() {}
    func go(_ address: String) {}
    func swap(_ address: String) {}
    func back() {}
    func act(_ action: Action) {}
    func ask(_ path: String, reply: @escaping (Int, String) -> Void) { reply(0, "") }
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) { reply(0, "") }
    func keep(_ profile: Profile) {}
    func keepAll(_ profiles: [Profile]) {}
    func speak(_ language: String?) {}
    func refreshNotifications() {}
}

/// The words these screens are given: the real English values out of `strings/en.json`, read on
/// 2026-11-14. A key wired to a label that `strings/` does not carry then shows up as an empty
/// label and the silence tests below catch it, instead of an English literal written here passing.
@MainActor
let visionWords: [String: String] = [
    "badge.new": "New",
    "badge.ranked": "#{rank}",
    "common.more_info": "More Info",
    "common.play": "Play",
    "games.soon": "Games are on the way.",
    "list.add": "My List",
    "list.added": "In My List",
    "mine.empty": "Titles you save, like or start will show up here.",
    "nav.films": "Films",
    "nav.home": "Home",
    "nav.search": "Search",
    "nav.series": "Series",
    "player.back": "Back",
    "player.down": "Not for me",
    "player.love": "Love this!",
    "player.restart": "Start Over",
    "player.up": "I like this",
    "row.continue_watching": "Continue Watching",
    "row.my_list": "My List",
    "say.chosen": "Selected",
    "say.loading": "Loading",
    "say.pause": "Pause",
    "say.position": "{at} of {length}",
    "say.watched_percent": "{percent}% watched",
    "title.continue_episode": "Continue Episode {n}",
    "title.episodes_more": "Episodes & More",
    "title.length.both": "{hours} {minutes}",
    "title.length.hours.many": "{n} hours",
    "title.length.hours.one": "{n} hour",
    "title.length.minutes": "{m}m",
    "title.length.minutes.many": "{n} minutes",
    "title.length.minutes.one": "{n} minute",
    "title.more_like": "More Like This",
    "title.play_episode": "Play Episode {n}",
    "title.resume": "Resume",
    "title.season.many": "{n} Seasons",
    "title.season.number": "Season {n}",
    "title.season.one": "{n} Season",
]

/// A tile as the catalogue sends one. `TitleKey` refuses anything but `movie:` or `tv:` and a
/// number, so the keys here are real ones and `try!` is the assertion that they are.
func tile(_ name: String, _ key: String, badges: [Contract.Badge]? = nil, percent: Int? = nil) -> Tile {
    Tile(
        badges: badges,
        images: [Contract.Image(role: .backdrop, url: "https://example.invalid/\(name).jpg")],
        name: name,
        percent: percent,
        tags: ["Thrilling", "Dark"],
        title: try! TitleKey(key)
    )
}

func section(_ id: String, _ title: String, _ kind: SectionKind, _ tiles: [Tile]) -> Contract.Section {
    Contract.Section(id: id, kind: kind, tiles: tiles, title: title)
}

/// A profile as the server sends one.
func person(_ name: String, _ id: String) -> Profile {
    Profile(hasPin: false, id: id, kids: false, main: id == "p1", name: name)
}

/// A page of rows as a first answer carries it: a hero, a plain row, a ranked row and a row the
/// viewer has started.
@MainActor
func aHomeScreen() -> Contract.Screen {
    Contract.Screen(
        name: .home,
        sections: [
            section("hero", "", .hero, [tile("Sunfall", "movie:101")]),
            section("picks", "Picked for You Today", .row, [
                tile("Harbour Light", "movie:102"),
                tile("The Long Field", "tv:103"),
                tile("Nightbus", "movie:104"),
            ]),
            section("top", "Top 10 Today", .ranked, [
                tile("Glasshouse", "movie:105", badges: [Contract.Badge(kind: .ranked, rank: 1)]),
                tile("Paper Moon", "tv:106", badges: [Contract.Badge(kind: .ranked, rank: 2)]),
            ]),
            section("continue", "Continue Watching", .continue, [
                tile("Salt Road", "tv:107", percent: 72),
            ]),
        ],
        ttlS: 60
    )
}

/// One title, as a detail answer carries it.
@MainActor
func aTitle(series: Bool = false) -> TitleDetail {
    TitleDetail(
        age: nil,
        genres: ["Thriller"],
        images: [Contract.Image(role: .backdrop, url: "https://example.invalid/hero.jpg")],
        inList: false,
        moreLikeThis: [tile("Nightbus", "movie:104"), tile("Paper Moon", "tv:106")],
        name: "Harbour Light",
        overview: "A harbourmaster finds a boat nobody will claim.",
        runtimeMin: 104,
        seasons: series ? [SeasonSummary(episodeCount: 6, number: 1)] : nil,
        title: try! TitleKey(series ? "tv:103" : "movie:102"),
        year: 2019
    )
}

/// A shell holding a page of rows, ready to draw.
@MainActor
func aBrowsingShell() -> NoShell {
    let shell = NoShell()
    shell.seen.words = visionWords
    shell.seen.screen = aHomeScreen()
    shell.seen.route = RouteTarget(screen: .home, target: .home)
    shell.seen.profiles = [person("Ibrahim", "p1")]
    shell.seen.active = "p1"
    shell.seen.nav = [
        NavItem(id: "home", label: "Home", screen: .home),
        NavItem(id: "series", label: "Series", screen: .series),
        NavItem(id: "films", label: "Films", screen: .films),
    ]
    return shell
}

/// A shell holding one title, ready to draw.
@MainActor
func aTitleShell(series: Bool = false) -> NoShell {
    let shell = NoShell()
    let detail = aTitle(series: series)
    shell.seen.words = visionWords
    shell.seen.detail = detail
    shell.seen.route = RouteTarget(target: .title, title: detail.title)
    return shell
}

/// The window this surface opens at, as a size a test can draw into.
@MainActor
let aWindow = CGSize(width: windowWide, height: windowHigh)

/// A whole screen drawn in that window and read once it has really been built.
///
/// Waited on rather than slept through. `Tests/Heard` records what a fixed settle cost on
/// 2026-11-14: with eighteen agents building, two tests drawing the same screen disagreed in one
/// run, because a duration asserts how fast the machine is as well as what the product does.
///
/// What is waited for is the picture port, and that is not an arbitrary signal. Every tile and
/// every backdrop on these screens draws through `Art`, which asks the port from its `task` -- and
/// a `task` runs only after that view's first frame is on screen. So `asked.count` reaching `n` is
/// the lazy stack reporting that it has built and laid out at least `n` of its children, which is
/// the exact condition these reads need and the exact one a duration was standing in for.
@MainActor
func heardWhenDrawn<V: SwiftUI.View>(_ view: V, _ shell: NoShell, pictures: Int) -> [Heard] {
    heard(view, size: aWindow) { shell.shown.asked.count >= pictures }
}

/// A part read until it has at least `least` elements in it, or until the reads stop helping.
///
/// For the parts that ask the picture port for nothing -- a bar of words, a row of buttons -- where
/// there is no port to wait on and the only other option is a fixed settle.
///
/// A fixed settle is what this replaces, and it was measured failing. On 2026-11-14, with mutation
/// runs building in the background, `heard(aNavBar(), size: aWindow)` at the default 0.1 s settle
/// returned fewer than its five controls intermittently: three tests failed in every mutated run
/// regardless of which line had been mutated, including runs whose mutation could not touch the bar
/// at all. Those failures said the machine was busy, not that the product was wrong, and a harness
/// that cannot tell those apart makes every mutation look caught.
///
/// So the wait escalates instead of being guessed: a quick read first, and a longer one only if the
/// quick one came back short. A fast machine still finishes in 0.1 s. A loaded one takes longer and
/// still gets the truth. A part that genuinely has too few controls fails on the count assertion in
/// the test, which is where that belongs, after the last try rather than the first.
@MainActor
func heardAtLeast<V: SwiftUI.View>(_ view: V, _ least: Int) -> [Heard] {
    var found: [Heard] = []
    for settle in [0.1, 0.4, 1.2, 3.0] {
        found = heard(view, size: aWindow, settle: settle)
        if found.count >= least { return found }
    }
    return found
}

// MARK: - The parts of a screen, each drawable on its own
//
// A whole screen cannot be read here: anything inside a `ScrollView` reaches `Tests/Heard` as
// nothing at all (measured 2026-11-14, written up in `Speech.swift`). So the screens' parts are
// built here once, with the same fixtures, and every test reads them rather than each test
// assembling its own and drifting from what the screens really draw.

@MainActor
func aNavBar() -> NavBar {
    NavBar(
        tabs: tabsOf([
            NavItem(id: "home", label: "Home", screen: .home),
            NavItem(id: "series", label: "Series", screen: .series),
            NavItem(id: "films", label: "Films", screen: .films),
        ]),
        active: "home",
        person: person("Ibrahim", "p1"),
        words: visionWords,
        go: { _ in }
    )
}

@MainActor
func aHero(_ shell: NoShell) -> Hero {
    Hero(
        pictures: shell.shown, tile: tile("Sunfall", "movie:101"),
        words: visionWords, width: windowWide, go: { _ in }
    )
}

@MainActor
func aTile(_ shell: NoShell, percent: Int? = nil) -> TileCard {
    TileCard(
        pictures: shell.shown,
        tile: tile("Salt Road", "tv:107", percent: percent),
        words: visionWords,
        box: tileBox(windowWide),
        open: { _ in }
    )
}

@MainActor
func aTitleButtons(saved: Bool) -> TitleButtons {
    let data = aTitle(series: true)
    return TitleButtons(
        data: data,
        words: visionWords,
        playing: visionWords["title.play_episode"]!.fill("n", 1),
        saved: saved,
        play: {}, episodes: {}, list: {}
    )
}

/// A shell with a film playing, for the player's own screen.
@MainActor
func aWatchingShell() -> NoShell {
    let shell = NoShell()
    shell.seen.words = visionWords
    shell.seen.route = RouteTarget(target: .watch, title: try! TitleKey("movie:102"))
    shell.seen.positionMs = 1_830_000
    shell.seen.lengthMs = 6_240_000
    return shell
}
#endif
