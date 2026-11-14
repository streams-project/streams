import Contract
import CoreGraphics
import Ports

/// Where the viewer is on the notifications feed, what the remote does from there, and the
/// geometry the feed is laid out on. It knows nothing about drawing.
///
/// The twin of the phone's `browse/Notifications.swift` for a surface with no touch: the same feed,
/// the same `GET /v1/notifications`, the same five kinds, and a key map instead of a scroll view.
/// It holds no SwiftUI at all, so every row of the key map below is pressed by `NoticesTests` with
/// no window and no simulator boot.
///
/// ## The key map, designed for this screen and not inherited
///
/// | word | what it does | at the edge |
/// | --- | --- | --- |
/// | Up | the line above | on the first line, nothing moves |
/// | Down | the line below | on the last line, nothing moves |
/// | Left | leaves the screen | every line is the left edge of this page |
/// | Right | nothing. There is no second column | -- |
/// | OK | opens the title, and the notice is read | -- |
/// | Back | leaves the screen | -- |
///
/// **Left.** This surface already decided what Left does at the left edge of a page that is not a
/// row screen: `settings/Settings.swift`, "On sections (left edge), Left goes back", and
/// `help/GetHelp.swift` says it again. A one column list is all left edge, so Left leaves from any
/// line. Left doing nothing was rejected: the left arrow is how a person backs out of a pane
/// everywhere else here, and a dead arrow on one screen is a difference a viewer feels without
/// being able to name it.
///
/// **Up and Down stop at the ends.** Nothing on this surface wraps - `Settings` stops at
/// `line > 0` and `line < rowsCount - 1`, `GetHelp` the same - and a list that jumped from the last
/// notice to the first would be the only place on the television that did.
///
/// **Left and Back are not moves.** ``press(_:)`` answers false for both, as it does for Ok:
/// leaving a page is not something that moved on it. ``leaves(_:)`` is what the screen asks after.
///
/// ## Focus on return
///
/// ``arrived(_:)`` keeps the focus on the notice it was on, found again *by its id* and not by its
/// place. The feed is the one list in this app that grows at the **top**: a notice that arrives
/// while the screen is open pushes every other one down, and a focus kept by number would silently
/// slide onto the notice above the one the viewer was reading, between two key presses.
struct Spot {
    /// The still on a line: **459 x 261**, measured.
    ///
    /// `reference/netflix-on-the-tv.md`, "Episodes screen": each episode card is a "459 x 261
    /// (16:9) still with the episode label burnt in bottom left, then title, synopsis and duration
    /// beside it". That is this screen - a picture with words beside it, stacked down a page - so
    /// the measured card is taken whole rather than a new size being invented for it.
    static let stillWide: CGFloat = 459

    /// And its height. Same measurement.
    static let stillHigh: CGFloat = 261

    /// The gap above, below and beside: **25 px**, measured.
    ///
    /// `reference/netflix-on-the-tv.md`, "Vertical rhythm of a row screen": "tiles 199-708, gap 25,
    /// facts line 733-755". 25 is the measured distance between a picture and the words under it on
    /// this television, so it is the distance between a picture and the words beside it here, and
    /// between one line and the next.
    static let gap: CGFloat = 25

    /// The first line's top: under the page head, by the measured gap.
    static let first = PageHead.under + gap

    /// One line to the next: the still plus the gap, as a row screen's pitch is tile plus gutter.
    static let step = stillHigh + gap

    /// How many lines are wholly on the screen: **3**.
    ///
    /// `(1080 - 203) / 286`. A derivation and not a measurement, from ``Round/pageHigh`` and the
    /// three numbers above, so a change to any of them moves it.
    static let visible = Int((pageHigh - first) / step)

    /// How close to the end the focus comes before the next page is asked for: one screenful.
    ///
    /// The phone asks when the last item drawn is the last item held
    /// (`browse/Notifications.swift`'s `onAppear`) and can, because a lazy stack says when a row
    /// comes into view. A remote has no such moment, so this screen asks from where the focus is;
    /// a screenful is the distance a viewer can cross before another answer could arrive.
    static let reach = visible

    /// The focus outline: **3 px**, measured.
    ///
    /// `reference/netflix-on-the-tv.md`, "A row screen, exact": focus outline `#FBFFFA`, 3 px. The
    /// colour taken is plain white, which is what `browse/Rows.swift` already draws the open card's
    /// edge in, rather than a second near-white on this surface.
    static let edge: CGFloat = 3

    /// What is drawn now: one answer to `GET /v1/notifications`, with later pages joined on the end.
    private(set) var feed = Notifications(cursor: nil, hasMore: false, items: [])

    /// Which line the remote is on.
    private(set) var at = 0

    /// The id of the notice the remote is on, so a fresh feed can find it again.
    private var kept = ""

    init(_ feed: Notifications? = nil) {
        if let feed { arrived(feed) }
    }

    /// A fresh feed for the same screen. The drawing is replaced and where the viewer is, is not.
    ///
    /// A notice that is gone leaves the focus at the same place in the new list, clamped to its
    /// end, which is the nearest thing to where the viewer was looking.
    mutating func arrived(_ feed: Notifications) {
        self.feed = feed
        if let found = feed.items.firstIndex(where: { $0.id == kept && !kept.isEmpty }) {
            at = found
        } else if at > last {
            at = max(0, last)
        }
        kept = idAt(at)
    }

    /// The next page joined on the end. The cursor and the flag come from the later answer, because
    /// they describe what is left rather than what has arrived.
    mutating func andThen(_ later: Notifications) {
        arrived(
            Notifications(
                cursor: later.cursor,
                hasMore: later.hasMore,
                items: feed.items + later.items
            )
        )
    }

    /// Everything down to and including a notice, marked read.
    ///
    /// `POST /v1/notifications/read` takes one id and means "and everything above it", so the
    /// screen draws what it has just asked for rather than waiting for the next answer to say it.
    mutating func readTo(_ id: String) {
        guard let to = feed.items.firstIndex(where: { $0.id == id }) else { return }
        var items = feed.items
        for index in 0...to where items[index].unread {
            items[index].unread = false
        }
        feed = Notifications(cursor: feed.cursor, hasMore: feed.hasMore, items: items)
    }

    /// The last line's place, or -1 for an empty feed.
    var last: Int { feed.items.count - 1 }

    /// Which notice Ok would open, or nothing on an empty feed.
    var chosen: NoticeItem? { feed.items[safe: at] }

    /// How many are unread, for the count the page head says.
    var unread: Int { feed.items.filter(\.unread).count }

    /// How far the list has slid up, in lines.
    ///
    /// The focus walks down to the last line that is wholly on the screen and the list scrolls from
    /// there, which is `Settings`'s own `max(0, line - 7)` shape for this screen's three lines. The
    /// row screen's rule is the other one - the reference measures the focused row pinned to
    /// y 199-708 in 24 of 24 frames, with the content scrolling under it - and it is not taken
    /// here: that band was measured on a screen of tall tiles where one row is the whole of the
    /// content, and pinning a three line list to its first line would scroll the page on the very
    /// first press of Down with two lines still unread below.
    var scrolled: Int { max(0, at - (Spot.visible - 1)) }

    /// Whether the screen should read its next page now: the server says there is one, and the
    /// focus is within ``reach`` of the last notice held.
    ///
    /// The android twin answers true for an empty feed as well, so that a screen with nothing on it
    /// can fill by asking. That line is not here, and its absence was measured rather than assumed:
    /// on this surface the first page is read by the screen's own `task` and `more()` refuses any
    /// feed the server has not given a cursor for, so an empty feed answering true could not change
    /// what the screen does. It would be a line no test could ever fail, which is the kind of line
    /// a mutation pass cannot tell from a working one.
    ///
    /// An empty feed the server says has more - `{"items": [], "has_more": true}` - still asks,
    /// because `at` is 0 and `last` is -1.
    var wantsMore: Bool {
        feed.hasMore && at >= last - Spot.reach
    }

    /// One word from the remote. True when something moved, which is the one thing that redraws.
    ///
    /// False for Left, Back and Ok: none of the three moves anything on this page. What they do
    /// instead is ``leaves(_:)`` and ``chosen``.
    mutating func press(_ word: Word) -> Bool {
        switch word {
        case .up:
            if at > 0 {
                at -= 1
                kept = idAt(at)
                return true
            }
            return false
        case .down:
            if at < last {
                at += 1
                kept = idAt(at)
                return true
            }
            return false
        case .left, .right, .ok, .back:
            return false
        }
    }

    /// Whether this word leaves the screen. Left and Back, and nothing else.
    ///
    /// Asked by the screen after ``press(_:)`` has answered false: a word that moved something is
    /// never a word that leaves.
    func leaves(_ word: Word) -> Bool {
        word == .left || word == .back
    }

    /// What a person hears as the remote moves, handed to `remote(says:)`.
    ///
    /// Four things in the order a person wants them: what kind of notice it is, the title's name,
    /// the day, and whether it has been read. They are joined by `sayBoth`, so the break is a line
    /// end and no language's punctuation or word order is decided here. A kind this build has no
    /// key for, and a word the run has not read yet, are both empty and drop out.
    ///
    /// An empty feed says the screen's own name and that there is nothing on it, because a reader
    /// that arrives on a screen with no stops would otherwise be handed silence.
    func says(_ words: [String: String]) -> String {
        guard let notice = chosen else {
            return sayBoth(words["notifications.title"], words["notifications.empty"])
        }
        let what = sayBoth(words[noticeKey(notice.kind)], notice.tile.name)
        let when = sayBoth(notice.date.rawValue, notice.unread ? words["say.unread"] : "")
        return sayBoth(what, when)
    }

    private func idAt(_ index: Int) -> String {
        feed.items[safe: index]?.id ?? ""
    }
}

/// The five kinds of notice this build knows. A kind from a newer server is not one of them.
let noticeKinds: Set<NoticeKind> = [.coming, .reminderLive, .newArrival, .newSeason, .newEpisode]

/// The words key for a notice's kind, or nothing for a kind this build does not know.
///
/// Derived rather than tabled: the five keys in `strings/` are the contract's own five kind values
/// under `notification.`, which `NoticesTests` holds against `strings/en.json` and against what the
/// phone draws. A table here would be a second copy of the phone's `labelFor` and would go stale
/// one kind at a time.
///
/// A sixth kind arriving from a newer server answers empty, draws no chip and is not said, which is
/// what the phone's `default: return ""` does. The notice still draws its name and its day, so a
/// viewer sees the thing that happened even when this build has no word for what kind it was.
func noticeKey(_ kind: NoticeKind) -> String {
    noticeKinds.contains(kind) ? "notification.\(kind.rawValue)" : ""
}
