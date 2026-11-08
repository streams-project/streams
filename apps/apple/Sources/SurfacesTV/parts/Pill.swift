import SwiftUI

private let changeMs = 0.12
private let quiet = Color.white.opacity(0.16)

/// A large button. In focus it turns white.
public struct Pill: View {
    public let word: String
    public let on: Bool
    public var icon: String?

    public init(_ word: String, on: Bool, icon: String? = nil) {
        self.word = word
        self.on = on
        self.icon = icon
    }

    public var body: some View {
        let ink = on ? Back : Color.white
        HStack(spacing: 12.ref) {
            if let icon { Icon(icon, 34, tint: ink) }
            Text(word).brand(32, 700).foregroundStyle(ink)
        }
        .padding(.horizontal, 46.ref)
        .frame(height: 84.ref)
        .background(on ? Color.white : quiet, in: Capsule())
        .animation(.linear(duration: changeMs), value: on)
    }
}
