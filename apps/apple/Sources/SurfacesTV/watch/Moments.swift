import Contract
import Ports
import SwiftUI

extension Seen {
    /// The window the picture is inside now, of one of these kinds.
    @MainActor
    func windowNow(_ kinds: Set<WindowKind>) -> TimeWindow? {
        play?.windows?.first { kinds.contains($0.kind) && positionMs >= Int64($0.startMs) && positionMs < Int64($0.endMs) }
    }
}

/// The pill offered at a moment of the film: skip this part, or go on to the next episode.
struct MomentPill: View {
    let word: String
    let on: Bool
    let raised: Bool

    var body: some View {
        Text(word)
            .brand(30, 700)
            .foregroundStyle(on ? Back : .white)
            .padding(.horizontal, 40.ref)
            .frame(height: 72.ref)
            .background(on ? Color.white : Color.white.opacity(0.2), in: Capsule())
            .padding(.trailing, 124.ref)
            .padding(.bottom, (raised ? 230 : 96).ref)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomTrailing)
            // The pill draws its own word - skip this part, or go on to the next episode - so it
            // gathers it. `on` is the white plate that says Ok will take this offer rather than
            // pause, and it is the only thing that says so.
            .saysItsWords(on: on)
    }
}

/// The age rating, once, as play starts.
struct AgeCard: View {
    let label: String

    var body: some View {
        HStack(spacing: 22.ref) {
            Rectangle().fill(Accent).frame(width: 6.ref)
            Text(label).brand(44, 700).foregroundStyle(.white).shadow(color: .black, radius: 6)
        }
        .frame(height: 84.ref)
        .offset(x: 96.ref, y: 260.ref)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        // The card shows once as play starts and nothing presses it, so it is read as it stands: the
        // age rating, and the bar beside it is a line with no words in it.
        .accessibilityElement(children: .combine)
    }
}

/// Several episodes went by with nobody touching the remote. The picture waits for an answer.
struct StillAsking: View {
    let seen: Seen
    let focus: Int

    var body: some View {
        ZStack {
            Color.black.opacity(0.8)
            VStack(spacing: 48.ref) {
                // The question the two buttons under it answer, so a reader can jump to it. The
                // buttons draw their own words and carry them themselves (`parts/Pill.swift`).
                Text(seen.say("player.still")).brand(56, 700).foregroundStyle(.white)
                    .accessibilityAddTraits(.isHeader)
                HStack(spacing: 24.ref) {
                    Pill(seen.say("player.still.yes"), on: focus == 0)
                    Pill(seen.say("player.still.no"), on: focus == 1)
                }
            }
        }
    }
}
