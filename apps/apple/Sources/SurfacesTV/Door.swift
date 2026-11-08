import Contract
import Observation
import Ports
import SwiftUI
import UIKit

private let emptyCard = 0.333
private let rowFade = 0.24
/// The lockup's mark height at the top left of the landing, measured on the reference: 58 of 1080.
private let lockupHigh: CGFloat = 58
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

/// Where the focus can be on the landing: the top row, the showreel card's buttons, a title card's button.
private enum Stop { case home, tab, topSignIn, help, signIn, join, watch }

/// Cards sit this far apart: 1542 wide and 39 between, so the next one shows at 1636.
private let cardStep: CGFloat = 1581

/// The cards hang this far from the top, under the top row.
private let cardTop: CGFloat = 169

/// The top row's centre line and height: the lockup, the home circle, the tab and Sign In share both.
private let rowMid: CGFloat = 104
private let rowHigh: CGFloat = 72

/// Inside a card, words and buttons keep this far from its left and right edges.
private let inset: CGFloat = 74
private let cardMove = 0.32
private let slideEvery = 4.5
private let slideFade = 0.7

private let titleSide = LinearGradient(
    stops: [.init(color: .black.opacity(0.8), location: 0), .init(color: .clear, location: 0.62)],
    startPoint: .leading, endPoint: .trailing
)
private let titleFoot = LinearGradient(
    stops: [.init(color: .clear, location: 0.45), .init(color: .black.opacity(0.75), location: 1)],
    startPoint: .top, endPoint: .bottom
)

/// The pictures of a title card in turn: its backdrop, then its episodes.
private func slidesOf(_ title: TitleDetail?) -> [String] {
    guard let title else { return [] }
    return [wide(title.images)].compactMap { $0 } + (title.images ?? []).filter { $0.role == .still }.map(\.url)
}

/// One title's pictures in turn while its card is the one in view, each fading into the next.
private struct Slideshow: View {
    let slides: [String]
    let running: Bool
    @State private var at = 0

    var body: some View {
        ZStack {
            ForEach(Array(slides.enumerated()), id: \.offset) { index, url in
                if index == at { Art(url).transition(.opacity) }
            }
        }
        .animation(.linear(duration: slideFade), value: at)
        .task(id: running) {
            guard running, slides.count > 1 else { return }
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: UInt64(slideEvery * 1e9))
                if Task.isCancelled { break }
                at = (at + 1) % slides.count
            }
        }
    }
}

/// A trending title as a card: its pictures, logo, facts, story and a way in.
private struct TitleCard: View {
    let title: TitleDetail
    let seen: Seen
    let current: Bool
    let on: Bool

    var body: some View {
        let shape = shapeOf(Round.panel)
        let logoUrl = (title.images ?? []).first { $0.role == .logo }?.url
        ZStack(alignment: .topLeading) {
            Raised
            Slideshow(slides: slidesOf(title), running: current)
            titleSide
            titleFoot
            ZStack(alignment: .bottomLeading) {
                if let logoUrl {
                    Logo(url: logoUrl, width: 520, height: 180)
                } else {
                    Text(title.name).brand(64, 800).lines(64, 70).foregroundStyle(.white).lineLimit(2)
                }
            }
            .frame(width: 520.ref, height: 180.ref, alignment: .bottomLeading)
            .offset(x: inset.ref, y: 424.ref)
            Facts(title, seen.words).offset(x: inset.ref, y: 628.ref)
            Text(title.overview ?? "")
                .brand(30).lines(30, 39)
                .foregroundStyle(.white.opacity(0.85))
                .lineLimit(3)
                .frame(width: 720.ref, alignment: .leading)
                .offset(x: inset.ref, y: 676.ref)
            HStack {
                Spacer(minLength: 0)
                Ask(word: seen.say("door.watch"), on: on, lit: Join, ink: .white)
            }
            .frame(width: (1542 - inset).ref)
            .offset(y: 720.ref)
        }
        .frame(width: 1542.ref, height: 864.ref, alignment: .topLeading)
        .clipShape(shape)
        .overlay(shape.stroke(cardEdge, lineWidth: 3.ref))
    }
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
    let ready: () -> Void
    // As in the reference, Get started holds the focus when the landing first shows.
    @State private var at = Stop.join
    // Which card is in view: 0 is the showreel, then the trending titles.
    @State private var page = 0
    @State private var cards: [TitleDetail]?
    @State private var filmReady = false
    @State private var peekReady = false
    @State private var told = false
    // The brand's lockup sits top left until the viewer first goes up, or along to the titles.
    @State private var raised = false
    @State private var film: Bool
    @State private var slide: CGFloat

    init(shell: Shell, seen: Seen, opening: Bool, roll: Bool, ready: @escaping () -> Void) {
        self.shell = shell
        self.seen = seen
        self.opening = opening
        self.roll = roll
        self.ready = ready
        _film = State(initialValue: opening)
        _slide = State(initialValue: opening ? 0 : slideSteps[0])
    }

    private func tell() {
        if !told && filmReady && cards != nil && peekReady {
            told = true
            ready()
        }
    }

    var body: some View {
        let shape = shapeOf(Round.panel)
        let titles = cards ?? []
        let onTop = at == .home || at == .tab || at == .topSignIn

        func cardStop() -> Stop { page == 0 ? .join : .watch }
        func show(_ card: Int) {
            page = card
            at = cardStop()
        }

        func press(_ word: Word) -> Bool {
            if word == .back {
                // Back walks out the way the viewer came in: the row to its card, a title to the
                // showreel, and only the showreel's buttons leave the app.
                if onTop { at = cardStop() } else if page > 0 { show(0) } else { return false }
                return true
            }
            if onTop {
                switch word {
                case .left: at = at == .topSignIn ? .tab : .home
                case .right:
                    if at == .home && !titles.isEmpty { at = .tab } else if at == .tab && page > 0 { at = .topSignIn }
                case .down: at = cardStop()
                case .ok:
                    switch at {
                    case .home: show(0)
                    case .tab: show(max(page, 1))
                    default: shell.go(Address.sign_in)
                    }
                default: break
                }
            } else if page == 0 {
                switch word {
                case .left: at = at == .join ? .signIn : .help
                // Past Get Started the next card comes in, as on the reference.
                case .right:
                    if at == .help { at = .signIn } else if at == .signIn { at = .join } else if !titles.isEmpty { show(1) }
                case .up: at = .home
                case .ok: shell.go(at == .help ? Address.help : Address.sign_in)
                default: break
                }
            } else {
                switch word {
                case .left: show(page - 1)
                case .right: if page < titles.count { show(page + 1) }
                case .up: at = .tab
                case .ok: shell.go(Address.sign_in)
                default: break
                }
            }
            if onTop || at == .home || at == .tab || page > 0 {
                if !raised { withAnimation(.linear(duration: rowFade)) { raised = true } }
            }
            return true
        }

        return ZStack(alignment: .topLeading) {
            Back
            glow
            if !raised {
                BrandLockup(high: lockupHigh).offset(x: pageEdge.ref, y: (rowMid - lockupHigh / 2).ref).transition(.opacity)
            } else {
                HStack(spacing: 16.ref) {
                    Circle()
                        .fill(at == .home ? Color.white : Color.white.opacity(0.12))
                        .frame(width: rowHigh.ref, height: rowHigh.ref)
                        .overlay(Icon("icon_home", 34, tint: at == .home ? Back : .white))
                    if !titles.isEmpty {
                        // The tab of the titles: grey while a title card is in view, white in focus.
                        Text(seen.say("door.trending"))
                            .brand(28, 600)
                            .foregroundStyle(at == .tab ? Back : .white)
                            .padding(.horizontal, 32.ref)
                            .frame(height: rowHigh.ref)
                            .background(at == .tab ? Color.white : page > 0 ? Color.white.opacity(0.16) : .clear, in: Capsule())
                    }
                }
                .offset(x: pageEdge.ref, y: (rowMid - rowHigh / 2).ref)
                .transition(.opacity)
                if page > 0 {
                    // On a title card the way to sign in moves up to the top right, outlined until in focus.
                    let on = at == .topSignIn
                    HStack {
                        Spacer(minLength: 0)
                        Text(seen.say("door.sign_in"))
                            .brand(28, 600)
                            .foregroundStyle(on ? Back : .white)
                            .padding(.horizontal, 40.ref)
                            .frame(minWidth: 200.ref)
                            .frame(height: rowHigh.ref)
                            .background(on ? Color.white : .clear, in: Capsule())
                            .overlay(Capsule().stroke(on ? .clear : Color.white.opacity(0.5), lineWidth: 3.ref))
                    }
                    .frame(width: (1920 - pageEdge).ref)
                    .offset(y: (rowMid - rowHigh / 2).ref)
                    .transition(.opacity)
                }
            }
            ZStack(alignment: .topLeading) {
                cardWash
                // The showreel rests while another card is in view.
                if film {
                    Film(shell: shell, url: "\(seen.server)/tv/showreel.mp4", loop: true, on: roll && page == 0, ready: {
                        filmReady = true
                        tell()
                    }) {
                        filmReady = true
                        tell()
                    }
                }
                // Get Help sits on the buttons' centre line; its words keep the card's inset, the pill grows around them.
                Text(seen.say("door.help"))
                    .brand(24, 600)
                    .foregroundStyle(at == .help ? Back : .white.opacity(0.85))
                    .padding(.horizontal, 24.ref)
                    .frame(height: 56.ref)
                    .background(at == .help ? Color.white : .clear, in: Capsule())
                    .offset(x: (inset - 24).ref, y: 732.ref)
                HStack(spacing: 32.ref) {
                    Spacer(minLength: 0)
                    Ask(word: seen.say("door.sign_in"), on: at == .signIn, lit: .white, ink: Back)
                    Ask(word: seen.say("door.join"), on: at == .join, lit: Join, ink: .white)
                }
                .frame(width: (1542 - inset).ref)
                .offset(y: 720.ref)
            }
            .frame(width: 1542.ref, height: 864.ref, alignment: .topLeading)
            .clipShape(shape)
            .overlay(shape.stroke(cardEdge, lineWidth: 3.ref))
            .offset(x: pageEdge.ref + slide * stage.ref - CGFloat(page) * cardStep.ref, y: cardTop.ref)
            // A card that leaves to the left fades as it goes, so no sliver of it waits at the edge.
            .opacity(page > 0 ? 0 : 1)
            // Each title card is built when it is near, one beyond each side so it slides in.
            ForEach(Array(titles.enumerated()), id: \.element.title) { index, title in
                let card = index + 1
                if card >= page - 1 && card <= page + 2 {
                    TitleCard(title: title, seen: seen, current: card == page, on: card == page && at == .watch)
                        .offset(x: (pageEdge + CGFloat(card - page) * cardStep).ref, y: cardTop.ref)
                        .opacity(card < page ? 0 : 1)
                }
            }
            // Every title's pictures load at once, so each card has them when it comes into view.
            ZStack {
                ForEach(Array(titles.enumerated()), id: \.offset) { index, title in
                    ForEach(Array(slidesOf(title).enumerated()), id: \.offset) { which, url in
                        Art(url) {
                            if index == 0 && which == 0 && !peekReady {
                                peekReady = true
                                tell()
                            }
                        }
                    }
                }
            }
            .frame(width: 4, height: 4)
            .opacity(0)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .animation(glide(cardMove), value: page)
        .remote { press($0) }
        .task {
            // The titles load under the opening when there is one, and the opening waits for the
            // showreel and the next card, so nothing comes in late.
            let list = await shell.read("/v1/public/trending")
                .flatMap { try? JSONDecoder().decode(PublicTitles.self, from: Data($0.utf8)).titles } ?? []
            cards = list
            if slidesOf(list.first).isEmpty { peekReady = true }
            tell()
        }
        .task {
            guard !opening else { return }
            for value in slideSteps.dropFirst() {
                withAnimation(.linear(duration: frameTime)) { slide = value }
                try? await Task.sleep(nanoseconds: UInt64(frameTime * 1e9))
            }
            try? await Task.sleep(nanoseconds: UInt64(emptyCard * 1e9))
            film = true
        }
    }
}

/// The way in: the landing. The first time, the opening is drawn over it (see `OpeningLayer`), and it
/// loads its showreel underneath so the arrow opens onto the film.
public struct Door: View {
    let shell: Shell
    let seen: Seen
    let state: OpeningState
    @State private var opening: Bool

    public init(shell: Shell, seen: Seen, state: OpeningState) {
        self.shell = shell
        self.seen = seen
        self.state = state
        _opening = State(initialValue: !state.opened)
    }

    public var body: some View {
        Landing(shell: shell, seen: seen, opening: opening, roll: state.roll) { state.landingReady = true }
    }
}
