package streams.tv

import android.net.Uri
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import contract.Address
import contract.Screen
import contract.Tile
import contract.contractJson
import kotlinx.coroutines.delay
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.ref
import streams.shared.refType

private const val SETTLE_MS = 350L
private const val OFFERED = 12
private const val GENRES = 6

internal enum class Zone { TOP, LETTERS, GENRES, RESULTS }

/** What a search holds. It outlives the page, so Back from a title finds it as it was left. */
@Stable
class SearchState {
    var text by mutableStateOf("")
    var offered by mutableStateOf(emptyList<Tile>())
    var results by mutableStateOf(emptyList<Tile>())
    internal var zone by mutableStateOf(Zone.LETTERS)
    var place by mutableStateOf(LetterPlace())
    var tab by mutableIntStateOf(SEARCH_TAB)
    var genre by mutableIntStateOf(0)
    var result by mutableIntStateOf(0)
}

private fun tilesOf(text: String, limit: Int): List<Tile> =
    contractJson.decodeFromString(Screen.serializer(), text).sections
        .flatMap { it.tiles.orEmpty() }
        .distinctBy { it.title }
        .take(limit)

/** Search: letters and genres on the left, posters on the right. */
@Composable
fun Search(shell: Shell, seen: Seen, state: SearchState) {
    val tabs = tabsOf(seen.nav)
    val genres = seen.nav.filter { it.screen.value == "genre" }.take(GENRES)
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()

    LaunchedEffect(Unit) {
        if (state.offered.isEmpty()) {
            state.offered = shell.read("/v1/screens/home")?.let { tilesOf(it, OFFERED) }.orEmpty()
        }
    }
    LaunchedEffect(state.text) {
        if (state.text.isBlank()) {
            state.results = emptyList()
            return@LaunchedEffect
        }
        delay(SETTLE_MS)
        val query = Uri.encode(state.text.trim())
        state.results = shell.read("/v1/search?q=$query")?.let { tilesOf(it, Int.MAX_VALUE) }.orEmpty()
        state.result = 0
    }
    val shown = if (state.text.isBlank()) state.offered else state.results

    fun press(word: Word): Boolean {
        when (state.zone) {
            Zone.TOP -> when (word) {
                Word.LEFT -> if (state.tab == 0) state.tab = SEARCH_TAB else if (state.tab > 0) state.tab -= 1
                Word.RIGHT -> if (state.tab == SEARCH_TAB) state.tab = 0 else if (state.tab < tabs.lastIndex) state.tab += 1
                Word.DOWN -> state.zone = Zone.LETTERS
                Word.OK -> tabs.getOrNull(state.tab)?.let { shell.go(if (it.screen.value == "home") Address.HOME else "/" + it.screen.value) }
                Word.BACK -> shell.back()
                Word.UP -> Unit
            }
            Zone.LETTERS -> when (word) {
                Word.OK -> state.text = state.place.press(state.text)
                Word.BACK -> shell.back()
                else -> {
                    val next = state.place.move(word)
                    when {
                        next != null -> state.place = next
                        word == Word.UP -> state.zone = Zone.TOP
                        word == Word.DOWN && genres.isNotEmpty() -> state.zone = Zone.GENRES
                        word == Word.RIGHT && shown.isNotEmpty() -> state.zone = Zone.RESULTS
                    }
                }
            }
            Zone.GENRES -> when (word) {
                Word.UP -> if (state.genre > 0) state.genre -= 1 else state.zone = Zone.LETTERS
                Word.DOWN -> if (state.genre < genres.lastIndex) state.genre += 1
                Word.RIGHT -> if (shown.isNotEmpty()) state.zone = Zone.RESULTS
                Word.OK -> genres.getOrNull(state.genre)?.genre?.let { shell.go("/genre/$it") }
                Word.BACK -> shell.back()
                Word.LEFT -> Unit
            }
            Zone.RESULTS -> when {
                word == Word.OK -> shown.getOrNull(state.result)?.let { shell.go(titleAddress(it.title)) }
                word == Word.BACK -> state.zone = Zone.LETTERS
                word == Word.LEFT && state.result % POSTERS_ACROSS == 0 -> state.zone = Zone.LETTERS
                else -> state.result = gridMove(state.result, word, shown.lastIndex)
            }
        }
        return true
    }

    Box(Modifier.fillMaxSize().remote(@DontMemoize { press(it) })) {
        PosterGrid(shown, if (state.zone == Zone.RESULTS) state.result else -1, 566, 232)
        TopBar(tabs, SEARCH_TAB, if (state.zone == Zone.TOP) state.tab else -1, person, seen.server)
        LetterGrid(if (state.zone == Zone.LETTERS) state.place else null, Modifier.offset(68.ref, 160.ref))
        Menu(
            genres.map { MenuLine(it.label) },
            if (state.zone == Zone.GENRES) state.genre else -1,
            Modifier.offset(40.ref, 636.ref),
            width = 440,
        )
        Text(
            state.text.ifBlank { seen.words["search.recommended"].orEmpty() },
            Modifier.offset(564.ref, 150.ref).width(1280.ref),
            fontSize = 40.refType,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
