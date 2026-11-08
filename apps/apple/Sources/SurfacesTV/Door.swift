import Contract
import Observation
import Ports
import SwiftUI
import UIKit

private let emptyCard = 0.333
private let peekAfter = 10.0
private let peekFade = 0.583
private let frameTime = 0.042
private let stage: CGFloat = 1920

// Share of the picture width the card still has to travel, one value per frame, as measured.
private let slideSteps: [CGFloat] = [0.97, 0.647, 0.323, 0.106, 0.077, 0.039, 0.017, 0.005, 0.001, 0]

private let cardWash = LinearGradient(
    stops: [
        .init(color: Color(red: 0x2B / 255.0, green: 0x3A / 255.0, blue: 0x78 / 255.0), location: 0),
        .init(color: Color(red: 0x42 / 255.0, green: 0x54 / 255.0, blue: 0x9C / 255.0), location: 0.7),
        .init(color: Color(red: 0x14 / 255.0, green: 0x17 / 255.0, blue: 0x30 / 255.0), location: 1),
    ],
    startPoint: .top, endPoint: .bottom
)
private let cardEdge = Color(red: 0xC4 / 255.0, green: 0xE4 / 255.0, blue: 0xFB / 255.0)

private enum Stop: Int, CaseIterable { case home, tab, help, signIn, join }

/// Whether the opening has played. It plays once for each run of the app.
@Observable
@MainActor
public final class DoorState {
    var opened = false
    public init() {}
}

/// A brand film or sound. It plays while `on`, and is handed back when it leaves the screen.
struct Film: UIViewRepresentable {
    let shell: Shell
    let url: String
    let loop: Bool
    var on = true
    var ready: () -> Void = {}
    let ended: () -> Void

    /// The newest answers, so a film made earlier reaches the page as it is now.
    final class Kept {
        var shell: Shell?
        var ready: () -> Void = {}
        var ended: () -> Void = {}
    }

    func makeCoordinator() -> Kept { Kept() }

    func makeUIView(context: Context) -> UIView {
        let kept = context.coordinator
        kept.shell = shell
        kept.ready = ready
        kept.ended = ended
        let view = shell.reel.picture(context, url: url, loop: loop, ready: { kept.ready() }, ended: { kept.ended() }) as? UIView ?? UIView()
        shell.reel.play(view, on)
        return view
    }

    func updateUIView(_ view: UIView, context: Context) {
        context.coordinator.ready = ready
        context.coordinator.ended = ended
        shell.reel.play(view, on)
    }

    static func dismantleUIView(_ view: UIView, coordinator: Kept) {
        coordinator.shell?.reel.release(view)
    }
}

private struct Ask: View {
    let word: String
    let on: Bool
    let lit: Color
    let ink: Color

    var body: some View {
        Text(word)
            .brand(28, 700)
            .foregroundStyle(on ? ink : .white)
            .frame(width: 268.ref, height: 80.ref)
            .background(on ? lit : Color.white.opacity(0.22), in: Capsule())
            .scaleEffect(on ? 1.04 : 1)
    }
}

/// The violet light in the top left corner behind the card, as the approved opening renders it:
/// strength (1 - (x / 1500 + y / 620)) ^ 1.5 of 5F28EB.
private let glow = LinearGradient(
    stops: [0, 0.25, 0.5, 0.75].map { f in
        Gradient.Stop(color: Color(red: 0x5F / 255, green: 0x28 / 255, blue: 0xEB / 255).opacity(pow(1 - f, 1.5)), location: f)
    } + [Gradient.Stop(color: .clear, location: 1)],
    startPoint: .topLeading,
    endPoint: UnitPoint(x: 219 / 1920, y: 530 / 1080)
)

/// The landing. After the opening it is already in place under it: the card does not slide, the
/// showreel is there from the first frame, says when it is `ready`, and plays once `roll` is true.
/// Coming back to it later, the card slides in and the showreel starts after it.
private struct Landing: View {
    let shell: Shell
    let seen: Seen
    let opening: Bool
    let roll: Bool
    let keys: Bool
    let ready: () -> Void
    @State private var at = Stop.signIn
    @State private var film: Bool
    @State private var peek = false
    @State private var slide: CGFloat

    init(shell: Shell, seen: Seen, opening: Bool, roll: Bool, keys: Bool, ready: @escaping () -> Void) {
        self.shell = shell
        self.seen = seen
        self.opening = opening
        self.roll = roll
        self.keys = keys
        self.ready = ready
        _film = State(initialValue: opening)
        _slide = State(initialValue: opening ? 0 : slideSteps[0])
    }

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: 24.ref)

        func press(_ word: Word) -> Bool {
            switch word {
            case .left: if at.rawValue > 0 { at = Stop(rawValue: at.rawValue - 1)! }
            case .right: if at.rawValue < Stop.allCases.count - 1 { at = Stop(rawValue: at.rawValue + 1)! }
            case .up: if at.rawValue >= Stop.help.rawValue { at = .tab }
            case .down: if at.rawValue < Stop.help.rawValue { at = .signIn }
            case .ok:
                switch at {
                case .signIn, .join: shell.go(Address.sign_in)
                case .help: shell.go(Address.help)
                default: break
                }
            case .back: return false
            }
            return true
        }

        return ZStack(alignment: .topLeading) {
            Back
            glow
            HStack(spacing: 40.ref) {
                Circle()
                    .fill(at == .home ? Color.white : Color.white.opacity(0.12))
                    .frame(width: 82.ref, height: 82.ref)
                    .overlay(Icon("icon_home", 38, tint: at == .home ? Back : .white))
                Text(seen.say("door.trending"))
                    .brand(30, 600)
                    .foregroundStyle(.white)
                    .opacity(at == .tab ? 1 : 0.85)
            }
            .offset(x: 96.ref, y: 78.ref)
            if peek {
                Art("\(seen.server)/tv/peek.jpg")
                    .frame(width: 284.ref, height: 866.ref)
                    .clipShape(UnevenRoundedRectangle(topLeadingRadius: 24.ref, bottomLeadingRadius: 24.ref))
                    .offset(x: 1636.ref, y: 167.ref)
                    .transition(.opacity.animation(.linear(duration: peekFade)))
            }
            ZStack(alignment: .topLeading) {
                cardWash
                if film { Film(shell: shell, url: "\(seen.server)/tv/showreel.mp4", loop: true, on: roll, ready: ready) { ready() } }
                Text(seen.say("door.help"))
                    .brand(24, 600)
                    .foregroundStyle(at == .help ? Back : .white.opacity(0.85))
                    .background(at == .help ? Color.white : .clear, in: RoundedRectangle(cornerRadius: 12.ref))
                    .offset(x: 96.ref, y: 770.ref)
                HStack(spacing: 32.ref) {
                    Ask(word: seen.say("door.sign_in"), on: at == .signIn, lit: .white, ink: Back)
                    Ask(word: seen.say("door.join"), on: at == .join, lit: Join, ink: .white)
                }
                .offset(x: 915.ref, y: 720.ref)
            }
            .frame(width: 1542.ref, height: 864.ref, alignment: .topLeading)
            .clipShape(shape)
            .overlay(shape.stroke(cardEdge, lineWidth: 3.ref))
            .offset(x: 55.ref + slide * stage.ref, y: 169.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote(on: keys) { press($0) }
        .task {
            guard !opening else { return }
            for value in slideSteps.dropFirst() {
                withAnimation(.linear(duration: frameTime)) { slide = value }
                try? await Task.sleep(nanoseconds: UInt64(frameTime * 1e9))
            }
            try? await Task.sleep(nanoseconds: UInt64(emptyCard * 1e9))
            film = true
        }
        .task(id: roll) {
            guard roll else { return }
            try? await Task.sleep(nanoseconds: UInt64(peekAfter * 1e9))
            withAnimation(.linear(duration: peekFade)) { peek = true }
        }
    }
}

/// The way in: the opening, drawn live, once for each run of the app, over the landing it opens onto.
/// The landing is built under it from the first frame so its showreel can load while the mark rests,
/// and the opening's sounds live here so they play to their end.
public struct Door: View {
    let shell: Shell
    let seen: Seen
    let state: DoorState
    @State private var opening: Bool
    @State private var ready = false
    @State private var roll: Bool
    @State private var shown: Bool
    @State private var notes = false
    @State private var swell = false

    public init(shell: Shell, seen: Seen, state: DoorState) {
        self.shell = shell
        self.seen = seen
        self.state = state
        let first = !state.opened
        _opening = State(initialValue: first)
        _roll = State(initialValue: !first)
        _shown = State(initialValue: !first)
    }

    public var body: some View {
        ZStack {
            // Under the opening's cover the landing loads but is not drawn until the first stage is over.
            Landing(shell: shell, seen: seen, opening: opening, roll: roll, keys: state.opened) { ready = true }
                .opacity(shown ? 1 : 0)
            if opening {
                Film(shell: shell, url: "\(seen.server)/tv/opening-chime.m4a", loop: false, on: notes) {}
                    .frame(width: 0, height: 0)
                Film(shell: shell, url: "\(seen.server)/tv/opening-swell.m4a", loop: false, on: swell) {}
                    .frame(width: 0, height: 0)
            }
            if !state.opened {
                Opening(ready: ready && !seen.words.isEmpty) { moment in
                    switch moment {
                    case .notes: notes = true
                    case .warm: shown = true
                    case .second: swell = true
                    case .roll: roll = true
                    case .done:
                        shown = true
                        roll = true
                        state.opened = true
                    }
                }
            }
        }
    }
}
