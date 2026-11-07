package streams.tv

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
import androidx.compose.foundation.shape.RoundedCornerShape
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
    Box(modifier.size(311.ref, 504.ref).clip(RoundedCornerShape(12.ref)).background(Raised)) {
        val url = tall(tile.images)
        if (url != null) Art(url, Modifier.fillMaxSize())
    }
}

/** The card stays still and its art crossfades. */
@Composable
private fun OpenCard(tile: Tile) {
    val shape = RoundedCornerShape(16.ref)
    Box(Modifier.offset(66.ref, 200.ref).size(904.ref, 508.ref).clip(shape).background(Raised)) {
        Crossfade(wide(tile.images), animationSpec = tween(SLIDE_MS, easing = LinearEasing), label = "art") { url ->
            Art(url, Modifier.fillMaxSize())
        }
        Box(Modifier.align(Alignment.BottomStart).fillMaxWidth().height(200.ref).background(CardFoot))
        Text(
            tile.name,
            Modifier.align(Alignment.BottomStart).padding(start = 24.ref, bottom = 16.ref).width(856.ref),
            fontSize = 44.refType,
            fontWeight = FontWeight.ExtraBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Box(Modifier.fillMaxSize().border(3.ref, Color.White, shape))
    }
}

/** The row that owns the screen: its focused title opens wide with the next ones beside it. */
@Composable
fun OpenRow(shell: Shell, words: Map<String, String>, store: DetailStore, section: Section, at: Int) {
    val tiles = section.tiles.orEmpty()
    val tile = tiles[at]
    val step = with(LocalDensity.current) { STEP.ref.toPx() }
    val moved by animateFloatAsState(at.toFloat(), tween(SLIDE_MS, easing = Glide), label = "posters")
    val detail by detailOf(shell, store, tile.title)
    val shown by animateFloatAsState(if (detail != null) 1f else 0f, tween(FADE_MS, easing = LinearEasing), label = "words")
    Box(Modifier.fillMaxSize()) {
        Text(
            section.title.orEmpty(),
            Modifier.offset(72.ref, 146.ref),
            fontSize = 32.refType,
            fontWeight = FontWeight.Bold,
        )
        if (at > 0) Poster(tiles[at - 1], Modifier.offset((-264).ref, 202.ref).alpha(0.55f))
        for (index in at + 1..minOf(at + AHEAD, tiles.lastIndex)) {
            key(tiles[index].title) {
                Poster(
                    tiles[index],
                    Modifier.offset(989.ref, 202.ref).graphicsLayer { translationX = (index - 1 - moved) * step },
                )
            }
        }
        OpenCard(tile)
        Box(Modifier.alpha(shown)) {
            detail?.let { Facts(it, words, Modifier.offset(70.ref, 728.ref)) }
            Text(
                detail?.overview.orEmpty(),
                Modifier.offset(70.ref, 768.ref).width(1000.ref),
                color = Color.White.copy(alpha = 0.8f),
                fontSize = 30.refType,
                lineHeight = 39.refType,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

/** The next row, dim, cut by the screen's edge. */
@Composable
fun NextRow(section: Section, high: Boolean) {
    Box(Modifier.fillMaxSize().alpha(0.5f)) {
        Text(
            section.title.orEmpty(),
            Modifier.offset(72.ref, (if (high) 946 else 944).ref),
            fontSize = 32.refType,
            fontWeight = FontWeight.Bold,
        )
        section.tiles.orEmpty().take(NEXT_POSTERS).forEachIndexed { index, tile ->
            Poster(tile, Modifier.offset((68 + index * STEP).ref, (if (high) 1001 else 998).ref))
        }
    }
}
