import Contract
import SwiftUI

private func marked(_ line: HelpLine, _ size: CGFloat = 30, _ weight: Int = 400) -> AttributedString {
    var out = AttributedString()
    for run in line.runs {
        var part = AttributedString(run.text)
        part.font = BrandFont.of(size, run.bold == true ? 700 : weight)
        out += part
    }
    return out
}

private struct Words: View {
    let line: HelpLine

    var body: some View {
        Text(marked(line)).lines(30, 44).foregroundStyle(.white.opacity(0.9)).fixedSize(horizontal: false, vertical: true)
    }
}

private struct Marked: View {
    let lines: [HelpLine]
    let mark: (Int) -> String

    var body: some View {
        VStack(alignment: .leading, spacing: 12.ref) {
            ForEach(Array(lines.enumerated()), id: \.offset) { index, line in
                HStack(alignment: .firstTextBaseline, spacing: 0) {
                    Text(mark(index)).brand(30, 700).foregroundStyle(.white).frame(width: 52.ref, alignment: .leading)
                    Words(line: line)
                }
            }
        }
    }
}

private struct Block: View {
    let block: HelpBlock

    var body: some View {
        let lines = block.lines ?? []
        switch block.kind {
        case .heading:
            Text(lines.first.map { marked($0, 36, 700) } ?? AttributedString()).foregroundStyle(.white)
        case .steps: Marked(lines: lines) { "\($0 + 1)." }
        case .list: Marked(lines: lines) { _ in "•" }
        case .note:
            if let first = lines.first {
                Words(line: first)
                    .padding(24.ref)
                    .background(Color.white.opacity(0.1), in: RoundedRectangle(cornerRadius: 12.ref))
            }
        case .paragraph:
            if let first = lines.first { Words(line: first) }
        // The actions are for apps that have them, and folds stay open on a television.
        default: EmptyView()
        }
    }
}

/// The sections of an article or policy, as every family receives them from the server.
struct HelpBody: View {
    let sections: [HelpSection]

    var body: some View {
        VStack(alignment: .leading, spacing: 28.ref) {
            ForEach(Array(sections.enumerated()), id: \.offset) { _, section in
                if let title = section.title { Text(title).brand(36, 700).foregroundStyle(.white) }
                ForEach(Array(section.blocks.enumerated()), id: \.offset) { _, block in Block(block: block) }
            }
        }
    }
}
