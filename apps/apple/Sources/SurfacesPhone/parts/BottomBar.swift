import SwiftUI

/// One place on the bar: its icon, its word, and whether it is the page in view.
struct BarItem: Identifiable {
    let icon: String
    let word: String
    let here: Bool
    let go: () -> Void
    var id: String { icon + word }
}

/// The bar at the foot of every main page, floating as the reference's does: a rounded plate over
/// the page, icons above short words, the page in view in white. It turns with a right to left
/// language. This equals BottomBar.kt.
struct BottomBar: View {
    let items: [BarItem]

    var body: some View {
        HStack(spacing: 0) {
            ForEach(items) { item in
                Button(action: item.go) {
                    VStack(spacing: 2) {
                        Icon(item.icon, 24, tint: item.here ? .white : .white.opacity(0.6))
                        Text(item.word).brand(11).foregroundStyle(item.here ? .white : .white.opacity(0.6)).lineLimit(1)
                    }
                    .frame(width: 84, height: 56)
                    .background(item.here ? Color.white.opacity(0.12) : .clear, in: Capsule())
                }
                .buttonStyle(.plain)
                .frame(maxWidth: .infinity)
            }
        }
        .padding(.horizontal, 8)
        .frame(maxWidth: 420, minHeight: 64)
        .background(Plate.opacity(0.92), in: Capsule())
        .overlay(Capsule().stroke(Color.white.opacity(0.12), lineWidth: 1))
    }
}
