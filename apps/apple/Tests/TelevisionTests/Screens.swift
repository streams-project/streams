#if canImport(UIKit)
import Contract
import Foundation
import Ports
import SwiftUI

import Stand

@testable import SurfacesTV

/// The words the television screens are given. These are the real English values out of
/// `strings/en.json`, read on 2026-11-12, so a test can assert that what is heard is what the
/// strings hold - and so a label wired to a key `strings/` does not carry shows up as an empty
/// label instead of passing on an English literal written here.
///
/// A key added to a label and left out of this list reads as "" and the silence tests catch it.
///
/// `say.shift` landed on 2026-11-12 and is here now. It reads "Shift" in English and names capital
/// letters in all eighteen languages - "Grossbuchstaben", "حروف كبيرة" - because not one of them
/// names that key for switching: `say.letters` already carries the switch verb and sits one key
/// away on the same page, so two switch-shaped labels would have been on adjacent keys.
@MainActor
let tvWords: [String: String] = [
    "common.back": "Back",
    "common.cancel": "Cancel",
    "common.next": "Next",
    "common.play": "Play",
    "door.help": "Get Help",
    "door.join": "Get Started",
    "door.sign_in": "Sign In",
    "door.trending": "Trending Now",
    "door.watch": "Join to Watch",
    "menu.help": "Get Help",
    "menu.switch": "Switch Profiles",
    "nav.home": "Home",
    "nav.search": "Search",
    "player.back": "Back",
    "player.down": "Not for me",
    "player.love": "Love this!",
    "player.next": "Next Episode",
    "player.restart": "Start Over",
    "player.still.no": "I'm Done",
    "player.still.yes": "Continue Watching",
    "player.up": "I like this",
    "profile.add": "Add Profile",
    "profile.edit": "Edit profile",
    "say.avatar_in_set": "Picture {n} in {set}",
    "say.chosen": "Selected",
    "say.delete": "Delete",
    "say.letters": "Switch letters",
    "say.pause": "Pause",
    "say.position": "{at} of {length}",
    "say.profile_locked": "{name}, locked with a PIN",
    "say.shift": "Shift",
    "say.space": "Space",
    "settings.autoplay_next": "Autoplay next episode",
    "settings.intro": "Change playback, language, subtitles and account options for this TV.",
    "settings.off": "Off",
    "settings.on": "On",
    "settings.title": "Settings",
    "signin.email": "Email address",
    "signin.password": "Password",
    "signin.phone": "Use Phone",
    "signin.remote": "Use Remote",
    "title.episode": "Episode {n}",
    "title.episodes.count": "{n} episodes",
    "title.season.number": "Season {n}",
]

/// A profile as the server sends one.
func person(_ name: String, _ id: String, hasPin: Bool = false, kids: Bool = false) -> Profile {
    Profile(hasPin: hasPin, id: id, kids: kids, main: id == "p1", name: name)
}

/// Answers the `Shell` port and records nothing. A screen reader never presses a control, so no
/// test here needs an answer back; the two that would reply at once reply with nothing, because a
/// stub that keeps its reply forever leaves a screen's `.task` suspended and a screen whose task
/// never finishes does not commit its accessibility tree.
@MainActor
final class NoTellShell: Shell {
    let seen: Seen
    /// The pictures this screen asked for, which a test may read, and which answer nothing drawn
    /// by default: every address in these fixtures is on `example.invalid`, so a refusal is what a
    /// real read of one would come to anyway.
    let shown = ToldPictures()
    var pictures: Pictures { shown }

    init(seen: Seen) {
        self.seen = seen
    }

    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { fatalError("no screen under test reels") }
    func connect(_ server: String) {}
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
#endif
