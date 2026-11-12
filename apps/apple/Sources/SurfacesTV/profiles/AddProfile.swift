import Contract
import Ports
import SwiftUI

private let addSmall: CGFloat = 130
private let addGrow = 0.2

private enum AddStep { case name, kids }

/// The last row in the profile column when another profile can be added.
struct AddProfileRow: View {
    let word: String
    let on: Bool

    var body: some View {
        let size = on ? faceLarge : addSmall
        ZStack(alignment: .topLeading) {
            shapeOf(Round.face(size))
                .fill(Raised)
                .overlay(Icon("icon_plus", on ? 64 : 44))
                .overlay(shapeOf(Round.face(size)).stroke(.white, lineWidth: on ? 4.ref : 0))
                .frame(width: size.ref, height: size.ref)
                .offset(x: (on ? 193 : 225).ref)
            if on {
                Text(word).brand(40, 700).foregroundStyle(.white).offset(x: 420.ref, y: 74.ref)
            }
        }
        .frame(width: 720.ref, height: size.ref, alignment: .topLeading)
        .animation(glide(addGrow), value: on)
        // Only the row in focus draws "Add Profile": off it there is a plus and nothing else, so
        // the row is given the word it would draw rather than gathering what is there now.
        .says(word, on: on)
    }
}

/// Two steps to create a new profile: name first, then the Kids choice.
struct AddProfile: View {
    let shell: Shell
    let seen: Seen
    let nameMax: Int
    let finish: (String?) -> Void
    @State private var keys = KeyboardState("")
    @State private var step = AddStep.name
    @State private var refused = false
    @State private var kidsAt = 0
    @State private var busy = false

    var body: some View {
        let preview = Profile(
            hasPin: false, id: "", kids: step == .kids && kidsAt == 1, main: false,
            name: keys.text.trimmingCharacters(in: .whitespaces).isEmpty ? "+" : keys.text
        )

        func create(_ kids: Bool) {
            if busy { return }
            busy = true
            Task {
                let trimmed = String(keys.text.trimmingCharacters(in: .whitespaces).prefix(nameMax))
                let body = String(data: try! JSONEncoder().encode(ProfileCreate(kids: kids, name: trimmed)), encoding: .utf8)!
                guard let answer = await shell.write("POST", "/v1/profiles", body),
                      let created = try? JSONDecoder().decode(Profile.self, from: Data(answer.utf8)) else {
                    refused = true
                    busy = false
                    step = .name
                    return
                }
                let list = await shell.read("/v1/profiles")
                    .flatMap { try? JSONDecoder().decode(ProfileList.self, from: Data($0.utf8)).profiles }
                shell.keepAll(list ?? seen.profiles + [created])
                finish(created.id)
            }
        }

        return Group {
            switch step {
            case .name:
                let cancel = seen.say("common.cancel")
                let done = seen.say("common.next")
                ZStack(alignment: .topLeading) {
                    PageHead(title: seen.say("profile.add"), help: seen.say("profile.add.help"))
                    Keyboard(state: keys, cancel: cancel, done: done, words: seen.words)
                        .offset(x: subEdge.ref, y: 302.ref)
                    ProfileCard(person: preview, server: seen.server, name: {
                        HStack(spacing: 3.ref) {
                            CardName(name: keys.text)
                            Rectangle().fill(.white).frame(width: 3.ref, height: 44.ref)
                        }
                    }, under: refused ? seen.say("error.name") : nil)
                }
                // The keyboard says which of its keys the remote is on; it is the part that knows.
                .remote(says: keys.hereIs(cancel, done, seen.words)) { word in
                    let handled = keys.press(word, cancel, done) { chosen in
                        guard let chosen else { return finish(nil) }
                        let trimmed = chosen.trimmingCharacters(in: .whitespaces)
                        if trimmed.isEmpty {
                            refused = true
                        } else {
                            keys.text = String(trimmed.prefix(nameMax))
                            refused = false
                            step = .kids
                        }
                    }
                    if keys.text.count > nameMax { keys.text = String(keys.text.prefix(nameMax)) }
                    return handled
                }
            case .kids:
                let lines = [MenuLine(word: seen.say("profile.add.kids.no")), MenuLine(word: seen.say("profile.add.kids.yes"))]
                ZStack(alignment: .topLeading) {
                    PageHead(title: seen.say("profile.add"), help: seen.say("profile.add.kids"))
                    Menu(lines: lines, focus: kidsAt, width: 460).offset(x: (subEdge - menuWords).ref, y: 397.ref)
                    ProfileCard(person: preview, server: seen.server, name: { CardName(name: keys.text) }, under: nil)
                }
                // What the remote is on: one of the two answers to the Kids question.
                .remote(says: lines[safe: kidsAt]?.word ?? "") { word in
                    switch word {
                    case .up: if kidsAt > 0 { kidsAt -= 1 }
                    case .down: if kidsAt < lines.count - 1 { kidsAt += 1 }
                    case .ok: create(kidsAt == 1)
                    case .back: finish(nil)
                    default: return false
                    }
                    return true
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
