import Contract
import Ports
import SwiftUI

public struct SignIn: View {
    public let shell: Shell
    public let seen: Seen

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        ZStack {
            Back.ignoresSafeArea()

            VStack(spacing: 48.ref) {
                Text(seen.words["signin.choose"] ?? "Sign In")
                    .font(.system(size: 48.refType, weight: .bold))
                    .foregroundStyle(.white)
                    .padding(.top, 80.ref)

                HStack(alignment: .top, spacing: 80.ref) {
                    VStack(alignment: .leading, spacing: 24.ref) {
                        HStack(spacing: 16.ref) {
                            Text("1")
                                .font(.system(size: 28.refType, weight: .bold))
                                .foregroundStyle(.white)
                                .frame(width: 52.ref, height: 52.ref)
                                .background(Color.white.opacity(0.2))
                                .clipShape(Circle())

                            Text(
                                (seen.words["signin.step1"] ?? "Visit {address} on your phone or computer")
                                    .replacingOccurrences(of: "{address}", with: seen.device?.verificationUri ?? "streams/activate")
                            )
                            .font(.system(size: 32.refType, weight: .medium))
                            .foregroundStyle(.white)
                            .lineLimit(2)
                        }
                    }
                    .frame(maxWidth: 600.ref)

                    VStack(alignment: .leading, spacing: 24.ref) {
                        HStack(spacing: 16.ref) {
                            Text("2")
                                .font(.system(size: 28.refType, weight: .bold))
                                .foregroundStyle(.white)
                                .frame(width: 52.ref, height: 52.ref)
                                .background(Color.white.opacity(0.2))
                                .clipShape(Circle())

                            Text(seen.words["signin.step2"] ?? "Enter this code:")
                                .font(.system(size: 32.refType, weight: .medium))
                                .foregroundStyle(.white)
                        }

                        let code = seen.device?.userCode ?? (seen.words["signin.getting"] ?? "Getting code...")
                        Text(code)
                            .font(.system(size: 80.refType, weight: .heavy))
                            .foregroundStyle(.white)
                            .tracking(4.ref)
                            .padding(.top, 16.ref)
                    }
                    .frame(maxWidth: 600.ref)
                }
                .padding(.horizontal, 100.ref)

                Spacer()
            }
        }
        .onAppear {
            shell.act(.deviceSignIn(ActionDeviceSignIn(deviceName: "Apple TV")))
        }
        .remote { word in
            switch word {
            case .back:
                shell.back()
                return true
            default:
                return true
            }
        }
    }
}
