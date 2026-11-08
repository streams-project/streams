package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import contract.TimeWindow
import contract.WindowKind
import streams.ports.Seen
import streams.shared.Accent
import streams.shared.Back
import streams.shared.ref
import streams.shared.refType

/** The window the picture is inside now, of one of these kinds. */
fun Seen.windowNow(kinds: Set<WindowKind>): TimeWindow? =
    play?.windows?.firstOrNull { it.kind in kinds && positionMs in it.startMs until it.endMs }

/** The pill offered at a moment of the film: skip this part, or go on to the next episode. */
@Composable
fun BoxScope.MomentPill(word: String, on: Boolean, raised: Boolean) {
    Box(
        Modifier
            .align(Alignment.BottomEnd)
            .padding(end = 124.ref, bottom = (if (raised) 230 else 96).ref)
            .height(72.ref)
            .background(if (on) Color.White else Color.White.copy(alpha = 0.2f), CircleShape)
            .padding(horizontal = 40.ref),
        contentAlignment = Alignment.Center,
    ) {
        Text(word, color = if (on) Back else Color.White, fontSize = 30.refType, fontWeight = FontWeight.Bold)
    }
}

/** The age rating, once, as play starts. */
@Composable
fun AgeNote(label: String) {
    Row(Modifier.offset(96.ref, 260.ref).height(84.ref)) {
        Box(Modifier.width(6.ref).fillMaxHeight().background(Accent))
        Text(
            label,
            Modifier.align(Alignment.CenterVertically).padding(start = 22.ref),
            style = TextStyle(shadow = Shadow(Color.Black, blurRadius = 12f)),
            fontSize = 44.refType,
            fontWeight = FontWeight.Bold,
        )
    }
}

/** Several episodes went by with nobody touching the remote. The picture waits for an answer. */
@Composable
fun StillAsking(words: Map<String, String>, focus: Int) {
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.8f)), contentAlignment = Alignment.Center) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(words["player.still"].orEmpty(), fontSize = 56.refType, fontWeight = FontWeight.Bold)
            Row(Modifier.padding(top = 48.ref), horizontalArrangement = Arrangement.spacedBy(24.ref)) {
                Pill(words["player.still.yes"].orEmpty(), focus == 0)
                Pill(words["player.still.no"].orEmpty(), focus == 1)
            }
        }
    }
}
