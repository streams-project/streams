import Contract
import Ports
import SwiftUI
import UIKit

/// Watching a film in a window in space.
///
/// ## What this platform is better at, and what this file does about it
///
/// A television plays a film on a panel whose size the room decided years ago. A phone plays it on
/// something the size of a hand. Here the screen is as large as the viewer wants and the room can
/// be taken out of the picture, which is the one thing this hardware does that neither of the
/// others can do at all. Not using it would be the whole point of the platform left on the table.
///
/// **The decision, and the two alternatives rejected.** Playback happens in a window that the
/// viewer scales, and a film is additionally *offered* a dimmed immersive space. Not a volume: a
/// volume is for bounded three-dimensional content that is walked around and looked into, and a
/// flat film in one gains nothing while losing the ability to be scaled past the volume's own
/// bounds -- a film is a rectangle, and a rectangle wants a window or a screen, not a box. And not
/// an immersive space by default: taking somebody's room away without being asked is the wrong
/// default for a person half-watching something with a cup of tea, and it is exactly the kind of
/// decision that should be the viewer's. So: a window that grows, and an offer.
///
/// **What is built here and what is not.** The window and its controls are in this file. The
/// immersive space is *not* built, and that is deliberate rather than unfinished: an
/// `ImmersiveSpace` cannot be laid out, drawn, entered or left by anything on this machine -- there
/// is no visionOS simulator and no device -- so shipping one would mean shipping a screen that has
/// never been drawn anywhere, with no way to find out. The README carries it as the first thing a
/// device has to settle. What this file does is leave the seam where it goes.
///
/// ## Why every control here is a discrete press
///
/// There is no scrub-by-drag. That is an accessibility decision and it is the reason the bar below
/// is a readout rather than a slider. A drag needs a pinch held while the hand moves, and the
/// people most likely to be using this platform's other input methods are the people least able to
/// do that: Dwell Control activates a target by resting on it and cannot hold-and-move at all, and
/// Pointer Control aims with a head or a wrist, where a held drag is a tremor away from a seek to
/// the wrong place. Both exist because not everyone can pinch reliably or aim their eyes.
///
/// So moving through a film is done with presses: Start Over, and the next episode. Ten-second
/// steps belong here too and are **not** built, because they would need words and `strings/` has
/// no key for them -- `icon_back_10` and `icon_forward_10` are in the assets with nothing to call
/// them. Inventing English in Swift is the one thing this family's `Says.swift` refuses. The two
/// keys that would unblock it are named in the report.
///
/// Everything below is a plain `Button`, which is the other half of the same decision: a standard
/// control is driven by VoiceOver's activate, by Dwell, by Pointer Control and by a hardware
/// keyboard without this file knowing any of them exist. A custom gesture recogniser would have
/// worked for eye-and-pinch and for nothing else.

/// The picture of what plays, from the player port. Nothing here knows what decodes it.
private struct Picture: UIViewRepresentable {
    let player: Player

    func makeUIView(context: Context) -> UIView {
        player.picture(context) as? UIView ?? UIView()
    }

    func updateUIView(_ view: UIView, context: Context) {}
}

/// A time as a clock reads it, in the interface language's digits.
func clock(_ ms: Int64) -> String {
    let seconds = max(0, ms / 1000)
    if seconds >= 3600 {
        return number(String(format: "%d:%02d:%02d", seconds / 3600, seconds / 60 % 60, seconds % 60))
    }
    return number(String(format: "%d:%02d", seconds / 60 % 60, seconds % 60))
}

/// How far through the film it is: drawn as a bar, and said as one control's value.
///
/// The bar carries no height of its own, and that is the result of a mutation rather than an
/// oversight. It had `.frame(height: Reach.target)` on the reader below; removing that on
/// 2026-11-16 made no test fail, which asked the question the method exists for -- dead line, or
/// blind test? It was the dead line. `reachable` already forces the whole control to the minimum
/// and a `GeometryReader` fills what it is given, so the line restated the minimum in a second
/// place. The same redundancy was removed from the navigation bar's tabs and icons earlier, for the
/// same reason: the minimum has one home (`app/Reach.swift`), and a second copy is a number that
/// can drift from the one that matters.
///
/// Named `PositionBar` and not `Progress` so it cannot be confused with the thin bar drawn inside a
/// tile in `browse/Rows.swift`, which means a different thing in a different place.
///
/// The bar is the one thing on a player a person listening cannot read any other way -- the two
/// clocks are text, and the bar between them holds the whole of the information and says nothing.
/// So it is one element, named by what pressing it does and valued by where the film is, with both
/// words from `strings/`. The same split the television's player makes, for the same reason.
struct PositionBar: SwiftUI.View {
    let words: [String: String]
    let atMs: Int64
    let lengthMs: Int64
    let playing: Bool
    let flip: () -> Void

    var body: some SwiftUI.View {
        let share = lengthMs > 0 ? min(1, max(0, CGFloat(atMs) / CGFloat(lengthMs))) : 0
        let said = lengthMs > 0
            ? (words["say.position"] ?? "").fill("at", clock(atMs)).fill("length", clock(lengthMs))
            : clock(atMs)
        return Button(action: flip) {
            HStack(spacing: 12) {
                Text(clock(atMs)).brand(13, 600).foregroundStyle(InkSoft).monospacedDigit()
                GeometryReader { space in
                    ZStack(alignment: .leading) {
                        Capsule().fill(InkSoft.opacity(0.35))
                        Capsule().fill(Accent).frame(width: space.size.width * share)
                    }
                    .frame(height: 6)
                    .frame(maxHeight: .infinity)
                }
                Text(lengthMs > 0 ? clock(lengthMs) : "").brand(13, 600).foregroundStyle(InkSoft).monospacedDigit()
            }
        }
        .buttonStyle(.plain)
        .reachable(
            says: words[playing ? "say.pause" : "common.play"] ?? "",
            state: said,
            corner: Round.field
        )
    }
}

/// The player: the picture, and the presses that drive it.
public struct Watch: SwiftUI.View {
    let shell: Shell
    let seen: Seen
    @State private var playing = true

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some SwiftUI.View {
        let words = seen.words
        return ZStack {
            Picture(player: shell.player)
                // The picture is what is being watched and names nothing: the title is named on the
                // page that opened it and by the controls below, and a reader stopping on the film
                // itself would hear the same words again with nothing to do about them.
                .accessibilityHidden(true)
            VStack {
                Spacer(minLength: 0)
                controls(words)
                    .padding(20)
                    .plate(Round.panel)
                    // Forward of the picture, so the controls read as being in front of the film
                    // rather than printed on it. On a television this is a gradient scrim; here it
                    // is the z axis, and it costs none of the picture.
                    .forward(Depth.controls)
                    .padding(24)
            }
        }
    }

    private func controls(_ words: [String: String]) -> some SwiftUI.View {
        VStack(spacing: 14) {
            PositionBar(
                words: words, atMs: seen.positionMs, lengthMs: seen.lengthMs, playing: playing,
                flip: flip
            )
            HStack(spacing: 12) {
                Button { shell.back() } label: {
                    WordButton(word: words["player.back"] ?? "", icon: "icon_back", dark: false)
                }
                .buttonStyle(.plain)
                .reachable(corner: Round.field)
                Button { shell.player.command(.seek(DriverCommandSeek(toMs: 0))) } label: {
                    WordButton(word: words["player.restart"] ?? "", icon: "icon_restart", dark: false)
                }
                .buttonStyle(.plain)
                .reachable(corner: Round.field)
                Button(action: flip) {
                    WordButton(
                        word: words[playing ? "say.pause" : "common.play"] ?? "",
                        icon: playing ? "icon_pause" : "icon_play",
                        dark: true
                    )
                }
                .buttonStyle(.plain)
                .reachable(corner: Round.field)
            }
        }
    }

    private func flip() {
        playing.toggle()
        shell.player.command(playing ? .play(DriverCommandPlay()) : .pause(DriverCommandPause()))
    }
}
