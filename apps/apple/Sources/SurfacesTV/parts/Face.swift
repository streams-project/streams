import Contract
import Ports
import SwiftUI

/// A profile's picture, or its first letter when it has none. Every screen draws this one part.
///
/// `says` is the words for a face that stands on its own, with no name drawn beside it: the
/// person's name, which is already in the viewer's language because they typed it. A face that sits
/// next to its own name leaves this out, so a screen reader does not say the name twice. The
/// default is decoration, because the letter standing in for a missing picture is not a word - a
/// reader would say "N" and stop - and because a face is almost always drawn inside a control that
/// already says whose it is. Mirrors `says` on android's `tv/parts/Face.kt` and `shared/Face.kt`.
public struct Face: View {
    public let pictures: Pictures
    public let person: Profile?
    public let size: CGFloat
    public let server: String
    public var says: String = ""

    public init(
        _ pictures: Pictures,
        _ person: Profile?,
        _ size: CGFloat,
        _ server: String,
        says: String = ""
    ) {
        self.pictures = pictures
        self.person = person
        self.size = size
        self.server = server
        self.says = says
    }

    public var body: some View {
        let avatar = person?.avatar.flatMap { $0.hasPrefix("/") || $0.hasPrefix("http") ? $0 : nil }
        let plate = person?.kids == true ? Join : Accent
        ZStack {
            plate
            if let avatar {
                Art(pictures, avatar.hasPrefix("/") ? server + avatar : avatar)
            } else {
                Text((person?.name.prefix(1) ?? "").uppercased())
                    .brand(size * 46 / 100, 800)
                    .foregroundStyle(.white)
                    .lineLimit(1)
            }
        }
        .frame(width: size.ref, height: size.ref)
        .clipShape(shapeOf(Round.face(size)))
        .modifier(FaceSays(words: says))
    }
}

/// A face's words, or the face gone from the tree. Not `says(_:)`, because a face is not a button:
/// it is drawn inside one, or beside the name that is.
private struct FaceSays: ViewModifier {
    let words: String

    func body(content: Content) -> some View {
        if words.isEmpty {
            content.decoration()
        } else {
            content.accessibilityElement(children: .ignore).accessibilityLabel(words)
        }
    }
}
