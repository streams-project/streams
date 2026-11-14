import SwiftUI

/// A question with two answers, and where the remote is on it.
///
/// Two lines and nothing else, which is the smallest state machine on this surface and is still
/// written down rather than spelled out again at each screen that asks something. It holds no
/// view, so every row of the key map below is pressed by a test on this machine with no window.
///
/// ## The key map
///
/// | word | what it does | at the edge |
/// | --- | --- | --- |
/// | Up | the line above | on the first line, nothing moves |
/// | Down | the line below | on the second line, nothing moves |
/// | Left | leaves | the two lines are the left edge of the panel |
/// | Right | nothing. There is no second column | -- |
/// | OK | the first line answers the question; the second leaves | -- |
/// | Back | leaves | -- |
///
/// This was `settings/Settings.swift`'s own confirm panel, written inline there as four branches
/// of `press`. It is one type now because the reminder screen asks a question of exactly the same
/// shape, and a second copy of these four branches would be a second place for the answer to drift
/// to. A viewer who has dismissed one two line panel on this television knows how to dismiss the
/// other, and now that is true by construction rather than by two files agreeing.
struct Question {
    /// The line that answers the question - clears the history, removes the device, sets the
    /// reminder.
    static let yes = 0

    /// The line that leaves.
    static let close = 1

    /// What a word did.
    enum Chose {
        /// The remote moved, so the panel redraws and nothing else happens.
        case moved
        /// The question was answered.
        case yes
        /// The panel was left: Ok on the second line, or Left, or Back.
        case close
        /// Nothing at all: Right, or Up and Down at the ends.
        case nothing
    }

    /// Which line the remote is on.
    ///
    /// It opens on the line that leaves. `Settings` opened its confirm there for a reason that
    /// holds for every question this draws: a panel that opens on the line that *does* the thing
    /// turns one stray Ok into an action nobody asked for.
    private(set) var at = Question.close

    /// One word from the remote.
    mutating func press(_ word: Word) -> Chose {
        switch word {
        case .up:
            if at > Question.yes {
                at -= 1
                return .moved
            }
            return .nothing
        case .down:
            if at < Question.close {
                at += 1
                return .moved
            }
            return .nothing
        case .ok:
            return at == Question.yes ? .yes : .close
        case .left, .back:
            return .close
        case .right:
            return .nothing
        }
    }

    /// The words of the line the remote is on, for the box that says it aloud.
    func word(yes: String, close: String) -> String {
        at == Question.close ? close : yes
    }
}

/// How a question with two answers is drawn: what is being asked, and the two lines that answer it.
///
/// The question is a heading, so a reader can jump to it, because it is what the two lines under it
/// answer. `under` is a second line of the question itself - the reminder screen puts the day a
/// title arrives there - and is left out when there is none.
struct QuestionPanel: View {
    let prompt: String
    var under: String = ""
    let yes: String
    let close: String
    let at: Int
    var width: CGFloat = 520

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(prompt)
                .brand(32)
                .foregroundStyle(.white)
                .accessibilityAddTraits(.isHeader)
                .padding(.leading, menuWords.ref)
            if !under.isEmpty {
                Text(under)
                    .brand(PageHead.helpType)
                    .foregroundStyle(Muted)
                    .padding(.leading, menuWords.ref)
                    .padding(.top, 8.ref)
            }
            Spacer().frame(height: 24.ref)
            Menu(
                lines: [MenuLine(word: yes), MenuLine(word: close)],
                focus: at,
                width: width
            )
        }
        .padding(.leading, 16.ref)
        .padding(.top, 8.ref)
    }
}
