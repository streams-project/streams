import SwiftUI

public struct NewBadge: View {
    public init() {}

    public var body: some View {
        Text("NEW")
            .font(.system(size: 20.refType, weight: .heavy))
            .foregroundStyle(.white)
            .padding(.horizontal, 12.ref)
            .padding(.vertical, 4.ref)
            .background(Color(red: 229/255.0, green: 9/255.0, blue: 20/255.0))
            .clipShape(RoundedRectangle(cornerRadius: 4.ref))
    }
}

public struct RankedBadge: View {
    public let rank: Int

    public init(rank: Int) {
        self.rank = rank
    }

    public var body: some View {
        HStack(spacing: 4.ref) {
            Text("TOP")
                .font(.system(size: 14.refType, weight: .heavy))
            Text("\(rank)")
                .font(.system(size: 20.refType, weight: .heavy))
        }
        .foregroundStyle(.white)
        .padding(.horizontal, 10.ref)
        .padding(.vertical, 4.ref)
        .background(Color(red: 229/255.0, green: 9/255.0, blue: 20/255.0))
        .clipShape(RoundedRectangle(cornerRadius: 4.ref))
    }
}
