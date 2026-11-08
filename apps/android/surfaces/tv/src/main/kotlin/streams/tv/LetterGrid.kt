package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
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

private const val ACROSS = 6
private const val STEP = 66
private const val CELL = 62
private const val LETTERS = "abcdefghijklmnopqrstuvwxyz1234567890"

/** How many lines the grid has, with space and delete above the letters. */
const val LETTER_LINES = 7

/** A place on the letter grid. Line 0 holds space and delete. */
data class LetterPlace(val line: Int = 1, val column: Int = 0) {
    fun move(word: Word): LetterPlace? = when (word) {
        Word.UP -> if (line > 0) copy(line = line - 1) else null
        Word.DOWN -> if (line < LETTER_LINES - 1) copy(line = line + 1) else null
        Word.LEFT -> when {
            line == 0 && column >= ACROSS / 2 -> copy(column = 0)
            line > 0 && column > 0 -> copy(column = column - 1)
            else -> null
        }
        Word.RIGHT -> when {
            line == 0 && column < ACROSS / 2 -> copy(column = ACROSS / 2)
            line > 0 && column < ACROSS - 1 -> copy(column = column + 1)
            else -> null
        }
        else -> null
    }

    /** Applies this key to the text. */
    fun press(text: String): String = when {
        line == 0 && column < ACROSS / 2 -> "$text "
        line == 0 -> text.dropLast(1)
        else -> text + LETTERS[(line - 1) * ACROSS + column]
    }
}

@Composable
private fun Cell(left: Int, top: Int, wide: Int, on: Boolean, content: @Composable (Color) -> Unit) {
    Box(
        Modifier
            .offset(left.ref, top.ref)
            .size(wide.ref, CELL.ref)
            .background(if (on) Color.White else Color.White.copy(alpha = 0.1f), shapeOf(Round.key)),
        contentAlignment = Alignment.Center,
    ) { content(if (on) Back else Color.White.copy(alpha = 0.75f)) }
}

/** Letters and digits, six across. `focus` is null when the grid has no focus. */
@Composable
fun LetterGrid(focus: LetterPlace?, modifier: Modifier = Modifier) {
    Box(modifier) {
        val half = ACROSS / 2 * STEP
        Cell(0, 0, half - 4, focus?.line == 0 && focus.column < ACROSS / 2) {
            Icon(painterResource(R.drawable.icon_space), null, Modifier.size(34.ref), tint = it)
        }
        Cell(half, 0, half - 4, focus?.line == 0 && focus.column >= ACROSS / 2) {
            Icon(painterResource(R.drawable.icon_erase), null, Modifier.size(34.ref), tint = it)
        }
        LETTERS.forEachIndexed { index, letter ->
            val line = index / ACROSS + 1
            val column = index % ACROSS
            Cell(column * STEP, line * STEP, CELL, focus == LetterPlace(line, column)) {
                Text(letter.toString(), color = it, fontSize = 30.refType)
            }
        }
    }
}
