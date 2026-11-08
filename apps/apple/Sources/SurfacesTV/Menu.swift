import SwiftUI

/// One line of a menu.
struct MenuLine {
    let word: String
    var icon: String? = nil
}

/// Plain rows of words. The one in focus is a white pill, and `kept` marks the line another part of
/// the page belongs to while the focus is there. Pages share this as it stands.
struct Menu: View {
    let lines: [MenuLine]
    let focus: Int
    var width: CGFloat = 395
    var kept: Int = -1

    var body: some View {
        VStack(alignment: .leading, spacing: 6.ref) {
            ForEach(Array(lines.enumerated()), id: \.offset) { index, line in
                let on = index == focus
                let ink = on ? Back : Color.white
                ZStack(alignment: .leading) {
                    Capsule().fill(on ? Color.white : index == kept ? Color.white.opacity(0.14) : .clear)
                    if let icon = line.icon { Icon(icon, 34, tint: ink).offset(x: 33.ref) }
                    Text(line.word).brand(30).foregroundStyle(ink).offset(x: 90.ref)
                }
                .frame(width: width.ref, height: 63.ref)
            }
        }
    }
}
