import Contract
import Ports
import SwiftUI

private enum Pick {
    case switchProfiles
    case other(Profile)
    case help
}

/// One line of the card: a face or an icon, a word, and what it does.
private struct Line<Lead: View>: View {
    let on: Bool
    let word: String
    var under: String? = nil
    @ViewBuilder let lead: (Color) -> Lead

    var body: some View {
        let ink = on ? Back : Color.white
        HStack(spacing: 20.ref) {
            lead(ink).frame(width: 60.ref, height: 60.ref)
            VStack(alignment: .leading, spacing: 0) {
                Text(word).brand(28, under != nil ? 700 : 400).foregroundStyle(ink).lineLimit(1)
                if let under { Text(under).brand(24).foregroundStyle(on ? Back : Muted).lineLimit(1) }
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 16.ref)
        .frame(height: 84.ref)
        .background(on ? Color.white : .clear, in: RoundedRectangle(cornerRadius: 16.ref))
    }
}

/// The card behind the face: who is watching, the other profiles, and help. A television app has no
/// Exit line on Apple TV: the system's own Back leaves the app, and there is no way for an app to quit.
struct ProfileMenu: View {
    let shell: Shell
    let seen: Seen
    let person: Profile?
    let leave: () -> Void
    @State private var at = 0

    var body: some View {
        let others = seen.profiles.filter { $0.id != person?.id }
        let picks: [Pick] = [.switchProfiles] + others.map(Pick.other) + [.help]

        func choose() {
            switch picks[at] {
            case .switchProfiles: shell.go(Address.profiles)
            case .other(let other):
                if other.hasPin {
                    shell.go(Address.profiles)
                } else {
                    shell.speak(other.language)
                    shell.act(.enterProfile(ActionEnterProfile(profile: other.id)))
                }
            case .help: shell.go(Address.help)
            }
        }

        return ZStack(alignment: .topLeading) {
            VStack(spacing: 0) {
                ForEach(Array(picks.enumerated()), id: \.offset) { index, pick in
                    let on = index == at
                    switch pick {
                    case .switchProfiles:
                        Line(on: on, word: person?.name ?? "", under: seen.say("menu.switch")) { _ in
                            Face(person, 60, 10, seen.server)
                        }
                    case .other(let other):
                        Line(on: on, word: other.name) { _ in Face(other, 60, 10, seen.server) }
                    case .help:
                        Line(on: on, word: seen.say("menu.help")) { Icon("icon_question", 38, tint: $0) }
                    }
                }
            }
            .padding(12.ref)
            .frame(width: 470.ref)
            .background(Plate, in: RoundedRectangle(cornerRadius: 24.ref))
            .offset(x: 78.ref, y: 36.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { word in
            switch word {
            case .up: if at > 0 { at -= 1 }
            case .down: if at < picks.count - 1 { at += 1 }
            case .ok: choose()
            case .back: leave()
            default: break
            }
            return true
        }
    }
}
