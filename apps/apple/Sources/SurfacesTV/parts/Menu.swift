import SwiftUI

/// Where a menu's words start inside its pill. A menu with icons puts its pill's edge on the page's
/// column; a menu without puts its words there, so it is placed this far to the left.
let menuWords: CGFloat = 36

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
        // Words follow the icons when the menu has any; without icons they start near the pill's round end.
        let wordsAt: CGFloat = lines.contains { $0.icon != nil } ? 90 : menuWords
        return VStack(alignment: .leading, spacing: 6.ref) {
            ForEach(Array(lines.enumerated()), id: \.offset) { index, line in
                let on = index == focus
                let ink = on ? Back : Color.white
                ZStack(alignment: .leading) {
                    Capsule().fill(on ? Color.white : index == kept ? Color.white.opacity(0.14) : .clear)
                    if let icon = line.icon { Icon(icon, 34, tint: ink).offset(x: 33.ref) }
                    Text(line.word).brand(30).foregroundStyle(ink).offset(x: wordsAt.ref)
                }
                .frame(width: width.ref, height: 63.ref)
                // A line of a menu is one thing with one word, not a pile of pictures and text, and
                // the line the remote is on says it is the chosen one: this surface draws its focus
                // as a white pill, which a person listening cannot see.
                .saysItsWords(on: index == focus)
            }
        }
    }
}
