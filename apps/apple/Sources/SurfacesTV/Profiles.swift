import Contract
import Ports
import SwiftUI

private let profileColors: [Color] = [
    Color(red: 229/255.0, green: 9/255.0, blue: 20/255.0),
    Color(red: 33/255.0, green: 150/255.0, blue: 243/255.0),
    Color(red: 76/255.0, green: 175/255.0, blue: 80/255.0),
    Color(red: 255/255.0, green: 152/255.0, blue: 0/255.0),
    Color(red: 156/255.0, green: 39/255.0, blue: 176/255.0),
]

public struct Profiles: View {
    public let shell: Shell
    public let seen: Seen
    @State private var at: Int = 0

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        ZStack {
            BrandTheme.back.ignoresSafeArea()

            VStack(spacing: 48.ref) {
                Spacer()

                Text(seen.words["profiles.who"] ?? "Who is watching?")
                    .font(.system(size: 64.refType, weight: .bold))
                    .foregroundStyle(.white)

                HStack(spacing: 40.ref) {
                    ForEach(Array(seen.profiles.enumerated()), id: \.element.id) { index, profile in
                        let on = index == at
                        let color = profileColors[index % profileColors.count]

                        VStack(spacing: 20.ref) {
                            ZStack {
                                color
                                    .frame(width: 180.ref, height: 180.ref)
                                    .clipShape(RoundedRectangle(cornerRadius: 16.ref))

                                Text(String(profile.name.prefix(1)).uppercased())
                                    .font(.system(size: 80.refType, weight: .heavy))
                                    .foregroundStyle(.white)
                            }
                            .overlay(
                                RoundedRectangle(cornerRadius: 16.ref)
                                    .stroke(on ? Color.white : Color.clear, lineWidth: 6.ref)
                            )
                            .scaleEffect(on ? 1.08 : 1.0)
                            .animation(.easeOut(duration: 0.15), value: on)

                            Text(profile.name)
                                .font(.system(size: 28.refType, weight: on ? .bold : .medium))
                                .foregroundStyle(on ? .white : BrandTheme.muted)
                        }
                    }
                }
                .padding(.horizontal, 80.ref)

                Spacer()
            }
        }
        .remote { word in
            let count = seen.profiles.count
            guard count > 0 else { return false }
            switch word {
            case .left:
                if at > 0 { at -= 1 }
                return true
            case .right:
                if at < count - 1 { at += 1 }
                return true
            case .ok:
                if let chosen = seen.profiles[safe: at] {
                    shell.act(.enterProfile(ActionEnterProfile(profile: chosen.id)))
                }
                return true
            case .back:
                shell.back()
                return true
            default:
                return true
            }
        }
    }
}

extension Collection {
    subscript(safe index: Index) -> Element? {
        return indices.contains(index) ? self[index] : nil
    }
}
