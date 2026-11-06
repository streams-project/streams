package streams.surfaces

import android.view.View
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import contract.ActionPause
import contract.ActionSeek
import contract.ActionUnpause
import contract.PlayState
import kotlinx.coroutines.delay
import streams.ports.Seen
import streams.ports.Shell

private const val CONTROLS_MS = 3000L
private const val SEEK_MS = 10_000L

private fun clock(ms: Long): String {
    val seconds = ms / 1000
    val body = "%d:%02d".format(seconds / 60 % 60, seconds % 60)
    return if (seconds >= 3600) "${seconds / 3600}:$body" else body
}

@Composable
fun Watch(shell: Shell, seen: Seen, tv: Boolean) {
    var woke by remember { mutableIntStateOf(1) }
    var shown by remember { mutableStateOf(true) }
    val focus = remember { FocusRequester() }
    fun toggle() = shell.act(if (seen.play?.state == PlayState.PLAYING) ActionPause else ActionUnpause)
    LaunchedEffect(woke) {
        shown = true
        delay(CONTROLS_MS)
        shown = false
    }
    LaunchedEffect(Unit) { focus.requestFocus() }
    Box(
        Modifier
            .fillMaxSize()
            .background(Color.Black)
            .focusRequester(focus)
            .focusable()
            .onKeyEvent { event ->
                if (event.type != KeyEventType.KeyDown) return@onKeyEvent false
                woke += 1
                when (event.key) {
                    Key.DirectionCenter, Key.Enter, Key.MediaPlayPause -> toggle()
                    Key.DirectionLeft -> shell.act(ActionSeek(toMs = (seen.positionMs - SEEK_MS).coerceAtLeast(0)))
                    Key.DirectionRight -> shell.act(ActionSeek(toMs = seen.positionMs + SEEK_MS))
                    else -> return@onKeyEvent false
                }
                true
            }
            .clickable {
                woke += 1
                if (!tv) toggle()
            },
    ) {
        AndroidView({ shell.player.picture(it) as View }, Modifier.fillMaxSize())
        if (shown) {
            Box(Modifier.align(Alignment.BottomStart).padding(24.dp).fillMaxWidth()) {
                Text(clock(seen.positionMs) + " / " + clock(seen.lengthMs), Modifier.padding(bottom = 14.dp))
                Box(Modifier.align(Alignment.BottomStart).fillMaxWidth().height(4.dp).background(Color(0x55FFFFFF))) {
                    val done = if (seen.lengthMs > 0) seen.positionMs.toFloat() / seen.lengthMs else 0f
                    Box(Modifier.fillMaxWidth(done).fillMaxHeight().background(Accent))
                }
            }
        }
    }
}
