import Contract
import SwiftUI

private let large: CGFloat = 300
private let small: CGFloat = 122

/// The profile large on the right of a page, with what is being edited under it.
struct ProfileCard<Name: View>: View {
    let person: Profile
    let server: String
    @ViewBuilder let name: () -> Name
    let under: String?

    var body: some View {
        ZStack(alignment: .topLeading) {
            Face(person, large, server).offset(x: 1237.ref, y: 336.ref)
            name().frame(width: 600.ref).offset(x: 1085.ref, y: 676.ref)
            if let under {
                Text(under).brand(30).foregroundStyle(.white).multilineTextAlignment(.center)
                    .frame(width: 600.ref).offset(x: 1085.ref, y: 748.ref)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

/// A profile's name as the card shows it.
struct CardName: View {
    let name: String

    var body: some View {
        Text(name).brand(40, 500).foregroundStyle(.white).lineLimit(1)
    }
}

/// The profile small at the top right, when the page needs the room.
struct ProfileCorner: View {
    let person: Profile
    let server: String

    var body: some View {
        HStack(spacing: 30.ref) {
            Text(person.name).brand(34, 500).foregroundStyle(.white).lineLimit(1)
                .frame(maxWidth: .infinity, alignment: .trailing)
            Face(person, small, server)
        }
        .frame(width: 702.ref)
        .offset(x: 1085.ref, y: 70.ref)
    }
}
