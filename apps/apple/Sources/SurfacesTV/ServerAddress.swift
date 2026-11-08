import Ports
import SwiftUI

public struct ServerAddress: View {
    public let shell: Shell
    @State private var address: String = "http://"
    @State private var focused: Bool = true

    public init(shell: Shell) {
        self.shell = shell
    }

    public var body: some View {
        ZStack {
            Back.ignoresSafeArea()
            VStack(spacing: 32.ref) {
                Text("Your server")
                    .font(.system(size: 48.refType, weight: .bold))
                    .foregroundStyle(.white)

                Text(address)
                    .font(.system(size: 32.refType, weight: .medium))
                    .foregroundStyle(Muted)
                    .padding(.horizontal, 32.ref)
                    .padding(.vertical, 16.ref)
                    .background(Color.white.opacity(0.1))
                    .clipShape(shapeOf(Round.field))

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
            }
        }
        .remote { word in
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
