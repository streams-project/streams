import Contract
import Ports
import SwiftUI

/// Titles like this one, as portrait posters to the right of its name.
struct MoreLike: View {
    let shell: Shell
    let seen: Seen
    let data: TitleDetail
    let leave: () -> Void
    @State private var at = 0

    var body: some View {
        let tiles = data.moreLikeThis ?? []
        ZStack(alignment: .topLeading) {
            Color.black
            PosterGrid(tiles: tiles, focus: at, left: 566, top: 232, words: seen.words)
            // The facts follow the name, however many lines it takes.
            VStack(alignment: .leading, spacing: 20.ref) {
                Text(data.name).brand(56, 800).lines(56, 62).foregroundStyle(.white).lineLimit(3)
                Facts(data, seen.words)
            }
            .frame(width: 440.ref, alignment: .leading)
            .offset(x: pageEdge.ref, y: 70.ref)
            Text(seen.say("title.more_like")).brand(40).foregroundStyle(.white).offset(x: 566.ref, y: 150.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { word in
            switch word {
            case .ok: if let tile = tiles[safe: at] { shell.go(titleAddress(tile.title)) }
            case .back: leave()
            // The first column sits at the title's name: Left there returns to the title.
            case .left where at % postersAcross == 0: leave()
            default: at = gridMove(at, word, tiles.count - 1)
            }
            return true
        }
    }
}
