import SwiftUI

/// The title and one line of help at the top left of a settings style page.
struct PageHead: View {
    let title: String
    let help: String

    var body: some View {
        ZStack(alignment: .topLeading) {
            Text(title).brand(44, 700).foregroundStyle(.white).offset(x: 130.ref, y: 74.ref)
            Text(help).brand(30).foregroundStyle(.white).offset(x: 128.ref, y: 148.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
