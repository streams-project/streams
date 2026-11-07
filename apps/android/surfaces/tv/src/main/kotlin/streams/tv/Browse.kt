package streams.tv

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import contract.Section
import contract.SectionKind
import contract.Tile
import contract.TitleKey
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Muted
import streams.shared.ref
import streams.shared.refType

private const val RISE = 810
private const val RISE_MS = 320

private enum class Where { TOP, HERO, ROW }

private val RowKinds = setOf(SectionKind.ROW, SectionKind.RANKED, SectionKind.CONTINUE, SectionKind.GRID)

/** Where the viewer is on a page of rows. It knows nothing about drawing. */
private class PageFocus(var rows: List<Section>, var tabs: Int, active: Int, var hero: Boolean) {
    private val above get() = if (hero) Where.HERO else Where.TOP
    var where by mutableStateOf(if (hero || rows.isEmpty()) above else Where.ROW)
    var row by mutableIntStateOf(0)
    var tab by mutableIntStateOf(active)
    var button by mutableIntStateOf(0)
    private val columns = mutableStateMapOf<Int, Int>()

    val column: Int get() = columns[row] ?: 0

    private fun lastColumn() = rows.getOrNull(row)?.tiles.orEmpty().lastIndex

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> when {
                where == Where.ROW && row > 0 -> row -= 1
                where == Where.ROW -> where = above
                where == Where.HERO -> where = Where.TOP
                else -> return false
            }
            Word.DOWN -> when {
                where == Where.TOP && hero -> where = Where.HERO
                (where == Where.HERO || where == Where.TOP) && rows.isNotEmpty() -> {
                    where = Where.ROW
                    row = 0
                }
                where == Where.ROW && row < rows.lastIndex -> row += 1
                else -> return false
            }
            Word.LEFT -> when {
                where == Where.TOP && tab > 0 -> tab -= 1
                where == Where.TOP && tab == 0 -> tab = SEARCH_TAB
                where == Where.HERO -> button = 0
                where == Where.ROW && column > 0 -> columns[row] = column - 1
                else -> return false
            }
            Word.RIGHT -> when {
                where == Where.TOP && tab == SEARCH_TAB -> tab = 0
                where == Where.TOP && tab < tabs - 1 -> tab += 1
                where == Where.HERO -> button = 1
                where == Where.ROW && column < lastColumn() -> columns[row] = column + 1
                else -> return false
            }
            Word.BACK -> when (where) {
                Where.ROW -> where = above
                Where.HERO -> where = Where.TOP
                Where.TOP -> return false
            }
            Word.OK -> return false
        }
        return true
    }
}

fun titleAddress(key: TitleKey) = "/title/" + key.value.replace(':', '/')

private fun watchAddress(key: TitleKey) = "/watch/" + key.value.replace(':', '/')

private fun Tile.isSeries() = title.value.startsWith("tv:")

private fun tabAddress(screen: String) = if (screen == "home") "/home" else "/$screen"

/** A page of rows. The parts draw, and this file only knows where the focus is. */
@Composable
fun Browse(shell: Shell, seen: Seen) {
    val data = seen.screen ?: return
    val sections = data.sections.filter { !it.tiles.isNullOrEmpty() }
    val rows = sections.filter { it.kind in RowKinds }
    val lead = sections.firstOrNull { it.kind == SectionKind.HERO }?.tiles?.first()
    val tabs = tabsOf(seen.nav)
    val current = seen.route?.screen?.value ?: "home"
    val active = tabs.indexOfFirst { it.screen.value == current }.coerceAtLeast(0)
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    // A page is drawn from what was kept and then from the fresh answer. The focus belongs to the page.
    val focus = remember(current, seen.route?.genre) { PageFocus(rows, tabs.size, active, lead != null) }
    focus.rows = rows
    focus.tabs = tabs.size
    focus.hero = lead != null
    val store = rememberDetails()
    val rise = with(LocalDensity.current) { RISE.ref.roundToPx() }

    fun choose() {
        when {
            focus.where == Where.TOP && focus.tab == SEARCH_TAB -> shell.go("/search")
            focus.where == Where.TOP -> tabs.getOrNull(focus.tab)?.let { shell.go(tabAddress(it.screen.value)) }
            focus.where == Where.ROW -> rows.getOrNull(focus.row)?.tiles?.getOrNull(focus.column)?.let {
                shell.go(titleAddress(it.title))
            }
            lead == null -> Unit
            focus.button == 1 || lead.isSeries() -> shell.go(titleAddress(lead.title))
            else -> shell.go(watchAddress(lead.title))
        }
    }

    Box(
        Modifier.fillMaxSize().remote { word ->
            when {
                word == Word.OK -> {
                    choose()
                    true
                }
                focus.press(word) -> true
                // Back from the top of any page but Home goes Home. From Home it leaves the app.
                word == Word.BACK && current != "home" -> {
                    shell.go("/home")
                    true
                }
                else -> false
            }
        },
    ) {
        val inRow = focus.where == Where.ROW
        TopBar(tabs, active, if (focus.where == Where.TOP) focus.tab else -1, person, seen.server)
        if (inRow) {
            AnimatedContent(
                focus.row,
                transitionSpec = {
                    val way = if (targetState > initialState) 1 else -1
                    val time = tween<Float>(RISE_MS, easing = Glide)
                    val move = tween<androidx.compose.ui.unit.IntOffset>(RISE_MS, easing = Glide)
                    (slideInVertically(move) { way * rise } + fadeIn(time, 0.4f)) togetherWith
                        (slideOutVertically(move) { -way * rise } + fadeOut(time))
                },
                label = "row",
            ) { row ->
                rows.getOrNull(row)?.let {
                    OpenRow(shell, seen.words, store, it, if (row == focus.row) focus.column else 0)
                }
            }
        } else if (lead != null) {
            HeroCard(shell, seen, store, lead, if (focus.where == Where.HERO) focus.button else -1)
        } else if (rows.isEmpty()) {
            Text(
                seen.words["mine.empty"].orEmpty(),
                Modifier.align(Alignment.Center),
                fontSize = 32.refType,
                color = Muted,
            )
        }
        (if (inRow) rows.getOrNull(focus.row + 1) else rows.firstOrNull())?.let { NextRow(it, !inRow) }
    }
}
