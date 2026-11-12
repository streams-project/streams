import SwiftUI

/// How many lines a column holds.
let gridLines = 8

private let columnX = [212, 612, 1015, 1425]
private let firstMiddle = 293
private let lineStep = 75

/// Words in four columns with one ticked. Language and subtitle pages share this.
///
/// `chosenWord` is what a ticked line is heard as after its own word. The tick is a picture of a
/// choice already made, and the focus ring is a picture of where the remote is: they are two
/// different things and a person listening needs both, so one is the value and the other the
/// selected trait. It is a word from `strings/` (`say.chosen`), handed in rather than read here,
/// because this part is given the words it draws and never the whole dictionary.
struct TickGrid: View {
    let words: [String]
    let ticked: Int
    let focus: Int
    var chosenWord: String = ""

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(Array(words.enumerated()), id: \.offset) { index, word in
                let x = columnX[min(index / gridLines, columnX.count - 1)]
                let middle = firstMiddle + (index % gridLines) * lineStep
                let on = index == focus
                let ink = on ? Back : Color.white
                ZStack(alignment: .leading) {
                    Capsule().fill(on ? Color.white : .clear)
                    if index == ticked { Icon("icon_check", 34, tint: ink).offset(x: 32.ref) }
                    Text(word).brand(30).foregroundStyle(ink).offset(x: 90.ref)
                }
                .frame(width: 390.ref, height: 63.ref)
                .saysItsWords(state: index == ticked ? chosenWord : "")
                .accessibilityAddTraits(index == focus ? [.isSelected] : [])
                .offset(x: CGFloat(x - 90).ref, y: CGFloat(middle - 31).ref)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
