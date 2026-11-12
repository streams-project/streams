package streams.tv

import streams.shared.counted
import streams.shared.number
import streams.shared.fill
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import contract.TitleDetail
import streams.shared.Muted
import streams.shared.ref
import streams.shared.refType

private fun formatLength(words: Map<String, String>, minutes: Long): String {
    val h = (minutes / 60).toInt()
    val m = (minutes % 60).toInt()
    return when {
        h > 0 && m > 0 -> {
            val hText = words.counted("title.length.hours", h)
            val mText = words.counted("title.length.minutes", m)
            val pattern = words["title.length.both"] ?: "{hours} {minutes}"
            pattern.replace("{hours}", hText).replace("{minutes}", mText)
        }
        h > 0 -> words.counted("title.length.hours", h)
        else -> words.counted("title.length.minutes", m)
    }
}

internal fun facts(detail: TitleDetail, words: Map<String, String>): List<String> = buildList {
    detail.genres?.firstOrNull()?.let(::add)
    detail.year?.let { add(number(it)) }
    val seasons = detail.seasons?.size ?: 0
    val minutes = detail.runtimeMin
    when {
        seasons > 0 -> add(words.counted("title.season", seasons))
        minutes != null && minutes > 0 -> add(formatLength(words, minutes))
    }
    detail.age?.let {
        val translated = words["age.${it.label}"] ?: number(it.label)
        add(translated)
    }
}

/** The line of facts about a title, in one order on every screen: genre, year, length, age. */
@Composable
fun Facts(detail: TitleDetail, words: Map<String, String>, modifier: Modifier = Modifier) {
    Row(modifier, verticalAlignment = Alignment.CenterVertically) {
        facts(detail, words).forEachIndexed { index, word ->
            if (index > 0) {
                Box(Modifier.padding(horizontal = 14.ref).size(6.ref).background(Muted, CircleShape))
            }
            Text(word, fontSize = 24.refType, fontWeight = FontWeight.Bold)
        }
    }
}
