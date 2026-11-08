import Contract
import SwiftUI

/// A profile's picture, or its first letter when it has none. Every screen draws this one part.
public struct Face: View {
    public let person: Profile?
    public let size: CGFloat
    public let server: String

    public init(_ person: Profile?, _ size: CGFloat, _ server: String) {
        self.person = person
        self.size = size
        self.server = server
    }

    public var body: some View {
        let avatar = person?.avatar.flatMap { $0.hasPrefix("/") || $0.hasPrefix("http") ? $0 : nil }
        let plate = person?.kids == true ? Join : Accent
        ZStack {
            plate
            if let avatar {
                Art(avatar.hasPrefix("/") ? server + avatar : avatar)
            } else {
                Text((person?.name.prefix(1) ?? "").uppercased())
                    .brand(size * 46 / 100, 800)
                    .foregroundStyle(.white)
                    .lineLimit(1)
            }
        }
        .frame(width: size.ref, height: size.ref)
        .clipShape(shapeOf(Round.face(size)))
    }
}
