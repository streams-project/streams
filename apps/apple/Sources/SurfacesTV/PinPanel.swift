import Contract
import Ports
import SwiftUI

private let defaultDigits = 4

/// A row of digit boxes for entering or choosing a PIN.
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
                    .background(on ? Color.white : Raised, in: RoundedRectangle(cornerRadius: 16.ref))
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

        return VStack(spacing: 0) {
            Face(person, faceLarge, 18, seen.server)
            Spacer().frame(height: 24.ref)
            Text(person.name).brand(56, 800).foregroundStyle(.white)
            Spacer().frame(height: 40.ref)
            PinBoxes(digits: digits.count == count ? digits : Array(repeating: 0, count: count), at: at)
            if refused {
                Spacer().frame(height: 28.ref)
                Text(seen.say("error.pin_wrong")).brand(30).foregroundStyle(Muted)
            }
        }
        .padding(.top, 200.ref)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .remote { word in
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
