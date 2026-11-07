import Contract
import Ports
import SwiftUI

public struct Television: View {
    public let shell: Shell

    public init(shell: Shell) {
        self.shell = shell
    }

    public var body: some View {
        let seen = shell.seen
        ZStack {
            BrandTheme.back.ignoresSafeArea()

            if seen.server.isEmpty {
                ServerAddress(shell: shell)
            } else if let target = seen.route?.target {
                switch target {
                case .start:
                    Door(shell: shell, seen: seen)
                case .signIn:
                    SignIn(shell: shell, seen: seen)
                case .profiles:
                    Profiles(shell: shell, seen: seen)
                case .home, .kidsHome, .screen:
                    Browse(shell: shell, seen: seen)
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
