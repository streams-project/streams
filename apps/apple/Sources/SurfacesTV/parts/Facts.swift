import Contract
import SwiftUI

private func formatLength(_ words: [String: String], _ minutes: Int) -> String {
    let h = minutes / 60
    let m = minutes % 60
    if h > 0 && m > 0 {
        let hText = words.counted("title.length.hours", h)
        let mText = words.counted("title.length.minutes", m)
        let pattern = words["title.length.both"] ?? "{hours} {minutes}"
        return pattern.replacingOccurrences(of: "{hours}", with: hText).replacingOccurrences(of: "{minutes}", with: mText)
    } else if h > 0 {
        return words.counted("title.length.hours", h)
    } else {
        return words.counted("title.length.minutes", m)
    }
}

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
        let translated = words["age.\(age.label)"] ?? number(age.label)
        out.append(translated)
    }
    return out
}

/// The line of facts about a title, in one order on every screen: genre, year, length, age.
public struct Facts: View {
    public let detail: TitleDetail
    public let words: [String: String]

    public init(_ detail: TitleDetail, _ words: [String: String]) {
        self.detail = detail
        self.words = words
    }

    public var body: some View {
        HStack(spacing: 0) {
            ForEach(Array(facts(detail, words).enumerated()), id: \.offset) { index, word in
                if index > 0 {
                    Circle().fill(Muted).frame(width: 6.ref, height: 6.ref).padding(.horizontal, 14.ref)
                }
                Text(word).brand(24, 700).foregroundStyle(.white)
            }
        }
    }
}
