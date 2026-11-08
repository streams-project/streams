package streams.tv

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import contract.HelpArticle
import contract.HelpHome
import contract.HelpPolicy
import contract.HelpSection
import contract.contractJson
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Muted
import streams.shared.ref
import streams.shared.refType

private const val MOVE_MS = 240
private const val PAGE = 320
private const val SCREEN = 1080
private const val FIRST_LINE = 262
private const val LINE = 78

private sealed interface Open
private data class Article(val slug: String) : Open
private data class Policy(val id: String) : Open

private fun Seen.query() = if (language.isEmpty()) "" else "?lang=$language"

/** A long page, moved a screenful at a time by the remote. */
@Composable
private fun Reading(title: String, sections: List<HelpSection>, leave: () -> Unit) {
    var moved by remember { mutableIntStateOf(0) }
    var high by remember { mutableIntStateOf(0) }
    val density = LocalDensity.current
    val shown by animateFloatAsState(moved.toFloat(), tween(MOVE_MS, easing = Glide), label = "page")
    val last = with(density) { (high.toDp() - (SCREEN - 160).ref).toPx() }.coerceAtLeast(0f)
    val step = with(density) { PAGE.ref.toPx() }.toInt()

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> moved = (moved - step).coerceAtLeast(0)
            Word.DOWN -> moved = (moved + step).coerceAtMost(last.toInt())
            // An article is read with Up and Down; Left and Back return to the list it came from.
            Word.BACK, Word.LEFT -> leave()
            else -> Unit
        }
        return true
    }

    Box(Modifier.fillMaxSize().background(Color.Black).remote(@DontMemoize { press(it) })) {
        Column(
            Modifier
                .wrapContentHeight(Alignment.Top, unbounded = true)
                .offset(SUB_EDGE.ref, 80.ref)
                .width(1500.ref)
                .graphicsLayer { translationY = -shown }
                .onSizeChanged { high = it.height },
        ) {
            Text(title, Modifier.padding(bottom = 36.ref), fontSize = 56.refType, fontWeight = FontWeight.ExtraBold)
            HelpBody(sections)
        }
    }
}

@Composable
private fun ArticlePage(shell: Shell, seen: Seen, slug: String, leave: () -> Unit) {
    val page by produceState<HelpArticle?>(null, slug, seen.language) {
        value = shell.read("/v1/help/articles/$slug${seen.query()}")
            ?.let { contractJson.decodeFromString(HelpArticle.serializer(), it) }
    }
    page?.let { Reading(it.title, it.sections, leave) }
}

@Composable
private fun PolicyPage(shell: Shell, seen: Seen, id: String, leave: () -> Unit) {
    val page by produceState<HelpPolicy?>(null, id, seen.language) {
        value = shell.read("/v1/help/policies/$id${seen.query()}")
            ?.let { contractJson.decodeFromString(HelpPolicy.serializer(), it) }
    }
    page?.let { Reading(it.title, it.sections, leave) }
}

/** One line of the right column: an article or policy, or a heading that takes no focus. */
private data class Entry(val word: String, val open: Open?)

private fun entries(home: HelpHome, topic: Int, policiesWord: String): List<Entry> {
    val topics = home.topics
    if (topic == topics.size) {
        return listOf(Entry(policiesWord, null)) + home.policies.map { Entry(it.title, Policy(it.id)) }
    }
    return topics.getOrNull(topic)?.groups.orEmpty().flatMap { group ->
        listOf(Entry(group.title, null)) + group.articles.map { Entry(it.title, Article(it.slug)) }
    }
}

/** Topics on the left and their articles on the right. The words come from the server. */
@Composable
fun HelpCentre(shell: Shell, seen: Seen, leave: () -> Unit) {
    val words = seen.words
    val home by produceState<HelpHome?>(null, seen.language) {
        value = shell.read("/v1/help${seen.query()}")
            ?.let { contractJson.decodeFromString(HelpHome.serializer(), it) }
    }
    var open by remember { mutableStateOf<Open?>(null) }
    var topic by remember { mutableIntStateOf(0) }
    var right by remember { mutableStateOf(false) }
    var line by remember { mutableIntStateOf(0) }
    when (val shown = open) {
        is Article -> {
            ArticlePage(shell, seen, shown.slug) { open = null }
            return
        }
        is Policy -> {
            PolicyPage(shell, seen, shown.id) { open = null }
            return
        }
        null -> Unit
    }
    val policies = words["help.policies"].orEmpty()
    val list = home?.let { entries(it, topic, policies) }.orEmpty()
    val focusable = list.indices.filter { list[it].open != null }
    val topics = home?.topics.orEmpty().map { MenuLine(it.title) } + MenuLine(policies)
    val at = focusable.getOrNull(line) ?: -1
    val scrolled by animateFloatAsState(maxOf(0, at - 7).toFloat(), tween(MOVE_MS, easing = Glide), label = "list")
    val step = with(LocalDensity.current) { LINE.ref.toPx() }

    fun press(word: Word): Boolean {
        if (home == null) {
            if (word == Word.BACK) leave()
            return true
        }
        when {
            word == Word.BACK && right -> right = false
            word == Word.BACK -> leave()
            right && word == Word.LEFT -> right = false
            // On the topics, the left edge of the page, Left goes back to Get Help.
            word == Word.LEFT -> leave()
            right && word == Word.UP -> if (line > 0) line -= 1
            right && word == Word.DOWN -> if (line < focusable.lastIndex) line += 1
            right && word == Word.OK -> open = list[focusable[line]].open
            word == Word.UP -> if (topic > 0) {
                topic -= 1
                line = 0
            }
            word == Word.DOWN -> if (topic < topics.lastIndex) {
                topic += 1
                line = 0
            }
            (word == Word.RIGHT || word == Word.OK) && focusable.isNotEmpty() -> right = true
        }
        return true
    }

    Box(Modifier.fillMaxSize().background(Color.Black).remote(@DontMemoize { press(it) })) {
        PageHead(words["help.centre"].orEmpty(), words["help.intro"].orEmpty())
        if (home == null) {
            Text(
                words["help.off"].orEmpty(),
                Modifier.align(Alignment.Center),
                fontSize = 32.refType,
                color = Muted,
            )
        } else {
            Menu(topics, if (right) -1 else topic, Modifier.offset((SUB_EDGE - MENU_WORDS).ref, FIRST_LINE.ref), width = 480, kept = topic)
            Box(Modifier.offset(700.ref, FIRST_LINE.ref).size(1100.ref, (SCREEN - FIRST_LINE - 60).ref).clipToBounds()) {
                Column(
                    Modifier
                        .wrapContentHeight(Alignment.Top, unbounded = true)
                        .graphicsLayer { translationY = -scrolled * step },
                ) {
                    list.forEachIndexed { index, entry ->
                        val on = right && index == at
                        Box(
                            Modifier
                                .width(1080.ref)
                                .padding(bottom = 6.ref)
                                .background(if (on) Color.White else Color.Transparent, CircleShape)
                                .padding(horizontal = 32.ref, vertical = 12.ref),
                        ) {
                            Text(
                                entry.word,
                                color = if (on) Back else if (entry.open == null) Muted else Color.White,
                                fontSize = 30.refType,
                                fontWeight = if (entry.open == null) FontWeight.Bold else FontWeight.Normal,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }
            }
        }
    }
}
