package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import contract.TitleDetail
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.ref
import streams.shared.refType

/** Titles like this one, as portrait posters to the right of its name. */
@Composable
fun MoreLike(shell: Shell, seen: Seen, data: TitleDetail, leave: () -> Unit) {
    val tiles = data.moreLikeThis.orEmpty()
    var at by remember { mutableIntStateOf(0) }
    Box(
        Modifier.fillMaxSize().background(Color.Black).remote { word ->
            when (word) {
                Word.OK -> shell.go(titleAddress(tiles[at].title))
                Word.BACK -> leave()
                else -> at = gridMove(at, word, tiles.lastIndex)
            }
            true
        },
    ) {
        PosterGrid(tiles, at, 566, 232)
        Text(
            data.name,
            Modifier.offset(66.ref, 70.ref).width(440.ref),
            fontSize = 56.refType,
            lineHeight = 62.refType,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 3,
            overflow = TextOverflow.Ellipsis,
        )
        Facts(data, seen.words, Modifier.offset(70.ref, 270.ref))
        Text(seen.words["title.more_like"].orEmpty(), Modifier.offset(564.ref, 150.ref), fontSize = 40.refType)
    }
}
