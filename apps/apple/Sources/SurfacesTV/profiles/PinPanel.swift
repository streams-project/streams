import Contract
import Ports
import SwiftUI

private let defaultDigits = 4

/// A row of digit boxes for entering or choosing a PIN.
///
/// Each box draws the digit it stands at, and Up and Down turn it. The digits are already drawn on
/// screen in plain figures - this is a dial, not a masked field - so reading them says nothing a
/// person in the room cannot already see.
struct PinBoxes: View {
    let digits: [Int]
    let at: Int

    var body: some View {
        HStack(spacing: 16.ref) {
            ForEach(Array(digits.enumerated()), id: \.offset) { index, digit in
                let on = index == at
                Text(String(digit))
                    .brand(48, 700)
                    .foregroundStyle(on ? Back : .white)
                    .frame(width: 96.ref, height: 96.ref)
                    .background(on ? Color.white : Raised, in: shapeOf(Round.field))
                    // The box the remote is on is drawn white, which is the only thing saying which
                    // of the four digits Up and Down would turn.
                    .saysItsWords(on: on)
            }
        }
    }
}

/// The PIN goes to the server and is never kept.
struct PinPanel: View {
    let shell: Shell
    let seen: Seen
    let person: Profile
    var digitsCount = defaultDigits
    let unlocked: (String) -> Void
    let leave: () -> Void
    @State private var digits: [Int] = []
    @State private var at = 0
    @State private var refused = false
    @State private var busy = false

    var body: some View {
        let count = max(1, digitsCount)

        func check() {
            if busy { return }
            busy = true
            let pin = digits.map(String.init).joined()
            Task {
                let body = String(data: try! JSONEncoder().encode(ProfileEnter(pin: pin)), encoding: .utf8)!
                let answer = await shell.write("POST", "/v1/profiles/\(person.id)/enter", body)
                busy = false
                if answer == nil { refused = true } else { unlocked(pin) }
            }
        }

        let shown = digits.count == count ? digits : Array(repeating: 0, count: count)

        // What the remote is on: one of the digit boxes. The panel says what it is asking for as
        // well, because nothing else on this screen does and a row of figures on its own does not
        // explain itself. A refusal is said over the top of that, since a viewer who pressed Ok and
        // heard nothing change has no other way to learn the PIN was wrong.
        let hereIs = refused
            ? seen.say("error.pin_wrong")
            : sayBoth(seen.say("say.pin"), shown[safe: at].map { number($0) })

        return VStack(spacing: 0) {
            // The face is decoration: the name is drawn under it and says whose profile this is.
            Face(shell.pictures, person, faceLarge, seen.server)
            Spacer().frame(height: 24.ref)
            Text(person.name).brand(56, 800).foregroundStyle(.white)
                .accessibilityAddTraits(.isHeader)
            Spacer().frame(height: 40.ref)
            PinBoxes(digits: shown, at: at)
            if refused {
                Spacer().frame(height: 28.ref)
                Text(seen.say("error.pin_wrong")).brand(30).foregroundStyle(Muted)
            }
        }
        .padding(.top, 200.ref)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .remote(says: hereIs) { word in
            if digits.count != count { digits = Array(repeating: 0, count: count) }
            switch word {
            // Left past the first digit goes back, as Back does.
            case .left: if at > 0 { at -= 1 } else { leave() }
            case .right: if at < count - 1 { at += 1 }
            case .up: digits[at] = (digits[at] + 1) % 10; refused = false
            case .down: digits[at] = (digits[at] + 9) % 10; refused = false
            case .ok: check()
            case .back: leave()
            }
            return true
        }
    }
}
