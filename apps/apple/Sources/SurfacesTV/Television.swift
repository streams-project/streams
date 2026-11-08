import Contract
import Ports
import SwiftUI

public struct Television: View {
    public let shell: Shell

    @State private var opening = OpeningState()
    @State private var memory = PageMemory()
    @State private var search = SearchState()
    // Whether this run has been past the profile chooser, so Back there returns rather than leaves.
    @State private var inside = false

    public init(shell: Shell) {
        self.shell = shell
    }

    public var body: some View {
        let seen = shell.seen
        // A language may ask for larger text (Arabic letters read small beside Latin at one size).
        BrandFont.scale = min(max(CGFloat(Double(seen.words["language.text_scale"] ?? "") ?? 1), 0.8), 1.4)
        Speaking.arabicDigits = seen.words["language.digits"] == "arab"
        Speaking.code = seen.language.isEmpty ? "en" : seen.language
        return ZStack {
            Color.black
            // Under the opening the first screen is built and loading, and drawn once the first stage is over.
            screens(seen)
                .opacity(opening.shown || opening.opened ? 1 : 0)
                .environment(\.covered, !opening.opened)
                .environment(\.pages, memory)
                // A right to left language mirrors every screen; the player keeps its own way (Watch).
                .environment(\.layoutDirection, seen.rtl ? .rightToLeft : .leftToRight)
            if !seen.server.isEmpty { OpeningLayer(shell: shell, seen: seen, state: opening) }
        }
        .frame(width: 1920.ref, height: 1080.ref, alignment: .topLeading)
        .ignoresSafeArea()
        // A run of the app with no server yet starts at the address screen, with no opening.
        .onChange(of: seen.server.isEmpty, initial: true) { _, none in if none { opening.over() } }
        .onChange(of: seen.route?.target) { _, target in
            if let target, ![.wait, .profiles, .start, .signIn].contains(target) { inside = true }
        }
    }

    @ViewBuilder
    private func screens(_ seen: Seen) -> some View {
        ZStack {
            if seen.server.isEmpty {
                ServerAddress(shell: shell)
            } else if let target = seen.route?.target {
                switch target {
                case .start:
                    Door(shell: shell, seen: seen, state: opening)
                case .signIn:
                    SignIn(shell: shell, seen: seen)
                case .profiles:
                    Profiles(shell: shell, seen: seen, fromInside: inside)
                case .title:
                    TitlePage(shell: shell, seen: seen)
                case .watch:
                    // The player keeps left to right in every language: its timeline and seek keys follow
                    // the film, not the reading direction.
                    Watch(shell: shell, seen: seen).environment(\.layoutDirection, .leftToRight)
                case .screen where seen.route?.screen == .search:
                    Search(shell: shell, seen: seen, state: search)
                case .screen where seen.route?.screen == .help:
                    GetHelp(shell: shell, seen: seen)
                case .home, .kidsHome, .screen:
                    Browse(shell: shell, seen: seen)
                        .id("\(seen.route?.screen?.rawValue ?? "home") \(seen.route?.genre ?? "")")
                case .wait:
                    Color.clear
                default:
                    Browse(shell: shell, seen: seen)
                }
            } else {
                Color.clear
            }
        }
    }
}
