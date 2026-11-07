package streams.tv

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.style.TextAlign
import streams.shared.ref
import streams.shared.refType

private const val TILE = 243
private const val ACROSS = 270
private const val DOWN = 311
private const val FIRST_X = 395
private const val FIRST_Y = 260
private const val MOVE_MS = 240
private const val AHEAD = 6

/** A themed row of pictures, each with where it sits on the server. */
data class PictureRow(val label: String, val pictures: List<String>)

/** Which picture has focus. */
@Stable
class PictureFocus {
    var row by mutableIntStateOf(0)
    private val columns = mutableStateMapOf<Int, Int>()
    fun column(of: Int) = columns[of] ?: 0

    fun press(word: Word, rows: List<PictureRow>): Boolean {
        val at = column(row)
        when (word) {
            Word.UP -> if (row > 0) row -= 1
            Word.DOWN -> if (row < rows.lastIndex) row += 1
            Word.LEFT -> if (at > 0) columns[row] = at - 1
            Word.RIGHT -> if (at < rows[row].pictures.lastIndex) columns[row] = at + 1
            else -> return false
        }
        return true
    }

    fun chosen(rows: List<PictureRow>) = rows.getOrNull(row)?.pictures?.getOrNull(column(row))
}

@Composable
private fun OneRow(row: PictureRow, index: Int, focus: PictureFocus, server: String) {
    val at = focus.column(index)
    val step = with(LocalDensity.current) { ACROSS.ref.toPx() }
    val moved by animateFloatAsState(at.toFloat(), tween(MOVE_MS, easing = Glide), label = "across")
    Text(
        row.label,
        Modifier.offset(90.ref, (index * DOWN + 94).ref).width(300.ref),
        fontSize = 30.refType,
        textAlign = TextAlign.Center,
        maxLines = 2,
    )
    val shape = RoundedCornerShape(16.ref)
    for (picture in maxOf(0, at - 1)..minOf(at + AHEAD, row.pictures.lastIndex)) {
        key(row.pictures[picture]) {
            Box(
                Modifier
                    .offset(FIRST_X.ref, (index * DOWN).ref)
                    .graphicsLayer { translationX = (picture - moved) * step }
                    .size(TILE.ref)
                    .clip(shape),
            ) {
                Art(server + row.pictures[picture], Modifier.fillMaxSize())
                if (index == focus.row && picture == at) {
                    Box(Modifier.fillMaxSize().border(4.ref, Color.White, shape))
                }
            }
        }
    }
}

/** Rows of pictures by theme, each scrolling sideways, with a label at the left. */
@Composable
fun PictureRows(rows: List<PictureRow>, focus: PictureFocus, server: String) {
    val step = with(LocalDensity.current) { DOWN.ref.toPx() }
    val moved by animateFloatAsState(focus.row.toFloat(), tween(MOVE_MS, easing = Glide), label = "down")
    Box(Modifier.offset(0.ref, FIRST_Y.ref).graphicsLayer { translationY = -moved * step }) {
        rows.forEachIndexed { index, row -> OneRow(row, index, focus, server) }
    }
}
