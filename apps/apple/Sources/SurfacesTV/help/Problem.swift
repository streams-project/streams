import Contract
import Ports
import SwiftUI

private let offlineWaitS: UInt64 = 8_000_000_000

private struct ProblemView: View {
    let heading: String
    let line: String
    var buttonWord: String? = nil
    var onPress: (() -> Void)? = nil
    var onBack: (() -> Void)? = nil

    private func press(_ word: Word) -> Bool {
        switch word {
        case .ok:
            onPress?()
            return true
        case .back:
            if let onBack {
                onBack()
                return true
            }
            return false
        default:
            return true
        }
    }

    var body: some View {
        ZStack {
            Color.black
            VStack(spacing: 0) {
                Text(heading)
                    .brand(52, 700)
                    .foregroundStyle(.white)
                    .multilineTextAlignment(.center)
                Spacer().frame(height: 20.ref)
                Text(line)
                    .brand(32)
                    .foregroundStyle(Muted)
                    .multilineTextAlignment(.center)
                if let buttonWord {
                    Spacer().frame(height: 44.ref)
                    Pill(buttonWord, on: true)
                }
            }
            .frame(width: 1000.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .remote { press($0) }
    }
}

/// Shown when the viewer navigates to an unknown route (`Target.notFound`).
public struct NotFoundProblem: View {
    let shell: Shell
    let seen: Seen

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        ProblemView(
            heading: seen.words["problem.not_found"] ?? "",
            line: seen.words["problem.not_found.help"] ?? "",
            buttonWord: seen.words["nav.home"] ?? "",
            onPress: { shell.go(Address.home) },
            onBack: { shell.go(Address.home) }
        )
    }
}

/// Shown when the server requires a newer client version (`Target.upgrade`).
public struct UpgradeProblem: View {
    let seen: Seen

    public init(seen: Seen) {
        self.seen = seen
    }

    public var body: some View {
        ProblemView(
            heading: seen.words["problem.upgrade"] ?? "",
            line: seen.words["problem.upgrade.help"] ?? ""
        )
    }
}

/// True when the screen the shell holds is the one this address asks for. Until it is, the answer for
/// the tab before is still in hand, and drawing it would show the page the viewer has just left.
@MainActor
public func screenHere(_ seen: Seen) -> Bool {
    let wanted: String? = switch seen.route?.target {
    case .home: "home"
    case .kidsHome: "kids"
    default: seen.route?.screen?.rawValue
    }
    guard let name = seen.screen?.name.rawValue else { return false }
    return wanted == nil || name == wanted
}

/// Draws `Browse`, or the offline problem screen when the target screen stays missing for 8 seconds.
public struct BrowseOrProblem: View {
    let shell: Shell
    let seen: Seen

    @State private var attempts = 0
    @State private var timedOut = false

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        let hasScreen = screenHere(seen)

        if timedOut && !hasScreen {
            let line = seen.notice.map { $0.isEmpty ? (seen.words["problem.offline.help"] ?? "") : $0 }
                ?? (seen.words["problem.offline.help"] ?? "")
            ProblemView(
                heading: seen.words["problem.offline"] ?? "",
                line: line,
                buttonWord: seen.words["problem.retry"] ?? "",
                onPress: {
                    attempts += 1
                    shell.go(Address.start)
                },
                onBack: { shell.back() }
            )
        } else {
            Browse(shell: shell, seen: seen)
                .task(id: "\(seen.route?.target.hashValue ?? 0)-\(hasScreen)-\(attempts)") {
                    if !hasScreen {
                        try? await Task.sleep(nanoseconds: offlineWaitS)
                        timedOut = true
                    } else {
                        timedOut = false
                    }
                }
        }
    }
}
