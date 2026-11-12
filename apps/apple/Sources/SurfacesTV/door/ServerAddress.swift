import Ports
import SwiftUI

public struct ServerAddress: View {
    public let shell: Shell
    @State private var address: String = "https://"
    @State private var focused: Bool = true

    public init(shell: Shell) {
        self.shell = shell
    }

    public var body: some View {
        ZStack {
            Back.ignoresSafeArea()
            VStack(spacing: 32.ref) {
                // This screen runs before any server has answered, so there are no words from
                // `strings/` to be had and its two English literals are what is drawn. They are a
                // defect of their own, recorded, and not changed here: changing visible text is
                // outside this work. A reader reaches them as text either way.
                Text("Your server")
                    .font(.system(size: 48.refType, weight: .bold))
                    .foregroundStyle(.white)
                    .accessibilityAddTraits(.isHeader)

                Text(address)
                    .font(.system(size: 32.refType, weight: .medium))
                    .foregroundStyle(Muted)
                    .padding(.horizontal, 32.ref)
                    .padding(.vertical, 16.ref)
                    .background(Color.white.opacity(0.1))
                    .clipShape(shapeOf(Round.field))

                // A refused address is said here, where it was typed, or the screen would simply
                // do nothing when the viewer presses Connect.
                if let notice = shell.seen.notice {
                    Text(notice)
                        .font(.system(size: 26.refType, weight: .medium))
                        .foregroundStyle(Join)
                        .multilineTextAlignment(.center)
                        .frame(width: 900.ref)
                }

                Button(action: {
                    shell.connect(address)
                }) {
                    Text("Connect")
                        .font(.system(size: 28.refType, weight: .bold))
                        .foregroundStyle(focused ? .black : .white)
                        .padding(.horizontal, 48.ref)
                        .padding(.vertical, 20.ref)
                        .background(focused ? Color.white : Color.white.opacity(0.2))
                        .clipShape(Capsule())
                }
                .buttonStyle(.plain)
                // The button draws its own word and is the only control on the screen, so the
                // remote is always on it: it says so always, which is true here and nowhere else.
                .saysItsWords(on: focused)
            }
        }
        .remote(says: address) { word in
            switch word {
            case .ok:
                shell.connect(address)
                return true
            default:
                return false
            }
        }
    }
}
