package streams.tv

import streams.shared.Art
import streams.shared.tall
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import contract.Tile
import streams.shared.Raised
import streams.shared.ref

/** Posters in one line of the grid. */
const val POSTERS_ACROSS = 4

private const val WIDE = 296
private const val HIGH = 414
private const val GAP_X = 10
private const val GAP_Y = 10
private const val MOVE_MS = 240

/** Moves a place in the grid. Answers the new place, or the same one at an edge. */
fun gridMove(at: Int, word: Word, last: Int): Int = when (word) {
    Word.UP -> if (at >= POSTERS_ACROSS) at - POSTERS_ACROSS else at
    Word.DOWN -> if (at + POSTERS_ACROSS <= last) at + POSTERS_ACROSS else at
    Word.LEFT -> if (at % POSTERS_ACROSS > 0) at - 1 else at
    Word.RIGHT -> if (at % POSTERS_ACROSS < POSTERS_ACROSS - 1 && at < last) at + 1 else at
    else -> at
}

/** Portrait posters, four across. `focus` is -1 when the grid has no focus. */
@Composable
fun PosterGrid(tiles: List<Tile>, focus: Int, left: Int, top: Int) {
    val line = maxOf(focus, 0) / POSTERS_ACROSS
    val step = with(LocalDensity.current) { (HIGH + GAP_Y).ref.toPx() }
    val moved by animateFloatAsState(line.toFloat(), tween(MOVE_MS, easing = Glide), label = "grid")
    val shape = shapeOf(Round.card)
    Box(Modifier.offset(0.ref, top.ref).graphicsLayer { translationY = -moved * step }) {
        tiles.forEachIndexed { index, tile ->
            val row = index / POSTERS_ACROSS
            if (row in line - 1..line + 2) {
                Box(
                    Modifier
                        .offset((left + index % POSTERS_ACROSS * (WIDE + GAP_X)).ref, (row * (HIGH + GAP_Y)).ref)
                        .size(WIDE.ref, HIGH.ref)
                        .clip(shape)
                        .background(Raised)
                        .says(tile.name, index == focus),
                ) {
                    Art(tall(tile.images), Modifier.fillMaxSize())
                    TileBadges(tile, Modifier.align(Alignment.BottomStart).padding(12.ref))
                    if (index == focus) Box(Modifier.fillMaxSize().border(4.ref, Color.White, shape))
                }
            }
        }
    }
}
