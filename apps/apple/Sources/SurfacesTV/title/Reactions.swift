import Contract
import SwiftUI

/// `word` is the key of what this rating is called; the player's rating pills say the same three.
private struct Reaction {
    let rating: Rating
    let plain: String
    let filled: String
    let word: String
}

private let choices = [
    Reaction(rating: .down, plain: "icon_thumb_down", filled: "icon_thumb_down_filled", word: "player.down"),
    Reaction(rating: .up, plain: "icon_thumb_up", filled: "icon_thumb_up_filled", word: "player.up"),
    Reaction(rating: .love, plain: "icon_love", filled: "icon_love_filled", word: "player.love"),
]

/// How many ratings the row offers.
let reactionCount = choices.count

/// The rating at a place in the row.
func reactionAt(_ index: Int) -> Rating { choices[index].rating }

/// What the rating at a place in the row is called, in the viewer's language. Three faces and no
/// words at all is the clearest case on this surface of a control a person cannot hear.
func reactionWord(_ index: Int, _ words: [String: String]) -> String {
    guard choices.indices.contains(index) else { return "" }
    return words[choices[index].word] ?? ""
}

/// Three ways to rate a title. `focus` is -1 when the row has no focus.
struct Reactions: View {
    let chosen: Rating?
    let focus: Int
    let left: CGFloat
    let top: CGFloat
    var words: [String: String] = [:]

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(Array(choices.enumerated()), id: \.offset) { index, choice in
                let on = index == focus
                Circle()
                    .fill(on ? Color.white : .clear)
                    .frame(width: 64.ref, height: 64.ref)
                    .overlay(Icon(chosen == choice.rating ? choice.filled : choice.plain, 38, tint: on ? Back : .white))
                    // A thumb is a picture with no words anywhere near it, and which of the three
                    // is filled in is the rating already given. So each one is given its name, and
                    // the filled one says it is the chosen rating.
                    .says(reactionWord(index, words), on: on)
                    .accessibilityValue(chosen == choice.rating ? (words["say.chosen"] ?? "") : "")
                    .offset(x: (left + CGFloat(index) * 90).ref, y: top.ref)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
