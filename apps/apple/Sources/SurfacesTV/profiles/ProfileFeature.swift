import Contract
import Ports
import SwiftUI

private let settle: UInt64 = 300
private let fade = 0.5

private let fromLeft = LinearGradient(
    stops: [.init(color: .black, location: 0), .init(color: .clear, location: 0.45)],
    startPoint: .leading, endPoint: .trailing
)
private let fromFoot = LinearGradient(
    stops: [.init(color: .clear, location: 0.6), .init(color: .black.opacity(0.7), location: 1)],
    startPoint: .top, endPoint: .bottom
)

/// A title chosen for the profile in focus, crossfading as the focus moves.
struct ProfileFeature: View {
    let shell: Shell
    let id: String
    @State private var kept: [String: Tile] = [:]
    @State private var shown: Tile?

    var body: some View {
        ZStack(alignment: .topLeading) {
            if let shown {
                ZStack(alignment: .bottomTrailing) {
                    ZStack {
                        Art(shell.pictures, wide(shown.images))
                        fromLeft
                        fromFoot
                    }
                    .frame(width: 1440.ref, height: 1080.ref)
                    .offset(x: 480.ref)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                    VStack(alignment: .trailing, spacing: 10.ref) {
                        Text(shown.name).brand(56, 800).foregroundStyle(.white)
                            .multilineTextAlignment(.trailing).lineLimit(2)
                            .frame(width: 900.ref, alignment: .trailing)
                        Text((shown.tags ?? []).joined(separator: " · ")).brand(26).foregroundStyle(.white.opacity(0.85))
                    }
                    .padding(.trailing, 64.ref)
                    .padding(.bottom, 52.ref)
                }
                .id(shown.title)
                .transition(.opacity)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .task(id: id) {
            if let tile = kept[id] {
                withAnimation(.linear(duration: fade)) { shown = tile }
                return
            }
            try? await Task.sleep(nanoseconds: settle * 1_000_000)
            if Task.isCancelled { return }
            guard let text = await shell.read("/v1/profiles/\(id)/feature"),
                  let tile = try? JSONDecoder().decode(Tile.self, from: Data(text.utf8)) else { return }
            kept[id] = tile
            if !Task.isCancelled { withAnimation(.linear(duration: fade)) { shown = tile } }
        }
    }
}
