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

/** The English letters, which every profile can switch to. */
const val ENGLISH_LETTERS = "abcdefghijklmnopqrstuvwxyz1234567890"

/**
 * The keys of the grid: letters six across under a top line of space, delete and, when the
 * profile's language has letters of its own, the globe that switches between them and English.
 */
data class LetterKeys(val letters: String, val globe: Boolean, val mirrored: Boolean = false) {
    /**
     * The letter at a place, or null. A left to right alphabet inside a right to left screen fills
     * each line from the left (`mirrored`), so English still reads a to z.
     */
    fun at(line: Int, column: Int): Char? {
        if (line < 1) return null
        val index = (line - 1) * ACROSS + if (mirrored) ACROSS - 1 - column else column
        return letters.getOrNull(index)
    }

    /** Lines of the grid, the top line included. */
    val lines: Int get() = 1 + (letters.length + ACROSS - 1) / ACROSS

    /** The top line split into its keys, as column ranges: space, globe, delete. */
    val parts: List<IntRange>
        get() = if (globe) listOf(0..1, 2..3, 4..5) else listOf(0..2, 3..5)

    /** The height of the grid in reference pixels. */
    val high: Int get() = lines * STEP
}

/** What a key on the top line does. */
enum class TopKey { SPACE, GLOBE, DELETE }

/** A place on the letter grid. Line 0 holds space, the globe and delete. */
data class LetterPlace(val line: Int = 1, val column: Int = 0) {
    /** The top line key at this place, or null on a letter. */
    fun top(keys: LetterKeys): TopKey? {
        if (line != 0) return null
        val part = keys.parts.indexOfFirst { column in it }
        return when {
            part == 0 -> TopKey.SPACE
            keys.globe && part == 1 -> TopKey.GLOBE
            else -> TopKey.DELETE
        }
    }

    fun move(word: Word, keys: LetterKeys): LetterPlace? {
        fun fits(place: LetterPlace) = place.line == 0 || keys.at(place.line, place.column) != null
        val lastLine = keys.lines - 1
        // On a short last line, Down lands on its nearest letter.
        fun nearest(line: Int): LetterPlace? =
            (0 until ACROSS).map { LetterPlace(line, it) }.filter(::fits).minByOrNull { kotlin.math.abs(it.column - column) }
        return when (word) {
            Word.UP -> if (line > 0) copy(line = line - 1) else null
            Word.DOWN -> copy(line = line + 1).let { below ->
                when {
                    fits(below) -> below
                    line + 1 <= lastLine -> nearest(line + 1)
                    else -> null
                }
            }
            Word.LEFT -> if (line == 0) {
                val part = keys.parts.indexOfFirst { column in it }
                if (part > 0) copy(column = keys.parts[part - 1].first) else null
            } else copy(column = column - 1).takeIf { column > 0 && fits(it) }
            Word.RIGHT -> if (line == 0) {
                val part = keys.parts.indexOfFirst { column in it }
                if (part < keys.parts.lastIndex) copy(column = keys.parts[part + 1].first) else null
            } else copy(column = column + 1).takeIf { column < ACROSS - 1 && fits(it) }
            else -> null
        }
    }

    /** Applies this key to the text. The globe is the screen's to handle. */
    fun press(text: String, keys: LetterKeys): String = when (top(keys)) {
        TopKey.SPACE -> "$text "
        TopKey.DELETE -> text.dropLast(1)
        TopKey.GLOBE -> text
        null -> keys.at(line, column)?.let { text + it } ?: text
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
fun LetterGrid(focus: LetterPlace?, keys: LetterKeys, modifier: Modifier = Modifier) {
    Box(modifier) {
        val icons = listOf(R.drawable.icon_space, R.drawable.icon_globe, R.drawable.icon_erase)
        keys.parts.forEachIndexed { part, columns ->
            val icon = if (keys.globe) icons[part] else icons[if (part == 0) 0 else 2]
            val on = focus?.line == 0 && focus.column in columns
            Cell(columns.first * STEP, 0, columns.count() * STEP - 4, on) {
                Icon(painterResource(icon), null, Modifier.size(34.ref), tint = it)
            }
        }
        for (line in 1 until keys.lines) {
            for (column in 0 until ACROSS) {
                val letter = keys.at(line, column) ?: continue
                Cell(column * STEP, line * STEP, CELL, focus == LetterPlace(line, column)) {
                    Text(letter.toString(), color = it, fontSize = 30.refType)
                }
            }
        }
    }
}
