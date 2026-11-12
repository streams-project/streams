import Contract
import Ports
import SwiftUI

private enum LockStep { case password, first, second }

private func json<T: Encodable>(_ value: T) -> String {
    String(data: (try? JSONEncoder().encode(value)) ?? Data(), encoding: .utf8) ?? "{}"
}

/// A password check first, then choosing and confirming a PIN, or clearing the one there is.
struct EditLock: View {
    let shell: Shell
    let seen: Seen
    let person: Profile
    let pinDigits: Int
    let finish: (Profile?) -> Void
    @State private var keys = KeyboardState("")
    @State private var step = LockStep.password
    @State private var refused = false
    @State private var mismatch = false
    @State private var firstPin = ""
    @State private var busy = false
    @State private var digits: [Int] = []
    @State private var at = 0

    private func savePin(_ pin: String?) {
        if busy { return }
        busy = true
        Task {
            guard await shell.write("PUT", "/v1/profiles/\(person.id)/pin", json(PinSet(pin: pin))) != nil else {
                refused = true
                busy = false
                return
            }
            let hasPin = pin != nil
            let read = await shell.read("/v1/profiles")
                .flatMap { try? JSONDecoder().decode(ProfileList.self, from: Data($0.utf8)).profiles }
            let updated = read ?? seen.profiles.map { profile in
                var changed = profile
                if profile.id == person.id { changed.hasPin = hasPin }
                return changed
            }
            shell.keepAll(updated)
            var fallback = person
            fallback.hasPin = hasPin
            finish(updated.first { $0.id == person.id } ?? fallback)
        }
    }

    private func confirmPassword(_ password: String) {
        if busy { return }
        busy = true
        Task {
            let answer = await shell.write("POST", "/v1/auth/confirm", json(Confirm(password: password)))
            // The password is done with, whatever the answer was.
            keys.text = ""
            busy = false
            if answer == nil {
                refused = true
            } else if person.hasPin {
                savePin(nil)
            } else {
                refused = false
                begin(.first)
            }
        }
    }

    private func begin(_ next: LockStep) {
        step = next
        digits = Array(repeating: 0, count: max(1, pinDigits))
        at = 0
    }

    var body: some View {
        switch step {
        case .password:
            let cancel = seen.say("common.cancel")
            let next = seen.say("common.next")
            ZStack(alignment: .topLeading) {
                PageHead(
                    title: seen.say(person.hasPin ? "profile.lock.remove" : "profile.lock"),
                    help: seen.say("profile.lock.password.help")
                )
                Keyboard(state: keys, cancel: cancel, done: next, words: seen.words)
                    .offset(x: subEdge.ref, y: 302.ref)
                ProfileCard(pictures: shell.pictures, person: person, server: seen.server, name: {
                    // The password as it is typed, drawn and heard as bullets: the characters are
                    // not read out, and the count still tells a person their presses are landing.
                    HStack(spacing: 3.ref) {
                        CardName(name: String(repeating: "•", count: keys.text.count))
                        Rectangle().fill(.white).frame(width: 3.ref, height: 44.ref)
                    }
                }, under: refused ? seen.say("error.sign_in") : nil)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            // The keyboard says which of its keys the remote is on; it is the part that knows.
            .remote(says: keys.hereIs(cancel, next, seen.words)) { word in
                keys.press(word, cancel, next) { chosen in
                    guard let chosen else {
                        keys.text = ""
                        return finish(nil)
                    }
                    if chosen.isEmpty {
                        refused = true
                    } else {
                        keys.text = ""
                        confirmPassword(chosen)
                    }
                }
            }
        case .first, .second:
            let count = max(1, pinDigits)
            let shown = digits.count == count ? digits : Array(repeating: 0, count: count)
            ZStack(alignment: .topLeading) {
                PageHead(
                    title: seen.say("profile.lock"),
                    help: seen.say(step == .first ? "profile.lock.pin.help" : "profile.lock.confirm.help")
                )
                VStack(spacing: 0) {
                    // The face is decoration: the name under it says whose profile this is.
                    Face(shell.pictures, person, faceLarge, seen.server)
                    Spacer().frame(height: 24.ref)
                    Text(person.name).brand(56, 800).foregroundStyle(.white)
                        .accessibilityAddTraits(.isHeader)
                    Spacer().frame(height: 40.ref)
                    PinBoxes(digits: shown, at: at)
                    if mismatch {
                        Spacer().frame(height: 28.ref)
                        Text(seen.say("profile.lock.mismatch")).brand(30).foregroundStyle(Muted)
                    }
                }
                .padding(.top, 240.ref)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            // What the remote is on: one of the digit boxes. The page says what it is asking for as
            // well, because a row of figures does not explain itself, and a mismatch is said over
            // the top of it - a viewer whose two PINs did not match hears why they are back at the
            // first box instead of being returned to it in silence.
            .remote(
                says: mismatch
                    ? seen.say("profile.lock.mismatch")
                    : sayBoth(seen.say("say.pin"), shown[safe: at].map { number($0) })
            ) { word in
                if digits.count != count { digits = Array(repeating: 0, count: count) }
                switch word {
                case .left: if at > 0 { at -= 1 }
                case .right: if at < count - 1 { at += 1 }
                case .up: digits[at] = (digits[at] + 1) % 10; mismatch = false
                case .down: digits[at] = (digits[at] + 9) % 10; mismatch = false
                case .ok:
                    let entered = digits.map(String.init).joined()
                    if step == .first {
                        firstPin = entered
                        mismatch = false
                        begin(.second)
                    } else if entered != firstPin {
                        firstPin = ""
                        mismatch = true
                        begin(.first)
                    } else {
                        savePin(entered)
                    }
                case .back: finish(nil)
                }
                return true
            }
        }
    }
}
