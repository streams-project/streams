package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import contract.TitleDetail
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Art
import streams.shared.Corner
import streams.shared.Plate
import streams.shared.tall
import streams.shared.titleAddress

/**
 * The More Like This tab on phones: a 3-column grid of tall portrait tiles.
 */
@Composable
fun MoreLike(shell: Shell, seen: Seen, data: TitleDetail) {
    val tiles = data.moreLikeThis.orEmpty()
    val chunked = tiles.chunked(3)

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 10.dp, vertical = 12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        chunked.forEach { rowTiles ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                rowTiles.forEach { tile ->
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .aspectRatio(119f / 166.6f)
                            .clip(RoundedCornerShape(Corner.CARD.dp))
                            .background(Plate)
                            .clickable {
                                shell.go(titleAddress(tile.title))
                            },
                    ) {
                        Art(tall(tile.images), Modifier.fillMaxSize())
                    }
                }
                repeat(3 - rowTiles.size) {
                    Spacer(modifier = Modifier.weight(1f))
                }
            }
        }
    }
}
