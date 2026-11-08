package streams.tv

import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import contract.Tile
import contract.contractJson
import kotlin.coroutines.resume
import kotlinx.coroutines.delay
import kotlinx.coroutines.suspendCancellableCoroutine
import streams.ports.Shell
import streams.shared.ref
import streams.shared.refType

private const val SETTLE_MS = 300L
private const val FADE_MS = 500

private val FromLeft = arrayOf(0f to Color.Black, 0.45f to Color.Transparent)
private val FromFoot = Brush.verticalGradient(0.6f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.7f))

@Composable
private fun featureOf(shell: Shell, kept: HashMap<String, Tile>, id: String): State<Tile?> =
    produceState(kept[id], id) {
        kept[id]?.let {
            value = it
            return@produceState
        }
        delay(SETTLE_MS)
        val tile = suspendCancellableCoroutine { waiting ->
            shell.ask("/v1/profiles/$id/feature") { status, text ->
                val found = if (status in 200..299) {
                    contractJson.decodeFromString(Tile.serializer(), text)
                } else {
                    null
                }
                if (waiting.isActive) waiting.resume(found)
            }
        }
        tile?.let {
            kept[id] = it
            value = it
        }
    }

/** A title chosen for the profile in focus, crossfading as the focus moves. */
@Composable
fun ProfileFeature(shell: Shell, id: String) {
    val kept = remember { HashMap<String, Tile>() }
    val tile by featureOf(shell, kept, id)
    Crossfade(tile, Modifier.fillMaxSize(), tween(FADE_MS, easing = LinearEasing), label = "feature") { shown ->
        if (shown != null) {
            Box(Modifier.fillMaxSize()) {
                Box(Modifier.offset(480.ref, 0.ref).size(1440.ref, 1080.ref)) {
                    Art(wide(shown.images), Modifier.fillMaxSize())
                    Box(Modifier.fillMaxSize().background(fromStart(*FromLeft)))
                    Box(Modifier.fillMaxSize().background(FromFoot))
                }
                Column(
                    Modifier.align(Alignment.BottomEnd).padding(end = 64.ref, bottom = 52.ref).width(900.ref),
                    horizontalAlignment = Alignment.End,
                ) {
                    Text(
                        shown.name,
                        Modifier.fillMaxWidth(),
                        textAlign = TextAlign.End,
                        fontSize = 56.refType,
                        fontWeight = FontWeight.ExtraBold,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        shown.tags.orEmpty().joinToString(" · "),
                        Modifier.padding(top = 10.ref),
                        color = Color.White.copy(alpha = 0.85f),
                        fontSize = 26.refType,
                    )
                }
            }
        }
    }
}
