import Contract
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
/// direction, so it turns with a right to left language.
struct BottomBar: View {
    let items: [BarItem]

    var body: some View {
        HStack(spacing: 0) {
            ForEach(items) { item in
                Button(action: item.go) {
                    VStack(spacing: 2) {
                        if let face = item.face {
                            BarFace(person: face, server: item.server)
                        } else {
                            Icon(item.icon, 24, tint: item.here ? .white : idleInk)
                        }
                        Text(item.word)
                            .brand(11, 400)
                            .foregroundStyle(item.here ? .white : idleInk)
                            .lineLimit(1)
                    }
                    .frame(width: 76, height: 48)
                    .background(item.here ? chosenFill : Color.clear, in: Capsule())
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 7)
        .frame(height: 63)
        .background(barFill, in: Capsule())
        .overlay(Capsule().stroke(Color.white.opacity(0.10), lineWidth: 1))
    }
}

/// The profile's face at the bar's icon size: its picture, or its first letter on its colour.
private struct BarFace: View {
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
                Art(fullUrl)
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
    }
}
