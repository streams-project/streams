import Contract
import Ports
import SwiftUI

/// The width of a line: the page between its two margins.
private let lineWide = pageWide - 2 * subEdge

/// The feed itself, and the call that marks a notice read. Written as the server's own paths, as
/// every other screen here writes the paths it reads (`settings/Settings.swift`, `help/GetHelp`).
private let feedPath = "/v1/notifications"
private let readPath = "/v1/notifications/read"

/// One notice: a still, and beside it what kind of notice it is, the title's name and the day.
///
/// ## What a reader is handed, and why it is one stop and not four
///
/// The line carries the whole reading - kind, name, day, and whether it has been read as the state
/// after them - and the still inside it is decoration (`Art` marks itself). A reader therefore
/// stops once per notice and hears all of it, instead of stopping four times and hearing a fragment
/// each time. It is the same bargain the phone's row makes
/// (`SurfacesPhone/browse/Notifications.swift`), with `on` added, because this surface paints its
/// own focus and a person listening cannot see a white outline.
/// Internal rather than private, as `SettingsRowItem` and `ProfileRow` are, so a test can draw one
/// line and read what a screen reader would find in it.
struct NoticeLine: View {
    /// The title's name, at the size this surface draws a title's name - the size a page head
    /// draws its own title at.
    static let nameType = PageHead.titleType

    /// The kind and the day, at this surface's small word size: a menu's lines and a page head's
    /// help line are both drawn at it.
    static let smallType = PageHead.helpType

    let pictures: Pictures
    let notice: NoticeItem
    let on: Bool
    let words: [String: String]

    var body: some View {
        let shape = shapeOf(Round.card)
        let kindWords = words[noticeKey(notice.kind)] ?? ""
        HStack(spacing: Spot.gap.ref) {
            ZStack {
                Raised
                // The picture for a wide place, through the one shared view and the one shared
                // choice of role: `SurfacesShared/Art.swift`'s `wide`, backdrop then still then
                // poster, in the box this line will draw it in.
                Art(
                    pictures,
                    wide(notice.tile.images),
                    box: CGSize(width: Spot.stillWide, height: Spot.stillHigh)
                )
            }
            .frame(width: Spot.stillWide.ref, height: Spot.stillHigh.ref)
            .clipShape(shape)
            VStack(alignment: .leading, spacing: Spot.gap.ref) {
                if !kindWords.isEmpty {
                    Text(kindWords).brand(Self.smallType, 600).foregroundStyle(Accent).lineLimit(1)
                }
                Text(notice.tile.name).brand(Self.nameType, 700).foregroundStyle(ink).lineLimit(1)
                Text(notice.date.rawValue).brand(Self.smallType).foregroundStyle(Muted).lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .frame(width: lineWide.ref, height: Spot.stillHigh.ref, alignment: .leading)
        .background(Plate, in: shape)
        .overlay(shape.stroke(.white, lineWidth: on ? Spot.edge.ref : 0))
        // Read and unread differ in the weight of the ink and not in a mark: a dot small enough to
        // sit in a line of text is a dot nobody sees from three metres away. So the dot the phone
        // draws is not drawn here - and the state is said either way, because it is the one thing
        // on the line a person listening could not otherwise know.
        .saysItsWords(state: notice.unread ? words["say.unread"] ?? "" : "", on: on)
    }

    /// A notice still unread is drawn in white, one already read in the muted ink.
    private var ink: Color { notice.unread ? .white : Muted }
}

/// The notifications screen: what has arrived for this viewer, newest first.
///
/// The television's answer to `SurfacesPhone/browse/Notifications.swift`. The phone had this screen
/// and the television had none, so a viewer who set a reminder on their phone was told when the
/// title arrived and the same person in front of the television was told nothing. The server has
/// served `GET /v1/notifications` to both all along.
///
/// What is this screen's own, and what is not: the feed, the five kinds and the read call are the
/// contract's and the phone's; the key map, where the focus goes when the feed changes, the
/// geometry and what a reader hears are this surface's, and they live in `Spot`.
public struct Notices: View {
    let shell: Shell
    let seen: Seen

    @State private var spot = Spot()
    @State private var loading = true
    @State private var reading = false

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        let words = seen.words
        let bare = spot.feed.items.isEmpty

        return ZStack(alignment: .topLeading) {
            Color.black
            // The count is the one thing about this screen that is not on a line of it, and
            // `say.notifications_unread` is the words this project already had for it: the phone
            // says them on the bell it has and the television had no caller for them at all.
            PageHead(title: words["notifications.title"] ?? "", help: head(words))
            if bare {
                // A screen with nothing on it says so in the viewer's own language, in the middle
                // of the screen, rather than drawing an empty page - and while the first answer is
                // still on its way it says that instead, because a page that is loading must not
                // be indistinguishable from a page that is empty.
                Text(words[loading ? "say.loading" : "notifications.empty"] ?? "")
                    .brand(NoticeLine.smallType)
                    .foregroundStyle(Muted)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
            } else {
                // Two boxes and not one: the outer one stands still and clips, so a line that has
                // slid up disappears under the head instead of drawing over it, and the stack
                // inside it is what moves.
                VStack(alignment: .leading, spacing: Spot.gap.ref) {
                    ForEach(Array(spot.feed.items.enumerated()), id: \.element.id) { index, notice in
                        NoticeLine(
                            pictures: shell.pictures,
                            notice: notice,
                            on: index == spot.at,
                            words: words
                        )
                    }
                }
                .offset(y: -(CGFloat(spot.scrolled) * Spot.step).ref)
                .animation(.easeOut(duration: 0.24), value: spot.at)
                .frame(width: lineWide.ref, height: (pageHigh - Spot.first).ref, alignment: .topLeading)
                .clipped()
                .offset(x: subEdge.ref, y: Spot.first.ref)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        // What the remote is on: the notice under it, or, on a screen with no notices, the screen's
        // own name and the line that says there are none.
        .remote(
            says: loading && bare
                ? sayBoth(words["notifications.title"], words["say.loading"])
                : spot.says(words)
        ) { press($0) }
        .task {
            if let answer = await read(feedPath) { spot.arrived(answer) }
            loading = false
        }
    }

    /// The line under the page's title: how many notices have not been read, in the words the
    /// strings give, or nothing at all when every one of them has.
    private func head(_ words: [String: String]) -> String {
        let count = spot.unread
        guard count > 0, let pattern = words["say.notifications_unread"] else { return "" }
        return pattern.fill("n", count)
    }

    /// One word from the remote. The order the other screens settled: Ok first, because choosing
    /// leaves the screen and is not a move; then what moved; then what leaves.
    private func press(_ word: Word) -> Bool {
        if word == .ok {
            if let notice = spot.chosen {
                if notice.unread {
                    // Drawn read at once and told to the server after, which is what the phone
                    // does: the ink on the line must not wait on the network.
                    spot.readTo(notice.id)
                    shell.send("POST", readPath, readBody(notice.id)) { _, _ in
                        shell.refreshNotifications()
                    }
                }
                shell.go(titleAddress(notice.tile.title))
            }
            // True whether or not there was a notice under it: the key was used either way.
            return true
        }
        if spot.press(word) {
            // A remote has no moment where a row comes into view, so the read of the next page
            // hangs off the focus instead.
            if spot.wantsMore { Task { await more() } }
            return true
        }
        if spot.leaves(word) {
            shell.back()
            return true
        }
        return false
    }

    /// `POST /v1/notifications/read`'s body, built from the contract's own type rather than
    /// written out as text.
    private func readBody(_ id: String) -> String {
        guard let json = try? JSONEncoder().encode(NotificationRead(upTo: id)) else { return "" }
        return String(decoding: json, as: UTF8.self)
    }

    /// The next page, when the server says there is one and nothing is already asking.
    private func more() async {
        guard !reading, spot.feed.hasMore, let cursor = spot.feed.cursor, !cursor.isEmpty else { return }
        reading = true
        if let answer = await read("\(feedPath)?cursor=\(cursor)") { spot.andThen(answer) }
        reading = false
    }

    private func read(_ path: String) async -> Notifications? {
        guard let text = await shell.read(path) else { return nil }
        return try? JSONDecoder().decode(Notifications.self, from: Data(text.utf8))
    }
}
