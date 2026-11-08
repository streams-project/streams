import SwiftUI

/// The row under the bar.
struct TrackStrip: View {
    let choices: [Choice]
    let focus: Int

    var body: some View {
        HStack(spacing: 14.ref) {
            ForEach(Array(choices.enumerated()), id: \.offset) { index, choice in
                let on = index == focus
                let ink = on ? Back : Color.white
                HStack(spacing: 10.ref) {
                    if choice.ticked { Icon("icon_check", 26, tint: ink) }
                    Text(choice.word).brand(24, 600).foregroundStyle(ink)
                }
                .padding(.horizontal, 24.ref)
                .frame(height: 54.ref)
                .background(on ? Color.white : Color.white.opacity(0.16), in: Capsule())
            }
        }
        .offset(x: 476.ref, y: 958.ref)
    }
}
