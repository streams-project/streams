import SwiftUI

/// The title and one line of help at the top left of a settings style page.
struct PageHead: View {
    /// Where the page's title sits, and the size it is drawn at.
    static let titleAt: CGFloat = 74
    static let titleType: CGFloat = 44

    /// Where the line under it sits, and its size.
    static let helpAt: CGFloat = 148
    static let helpType: CGFloat = 30

    /// Where the head ends and a page's own content may begin.
    ///
    /// Named here rather than worked out again by each page that lays out under a head: a page
    /// that wrote 178 of its own would keep drawing where the head used to end the day the head
    /// moved. `notifications/Spot.swift` is the first caller.
    static let under = helpAt + helpType

    let title: String
    let help: String

    var body: some View {
        ZStack(alignment: .topLeading) {
            // The page's title is a heading, so a screen reader can jump to it. The line under it
            // is the page's own words and is read as it stands.
            Text(title).brand(Self.titleType, 700).foregroundStyle(.white)
                .offset(x: subEdge.ref, y: Self.titleAt.ref)
                .accessibilityAddTraits(.isHeader)
            Text(help).brand(Self.helpType).foregroundStyle(.white)
                .offset(x: subEdge.ref, y: Self.helpAt.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
