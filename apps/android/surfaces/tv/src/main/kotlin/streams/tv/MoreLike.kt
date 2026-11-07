package streams.tv

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import contract.TitleDetail
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Raised
import streams.shared.ref
import streams.shared.refType

private const val ACROSS = 4
private const val WIDE = 296
private const val HIGH = 414
private const val GAP_X = 16
private const val GAP_Y = 24
private const val FIRST_X = 562
private const val FIRST_Y = 230
private const val MOVE_MS = 240

/** Titles like this one, as portrait posters to the right of its name. */
@Composable
fun MoreLike(shell: Shell, seen: Seen, data: TitleDetail, leave: () -> Unit) {
    val tiles = data.moreLikeThis.orEmpty()
    var at by remember { mutableIntStateOf(0) }
    val step = with(LocalDensity.current) { (HIGH + GAP_Y).ref.toPx() }
    val moved by animateFloatAsState((at / ACROSS).toFloat(), tween(MOVE_MS, easing = Glide), label = "down")

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> if (at >= ACROSS) at -= ACROSS
            Word.DOWN -> if (at + ACROSS <= tiles.lastIndex) at += ACROSS
            Word.LEFT -> if (at % ACROSS > 0) at -= 1
            Word.RIGHT -> if (at % ACROSS < ACROSS - 1 && at < tiles.lastIndex) at += 1
            Word.OK -> shell.go(titleAddress(tiles[at].title))
            Word.BACK -> leave()
        }
        return true
    }

    val shape = RoundedCornerShape(12.ref)
    Box(Modifier.fillMaxSize().background(Color.Black).remote(::press)) {
        Box(Modifier.offset(0.ref, FIRST_Y.ref).graphicsLayer { translationY = -moved * step }) {
            tiles.forEachIndexed { index, tile ->
                val line = index / ACROSS
                if (line in at / ACROSS - 1..at / ACROSS + 2) {
                    Box(
                        Modifier
                            .offset((FIRST_X + index % ACROSS * (WIDE + GAP_X)).ref, (line * (HIGH + GAP_Y)).ref)
                            .size(WIDE.ref, HIGH.ref)
                            .clip(shape)
                            .background(Raised),
                    ) {
                        Art(tall(tile.images), Modifier.fillMaxSize())
                        if (index == at) Box(Modifier.fillMaxSize().border(4.ref, Color.White, shape))
                    }
                }
            }
        }
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
        Text(
            seen.words["title.more_like"].orEmpty(),
            Modifier.offset(FIRST_X.ref, 150.ref),
            fontSize = 40.refType,
        )
    }
}
