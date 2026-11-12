import SwiftUI

/// The row under the bar.
///
/// `chosenWord` is what the ticked choice is heard as after its own words. The tick is a picture of
/// the sound and subtitles in play; the white plate is where the remote is. They are two different
/// things and a person listening needs both. From `strings/` (`say.chosen`), handed in by the
/// screen, which is the part that holds the words.
struct TrackStrip: View {
    let choices: [Choice]
    let focus: Int
    var chosenWord: String = ""

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
                // The choice draws its own words - a language's name, or that name with subtitles -
                // so it gathers them. The tick beside them is decoration inside this one node.
                .saysItsWords(state: choice.ticked ? chosenWord : "")
                .accessibilityAddTraits(on ? [.isSelected] : [])
            }
        }
        .offset(x: 476.ref, y: 958.ref)
    }
}
