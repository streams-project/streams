import Contract
import Ports
import SwiftUI

/// One title that has not arrived yet, and whether this viewer asked to be told when it does.
///
/// Read straight out of `GET /v1/titles/{key}`'s `TitleDetail`, which the title page has already
/// read into `seen.detail`, so this screen asks the server for nothing on the way in.
///
/// A title that is already here has no `available_from` and nothing to be reminded of, so there is
/// no `Coming` for it at all and the screen cannot be opened on one. That is the same condition the
/// phone's button hangs on (`SurfacesPhone/title/Title.swift`, `if let availableFrom`).
struct Coming: Equatable {
    /// The title, for the command the host sends.
    let key: TitleKey

    /// The title's own name, from the catalogue, already in the viewer's language.
    let name: String

    /// The day it arrives, as the server sent it.
    let day: String

    /// Whether this profile has already asked to be told.
    ///
    /// Named `reminded` and not `set`, which is what it is called everywhere else in this project:
    /// `set` opens a setter inside a computed property's braces and does not compile here.
    var reminded: Bool

    init(key: TitleKey, name: String, day: String, reminded: Bool) {
        self.key = key
        self.name = name
        self.day = day
        self.reminded = reminded
    }

    /// Nothing for a title that is already here, or that has no name to ask about.
    init?(_ detail: TitleDetail) {
        guard let day = detail.availableFrom?.rawValue, !day.isEmpty, !detail.name.isEmpty else {
            return nil
        }
        self.init(key: detail.title, name: detail.name, day: day, reminded: detail.reminderSet == true)
    }

    /// The words on the line that turns the reminder: what pressing Ok there would leave behind.
    ///
    /// `remind.me` when there is no reminder and `remind.set` when there is, which is the phone's
    /// own pair on its title page.
    var turnKey: String { reminded ? "remind.set" : "remind.me" }

    /// When it arrives, in the viewer's language: `gate.coming` with its `{date}` filled, which is
    /// the one line the phone draws above its button.
    ///
    /// Filled through `fill`, which wraps the day in the bidirectional isolates, and not by a plain
    /// replacement: a date dropped into an Arabic sentence without them comes out in the wrong
    /// order on screen. Empty words stay empty - an unfilled `gate.coming` is a line this run has
    /// no language for, and filling a template nobody will read would put a bare date on screen.
    func comingWords(_ words: [String: String]) -> String {
        guard let pattern = words["gate.coming"], !pattern.isEmpty else { return "" }
        return pattern.fill("date", day)
    }

    /// What a person hears at either stop: what this screen is about - the title's name and when it
    /// arrives - and then the line the remote is standing on.
    ///
    /// The title is repeated at both stops on purpose. This screen is two lines on an otherwise
    /// empty page, and a reader that moved from "Remind Me" to "Cancel" and heard only "Cancel"
    /// would have lost what was being cancelled.
    func says(_ words: [String: String], at: Int) -> String {
        let what = sayBoth(name, comingWords(words))
        let line = at == Question.close ? words["common.cancel"] : words[turnKey]
        return sayBoth(what, line)
    }
}

/// The reminder screen: a title that has not arrived, and one question about it.
///
/// The television's answer to the Remind Me button on the phone's title page. The phone had it and
/// the television had none, so a viewer could ask to be told about a coming title from their pocket
/// and not from the room it would be watched in.
///
/// It is a question of its own and not a button on the title page because a remote has no thumb to
/// reach a button with, and because a question with two answers already has a shape on this
/// surface: the settings confirm panel. It is the same `parts/Question` and the same `parts/QuestionPanel`,
/// so a viewer who has dismissed one two line panel here knows how to dismiss this one, and it
/// opens on the line that leaves for the reason that panel does.
///
/// ## What the television does not do, and why
///
/// The phone sets a local notification as well, so the device raises its own alert on the day
/// (`SurfacesPhone/title/Title.swift`). This screen does not, and that is a decision rather than a
/// gap:
///
/// * `UNMutableNotificationContent`'s `title`, `body` and `sound` **are not available on tvOS**.
///   That is not an opinion: the phone's own file is already `#if os(tvOS)`'d out to two empty
///   functions because, unguarded, those three unavailable properties stopped the whole package
///   compiling for tvOS. There is no local alarm to port.
/// * The server already makes the notice: a reminder that comes true arrives in
///   `GET /v1/notifications` as `reminder_live`, which is the other half of this pass and is the
///   screen next door.
/// * And a television is unplugged, asleep or on another input most of the time, so the device that
///   would raise the alert is the least likely of a viewer's devices to be awake for it.
///
/// What this costs is real and is not hidden: a viewer whose only device is the television is told
/// when they next open the app, not on the day. Closing it means a system notification from the app
/// target, which is the lead's to route.
struct Reminder: View {
    let shell: Shell
    let seen: Seen
    /// The title this screen asks about. The caller has it already; this screen reads nothing.
    let coming: Coming
    /// Ok on the second line, Left, or Back.
    let leave: () -> Void

    @State private var question = Question()
    /// Whether the reminder is on now, which starts as the answer said and is turned here the
    /// moment Ok is pressed. The words on the line must not wait on the network.
    @State private var reminded: Bool

    init(shell: Shell, seen: Seen, coming: Coming, leave: @escaping () -> Void) {
        self.shell = shell
        self.seen = seen
        self.coming = coming
        self.leave = leave
        _reminded = State(initialValue: coming.reminded)
    }

    var body: some View {
        let words = seen.words
        let now = Coming(key: coming.key, name: coming.name, day: coming.day, reminded: reminded)

        return ZStack {
            Color.black
            QuestionPanel(
                prompt: now.name,
                under: now.comingWords(words),
                yes: words[now.turnKey] ?? "",
                close: words["common.cancel"] ?? "",
                at: question.at
            )
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        // What the remote is on: the line it stands on, with what this screen is about said before
        // it at both stops.
        .remote(says: now.says(words, at: question.at)) { press($0) }
    }

    private func press(_ word: Word) -> Bool {
        switch question.press(word) {
        case .yes:
            // Drawn turned at once and told to the server after, which is what the phone does.
            // The new way is held in a value rather than read back out of the state, because what
            // a `@State` reads a moment after it is written is SwiftUI's business and not this
            // screen's - and it is the command's whole meaning.
            let next = !reminded
            reminded = next
            shell.act(.command(ActionCommand(command: .remind(CommandRemind(on: next, title: coming.key)))))
        case .close:
            leave()
        case .moved, .nothing:
            break
        }
        return true
    }
}
