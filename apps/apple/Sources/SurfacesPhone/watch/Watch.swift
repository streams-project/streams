import Contract
import Ports
import SwiftUI
import UIKit

private let controlsFor = 3.0

private func clock(_ ms: Int64) -> String {
    let seconds = max(0, ms / 1000)
    if seconds >= 3600 {
        return number(String(format: "%d:%02d:%02d", seconds / 3600, seconds / 60 % 60, seconds % 60))
    }
    return number(String(format: "%d:%02d", seconds / 60 % 60, seconds % 60))
}

/// The picture of what plays, from the player port.
private struct Picture: UIViewRepresentable {
    let shell: Shell

    func makeUIView(context: Context) -> UIView {
        shell.player.picture(context) as? UIView ?? UIView()
    }

    func updateUIView(_ view: UIView, context: Context) {}
}

/// The picture and its bar on a phone or tablet: tapping toggles play and pause, and the control
/// is named by what a tap does and valued by how far through the film it is.
public struct Watch: View {
    let shell: Shell
    let seen: Seen
    @State private var woke = 1
    @State private var shown = true

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        let playing = seen.play?.state == .playing
        let done = seen.lengthMs > 0
            ? min(max(Double(seen.positionMs) / Double(seen.lengthMs), 0), 1)
            : 0
        let position = seen.lengthMs > 0
            ? seen.say("say.position")
                .fill("at", clock(seen.positionMs))
                .fill("length", clock(seen.lengthMs))
            : clock(seen.positionMs)

        func toggle() {
            shell.act(playing ? .pause(ActionPause()) : .unpause(ActionUnpause()))
        }

        return ZStack(alignment: .bottomLeading) {
            Color.black.ignoresSafeArea()
            Picture(shell: shell).decoration()
            if shown {
                VStack(alignment: .leading, spacing: 10) {
                    Text(clock(seen.positionMs) + " / " + clock(seen.lengthMs))
                        .brand(14)
                        .foregroundStyle(.white)
                    GeometryReader { bar in
                        ZStack(alignment: .leading) {
                            Rectangle().fill(Color.white.opacity(0.33))
                            Rectangle().fill(Accent).frame(width: bar.size.width * done)
                        }
                    }
                    .frame(height: 4)
                }
                .padding(24)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .contentShape(Rectangle())
        .onTapGesture {
            woke += 1
            toggle()
        }
        .says(seen.say(playing ? "say.pause" : "common.play"), state: position)
        .environment(\.layoutDirection, .leftToRight)
        .task(id: woke) {
            shown = true
            try? await Task.sleep(nanoseconds: UInt64(controlsFor * 1e9))
            if !Task.isCancelled { shown = false }
        }
    }
}
