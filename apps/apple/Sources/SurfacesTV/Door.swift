import Contract
import Ports
import SwiftUI

private enum Stop: Int, CaseIterable {
    case signIn = 0
    case help = 1
}

public struct Door: View {
    public let shell: Shell
    public let seen: Seen
    @State private var at: Stop = .signIn

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        ZStack {
            BrandTheme.back.ignoresSafeArea()

            VStack(spacing: 40.ref) {
                Spacer()

                VStack(spacing: 20.ref) {
                    Text(seen.brand.isEmpty ? "Streams" : seen.brand)
                        .font(.system(size: 80.refType, weight: .heavy))
                        .foregroundStyle(.white)

                    Text(seen.words["door.title"] ?? "Unlimited films, TV programmes and more")
                        .font(.system(size: 36.refType, weight: .medium))
                        .foregroundStyle(BrandTheme.muted)
                }

                Spacer()

                HStack(spacing: 32.ref) {
                    Text(seen.words["common.sign_in"] ?? "Sign In")
                        .font(.system(size: 28.refType, weight: .bold))
                        .foregroundStyle(at == .signIn ? .black : .white)
                        .padding(.horizontal, 48.ref)
                        .padding(.vertical, 22.ref)
                        .background(at == .signIn ? Color.white : Color.white.opacity(0.2))
                        .clipShape(Capsule())

                    Text(seen.words["common.help"] ?? "Help")
                        .font(.system(size: 28.refType, weight: .bold))
                        .foregroundStyle(at == .help ? .black : .white)
                        .padding(.horizontal, 48.ref)
                        .padding(.vertical, 22.ref)
                        .background(at == .help ? Color.white : Color.white.opacity(0.2))
                        .clipShape(Capsule())
                }
                .padding(.bottom, 120.ref)
            }
        }
        .remote { word in
            switch word {
            case .left:
                at = .signIn
                return true
            case .right:
                at = .help
                return true
            case .ok:
                if at == .signIn {
                    shell.go(Address.sign_in)
                } else {
                    shell.go(Address.help)
                }
                return true
            case .back:
                return false
            default:
                return true
            }
        }
    }
}
