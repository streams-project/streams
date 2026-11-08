package streams.tv

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

private fun facts(detail: TitleDetail, words: Map<String, String>): List<String> = buildList {
    detail.genres?.firstOrNull()?.let(::add)
    detail.year?.let { add(it.toString()) }
    val seasons = detail.seasons?.size ?: 0
    val minutes = detail.runtimeMin
    when {
        seasons > 0 -> {
            val word = words[if (seasons == 1) "title.season.one" else "title.season.many"].orEmpty()
            add(word.fill("n", seasons.toString()))
        }
        minutes != null && minutes >= 60 ->
            add(words["title.length.hours"].orEmpty().fill("h", minutes / 60).fill("m", minutes % 60))
        minutes != null -> add(words["title.length.minutes"].orEmpty().fill("m", minutes))
    }
    detail.age?.let { add(it.label) }
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
