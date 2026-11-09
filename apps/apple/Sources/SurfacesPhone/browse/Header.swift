import Contract
import Ports
import SurfacesShared
import SwiftUI

/// The top bar's height (reference: 51.9 pt) and the chip row's (73.4 pt, chips 49.2 tall).
public let topBarHeight: CGFloat = 52
public let chipRowHeight: CGFloat = 73
private let chipHigh: CGFloat = 49
private let outlineHigh: CGFloat = 43

/// The top of a main page, pinned: the brand's mark and the page's name on the left, the bell on the
/// right (reference: mark box 48 x 44 at 3.9 pt, name from 51.9 pt, 44 pt buttons). On a page reached
/// from a chip (Series, Films) the mark gives way to a back arrow, as the reference's does.
struct TopBar: View {
    let server: String
    let name: String
    let back: (() -> Void)?
    let bell: (() -> Void)?
    var unreadCount: Int = 0

    var body: some View {
        HStack(spacing: 0) {
            Button {
                back?()
            } label: {
                ZStack {
                    if back != nil {
                        Icon("icon_back", 24, tint: .white)
                    } else {
                        Mark(30)
                    }
                }
                .frame(width: 48, height: 44)
            }
            .buttonStyle(.plain)
            .disabled(back == nil)
            .padding(.leading, 4)

            Text(name)
                .brand(24, 700)
                .foregroundStyle(.white)
                .lineLimit(1)
                .truncationMode(.tail)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.trailing, bell == nil ? 8 : 0)

            if let bell {
                Button(action: bell) {
                    ZStack(alignment: .topTrailing) {
                        Icon("icon_bell", 24, tint: .white)
                            .frame(width: 44, height: 44)
                        if unreadCount > 0 {
                            let countText = unreadCount > 9 ? "9+" : "\(unreadCount)"
                            Text(countText)
                                .brand(10, 700)
                                .foregroundStyle(.white)
                                .padding(.horizontal, 4)
                                .padding(.vertical, 1)
                                .background(Accent, in: Capsule())
                                .offset(x: -4, y: 4)
                        }
                    }
                }
                .buttonStyle(.plain)
                .padding(.trailing, 4)
            }
        }
        .frame(height: topBarHeight)
    }
}

/// The chips' words, the reference's grey (191, 191, 190); black on a chosen chip.
private let chipInk = Color(red: 191 / 255, green: 191 / 255, blue: 190 / 255)

/// One chip of the row: what it says, whether it is chosen, and where it goes.
struct Chip: Identifiable {
    let word: String
    let chosen: Bool
    var drops: Bool = false
    let go: () -> Void
    var id: String { word }
}

/// The chip row under the top bar, as the reference's: 49 pt chips from 16 pt in, 6 pt apart, 16 pt
/// padding, a 10 % white fill under a 1 pt edge of 10 % white, words 15 pt medium. The row reads as
/// one group: the first chip's outer side and the last chip's are round (brand.json
/// `chip_end_radius`), every other corner takes `chip_radius`. Start and end follow the reading
/// direction, so the group mirrors in Arabic and Hebrew.
struct ChipRow: View {
    @Environment(\.layoutDirection) private var layoutDirection
    let chips: [Chip]
    let shown: CGFloat

    var body: some View {
        let outline = chips.count == 1 && chips[0].drops
        let high = outline ? outlineHigh : chipHigh
        let inner = Corner.chip
        let outer = min(Corner.chipEnd, high / 2)
        let isRTL = layoutDirection == .rightToLeft

        ScrollView(.horizontal, showsIndicators: false) {
            LazyHStack(spacing: 6) {
                ForEach(Array(chips.enumerated()), id: \.element.id) { index, chip in
                    let first = index == 0
                    let last = index == chips.count - 1
                    let leadRadius = outline ? inner : (isRTL ? (last ? outer : inner) : (first ? outer : inner))
                    let trailRadius = outline ? inner : (isRTL ? (first ? outer : inner) : (last ? outer : inner))
                    let shape = UnevenRoundedRectangle(
                        topLeadingRadius: leadRadius,
                        bottomLeadingRadius: leadRadius,
                        bottomTrailingRadius: trailRadius,
                        topTrailingRadius: trailRadius
                    )
                    let fill = chip.chosen ? Color.white : outline ? Color.clear : Color.white.opacity(0.10)
                    let edge = chip.chosen ? Color.clear : outline ? Color.white.opacity(0.40) : Color.white.opacity(0.10)
                    let ink = chip.chosen ? Color.black : chipInk

                    Button(action: chip.go) {
                        HStack(spacing: 4) {
                            Text(chip.word)
                                .brand(15, 500)
                                .foregroundStyle(ink)
                                .lineLimit(1)
                            if chip.drops {
                                Icon("icon_caret_down", 18, tint: ink)
                            }
                        }
                        .fixedSize(horizontal: true, vertical: false)
                        .padding(.horizontal, 16)
                        .frame(height: high)
                        .background(fill, in: shape)
                        .overlay(shape.stroke(edge, lineWidth: 1))
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 16)
        }
        .frame(height: chipRowHeight)
        .opacity(Double(shown))
    }
}

/// The header block: top bar and chip row together.
struct Header: View {
    let server: String
    let name: String
    let back: (() -> Void)?
    let bell: (() -> Void)?
    var unreadCount: Int = 0
    let chips: [Chip]
    let chipShift: CGFloat

    var body: some View {
        VStack(spacing: 0) {
            TopBar(server: server, name: name, back: back, bell: bell, unreadCount: unreadCount)
            ChipRow(chips: chips, shown: 1.0 - chipShift)
                .frame(height: chipRowHeight * max(0, 1.0 - chipShift))
                .clipped()
                .offset(y: -chipShift * chipRowHeight)
        }
    }
}
