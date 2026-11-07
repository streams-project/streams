package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import contract.ActionCommand
import contract.CommandList
import contract.CommandRate
import contract.Rating
import contract.Season
import contract.TitleDetail
import contract.contractJson
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

private val Side = Brush.horizontalGradient(0f to Color.Black.copy(alpha = 0.82f), 0.62f to Color.Black.copy(alpha = 0.12f))
private val Foot = Brush.verticalGradient(0.42f to Color.Transparent, 0.92f to Back)

private enum class Part { REACTIONS, MENU }
private enum class Page { TITLE, EPISODES, MORE }
private enum class Row { PLAY, EPISODES, MORE, LIST }

private fun TitleDetail.isSeries() = title.value.startsWith("tv:")

private fun TitleDetail.path() = title.value.replace(':', '/')

/** The artwork and its shades, behind a title's pages. */
@Composable
fun TitleBackdrop(data: TitleDetail) {
    Art(wide(data.images), Modifier.fillMaxSize())
    Box(Modifier.fillMaxSize().background(Side))
    Box(Modifier.fillMaxSize().background(Foot))
}

/** One title: its facts, three ratings, and a menu of what to do next. */
@Composable
fun TitlePage(shell: Shell, seen: Seen) {
    val data = seen.detail?.takeIf { it.title == seen.route?.title } ?: return
    var page by remember { mutableStateOf(Page.TITLE) }
    when (page) {
        Page.EPISODES -> Episodes(shell, seen, data) { page = Page.TITLE }
        Page.MORE -> MoreLike(shell, seen, data) { page = Page.TITLE }
        Page.TITLE -> Summary(shell, seen, data) { page = it }
    }
}

@Composable
private fun Summary(shell: Shell, seen: Seen, data: TitleDetail, open: (Page) -> Unit) {
    val words = seen.words
    val rows = buildList {
        add(Row.PLAY)
        if (!data.seasons.isNullOrEmpty()) add(Row.EPISODES)
        if (!data.moreLikeThis.isNullOrEmpty()) add(Row.MORE)
        add(Row.LIST)
    }
    var part by remember { mutableStateOf(Part.MENU) }
    var row by remember { mutableIntStateOf(0) }
    var reaction by remember { mutableIntStateOf(1) }
    var rating by remember { mutableStateOf<Rating?>(null) }
    var saved by remember(data.title, data.inList) { mutableStateOf(data.inList == true) }
    val scope = rememberCoroutineScope()

    fun play() {
        if (!data.isSeries()) {
            shell.go("/watch/" + data.path())
            return
        }
        val number = data.seasons?.firstOrNull()?.number ?: 1
        scope.launch {
            val season = shell.read("/v1/titles/${data.path()}/seasons/$number")
                ?.let { contractJson.decodeFromString(Season.serializer(), it) }
            shell.go("/watch/${data.path()}/$number/${season?.episodes?.firstOrNull()?.number ?: 1}")
        }
    }

    fun rate() {
        val asked = reactionAt(reaction)
        rating = if (rating == asked) null else asked
        shell.act(ActionCommand(CommandRate(rating = rating, title = data.title)))
    }

    fun toggleList() {
        saved = !saved
        shell.act(ActionCommand(CommandList(inList = saved, title = data.title)))
    }

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> when {
                part == Part.MENU && row > 0 -> row -= 1
                part == Part.MENU -> part = Part.REACTIONS
                else -> Unit
            }
            Word.DOWN -> when {
                part == Part.REACTIONS -> part = Part.MENU
                row < rows.lastIndex -> row += 1
            }
            Word.LEFT -> if (part == Part.REACTIONS && reaction > 0) reaction -= 1
            Word.RIGHT -> if (part == Part.REACTIONS && reaction < ReactionCount - 1) reaction += 1
            Word.OK -> when {
                part == Part.REACTIONS -> rate()
                rows[row] == Row.PLAY -> play()
                rows[row] == Row.EPISODES -> open(Page.EPISODES)
                rows[row] == Row.LIST -> toggleList()
                else -> open(Page.MORE)
            }
            Word.BACK -> return false
        }
        return true
    }

    val lines = rows.map { line ->
        when (line) {
            Row.PLAY -> MenuLine(
                if (data.isSeries()) words["title.play_episode"].orEmpty().replace("{n}", "1") else words["common.play"].orEmpty(),
                R.drawable.icon_play,
            )
            Row.EPISODES -> MenuLine(words["title.episodes_more"].orEmpty(), R.drawable.icon_episodes)
            Row.MORE -> MenuLine(words["title.more_like"].orEmpty(), R.drawable.icon_grid)
            Row.LIST -> MenuLine(
                words[if (saved) "list.added" else "list.add"].orEmpty(),
                if (saved) R.drawable.icon_check else R.drawable.icon_plus,
            )
        }
    }
    Box(Modifier.fillMaxSize().remote(@DontMemoize { press(it) })) {
        TitleBackdrop(data)
        Box(Modifier.offset(60.ref, 50.ref).size(900.ref, 176.ref), contentAlignment = Alignment.BottomStart) {
            Text(
                data.name,
                fontSize = 76.refType,
                lineHeight = 84.refType,
                fontWeight = FontWeight.ExtraBold,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Facts(data, words, Modifier.offset(60.ref, 244.ref))
        Text(
            data.overview.orEmpty(),
            Modifier.offset(60.ref, 300.ref).width(820.ref),
            fontSize = 28.refType,
            lineHeight = 38.refType,
            color = Color.White.copy(alpha = 0.85f),
            maxLines = 4,
            overflow = TextOverflow.Ellipsis,
        )
        Reactions(rating, if (part == Part.REACTIONS) reaction else -1, 88, 504)
        Menu(lines, if (part == Part.MENU) row else -1, Modifier.offset(68.ref, 581.ref), width = 570)
    }
}
