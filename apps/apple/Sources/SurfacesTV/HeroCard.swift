import Contract
import Ports
import SwiftUI

private let side = LinearGradient(
    stops: [.init(color: .black.opacity(0.8), location: 0), .init(color: .clear, location: 0.62)],
    startPoint: .leading, endPoint: .trailing
)
private let foot = LinearGradient(
    stops: [.init(color: .clear, location: 0.55), .init(color: .black.opacity(0.55), location: 1)],
    startPoint: .top, endPoint: .bottom
)

/// A logo at its own shape, no larger than its box, sitting on the box's bottom left.
struct Logo: View {
    let url: String
    let width: CGFloat
    let height: CGFloat

    var body: some View {
        if let kept = Pictures.now(url) {
            shaped(kept)
        } else if let address = URL(string: url) {
            AsyncImage(url: address) { phase in
                if case .success(let image) = phase {
                    shaped(image).onAppear { Pictures.keep(url, image) }
                }
            }
        }
    }

    private func shaped(_ image: SwiftUI.Image) -> some View {
        image.resizable()
            .aspectRatio(contentMode: .fit)
            .frame(maxWidth: width.ref, maxHeight: height.ref, alignment: .bottomLeading)
    }
}

/// The rounded card at the top of a page.
struct HeroCard: View {
    let shell: Shell
    let seen: Seen
    let store: DetailStore
    let lead: Tile
    let button: Int
    @State private var detail: TitleDetail?

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: 24.ref)
        let logoUrl = logo((lead.images ?? []) + (detail?.images ?? []))
        ZStack(alignment: .bottomLeading) {
            Plate
            Art(wide(lead.images))
            side
            foot
            VStack(alignment: .leading, spacing: 0) {
                if let logoUrl {
                    Logo(url: logoUrl, width: 600, height: 200)
                } else {
                    Text(lead.name).brand(76, 800).lines(76, 80).foregroundStyle(.white)
                }
                Spacer().frame(height: 16.ref)
                if let detail { Facts(detail, seen.words) }
                Spacer().frame(height: 16.ref)
                Text(detail?.overview ?? "")
                    .brand(30)
                    .lines(30, 39)
                    .foregroundStyle(.white.opacity(0.8))
                    .lineLimit(3)
                    .frame(width: 760.ref, alignment: .leading)
                Spacer().frame(height: 30.ref)
                HStack(spacing: 16.ref) {
                    Pill(seen.say("common.play"), on: button == 0, icon: "icon_play")
                    Pill(seen.say("common.more_info"), on: button == 1)
                }
            }
            .frame(width: 760.ref, alignment: .leading)
            .padding(.leading, 30.ref)
            .padding(.bottom, 34.ref)
        }
        .frame(width: 1789.ref, height: 767.ref)
        .clipShape(shape)
        .overlay(shape.stroke(.white.opacity(0.28), lineWidth: 2.ref))
        .offset(x: 75.ref, y: 143.ref)
        .task(id: lead.title) { detail = await detailOf(shell, store, lead.title) }
    }
}
