package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import contract.HelpBlock
import contract.HelpBlockKind
import contract.HelpLine
import contract.HelpSection
import streams.shared.ref
import streams.shared.refType

private fun marked(line: HelpLine): AnnotatedString = buildAnnotatedString {
    line.runs.forEach { run ->
        if (run.bold == true) {
            withStyle(SpanStyle(fontWeight = FontWeight.Bold)) { append(run.text) }
        } else {
            append(run.text)
        }
    }
}

@Composable
private fun Words(line: HelpLine, modifier: Modifier = Modifier) {
    Text(marked(line), modifier, fontSize = 30.refType, lineHeight = 44.refType, color = Color.White.copy(alpha = 0.9f))
}

@Composable
private fun Marked(lines: List<HelpLine>, mark: (Int) -> String) {
    Column(verticalArrangement = Arrangement.spacedBy(12.ref)) {
        lines.forEachIndexed { index, line ->
            Row {
                Text(mark(index), Modifier.width(52.ref), fontSize = 30.refType, lineHeight = 44.refType, fontWeight = FontWeight.Bold)
                Words(line)
            }
        }
    }
}

@Composable
private fun Block(block: HelpBlock) {
    val lines = block.lines.orEmpty()
    when (block.kind) {
        HelpBlockKind.HEADING -> Text(
            lines.firstOrNull()?.let(::marked) ?: AnnotatedString(""),
            fontSize = 36.refType,
            fontWeight = FontWeight.Bold,
        )
        HelpBlockKind.STEPS -> Marked(lines) { "${it + 1}." }
        HelpBlockKind.LIST -> Marked(lines) { "•" }
        HelpBlockKind.NOTE -> Box(
            Modifier
                .background(Color.White.copy(alpha = 0.1f), RoundedCornerShape(12.ref))
                .padding(24.ref),
        ) { lines.firstOrNull()?.let { Words(it) } }
        HelpBlockKind.PARAGRAPH -> lines.firstOrNull()?.let { Words(it) }
        // The actions are for apps that have them, and folds stay open on a television.
        else -> Unit
    }
}

/** The sections of an article or policy, as every family receives them from the server. */
@Composable
fun HelpBody(sections: List<HelpSection>, modifier: Modifier = Modifier) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(28.ref)) {
        sections.forEach { section ->
            section.title?.let { Text(it, fontSize = 36.refType, fontWeight = FontWeight.Bold) }
            section.blocks.forEach { Block(it) }
        }
    }
}
