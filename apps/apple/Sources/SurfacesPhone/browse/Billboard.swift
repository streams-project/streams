import Contract
import CoreImage
import Ports
import SurfacesShared
import SwiftUI
import UIKit

/// The card's shape on the reference: 0.712 wide for every 1 high, 20 pt in from each side.
private let cardRatio: CGFloat = 0.712
public let cardSide: CGFloat = 20

/// The tag line's grey, as the reference's.
private let tagInk = Color(red: 0xD4 / 255, green: 0xD4 / 255, blue: 0xD1 / 255)

/// The My List button's fill over the tinted page, as the reference's.
private let secondFill = Color(red: 0x3A / 255, green: 0x3F / 255, blue: 0x47 / 255).opacity(0.9)

private let fade = LinearGradient(
    stops: [
        .init(color: .clear, location: 0.45),
        .init(color: Color.black.opacity(0.88), location: 1.0),
    ],
    startPoint: .top,
    endPoint: .bottom
)

/// The billboard: a tall card of a title's picture with its logo, a line of tags and two buttons
/// (reference: 381.7 x 536.2 pt, logo about 70 percent wide within 108 pt, tags 14 pt with the kind in
/// bold, two 48 pt buttons 16 pt apart, 16 pt in, 13.7 pt from the bottom). Our brand rounds the card
/// and corners both buttons from brand.json. `tint` gets the picture's main colour, for the page behind.
struct Billboard: View {
    let shell: Shell
    let tile: Tile
    let words: [String: String]
    let saved: Bool
    let play: () -> Void
    let list: () -> Void
    let open: () -> Void
    let tint: (Color) -> Void

    var body: some View {
        GeometryReader { proxy in
            let wide = proxy.size.width
            let cardHeight = wide / cardRatio
            let shape = RoundedRectangle(cornerRadius: Corner.panel)
            let picture = tall(tile.images)

            ZStack(alignment: .bottom) {
                if let picture {
                    Art(shell.pictures, picture)
                        .frame(width: wide, height: cardHeight)
                        .clipped()
                }

                fade.frame(width: wide, height: cardHeight)

                VStack(spacing: 0) {
                    // The title's own treatment is a picture of its name, so it is the one picture
                    // on this card that carries meaning: with no words it would be the only thing
                    // naming the title and would say nothing. Where the catalogue has no treatment
                    // the name is already drawn as text below and the picture does not arise.
                    if let logoUrl = logo(tile.images) {
                        Art(shell.pictures, logoUrl, fit: true, says: tile.name)
                            .frame(maxWidth: wide * 0.7, maxHeight: 108)
                    } else {
                        Text(tile.name)
                            .brand(34, 800)
                            .foregroundStyle(.white)
                            .multilineTextAlignment(.center)
                            .lineLimit(2)
                    }

                    Spacer().frame(height: 16)
                    Tags(tile: tile, words: words)
                    Spacer().frame(height: 14)

                    HStack(spacing: 16) {
                        WideButton(
                            word: words["common.play"] ?? "Play",
                            icon: "icon_play",
                            fill: .white,
                            ink: .black,
                            go: play
                        )
                        WideButton(
                            word: words["list.add"] ?? "My List",
                            icon: saved ? "icon_check" : "icon_plus",
                            fill: secondFill,
                            ink: .white,
                            go: list
                        )
                    }
                }
                .padding(.horizontal, 16)
                .padding(.bottom, 14)
            }
            .frame(width: wide, height: cardHeight)
            .background(Color(red: 0x16 / 255, green: 0x16 / 255, blue: 0x1D / 255))
            .clipShape(shape)
            .overlay(shape.stroke(Color.white.opacity(0.28), lineWidth: 1))
            .contentShape(shape)
            .onTapGesture(perform: open)
            // The whole card opens the title as well as its two buttons, and that tap is the one
            // affordance here a person listening cannot find: `children: .contain` leaves the two
            // buttons and the tag line reachable on their own while giving the card itself a name.
            .accessibilityElement(children: .contain)
            .accessibilityLabel(tile.name)
            .task(id: picture) {
                guard let picture else { return }
                // `pixels: true` says what this is for: the tint is read out of the artwork's own
                // pixels. The card draws the same address through `Art` just above, so both take
                // one fetch and one cache entry rather than two.
                if case .painted(let held) = await shell.pictures.picture(Want(url: picture, pixels: true)),
                   let uiImage = held as? UIImage,
                   let darkColor = extractDarkColor(from: uiImage) {
                    tint(darkColor)
                }
            }
        }
        .aspectRatio(cardRatio, contentMode: .fit)
        .padding(.horizontal, cardSide)
    }
}

/// The line of tags: the title's kind in bold, then its tags, each apart by a dot.
private struct Tags: View {
    let tile: Tile
    let words: [String: String]

    var body: some View {
        let kind = words[tile.isSeries ? "title.kind.series" : "title.kind.film"] ?? (tile.isSeries ? "Series" : "Film")
        let tags = tile.tags?.prefix(4) ?? []
        let extra = tags.isEmpty ? "" : "  •  " + tags.joined(separator: "  •  ")
        (Text(kind).font(BrandFont.of(14, 700)).foregroundColor(.white) +
         Text(extra).font(BrandFont.of(14, 400)).foregroundColor(tagInk))
            .lineLimit(1)
            .truncationMode(.tail)
            .padding(.horizontal, 8)
    }
}

/// One of the billboard's two buttons: 48 pt high, cornered by brand.json `button_radius`.
struct WideButton: View {
    let word: String
    let icon: String
    let fill: Color
    let ink: Color
    let go: () -> Void

    var body: some View {
        Button(action: go) {
            HStack(spacing: 6) {
                Icon(icon, 22, tint: ink)
                Text(word)
                    .brand(15, 600)
                    .foregroundStyle(ink)
                    .lineLimit(1)
                    .minimumScaleFactor(0.75)
            }
            .padding(.horizontal, 8)
            .frame(maxWidth: .infinity, minHeight: 48)
            .background(fill, in: RoundedRectangle(cornerRadius: min(Corner.button, 24)))
        }
        .buttonStyle(.plain)
        // The word is drawn in the button, so the button gathers it rather than being given a
        // second copy from a key of its own. The icon beside it repeats the word.
        .saysItsWords()
    }
}

/// Extract darker average tone from the lower half of an image using Core Image CIAreaAverage.
private func extractDarkColor(from image: UIImage) -> Color? {
    guard let ciImage = CIImage(image: image) else { return nil }
    let extent = ciImage.extent
    let lowerHalf = CGRect(
        x: extent.origin.x,
        y: extent.origin.y,
        width: extent.width,
        height: extent.height / 2
    )
    guard let filter = CIFilter(name: "CIAreaAverage", parameters: [
        kCIInputImageKey: ciImage,
        kCIInputExtentKey: CIVector(cgRect: lowerHalf),
    ]), let output = filter.outputImage else { return nil }

    var bitmap = [UInt8](repeating: 0, count: 4)
    let context = CIContext(options: [.workingColorSpace: NSNull()])
    context.render(
        output,
        toBitmap: &bitmap,
        rowBytes: 4,
        bounds: CGRect(x: 0, y: 0, width: 1, height: 1),
        format: .RGBA8,
        colorSpace: nil
    )

    let r = Double(bitmap[0]) / 255.0 * 0.6
    let g = Double(bitmap[1]) / 255.0 * 0.6
    let b = Double(bitmap[2]) / 255.0 * 0.6
    return Color(red: r, green: g, blue: b)
}
