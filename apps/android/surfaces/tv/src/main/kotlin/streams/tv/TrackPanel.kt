package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import contract.ActionSelectAudio
import contract.Track
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.ref
import streams.shared.refType

/** Where the focus is in the full lists. The first subtitle line is Off. */
class TrackPanelState {
    var open by mutableStateOf(false)
    var column by mutableIntStateOf(0)
    var line by mutableIntStateOf(0)

    fun show(seen: Seen) {
        open = true
        column = 0
        line = oneOfEach(seen.audio).indexOfFirst { it.playing }.coerceAtLeast(0)
    }

    /** The panel takes every key while it is open. */
    fun press(word: Word, shell: Shell, seen: Seen, files: List<Track>, chooseLine: (Track?) -> Unit) {
        val sounds = oneOfEach(seen.audio)
        val lines = linesOf(seen, files)
        val last = if (column == 0) sounds.lastIndex else lines.size
        when (word) {
            Word.BACK -> open = false
            Word.UP -> if (line > 0) line -= 1
            Word.DOWN -> if (line < last) line += 1
            Word.LEFT -> if (column == 1) {
                column = 0
                line = sounds.indexOfFirst { it.playing }.coerceAtLeast(0)
            }
            Word.RIGHT -> if (column == 0) {
                column = 1
                line = lines.indexOfFirst { it.playing } + 1
            }
            Word.OK -> if (column == 0) {
                sounds.getOrNull(line)?.let { shell.act(ActionSelectAudio(track = it.id)) }
            } else {
                chooseLine(lines.getOrNull(line - 1))
            }
        }
    }
}

@Composable
private fun Tracks(head: String, lines: List<String>, focus: Int, kept: Int, x: Int) {
    Column(Modifier.offset(x.ref, 180.ref)) {
        Text(head, Modifier.padding(start = 90.ref, bottom = 24.ref), fontSize = 40.refType, fontWeight = FontWeight.Bold)
        Menu(lines.map { MenuLine(it) }, focus, width = 620, kept = kept)
    }
}

/** Every sound and every subtitle, over the picture, which goes on playing. */
@Composable
fun TrackPanel(state: TrackPanelState, seen: Seen, files: List<Track>) {
    val words = seen.words
    val sounds: List<Track> = oneOfEach(seen.audio)
    val lines: List<Track> = linesOf(seen, files)
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.8f))) {
        Tracks(
            words["player.audio"].orEmpty(),
            sounds.map { nameOf(it, seen.language) },
            if (state.column == 0) state.line else -1,
            sounds.indexOfFirst { it.playing },
            160,
        )
        Tracks(
            words["player.subtitles"].orEmpty(),
            listOf(words["player.subtitles_off"].orEmpty()) + lines.map { nameOf(it, seen.language) },
            if (state.column == 1) state.line else -1,
            lines.indexOfFirst { it.playing } + 1,
            960,
        )
    }
}
