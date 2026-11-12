import Contract
import Ports
import SurfacesShared
import SwiftUI

/// One place on the bar: its icon, its word, whether it is the page in view, and where it goes.
/// The bar shows the profile's face in place of an icon, as the reference shows the viewer's.
struct BarItem: Identifiable {
    let icon: String
    let word: String
    let here: Bool
    var face: Profile? = nil
    var server: String = ""
    let go: () -> Void
    var id: String { icon + word }
}

/// The bar's plate, the chosen item's plate and the idle words, as the reference's.
private let barFill = Color(red: 0x2D / 255, green: 0x2D / 255, blue: 0x2D / 255)
private let chosenFill = Color(red: 0x38 / 255, green: 0x38 / 255, blue: 0x38 / 255)
private let idleInk = Color(red: 0xC0 / 255, green: 0xC0 / 255, blue: 0xC0 / 255)

/// The bar at the foot of every main page, floating as the reference's does. It sits in the reading
/// direction, so it turns with a right to left language. All shapes and corners come from brand.json
/// (`Corner.button`), never hardcoded Capsule.
struct BottomBar: View {
    let pictures: Pictures
    let items: [BarItem]

    var body: some View {
        let barCorner = min(Corner.button, 32)
        let itemCorner = min(Corner.button, 24)
        let minItemWidth: CGFloat = items.count > 4 ? 58 : 66

        HStack(spacing: 0) {
            ForEach(items) { item in
                Button(action: item.go) {
                    VStack(spacing: 2) {
                        if let face = item.face {
                            BarFace(pictures: pictures, person: face, server: item.server)
                        } else {
                            Icon(item.icon, 24, tint: item.here ? .white : idleInk)
                        }
                        Text(item.word)
                            .brand(11, 400)
                            .foregroundStyle(item.here ? .white : idleInk)
                            .lineLimit(1)
                            .truncationMode(.tail)
                            .minimumScaleFactor(0.8)
                    }
                    .padding(.horizontal, 6)
                    .frame(height: 48)
                    .frame(minWidth: minItemWidth)
                    .background(item.here ? chosenFill : Color.clear, in: RoundedRectangle(cornerRadius: itemCorner))
                }
                .buttonStyle(.plain)
                .padding(.horizontal, 2)
                // Each place draws its own word under its icon, so the word is gathered rather than
                // given a second copy from a key. The face shown in place of an icon is decoration
                // (`BarFace` marks it): without that the place was heard as the profile's first
                // letter, a comma, and then its word - "I," ahead of the word for `nav.mine`
                // (measured 2026-11-12 with a profile named Ibrahim).
                //
                // `on` is where this family parts from the android one, which labels this bar with
                // the words alone. The bar draws the page in view on a plate of its own, and a plate
                // is not something a person listening can see, so the chosen place says it is chosen.
                .saysItsWords(on: item.here)
            }
        }
        .padding(.horizontal, 7)
        .frame(height: 63)
        .background(barFill, in: RoundedRectangle(cornerRadius: barCorner))
        .overlay(RoundedRectangle(cornerRadius: barCorner).stroke(Color.white.opacity(0.10), lineWidth: 1))
    }
}

/// The profile's face at the bar's icon size: its picture, or its first letter on its colour.
private struct BarFace: View {
    let pictures: Pictures
    let person: Profile
    let server: String

    var body: some View {
        let avatar = person.avatar.flatMap {
            $0.hasPrefix("/") || $0.hasPrefix("http") ? $0 : nil
        }
        let fullUrl = avatar.map { $0.hasPrefix("/") ? server + $0 : $0 }
        let backColor = person.kids ? Join : Accent

        ZStack {
            if let fullUrl {
                Art(pictures, fullUrl)
                    .frame(width: 24, height: 24)
                    .clipped()
            } else {
                Text(person.name.prefix(1).uppercased())
                    .brand(13, 800)
                    .foregroundStyle(.white)
            }
        }
        .frame(width: 24, height: 24)
        .background(backColor)
        .clipShape(RoundedRectangle(cornerRadius: 6))
        // The face stands where the other places draw an icon, and the word beside it already names
        // the place. Its fallback is the profile's first letter, which is a letter and not a word.
        .decoration()
    }
}
