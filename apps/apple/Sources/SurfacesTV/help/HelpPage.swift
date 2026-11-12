import Contract
import Ports
import SwiftUI

private let moveTime = 0.24
private let page: CGFloat = 320
private let screen: CGFloat = 1080
private let firstLine: CGFloat = 262
private let lineStep: CGFloat = 78

private enum Open: Equatable {
    case article(String)
    case policy(String)
}

private func decode<T: Decodable>(_ type: T.Type, _ text: String?) -> T? {
    text.flatMap { try? JSONDecoder().decode(type, from: Data($0.utf8)) }
}

extension Seen {
    @MainActor var query: String { language.isEmpty ? "" : "?lang=\(language)" }
}

private struct Tall: PreferenceKey {
    static let defaultValue: CGFloat = 0
    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) { value = max(value, nextValue()) }
}

/// A long page, moved a screenful at a time by the remote.
private struct Reading: View {
    let title: String
    let sections: [HelpSection]
    let leave: () -> Void
    @State private var moved: CGFloat = 0
    @State private var high: CGFloat = 0

    var body: some View {
        let last = max(0, high - (screen - 160).ref)
        ZStack(alignment: .topLeading) {
            Color.black
            VStack(alignment: .leading, spacing: 36.ref) {
                // The article's own title, from the server, and a heading so a reader can jump to
                // it. The body under it is prose and is read as it stands.
                Text(title).brand(56, 800).foregroundStyle(.white)
                    .accessibilityAddTraits(.isHeader)
                HelpBody(sections: sections)
            }
            .frame(width: 1500.ref, alignment: .leading)
            .fixedSize(horizontal: false, vertical: true)
            .background(GeometryReader { Color.clear.preference(key: Tall.self, value: $0.size.height) })
            .offset(x: subEdge.ref, y: 80.ref - moved)
            .animation(glide(moveTime), value: moved)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .clipped()
        .onPreferenceChange(Tall.self) { high = $0 }
        // An article has no controls at all: Up and Down move the page under a fixed window, so
        // there is nothing for the remote to be "on" and nothing to announce. A reader walks the
        // prose itself, which is what the window is showing. Announcing the title on every press
        // would say the same words over and over while the words a person wants went unread.
        .remote { word in
            switch word {
            case .up: moved = max(0, moved - page.ref)
            case .down: moved = min(last, moved + page.ref)
            // An article is read with Up and Down; Left and Back return to the list it came from.
            case .back, .left: leave()
            default: break
            }
            return true
        }
    }
}

private struct ArticlePage: View {
    let shell: Shell
    let seen: Seen
    let slug: String
    let leave: () -> Void
    @State private var page: HelpArticle?

    var body: some View {
        ZStack {
            if let page { Reading(title: page.title, sections: page.sections, leave: leave) }
            // Black while the article is on its way: nothing drawn, nothing to be on, and only
            // Back does anything. A reader finds an empty screen, which is what it is.
            else { Color.black.remote { if $0 == .back { leave() }; return true } }
        }
        .task(id: slug + seen.language) {
            page = decode(HelpArticle.self, await shell.read("/v1/help/articles/\(slug)\(seen.query)"))
        }
    }
}

private struct PolicyPage: View {
    let shell: Shell
    let seen: Seen
    let id: String
    let leave: () -> Void
    @State private var page: HelpPolicy?

    var body: some View {
        ZStack {
            if let page { Reading(title: page.title, sections: page.sections, leave: leave) }
            // Black while the article is on its way: nothing drawn, nothing to be on, and only
            // Back does anything. A reader finds an empty screen, which is what it is.
            else { Color.black.remote { if $0 == .back { leave() }; return true } }
        }
        .task(id: id + seen.language) {
            page = decode(HelpPolicy.self, await shell.read("/v1/help/policies/\(id)\(seen.query)"))
        }
    }
}

/// One line of the right column: an article or policy, or a heading that takes no focus.
private struct Entry {
    let word: String
    let open: Open?
}

/// A line of the list said as what it is: a control the remote can stop on, or a heading over the
/// controls under it. Calling every line a button would promise sixty presses that do nothing.
private struct EntrySays: ViewModifier {
    let control: Bool
    let on: Bool

    func body(content: Content) -> some View {
        if control {
            content.saysItsWords(on: on)
        } else {
            content.accessibilityAddTraits(.isHeader)
        }
    }
}

private func entries(_ home: HelpHome, _ topic: Int, _ policiesWord: String) -> [Entry] {
    if topic == home.topics.count {
        return [Entry(word: policiesWord, open: nil)] + home.policies.map { Entry(word: $0.title, open: .policy($0.id)) }
    }
    return (home.topics[safe: topic]?.groups ?? []).flatMap { group in
        [Entry(word: group.title, open: nil)] + group.articles.map { Entry(word: $0.title, open: .article($0.slug)) }
    }
}

/// Topics on the left and their articles on the right. The words come from the server.
struct HelpCentre: View {
    let shell: Shell
    let seen: Seen
    let leave: () -> Void
    @State private var home: HelpHome?
    @State private var asked = false
    @State private var open: Open?
    @State private var topic = 0
    @State private var right = false
    @State private var line = 0

    var body: some View {
        Group {
            switch open {
            case .article(let slug)?: ArticlePage(shell: shell, seen: seen, slug: slug) { open = nil }
            case .policy(let id)?: PolicyPage(shell: shell, seen: seen, id: id) { open = nil }
            case nil: centre
            }
        }
        .task(id: seen.language) {
            home = decode(HelpHome.self, await shell.read("/v1/help\(seen.query)"))
            asked = true
        }
    }

    private var centre: some View {
        let policies = seen.say("help.policies")
        let list = home.map { entries($0, topic, policies) } ?? []
        let focusable = list.indices.filter { list[$0].open != nil }
        let topics = (home?.topics ?? []).map { MenuLine(word: $0.title) } + [MenuLine(word: policies)]
        let at = focusable[safe: line] ?? -1
        let scrolled = CGFloat(max(0, at - 7))

        func press(_ word: Word) -> Bool {
            guard home != nil else {
                if word == .back { leave() }
                return true
            }
            if word == .back && right { right = false }
            else if word == .back { leave() }
            else if right && word == .left { right = false }
            // On the topics, the left edge of the page, Left goes back to Get Help.
            else if word == .left { leave() }
            else if right && word == .up { if line > 0 { line -= 1 } }
            else if right && word == .down { if line < focusable.count - 1 { line += 1 } }
            else if right && word == .ok { open = list[focusable[line]].open }
            else if word == .up { if topic > 0 { topic -= 1; line = 0 } }
            else if word == .down { if topic < topics.count - 1 { topic += 1; line = 0 } }
            else if (word == .right || word == .ok) && !focusable.isEmpty { right = true }
            return true
        }

        return ZStack(alignment: .topLeading) {
            Color.black
            PageHead(title: seen.say("help.centre"), help: seen.say("help.intro"))
            if home == nil {
                if asked {
                    Text(seen.say("help.off")).brand(32).foregroundStyle(Muted)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            } else {
                Menu(lines: topics, focus: right ? -1 : topic, width: 480, kept: topic).offset(x: (subEdge - menuWords).ref, y: firstLine.ref)
                VStack(alignment: .leading, spacing: 6.ref) {
                    ForEach(Array(list.enumerated()), id: \.offset) { index, entry in
                        let on = right && index == at
                        Text(entry.word)
                            .brand(30, entry.open == nil ? 700 : 400)
                            .foregroundStyle(on ? Back : entry.open == nil ? Muted : .white)
                            .lineLimit(1)
                            .padding(.horizontal, 32.ref)
                            .padding(.vertical, 12.ref)
                            .frame(width: 1080.ref, alignment: .leading)
                            .background(on ? Color.white : .clear, in: Capsule())
                            // An entry that opens something is a control and draws its own word. A
                            // group's name opens nothing and takes no focus, so it stays plain text
                            // rather than becoming a button nobody can press: the only thing
                            // separating the two on screen is the weight they are drawn in.
                            .modifier(EntrySays(control: entry.open != nil, on: on))
                    }
                }
                .offset(y: -scrolled * lineStep.ref)
                .animation(glide(moveTime), value: scrolled)
                .frame(width: 1100.ref, height: (screen - firstLine - 60).ref, alignment: .topLeading)
                .clipped()
                .offset(x: 700.ref, y: firstLine.ref)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        // What the remote is on: a topic in the column on the left, or an article in the list on the
        // right. Both words come from the server, already in the viewer's language.
        .remote(says: right ? (list[safe: at]?.word ?? "") : (topics[safe: topic]?.word ?? "")) { press($0) }
    }
}
