package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import streams.shared.Back
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

/** How many lines a column holds. */
const val GRID_LINES = 8

private val ColumnX = listOf(212, 612, 1015, 1425)
private const val FIRST_MIDDLE = 293
private const val LINE = 75

/** Words in four columns with one ticked. Language and subtitle pages share this. */
@Composable
fun TickGrid(words: List<String>, ticked: Int, focus: Int) {
    words.forEachIndexed { index, word ->
        val x = ColumnX[(index / GRID_LINES).coerceAtMost(ColumnX.lastIndex)]
        val middle = FIRST_MIDDLE + (index % GRID_LINES) * LINE
        val on = index == focus
        val ink = if (on) Back else Color.White
        // OWNER: which line is ticked is drawn as a tick and is not said. Saying it needs a word
        // in strings/ for "chosen", which this surface may not invent (see the handoff list).
        Box(
            Modifier
                .offset((x - 90).ref, (middle - 31).ref)
                .width(390.ref)
                .height(63.ref)
                .background(if (on) Color.White else Color.Transparent, CircleShape)
                .saysItsWords(on),
        ) {
            if (index == ticked) {
                Icon(
                    painterResource(R.drawable.icon_check),
                    null,
                    Modifier.align(Alignment.CenterStart).offset(32.ref).size(34.ref),
                    tint = ink,
                )
            }
            Text(word, Modifier.align(Alignment.CenterStart).offset(90.ref), color = ink, fontSize = 30.refType)
        }
    }
}
