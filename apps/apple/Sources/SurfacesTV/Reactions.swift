import Contract
import SwiftUI

private struct Choice {
    let rating: Rating
    let plain: String
    let filled: String
}

private let choices = [
    Choice(rating: .down, plain: "icon_thumb_down", filled: "icon_thumb_down_filled"),
    Choice(rating: .up, plain: "icon_thumb_up", filled: "icon_thumb_up_filled"),
    Choice(rating: .love, plain: "icon_love", filled: "icon_love_filled"),
]

/// How many ratings the row offers.
let reactionCount = choices.count

/// The rating at a place in the row.
func reactionAt(_ index: Int) -> Rating { choices[index].rating }

/// Three ways to rate a title. `focus` is -1 when the row has no focus.
struct Reactions: View {
    let chosen: Rating?
    let focus: Int
    let left: CGFloat
    let top: CGFloat

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(Array(choices.enumerated()), id: \.offset) { index, choice in
                let on = index == focus
                Circle()
                    .fill(on ? Color.white : .clear)
                    .frame(width: 64.ref, height: 64.ref)
                    .overlay(Icon(chosen == choice.rating ? choice.filled : choice.plain, 38, tint: on ? Back : .white))
                    .offset(x: (left + CGFloat(index) * 90).ref, y: top.ref)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
