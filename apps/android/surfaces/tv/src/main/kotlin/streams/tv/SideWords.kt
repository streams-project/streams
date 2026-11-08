package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import contract.Cue
import contract.SubtitleBody
import contract.SubtitleLanguage
import contract.SubtitleList
import contract.Track
import contract.contractJson
import kotlinx.coroutines.delay
import streams.ports.Shell
import streams.shared.ref
import streams.shared.refType

private const val POLL_MS = 200L
private const val RAISED = 220
private const val RESTING = 70

/** A file that ends far from the film's length belongs to another cut. */
private const val SHORTEST_SHARE = 0.5
private const val LONGEST_SHARE = 1.1

private const val FILE_MARK = "file:"

/** A subtitle track that is a file on the server, not part of the stream. */
fun Track.isFile() = id.startsWith(FILE_MARK)

/** Subtitle files the server offers for what is playing, and the cues of the one in use. */
class SideWords {
    private var languages by mutableStateOf<List<SubtitleLanguage>>(emptyList())
    var chosen by mutableStateOf<String?>(null)
        private set
    var cues by mutableStateOf<List<Cue>>(emptyList())
        private set

    /** The server's languages, as tracks beside the stream's own. */
    val tracks: List<Track>
        get() = languages.map { Track(id = FILE_MARK + it.lang, label = it.label, lang = it.lang, selected = it.lang == chosen) }

    suspend fun load(shell: Shell, title: String, episode: String?) {
        val ask = "/v1/subtitles?title=$title" + episode?.let { "&episode=$it" }.orEmpty()
        languages = shell.read(ask)?.let { contractJson.decodeFromString(SubtitleList.serializer(), it).languages }.orEmpty()
    }

    fun has(language: String) = languages.any { it.lang == language }

    fun off() {
        chosen = null
        cues = emptyList()
    }

    /** The first file of the language that fits the film. */
    suspend fun pick(shell: Shell, language: String, lengthMs: Long) {
        val files = languages.firstOrNull { it.lang == language }?.files.orEmpty()
        for (file in files) {
            val body = shell.read("/v1/subtitles/${file.id}")
                ?.let { contractJson.decodeFromString(SubtitleBody.serializer(), it) } ?: continue
            val share = if (lengthMs > 0) body.lastEndMs.toDouble() / lengthMs else 1.0
            if (share in SHORTEST_SHARE..LONGEST_SHARE) {
                chosen = language
                cues = body.cues
                return
            }
        }
        off()
    }
}

private fun List<Cue>.at(ms: Long): Cue? {
    var low = 0
    var high = lastIndex
    while (low <= high) {
        val middle = (low + high) / 2
        val cue = this[middle]
        when {
            ms < cue.startMs -> high = middle - 1
            ms >= cue.endMs -> low = middle + 1
            else -> return cue
        }
    }
    return null
}

/** The line of the moment, drawn over the picture. */
@Composable
fun BoxScope.SideLine(words: SideWords, shell: Shell, raised: Boolean) {
    var line by androidx.compose.runtime.remember { mutableStateOf<String?>(null) }
    LaunchedEffect(words.cues) {
        while (true) {
            line = words.cues.at(shell.player.now())?.text
            delay(POLL_MS)
        }
    }
    line?.let {
        Box(
            Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = (if (raised) RAISED else RESTING).ref)
                .widthIn(max = 1500.ref)
                .background(Color.Black.copy(alpha = 0.6f), shapeOf(Round.chip))
                .padding(horizontal = 16.ref, vertical = 6.ref),
        ) {
            Text(it, color = Color.White, fontSize = 40.refType, lineHeight = 50.refType, textAlign = TextAlign.Center)
        }
    }
}
