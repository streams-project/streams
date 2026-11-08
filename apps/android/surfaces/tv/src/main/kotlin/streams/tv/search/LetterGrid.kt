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
import streams.shared.number
import streams.shared.ref
import streams.shared.refType

private const val ACROSS = 7
private const val STEP = 66
private const val CELL = 62

/**
 * The English letters, which every profile can switch to. Each alphabet fills whole lines of seven
 * with the marks titles use, so no line ends half empty: 26 letters and ' and - make four lines.
 */
const val ENGLISH_LETTERS = "abcdefghijklmnopqrstuvwxyz'-"

/** Behind the 123 key: digits and the marks titles use, two whole lines of seven. */
const val DIGIT_KEYS = "1234567890-&:."

/**
 * The keys of the grid: letters seven across under a top line of space, 123 (digits), the globe
 * (when the profile's language has letters of its own) and delete. These equal LetterKeys in
 * LetterGrid.swift.
 */
data class LetterKeys(
    val letters: String,
    val globe: Boolean,
    val mirrored: Boolean = false,
    /** What the 123 key says: 123, or the alphabet's first letters while the digits show. */
    val switchWord: String = "123",
) {
    /**
     * The letter at a place, or null. A left to right set inside a right to left screen fills each
     * line from the left (`mirrored`), so English and digits still read from the left.
     */
    fun at(line: Int, column: Int): Char? {
        if (line < 1) return null
        val index = (line - 1) * ACROSS + if (mirrored) ACROSS - 1 - column else column
        return letters.getOrNull(index)
    }

    /** Lines of the grid, the top line included. */
    val lines: Int get() = 1 + (letters.length + ACROSS - 1) / ACROSS

    /** The top line's keys, in order. */
    val tops: List<TopKey> get() = if (globe) TopKey.entries else TopKey.entries - TopKey.GLOBE

    /** The top line split into its keys, as column ranges. */
    val parts: List<IntRange>
        get() = if (globe) listOf(0..1, 2..3, 4..4, 5..6) else listOf(0..2, 3..4, 5..6)

    /** The height of the grid in reference pixels. */
    val high: Int get() = lines * STEP
}

/** What a key on the top line does. */
enum class TopKey { SPACE, DIGITS, GLOBE, DELETE }

/** A place on the letter grid. Line 0 holds space, 123, the globe and delete. */
data class LetterPlace(val line: Int = 1, val column: Int = 0) {
    /** The top line key at this place, or null on a letter. */
    fun top(keys: LetterKeys): TopKey? {
        if (line != 0) return null
        val part = keys.parts.indexOfFirst { column in it }.coerceAtLeast(0)
        return keys.tops[part]
    }

    fun move(word: Word, keys: LetterKeys): LetterPlace? {
        fun fits(place: LetterPlace) = place.line == 0 || keys.at(place.line, place.column) != null
        val lastLine = keys.lines - 1
        // On a short line, Down lands on its nearest letter.
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

    /** Applies this key to the text. The 123 key and the globe are the screen's to handle. */
    fun press(text: String, keys: LetterKeys): String = when (top(keys)) {
        TopKey.SPACE -> "$text "
        TopKey.DELETE -> text.dropLast(1)
        TopKey.GLOBE, TopKey.DIGITS -> text
        null -> keys.at(line, column)?.let { text + number(it) } ?: text
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

/** Letters seven across under the top line. `focus` is null when the grid has no focus. */
@Composable
fun LetterGrid(focus: LetterPlace?, keys: LetterKeys, modifier: Modifier = Modifier) {
    Box(modifier) {
        keys.parts.forEachIndexed { part, columns ->
            val on = focus?.line == 0 && focus.column in columns
            Cell(columns.first * STEP, 0, columns.count() * STEP - 4, on) { ink ->
                when (keys.tops[part]) {
                    TopKey.SPACE -> Icon(painterResource(R.drawable.icon_space), null, Modifier.size(34.ref), tint = ink)
                    TopKey.DIGITS -> Text(keys.switchWord, color = ink, fontSize = 26.refType, maxLines = 1)
                    TopKey.GLOBE -> Icon(painterResource(R.drawable.icon_globe), null, Modifier.size(34.ref), tint = ink)
                    TopKey.DELETE -> Icon(painterResource(R.drawable.icon_erase), null, Modifier.size(34.ref), tint = ink)
                }
            }
        }
        for (line in 1 until keys.lines) {
            for (column in 0 until ACROSS) {
                val letter = keys.at(line, column) ?: continue
                Cell(column * STEP, line * STEP, CELL, focus == LetterPlace(line, column)) {
                    Text(number(letter), color = it, fontSize = 30.refType)
                }
            }
        }
    }
}
