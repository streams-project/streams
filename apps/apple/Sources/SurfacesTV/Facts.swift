import Contract
import SwiftUI

private func facts(_ detail: TitleDetail, _ words: [String: String]) -> [String] {
    var out: [String] = []
    if let genre = detail.genres?.first { out.append(genre) }
    if let year = detail.year { out.append(String(year)) }
    let seasons = detail.seasons?.count ?? 0
    if seasons > 0 {
        let word = words[seasons == 1 ? "title.season.one" : "title.season.many"] ?? ""
        out.append(word.replacingOccurrences(of: "{n}", with: String(seasons)))
    } else if let minutes = detail.runtimeMin {
        out.append("\(minutes / 60)h \(minutes % 60)m")
    }
    if let age = detail.age { out.append(age.label) }
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
