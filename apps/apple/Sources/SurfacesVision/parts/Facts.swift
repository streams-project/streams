import Contract
import SwiftUI

/// The facts about a title, in one order on every screen: genre, year, length or seasons, age.
///
/// A third copy of the same short list, and the duplication is named rather than hidden. The
/// television has one in `SurfacesTV/parts/Facts.swift` and the phone draws the same four in its
/// own title page. This belongs in `SurfacesShared`, where the other three surfaces could all take
/// it -- and it is not there today for a mechanical reason, not a judgement: `SurfacesShared` is
/// `@_exported` into every surface, so a public `Facts` there and the television's existing public
/// `Facts` would be two of that name in the television's namespace, and resolving that means
/// editing `SurfacesTV`, which this change is not allowed to touch.
///
/// Nothing measured is duplicated by it. The order is a content decision and the sizes below are
/// this surface's own, so there is no second home for a number here -- only for a list.
private func facts(_ detail: TitleDetail, _ words: [String: String]) -> [String] {
    var out: [String] = []
    if let genre = detail.genres?.first { out.append(genre) }
    if let year = detail.year { out.append(number(year)) }
    let seasons = detail.seasons?.count ?? 0
    if seasons > 0 {
        out.append(words.counted("title.season", seasons))
    } else if let minutes = detail.runtimeMin, minutes > 0 {
        out.append(formatLength(words, minutes))
    }
    if let age = detail.age {
        out.append(words["age.\(age.label)"] ?? number(age.label))
    }
    return out
}

/// The line of facts, with a dot between each pair.
///
/// One accessibility element and not five, because a reader stopping four times to hear "2019"
/// and then "15" has heard a list rather than a sentence. The dots are drawn and never spoken: a
/// separator read aloud is the picture of the punctuation, not the punctuation.
public struct Facts: SwiftUI.View {
    public let detail: TitleDetail
    public let words: [String: String]

    public init(_ detail: TitleDetail, _ words: [String: String]) {
        self.detail = detail
        self.words = words
    }

    public var body: some SwiftUI.View {
        let said = facts(detail, words)
        return HStack(spacing: 0) {
            ForEach(Array(said.enumerated()), id: \.offset) { index, word in
                if index > 0 {
                    Circle()
                        .fill(InkSoft)
                        .frame(width: 4, height: 4)
                        .padding(.horizontal, 10)
                        .decoration()
                }
                Text(word).brand(15, 600).foregroundStyle(Ink)
            }
        }
        .accessibilityElement(children: .combine)
    }
}
