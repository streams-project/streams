package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.Section
import contract.SectionKind
import contract.Tile
import streams.shared.Accent
import streams.shared.Art
import streams.shared.Corner
import streams.shared.Raised
import streams.shared.fill
import streams.shared.tall

/** The rows a page shows, in the server's order. */
val RowKinds = setOf(SectionKind.ROW, SectionKind.RANKED, SectionKind.CONTINUE, SectionKind.GRID)

/** Tiles on the reference: standard 119 x 166.6 dp (5 : 7), the first row's tall panels 184.6 x 369.2. */
private val TileWide = 119.dp
private val TileHigh = 166.6.dp
private val TallWide = 184.6.dp
private val TallHigh = 369.2.dp

/** Between tiles, and the row's start from the screen's edge (reference: 5.9 and 12.1 dp). */
private val TileGap = 6.dp
private val RowStart = 12.dp

/**
 * One row: its name over a sideways list of tiles (reference: the name 34.3 dp high, bold, 18 sp; 4 dp
 * to the tiles; 12 dp to the next row). The page's first row shows tall panels, as the reference's.
 */
@Composable
fun SectionRow(section: Section, words: Map<String, String>, tallTiles: Boolean, open: (Tile) -> Unit) {
    val tiles = section.tiles.orEmpty()
    if (tiles.isEmpty()) return
    Column(Modifier.fillMaxWidth().padding(bottom = 12.dp)) {
        Box(Modifier.height(34.dp).padding(start = RowStart), contentAlignment = Alignment.CenterStart) {
            Text(section.title.orEmpty(), color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.Bold, maxLines = 1)
        }
        Spacer(Modifier.height(4.dp))
        LazyRow(
            contentPadding = PaddingValues(horizontal = RowStart),
            horizontalArrangement = Arrangement.spacedBy(TileGap),
        ) {
            items(tiles, key = { it.title.value }) { tile ->
                val rank = tile.badges.orEmpty().firstOrNull { it.rank != null }?.rank
                TileCard(
                    tile,
                    if (tallTiles) TallWide else TileWide,
                    if (tallTiles) TallHigh else TileHigh,
                    rank = if (section.kind == SectionKind.RANKED) rank else null,
                    percent = if (section.kind == SectionKind.CONTINUE) tile.percent else null,
                    words = words,
                ) { open(tile) }
            }
        }
    }
}

/**
 * One tile: the title's tall picture, rounded by the brand (reference about 6 dp). A ranked row marks
 * the rank in the top corner and Continue Watching shows how far the viewer is, both in our blue
 * where the reference uses red.
 */
@Composable
private fun TileCard(
    tile: Tile,
    wide: Dp,
    high: Dp,
    rank: Long?,
    percent: Long?,
    words: Map<String, String>,
    open: () -> Unit,
) {
    val shape = RoundedCornerShape(Corner.CARD.dp)
    Box(Modifier.size(wide, high).clip(shape).background(Raised).clickable(onClick = open)) {
        Art(tall(tile.images), Modifier.fillMaxSize())
        if (rank != null) {
            Text(
                words["badge.ranked"].orEmpty().fill("rank", rank),
                Modifier
                    .padding(6.dp)
                    .background(Accent, RoundedCornerShape(6.dp))
                    .padding(horizontal = 6.dp, vertical = 2.dp),
                color = Color.White,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
            )
        }
        if (percent != null) {
            Box(
                Modifier
                    .align(Alignment.BottomStart)
                    .fillMaxWidth()
                    .height(3.dp)
                    .background(Color.White.copy(alpha = 0.3f)),
            ) {
                Box(Modifier.fillMaxWidth(percent.coerceIn(0L, 100L) / 100f).height(3.dp).background(Accent))
            }
        }
    }
}
