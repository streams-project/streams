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
internal sealed interface Press
internal data class Letter(val text: String) : Press
internal data object Shift : Press
internal data object Marks : Press
internal data object Accents : Press
internal data object Space : Press
internal data object Erase : Press
internal data object Cancel : Press
internal data object Done : Press

internal class Cap(val press: Press, val column: Int, val span: Int, val word: String? = null, val icon: Int? = null) {
    fun holds(at: Int): Boolean = at in column until column + span
}

private val Letters = listOf("1234567890", "qwertyuiop", "asdfghjkl-", "zxcvbnm'")
private val Symbols = listOf("1234567890", "!@#$%^&*()", "_=+[]{};:\"", ",.?/\\|~")
private val Accented = listOf("àáâäãåæçèé", "êëìíîïñòóô", "õöøùúûüýßÿ", "œšžćčđğ")

private fun line(letters: String, from: Int, upper: Boolean) = letters.mapIndexed { index, letter ->
    val text = if (upper) letter.uppercase() else letter.toString()
    Cap(Letter(text), from + index, 1, text)
}

internal fun caps(page: List<String>, upper: Boolean, cancel: String, done: String): List<List<Cap>> = listOf(
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

    /**
     * Which key of a row the remote is on, as a place in that row's own list, or -1 for a row it is
     * not on. The painter's question and `press`'s question are this one question, so the key that
     * lights is the key that answers.
     */
    fun lit(atRow: Int): Int {
        val rows = caps(page, upper, "", "")
        if (atRow != row || atRow !in rows.indices) return -1
        val line = rows[atRow]
        val at = line.indexOfFirst { it.holds(column) }
        return if (at >= 0) at else line.lastIndex
    }
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
                val lit = state.lit(row)
                caps.forEachIndexed { at, cap ->
                    CapView(cap, row, at == lit)
                }
            }
        }
    }
}

/** Answers a word from the remote. `finish` gets the text on Done, or nothing on Cancel. */
fun KeyboardState.press(word: Word, cancel: String, done: String, finish: (String?) -> Unit): Boolean {
    val rows = caps(page, upper, cancel, done)
    // A row whose letters stop short of the last column has no key under a remote that came down
    // the right hand edge: `Symbols` is `,.?/\|~` and `Accented` is `œšžćčđğ`, seven keys from column 2,
    // so both end at 8 while the rows above them end at 9. `lit` holds the last key of the row
    // rather than throwing, and `Keyboard` above asks the same `lit` when it draws, so the key that
    // lights is the key that answers. Until 2026-11-13 this was `first`, which raised
    // NoSuchElementException on the eleventh press from where the keyboard opens and closed the
    // screen on a viewer mid password, on all four screens that draw one (`KeyboardReachTest`).
    val here = rows[row][lit(row)]
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
