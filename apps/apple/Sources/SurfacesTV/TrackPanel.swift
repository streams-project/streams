import Contract
import Observation
import Ports
import SwiftUI

/// Where the focus is in the full lists. The first subtitle line is Off.
@Observable
@MainActor
final class TrackPanelState {
    var open = false
    var column = 0
    var line = 0

    func show(_ seen: Seen) {
        open = true
        column = 0
        line = max(0, oneOfEach(seen.audio).firstIndex { $0.playing } ?? 0)
    }

    /// The panel takes every key while it is open.
    func press(_ word: Word, _ shell: Shell, _ seen: Seen, _ files: [Track], _ chooseLine: (Track?) -> Void) {
        let sounds = oneOfEach(seen.audio)
        let lines = linesOf(seen, files)
        let last = column == 0 ? sounds.count - 1 : lines.count
        switch word {
        case .back: open = false
        case .up: if line > 0 { line -= 1 }
        case .down: if line < last { line += 1 }
        case .left:
            // From the subtitles Left returns to the sounds; from the sounds it closes the panel.
            if column == 1 {
                column = 0
                line = max(0, sounds.firstIndex { $0.playing } ?? 0)
            } else {
                open = false
            }
        case .right:
            if column == 0 {
                column = 1
                line = (lines.firstIndex { $0.playing } ?? -1) + 1
            }
        case .ok:
            if column == 0 {
                if let sound = sounds[safe: line] { shell.act(.selectAudio(ActionSelectAudio(track: sound.id))) }
            } else {
                chooseLine(lines[safe: line - 1])
            }
        }
    }
}

private struct Tracks: View {
    let head: String
    let lines: [String]
    let focus: Int
    let kept: Int
    let x: CGFloat

    var body: some View {
        VStack(alignment: .leading, spacing: 24.ref) {
            Text(head).brand(40, 700).foregroundStyle(.white).padding(.leading, menuWords.ref)
            Menu(lines: lines.map { MenuLine(word: $0) }, focus: focus, width: 620, kept: kept)
        }
        .offset(x: x.ref, y: 180.ref)
    }
}

/// Every sound and every subtitle, over the picture, which goes on playing.
struct TrackPanel: View {
    let state: TrackPanelState
    let seen: Seen
    let files: [Track]

    var body: some View {
        let sounds = oneOfEach(seen.audio)
        let lines = linesOf(seen, files)
        ZStack(alignment: .topLeading) {
            Color.black.opacity(0.8)
            Tracks(
                head: seen.say("player.audio"),
                lines: sounds.map { nameOf($0, seen.language) },
                focus: state.column == 0 ? state.line : -1,
                kept: sounds.firstIndex { $0.playing } ?? -1,
                x: 160
            )
            Tracks(
                head: seen.say("player.subtitles"),
                lines: [seen.say("player.subtitles_off")] + lines.map { nameOf($0, seen.language) },
                focus: state.column == 1 ? state.line : -1,
                kept: (lines.firstIndex { $0.playing } ?? -1) + 1,
                x: 960
            )
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
