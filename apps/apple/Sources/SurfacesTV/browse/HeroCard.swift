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

/// The hero card's width: the page less its two margins. Parts scaled against the hero take it from
/// here rather than repeating the arithmetic.
let heroWide = pageWide - 2 * pageEdge

/// The box the title's own treatment is fitted into on the hero. No measurement sets it; it is this
/// surface's own, and the focused row card scales it (`Rows.swift`).
let heroMark = CGSize(width: 600, height: 200)

/// A logo at its own shape, no larger than its box, sitting on the box's bottom left.
///
/// `says` is the title's name: a treatment is a picture of a name, so where one is drawn it is the
/// only thing naming the title and it has to carry the words. Where it is left out the picture is
/// decoration, which is right for a logo drawn inside a part that already says the name.
struct Logo: View {
    let pictures: Pictures
    let url: String
    let width: CGFloat
    let height: CGFloat
    var says: String = ""

    /// The treatment this view read, and the address it is of, for the same reason `Art` keeps
    /// both: a row hands one of these views to a second title as the viewer moves along it.
    @State private var shown: SwiftUI.Image?
    @State private var shownFor: String?

    var body: some View {
        // Read in the draw, not in the task: a task runs after the first frame, and a treatment
        // the port is already holding would otherwise appear a frame after the card it sits on.
        let now = (shownFor == url ? shown : nil) ?? drawn(pictures.kept(Want(url: url)))
        return Group {
            if let now {
                shaped(now)
            }
        }
        .task(id: url) {
            if shownFor == url && shown != nil { return }
            if drawn(pictures.kept(Want(url: url))) != nil { return }
            if let picture = drawn(await pictures.picture(Want(url: url))) {
                shown = picture
                shownFor = url
            }
        }
    }

    private func shaped(_ image: SwiftUI.Image) -> some View {
        image.resizable()
            .aspectRatio(contentMode: .fit)
            .frame(maxWidth: width.ref, maxHeight: height.ref, alignment: .bottomLeading)
            .modifier(LogoSays(words: says))
    }
}

/// A treatment's words, or none, so a logo is never read out as the name of its own file.
private struct LogoSays: ViewModifier {
    let words: String

    func body(content: Content) -> some View {
        if words.isEmpty {
            content.accessibilityHidden(true)
        } else {
            content.accessibilityLabel(words)
        }
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
        let shape = shapeOf(Round.panel)
        let logoUrl = logo((lead.images ?? []) + (detail?.images ?? []))
        ZStack(alignment: .bottomLeading) {
            Plate
            Art(shell.pictures, wide(lead.images))
            side
            foot
            VStack(alignment: .leading, spacing: 0) {
                if let logoUrl {
                    Logo(pictures: shell.pictures, url: logoUrl, width: heroMark.width, height: heroMark.height, says: lead.name)
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
        .frame(width: heroWide.ref, height: 767.ref)
        .clipShape(shape)
        .overlay(shape.stroke(.white.opacity(0.28), lineWidth: 2.ref))
        .offset(x: pageEdge.ref, y: 143.ref)
        .task(id: lead.title) { detail = await detailOf(shell, store, lead.title) }
    }
}
