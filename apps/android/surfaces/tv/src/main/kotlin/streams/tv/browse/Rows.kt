package streams.tv

import streams.shared.Art
import streams.shared.wide
import streams.shared.tall
import streams.shared.DetailStore
import streams.shared.detailOf
import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import contract.Section
import contract.Tile
import streams.ports.Shell
import streams.shared.Raised
import streams.shared.ref
import streams.shared.refType

private const val STEP = 328
private const val AHEAD = 5
private const val NEXT_POSTERS = 6
private const val SLIDE_MS = 240
private const val FADE_MS = 200

/** The curve every move on this page uses. */
val Glide = CubicBezierEasing(0.22f, 0.61f, 0.2f, 1f)

private val CardFoot = Brush.verticalGradient(0f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.7f))

@Composable
private fun Poster(tile: Tile, modifier: Modifier) {
    Box(modifier.size(311.ref, 504.ref).clip(shapeOf(Round.card)).background(Raised)) {
        val url = tall(tile.images)
        if (url != null) Art(url, Modifier.fillMaxSize())
        TileBadges(tile, Modifier.align(Alignment.BottomStart).padding(12.ref))
    }
}

/**
 * The card stays still and its art crossfades.
 *
 * The name and the chips are drawn over the art. Netflix composites them into the artwork instead,
 * which notes/netflix-reference/00-gaps.md records as gap 4 and as an open owner decision, so this
 * stays as it is until he takes it. 505652c swapped the name for the title's LOGO asset in a
 * 240 x 83 box inset 17 and 23, and moved the chips to the far corner at 20 and 30; none of those
 * five numbers is measured anywhere in notes/netflix-reference - `row_screen` in
 * raw/tv-geometry-1920x1080.json carries tile sizes, pitch, gutter and the vertical rhythm and says
 * nothing about what is drawn inside a card, and "badge chips bottom right" in that file is the Home
 * hero card - so they are removed rather than kept on an unfiled reading of a frame.
 */
@Composable
private fun OpenCard(tile: Tile) {
    val shape = shapeOf(Round.card)
    Box(Modifier.offset(PAGE_EDGE.ref, 200.ref).size(904.ref, 508.ref).clip(shape).background(Raised)) {
        Crossfade(wide(tile.images), animationSpec = tween(SLIDE_MS, easing = LinearEasing), label = "art") { url ->
            Art(url, Modifier.fillMaxSize())
        }
        Box(Modifier.align(Alignment.BottomStart).fillMaxWidth().height(200.ref).background(CardFoot))
        TileBadges(tile, Modifier.align(Alignment.BottomStart).padding(start = 24.ref, bottom = 78.ref))
        Text(
            tile.name,
            // The card is 904 wide (netflix-reference/tv/raw/tv-geometry-1920x1080.json,
            // `row_screen.focused_tile` 906 x 510, matched to 904 x 508 in 00-gaps.md gap 9), less the
            // 24 inset on each side.
            Modifier.align(Alignment.BottomStart).padding(start = 24.ref, bottom = 16.ref).width(856.ref),
            fontSize = 44.refType,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = storyStyle(),
        )
        Box(Modifier.fillMaxSize().border(3.ref, Color.White, shape))
    }
}

/** The row that owns the screen: its focused title opens wide with the next ones beside it. */
@Composable
fun OpenRow(shell: Shell, words: Map<String, String>, store: DetailStore, section: Section, at: Int) {
    keptWords = words
    val tiles = section.tiles.orEmpty()
    val tile = tiles[at.coerceAtMost(tiles.lastIndex)]
    // In a right to left language the row runs leftwards: every move along it turns.
    val step = with(LocalDensity.current) { STEP.ref.toPx() } * along()
    val moved by animateFloatAsState(at.toFloat(), tween(SLIDE_MS, easing = Glide), label = "posters")
    val detail by detailOf(shell, store, tile.title)
    val shown by animateFloatAsState(if (detail != null) 1f else 0f, tween(FADE_MS, easing = LinearEasing), label = "words")
    Box(Modifier.fillMaxSize()) {
        Text(
            section.title.orEmpty(),
            Modifier.offset(PAGE_EDGE.ref, 146.ref),
            fontSize = 32.refType,
            fontWeight = FontWeight.Bold,
        )
        if (at > 0) Poster(tiles[at - 1], Modifier.offset((-264).ref, 202.ref).alpha(0.55f))
        for (index in at + 1..minOf(at + AHEAD, tiles.lastIndex)) {
            key(tiles[index].title) {
                Poster(
                    tiles[index],
                    Modifier.offset((PAGE_EDGE + 904 + 19).ref, 202.ref).graphicsLayer { translationX = (index - 1 - moved) * step },
                )
            }
        }
        OpenCard(tile)
        Box(Modifier.alpha(shown)) {
            detail?.let { Facts(it, words, Modifier.offset(PAGE_EDGE.ref, 728.ref)) }
            Text(
                detail?.overview.orEmpty(),
                Modifier.offset(PAGE_EDGE.ref, 768.ref).width(1000.ref),
                color = Color.White.copy(alpha = 0.8f),
                fontSize = 30.refType,
                lineHeight = 39.refType,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
                style = storyStyle(),
            )
        }
    }
}

/**
 * The next row, dim, cut by the screen's edge. It shows where the viewer left it: the kept tile
 * first, the one before it at the edge, as the open row does.
 */
@Composable
fun NextRow(section: Section, high: Boolean, at: Int = 0) {
    val tiles = section.tiles.orEmpty()
    val top = if (high) 1001 else 998
    Box(Modifier.fillMaxSize().alpha(0.5f)) {
        Text(
            section.title.orEmpty(),
            Modifier.offset(PAGE_EDGE.ref, (if (high) 946 else 944).ref),
            fontSize = 32.refType,
            fontWeight = FontWeight.Bold,
        )
        if (at > 0) tiles.getOrNull(at - 1)?.let { Poster(it, Modifier.offset((PAGE_EDGE - STEP).ref, top.ref)) }
        tiles.drop(at).take(NEXT_POSTERS).forEachIndexed { index, tile ->
            Poster(tile, Modifier.offset((PAGE_EDGE + index * STEP).ref, top.ref))
        }
    }
}
