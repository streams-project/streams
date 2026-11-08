import Contract
import Observation
import Ports
import SwiftUI

private let pollMs: UInt64 = 200
private let raisedBottom: CGFloat = 220
private let restingBottom: CGFloat = 70
/// A file that ends far from the film's length belongs to another cut.
private let shortestShare = 0.5
private let longestShare = 1.1
private let fileMark = "file:"

extension Track {
    /// A subtitle track that is a file on the server, not part of the stream.
    var isFile: Bool { id.hasPrefix(fileMark) }
}

/// Subtitle files the server offers for what is playing, and the cues of the one in use.
@Observable
@MainActor
final class SideWords {
    private var languages: [SubtitleLanguage] = []
    private(set) var chosen: String?
    private(set) var cues: [Cue] = []

    /// The server's languages, as tracks beside the stream's own.
    var tracks: [Track] {
        languages.map { Track(id: fileMark + $0.lang, label: $0.label, lang: $0.lang, selected: $0.lang == chosen) }
    }

    func load(_ shell: Shell, _ title: String, _ episode: String?) async {
        let ask = "/v1/subtitles?title=\(title)" + (episode.map { "&episode=\($0)" } ?? "")
        languages = await shell.read(ask)
            .flatMap { try? JSONDecoder().decode(SubtitleList.self, from: Data($0.utf8)).languages } ?? []
    }

    func has(_ language: String) -> Bool { languages.contains { $0.lang == language } }

    func off() {
        chosen = nil
        cues = []
    }

    /// The first file of the language that fits the film.
    func pick(_ shell: Shell, _ language: String, _ lengthMs: Int64) async {
        for file in languages.first(where: { $0.lang == language })?.files ?? [] {
            guard let body = await shell.read("/v1/subtitles/\(file.id)")
                .flatMap({ try? JSONDecoder().decode(SubtitleBody.self, from: Data($0.utf8)) }) else { continue }
            let share = lengthMs > 0 ? Double(body.lastEndMs) / Double(lengthMs) : 1
            if share >= shortestShare && share <= longestShare {
                chosen = language
                cues = body.cues
                return
            }
        }
        off()
    }
}

private func cueAt(_ cues: [Cue], _ ms: Int64) -> Cue? {
    var low = 0
    var high = cues.count - 1
    while low <= high {
        let middle = (low + high) / 2
        let cue = cues[middle]
        if ms < Int64(cue.startMs) { high = middle - 1 } else if ms >= Int64(cue.endMs) { low = middle + 1 } else { return cue }
    }
    return nil
}

/// The line of the moment, drawn over the picture.
struct SideLine: View {
    let words: SideWords
    let shell: Shell
    let raised: Bool
    @State private var line: String?

    var body: some View {
        ZStack {
            if let line {
                Text(line)
                    .brand(40).lines(40, 50)
                    .foregroundStyle(.white)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 16.ref)
                    .padding(.vertical, 6.ref)
                    .background(Color.black.opacity(0.6), in: RoundedRectangle(cornerRadius: 6.ref))
                    .frame(maxWidth: 1500.ref)
            }
        }
        .padding(.bottom, (raised ? raisedBottom : restingBottom).ref)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
        .task(id: words.cues.count) {
            while !Task.isCancelled {
                line = cueAt(words.cues, shell.player.now())?.text
                try? await Task.sleep(nanoseconds: pollMs * 1_000_000)
            }
        }
    }
}
