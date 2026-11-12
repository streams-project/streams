import SwiftUI

/// The title and one line of help at the top left of a settings style page.
struct PageHead: View {
    let title: String
    let help: String

    var body: some View {
        ZStack(alignment: .topLeading) {
            // The page's title is a heading, so a screen reader can jump to it. The line under it
            // is the page's own words and is read as it stands.
            Text(title).brand(44, 700).foregroundStyle(.white).offset(x: subEdge.ref, y: 74.ref)
                .accessibilityAddTraits(.isHeader)
            Text(help).brand(30).foregroundStyle(.white).offset(x: subEdge.ref, y: 148.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
