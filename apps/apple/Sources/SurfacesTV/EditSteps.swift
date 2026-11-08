import Contract
import Ports
import SwiftUI

/// A small pill low on the left of a page, for Done or Cancel.
struct LowPill: View {
    let word: String
    let on: Bool
    let top: CGFloat

    var body: some View {
        Text(word)
            .brand(30, 700)
            .foregroundStyle(on ? Back : .white)
            .frame(width: 224.ref, height: 61.ref)
            .background(on ? Color.white : Color.white.opacity(0.16), in: Capsule())
            .offset(x: subEdge.ref, y: top.ref)
    }
}

/// What this profile may watch. The server holds the rule, and this only says it.
struct AgeNote: View {
    let seen: Seen
    let person: Profile
    let limit: String
    var top: CGFloat = 690

    var body: some View {
        let key = person.kids ? "kids" : limit
        let chip = seen.words["profile.age.\(key)"] ?? seen.say("profile.age.limit").replacingOccurrences(of: "{age}", with: limit)
        let line = seen.words["profile.age.\(key).help"] ?? seen.say("profile.age.limit.help")
        ZStack(alignment: .topLeading) {
            Text(chip).brand(24, 700).foregroundStyle(.white)
                .padding(.horizontal, 10.ref)
                .background(Color.white.opacity(0.2), in: shapeOf(Round.chip))
                .offset(x: 157.ref, y: top.ref)
            Text(line).brand(30).foregroundStyle(.white)
                .frame(width: 900.ref, alignment: .leading)
                .offset(x: 157.ref, y: (top + 48).ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

/// The first page: what can be changed.
struct EditMenu: View {
    let seen: Seen
    let person: Profile
    let limit: String
    let language: String?
    let pictures: Bool
    let at: Int
    let move: (Int) -> Void
    let open: (EditStep) -> Void
    let leave: () -> Void
    @State private var asking = false
    @State private var askAt = 1

    var body: some View {
        var lines: [(EditStep, MenuLine)] = [(.name, MenuLine(word: seen.say("profile.name"), icon: "icon_person"))]
        if pictures { lines.append((.icon, MenuLine(word: seen.say("profile.icon"), icon: "icon_face"))) }
        lines.append((.language, MenuLine(word: seen.say("profile.language"), icon: "icon_language")))
        lines.append((.lock, MenuLine(
            word: seen.say(person.hasPin ? "profile.lock.remove" : "profile.lock"),
            icon: person.hasPin ? "icon_lock_open" : "icon_locked"
        )))
        if !person.main { lines.append((.delete, MenuLine(word: seen.say("profile.delete")))) }
        let askLines = [MenuLine(word: seen.say("profile.delete")), MenuLine(word: seen.say("common.cancel"))]

        return ZStack(alignment: .topLeading) {
            PageHead(title: seen.say("profile.edit"), help: seen.say(asking ? "profile.delete.ask" : "profile.edit.help"))
            if asking {
                Menu(lines: askLines, focus: askAt).offset(x: (subEdge - menuWords).ref, y: 397.ref)
            } else {
                Menu(lines: lines.map(\.1), focus: at).offset(x: subEdge.ref, y: 397.ref)
                AgeNote(seen: seen, person: person, limit: limit, top: lines.count > 4 ? 750 : 690)
                LowPill(word: seen.say("common.done"), on: at == lines.count, top: 897)
            }
            ProfileCard(person: person, server: seen.server, name: { CardName(name: person.name) }, under: language)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { word in
            if asking {
                switch word {
                case .up: if askAt > 0 { askAt -= 1 }
                case .down: if askAt < askLines.count - 1 { askAt += 1 }
                case .ok: if askAt == 0 { open(.delete) } else { asking = false }
                case .back, .left: asking = false
                default: return false
                }
                return true
            }
            switch word {
            case .up: if at > 0 { move(at - 1) }
            case .down: if at < lines.count { move(at + 1) }
            case .ok:
                switch lines[safe: at]?.0 {
                case nil: leave()
                case .delete?: asking = true; askAt = 1
                case let chosen?: open(chosen)
                }
            // The menu is the left edge of the page: Left goes back, as Back does.
            case .back, .left: leave()
            default: return false
            }
            return true
        }
    }
}

/// The keyboard, with the name growing under the picture.
struct EditName: View {
    let seen: Seen
    let person: Profile
    let refused: Bool
    let finish: (String?) -> Void
    @State private var keys: KeyboardState

    init(seen: Seen, person: Profile, refused: Bool, finish: @escaping (String?) -> Void) {
        self.seen = seen
        self.person = person
        self.refused = refused
        self.finish = finish
        _keys = State(initialValue: KeyboardState(person.name))
    }

    var body: some View {
        let cancel = seen.say("common.cancel")
        let done = seen.say("common.done")
        ZStack(alignment: .topLeading) {
            PageHead(title: seen.say("profile.edit"), help: seen.say("profile.name.help"))
            Keyboard(state: keys, cancel: cancel, done: done).offset(x: subEdge.ref, y: 302.ref)
            ProfileCard(person: person, server: seen.server, name: {
                HStack(spacing: 3.ref) {
                    CardName(name: keys.text)
                    Rectangle().fill(.white).frame(width: 3.ref, height: 44.ref)
                }
            }, under: refused ? seen.say("error.name") : nil)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { keys.press($0, cancel, done, finish) }
    }
}

/// Rows of pictures by theme.
struct EditIcon: View {
    let seen: Seen
    let person: Profile
    let rows: [PictureRow]
    let finish: (String?) -> Void
    @State private var focus = PictureFocus()

    var body: some View {
        ZStack(alignment: .topLeading) {
            PictureRows(rows: rows, focus: focus, server: seen.server)
            PageHead(title: seen.say("profile.edit"), help: seen.say("profile.icon.help"))
            ProfileCorner(person: person, server: seen.server)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { word in
            switch word {
            case .ok: finish(focus.chosen(rows))
            case .back: finish(nil)
            default: _ = focus.press(word, rows)
            }
            return true
        }
    }
}

/// Every language this server has words for.
struct EditLanguage: View {
    let seen: Seen
    let list: [Language]
    let current: String?
    let finish: (String?) -> Void
    @State private var at: Int?

    var body: some View {
        let ticked = max(0, list.firstIndex(where: { $0.code == current }) ?? 0)
        let at = self.at ?? ticked
        let cancelAt = list.count
        ZStack(alignment: .topLeading) {
            PageHead(title: seen.say("profile.edit"), help: seen.say("profile.language.help"))
            TickGrid(words: list.map(\.name), ticked: ticked, focus: at)
            LowPill(word: seen.say("common.cancel"), on: at == cancelAt, top: 900)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { word in
            var next = at
            switch word {
            case .up:
                if at == cancelAt { next = min(gridLines, list.count) - 1 }
                else if at % gridLines > 0 { next -= 1 }
            case .down:
                if at == cancelAt { break }
                if at % gridLines == gridLines - 1 || at == list.count - 1 { next = cancelAt } else { next += 1 }
            // Left from the first column, or from Cancel, leaves the languages unchanged.
            case .left:
                if at != cancelAt && at >= gridLines { next -= gridLines } else { finish(nil); return true }
            case .right: if at != cancelAt && at + gridLines <= list.count - 1 { next += gridLines }
            case .ok: finish(list[safe: at]?.code)
            case .back: finish(nil)
            }
            self.at = next
            return true
        }
    }
}
