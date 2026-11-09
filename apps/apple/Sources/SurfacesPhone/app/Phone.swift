import Contract
import Ports
import SwiftUI

/// The root of the phone and tablet app: the language's direction, digits, plural forms and text
/// size, then the page the core routes to. Pages here are the M1 skeleton;
public struct Phone: View {
    let shell: Shell

    public init(shell: Shell) {
        self.shell = shell
    }

    public var body: some View {
        let seen = shell.seen
        BrandFont.scale = min(max(CGFloat(Double(seen.words["language.text_scale"] ?? "") ?? 1), 0.8), 1.4)
        Speaking.digits = seen.words["language.digits"] ?? "latn"
        Speaking.code = seen.language.isEmpty ? "en" : seen.language
        return ZStack {
            Back.ignoresSafeArea()
            page(seen)
        }
        .environment(\.layoutDirection, seen.rtl ? .rightToLeft : .leftToRight)
        .preferredColorScheme(.dark)
    }

    @ViewBuilder
    private func page(_ seen: Seen) -> some View {
        let target = seen.route?.target
        if seen.server.isEmpty {
            ServerAddress(shell: shell)
        } else if target == .start {
            Door(shell: shell, seen: seen)
        } else if target == .signIn {
            SignIn(shell: shell, seen: seen)
        } else if target == .profiles {
            GateView(shell: shell, seen: seen)
        } else if target == .title {
            Title(shell: shell, seen: seen)
        } else if target == .screen && seen.route?.screen == .notifications {
            NotificationsView(shell: shell, seen: seen)
        } else if target == .home || target == .kidsHome || target == .screen || target == .watch {
            Pages(shell: shell, seen: seen)
        } else {
            ProgressView().tint(.white)
        }
    }
}
