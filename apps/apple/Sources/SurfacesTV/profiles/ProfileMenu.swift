import Contract
import Ports
import SwiftUI

private enum Pick {
    case switchProfiles
    case other(Profile)
    case help
}

/// The card's padding, and a line's padding before its face.
let menuInset: CGFloat = 12
private let linePad: CGFloat = 16
private let lineHigh: CGFloat = 84
private let menuFace: CGFloat = 60

/// The card opens from the face in the top bar: its first face is centred where that face is
/// (the bar's face is 48 at the page edge, centre 68), so the menu seems to grow out of it.
private let menuX = pageEdge + 24 - menuFace / 2 - linePad - menuInset
private let menuY = 68 - lineHigh / 2 - menuInset

/// One line of the card: a face or an icon, a word, and what it does.
private struct Line<Lead: View>: View {
    let on: Bool
    let word: String
    var under: String? = nil
    @ViewBuilder let lead: (Color) -> Lead

    var body: some View {
        let ink = on ? Back : Color.white
        HStack(spacing: 20.ref) {
            lead(ink).frame(width: menuFace.ref, height: menuFace.ref)
            VStack(alignment: .leading, spacing: 0) {
                Text(word).brand(28, under != nil ? 700 : 400).foregroundStyle(ink).lineLimit(1).frame(height: 34.ref)
                if let under { Text(under).brand(22).foregroundStyle(on ? Back : Muted).lineLimit(1).frame(height: 28.ref) }
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, linePad.ref)
        .frame(height: lineHigh.ref)
        .background(on ? Color.white : .clear, in: shapeOf(Round.inside(Round.panel, menuInset)))
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
                            Face(person, menuFace, seen.server)
                        }
                    case .other(let other):
                        Line(on: on, word: other.name) { _ in Face(other, menuFace, seen.server) }
                    case .help:
                        Line(on: on, word: seen.say("menu.help")) { Icon("icon_question", 38, tint: $0) }
                    }
                }
            }
            .padding(menuInset.ref)
            .frame(width: 470.ref)
            .background(Plate, in: shapeOf(Round.panel))
            .offset(x: menuX.ref, y: menuY.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { word in
            switch word {
            case .up: if at > 0 { at -= 1 }
            case .down: if at < picks.count - 1 { at += 1 }
            case .ok: choose()
            // The card hangs from the face at the left, so Left closes it as Back does.
            case .back, .left: leave()
            default: break
            }
            return true
        }
    }
}
