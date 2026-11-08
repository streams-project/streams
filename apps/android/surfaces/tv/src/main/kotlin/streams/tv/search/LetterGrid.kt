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
        TopKey.DELETE -> deleteLetter(text)
        TopKey.GLOBE, TopKey.DIGITS -> text
        null -> keys.at(line, column)?.let { addLetter(text, it) } ?: text
    }
}

private const val HANGUL_BASE = 0xAC00
private const val HANGUL_END = 0xD7A3

private val CHOSEONG = listOf(
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ',
    'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'
)

private val JUNGSEONG = listOf(
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ',
    'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'
)

private fun hangul(cho: Int, jung: Int, jong: Int = 0): Char =
    (HANGUL_BASE + (cho * 21 + jung) * 28 + jong).toChar()

private fun singleJong(c: Char): Int = when (c) {
    'ㄱ' -> 1; 'ㄲ' -> 2; 'ㄴ' -> 4; 'ㄷ' -> 7; 'ㄹ' -> 8; 'ㅁ' -> 16
    'ㅂ' -> 17; 'ㅅ' -> 19; 'ㅆ' -> 20; 'ㅇ' -> 21; 'ㅈ' -> 22; 'ㅊ' -> 23
    'ㅋ' -> 24; 'ㅌ' -> 25; 'ㅍ' -> 26; 'ㅎ' -> 27
    else -> 0
}

private fun combineJong(firstJong: Int, c: Char): Int = when (firstJong) {
    1 -> if (c == 'ㅅ') 3 else 0
    4 -> when (c) { 'ㅈ' -> 5; 'ㅎ' -> 6; else -> 0 }
    8 -> when (c) {
        'ㄱ' -> 9; 'ㅁ' -> 10; 'ㅂ' -> 11; 'ㅅ' -> 12; 'ㅌ' -> 13; 'ㅍ' -> 14; 'ㅎ' -> 15
        else -> 0
    }
    17 -> if (c == 'ㅅ') 18 else 0
    else -> 0
}

private fun decomposeJong(jong: Int): Int = when (jong) {
    3 -> 1; 5 -> 4; 6 -> 4; 9 -> 8; 10 -> 8; 11 -> 8; 12 -> 8; 13 -> 8; 14 -> 8; 15 -> 8; 18 -> 17
    else -> 0
}

private fun detachJong(jong: Int): Pair<Int, Int>? = when (jong) {
    1 -> 0 to 0; 2 -> 0 to 1; 3 -> 1 to 9; 4 -> 0 to 2; 5 -> 4 to 12; 6 -> 4 to 18
    7 -> 0 to 3; 8 -> 0 to 5; 9 -> 8 to 0; 10 -> 8 to 6; 11 -> 8 to 7; 12 -> 8 to 9
    13 -> 8 to 16; 14 -> 8 to 17; 15 -> 8 to 18; 16 -> 0 to 6; 17 -> 0 to 7; 18 -> 17 to 9
    19 -> 0 to 9; 20 -> 0 to 10; 21 -> 0 to 11; 22 -> 0 to 12; 23 -> 0 to 14; 24 -> 0 to 15
    25 -> 0 to 16; 26 -> 0 to 17; 27 -> 0 to 18
    else -> null
}

private fun voiceKana(last: Char, mark: Char): Char? = when (mark) {
    '゛', '\u309B' -> when (last) {
        'か' -> 'が'; 'き' -> 'ぎ'; 'く' -> 'ぐ'; 'け' -> 'げ'; 'こ' -> 'ご'
        'さ' -> 'ざ'; 'し' -> 'じ'; 'す' -> 'ず'; 'せ' -> 'ぜ'; 'そ' -> 'ぞ'
        'た' -> 'だ'; 'ち' -> 'ぢ'; 'つ' -> 'づ'; 'て' -> 'で'; 'と' -> 'ど'
        'は' -> 'ば'; 'ひ' -> 'び'; 'ふ' -> 'ぶ'; 'へ' -> 'べ'; 'ほ' -> 'ぼ'
        'う' -> 'ゔ'
        'カ' -> 'ガ'; 'キ' -> 'ギ'; 'ク' -> 'グ'; 'ケ' -> 'ゲ'; 'コ' -> 'ゴ'
        'サ' -> 'ザ'; 'シ' -> 'ジ'; 'ス' -> 'ズ'; 'セ' -> 'ゼ'; 'ソ' -> 'ゾ'
        'タ' -> 'ダ'; 'チ' -> 'ヂ'; 'ツ' -> 'ヅ'; 'テ' -> 'デ'; 'ト' -> 'ド'
        'ハ' -> 'バ'; 'ヒ' -> 'ビ'; 'フ' -> 'ブ'; 'ヘ' -> 'ベ'; 'ホ' -> 'ボ'
        'ウ' -> 'ヴ'
        else -> null
    }
    '゜', '\u309C' -> when (last) {
        'は' -> 'ぱ'; 'ひ' -> 'ぴ'; 'ふ' -> 'ぷ'; 'へ' -> 'ぺ'; 'ほ' -> 'ぽ'
        'ハ' -> 'パ'; 'ヒ' -> 'ピ'; 'フ' -> 'プ'; 'ヘ' -> 'ペ'; 'ホ' -> 'ポ'
        else -> null
    }
    else -> null
}

private fun addLetter(text: String, c: Char): String {
    if (c == '゛' || c == '\u309B' || c == '゜' || c == '\u309C') {
        if (text.isNotEmpty()) {
            val voiced = voiceKana(text.last(), c)
            if (voiced != null) return text.dropLast(1) + voiced
        }
        return text + c
    }
    if (text.isNotEmpty()) {
        val last = text.last()
        val vIdx = JUNGSEONG.indexOf(c)
        val cIdx = CHOSEONG.indexOf(c)
        if (last in CHOSEONG && vIdx >= 0) {
            val cho = CHOSEONG.indexOf(last)
            return text.dropLast(1) + hangul(cho, vIdx, 0)
        }
        if (last.code in HANGUL_BASE..HANGUL_END) {
            val code = last.code - HANGUL_BASE
            val cho = code / (21 * 28)
            val jung = (code % (21 * 28)) / 28
            val jong = code % 28
            if (vIdx >= 0 && jong > 0) {
                val detached = detachJong(jong)
                if (detached != null) {
                    return text.dropLast(1) + hangul(cho, jung, detached.first) + hangul(detached.second, vIdx, 0)
                }
            } else if (cIdx >= 0) {
                if (jong == 0) {
                    val j = singleJong(c)
                    if (j > 0) return text.dropLast(1) + hangul(cho, jung, j)
                } else {
                    val comp = combineJong(jong, c)
                    if (comp > 0) return text.dropLast(1) + hangul(cho, jung, comp)
                }
            }
        }
    }
    return text + number(c)
}

private fun deleteLetter(text: String): String {
    if (text.isEmpty()) return ""
    val last = text.last()
    if (last.code in HANGUL_BASE..HANGUL_END) {
        val code = last.code - HANGUL_BASE
        val cho = code / (21 * 28)
        val jung = (code % (21 * 28)) / 28
        val jong = code % 28
        return if (jong > 0) {
            val rem = decomposeJong(jong)
            text.dropLast(1) + hangul(cho, jung, rem)
        } else {
            text.dropLast(1) + CHOSEONG[cho]
        }
    }
    return text.dropLast(1)
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
