package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.Stable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import streams.shared.Back
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

private const val KEY = 63
private const val STEP = 66
private const val COLUMNS = 10
private const val BUTTONS = 5

/** What a key does. A letter key carries its text. */
private sealed interface Press
private data class Letter(val text: String) : Press
private data object Shift : Press
private data object Marks : Press
private data object Accents : Press
private data object Space : Press
private data object Erase : Press
private data object Cancel : Press
private data object Done : Press

private class Cap(val press: Press, val column: Int, val span: Int, val word: String? = null, val icon: Int? = null)

private val Letters = listOf("1234567890", "qwertyuiop", "asdfghjkl-", "zxcvbnm'")
private val Symbols = listOf("1234567890", "!@#$%^&*()", "_=+[]{};:\"", ",.?/\\|~")
private val Accented = listOf("àáâäãåæçèé", "êëìíîïñòóô", "õöøùúûüýßÿ", "œšžćčđğ")

private fun line(letters: String, from: Int, upper: Boolean) = letters.mapIndexed { index, letter ->
    val text = if (upper) letter.uppercase() else letter.toString()
    Cap(Letter(text), from + index, 1, text)
}

private fun caps(page: List<String>, upper: Boolean, cancel: String, done: String): List<List<Cap>> = listOf(
    line(page[0], 0, upper),
    line(page[1], 0, upper),
    line(page[2], 0, upper),
    listOf(Cap(Shift, 0, 2, icon = R.drawable.icon_shift)) + line(page[3], 2, upper),
    listOf(
        Cap(Marks, 0, 2, "!#$"),
        Cap(Accents, 2, 2, "àáâ"),
        Cap(Space, 4, 3, icon = R.drawable.icon_space),
        Cap(Erase, 7, 3, icon = R.drawable.icon_erase),
    ),
    listOf(Cap(Cancel, 0, BUTTONS, cancel), Cap(Done, BUTTONS, BUTTONS, done)),
)

/** Where the viewer is on the keys, and the text so far. */
@Stable
class KeyboardState(start: String) {
    var text by mutableStateOf(start)
    var page by mutableStateOf(Letters)
    var upper by mutableStateOf(false)
    var row by mutableIntStateOf(2)
    var column by mutableIntStateOf(4)
}

@Composable
private fun CapView(cap: Cap, row: Int, on: Boolean) {
    val pill = cap.press == Cancel || cap.press == Done
    val shape = if (pill) CircleShape else shapeOf(Round.key)
    val top = if (pill) 356 else row * STEP
    val high = if (pill) 62 else KEY
    val left = if (pill) (if (cap.press == Cancel) 0 else 331) else cap.column * STEP
    val wide = if (pill) 327 else cap.span * STEP - (STEP - KEY)
    val ink = if (on) Back else Color.White
    Box(
        Modifier
            .offset(left.ref, top.ref)
            .size(wide.ref, high.ref)
            .background(if (on) Color.White else Color.White.copy(alpha = 0.16f), shape),
        contentAlignment = Alignment.Center,
    ) {
        cap.word?.let {
            Text(it, color = ink, fontSize = 26.refType, fontWeight = if (pill) FontWeight.Bold else FontWeight.Medium)
        }
        cap.icon?.let { Icon(painterResource(it), null, Modifier.size(34.ref), tint = ink) }
    }
}

/** A letter grid for a remote. Search and sign in share it as it stands. */
@Composable
fun Keyboard(
    state: KeyboardState,
    cancel: String,
    done: String,
    modifier: Modifier = Modifier,
) {
    val rows = caps(state.page, state.upper, cancel, done)
    Box(
        modifier
            .size(722.ref, 476.ref)
            .background(Color.White.copy(alpha = 0.08f), shapeOf(Round.panel))
            .border(1.ref, Color.White.copy(alpha = 0.25f), shapeOf(Round.panel)),
    ) {
        Box(Modifier.offset(30.ref, 29.ref)) {
            rows.forEachIndexed { row, caps ->
                caps.forEach { cap ->
                    val on = row == state.row && state.column in cap.column until cap.column + cap.span
                    CapView(cap, row, on)
                }
            }
        }
    }
}

/** Answers a word from the remote. `finish` gets the text on Done, or nothing on Cancel. */
fun KeyboardState.press(word: Word, cancel: String, done: String, finish: (String?) -> Unit): Boolean {
    val rows = caps(page, upper, cancel, done)
    val caps = rows[row]
    val here = caps.first { column in it.column until it.column + it.span }
    when (word) {
        Word.UP -> if (row > 0) row -= 1 else return false
        Word.DOWN -> if (row < rows.lastIndex) row += 1
        Word.LEFT -> if (here.column > 0) column = here.column - 1
        Word.RIGHT -> if (here.column + here.span < COLUMNS) column = here.column + here.span
        Word.BACK -> finish(null)
        Word.OK -> when (val press = here.press) {
            is Letter -> {
                text += press.text
                upper = false
            }
            Shift -> upper = !upper
            Marks -> page = if (page === Symbols) Letters else Symbols
            Accents -> page = if (page === Accented) Letters else Accented
            Space -> text += " "
            Erase -> text = text.dropLast(1)
            Cancel -> finish(null)
            Done -> finish(text)
        }
    }
    return true
}
