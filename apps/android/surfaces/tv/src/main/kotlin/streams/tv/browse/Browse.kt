package streams.tv

import streams.shared.rememberDetails
import streams.shared.titleAddress
import streams.shared.watchAddress
import streams.shared.tabAddress
import streams.shared.isSeries
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
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import contract.Address
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

/** Where the viewer was on each page of rows, for the run of the app. */
val LocalPages = staticCompositionLocalOf { HashMap<String, Any>() }

private val RowKinds = setOf(SectionKind.ROW, SectionKind.RANKED, SectionKind.CONTINUE, SectionKind.GRID)

/** Where the viewer is on a page of rows. It knows nothing about drawing. */
private class PageFocus(var rows: List<Section>, var tabs: Int, private val active: Int, var hero: Boolean) {
    private val above get() = if (hero) Where.HERO else Where.TOP
    var where by mutableStateOf(if (hero || rows.isEmpty()) above else Where.ROW)
    var row by mutableIntStateOf(0)
    var tab by mutableIntStateOf(active)
    var button by mutableIntStateOf(0)
    /**
     * Where each row was left, by the row's id: the place and the title on it. A row the viewer comes
     * back to, or sees below, opens there; a fresh answer that moved the title keeps the title.
     */
    private val columns = mutableStateMapOf<String, Pair<Int, TitleKey?>>()

    /** The tile a row was left on, or its first. */
    fun columnOf(index: Int): Int {
        val section = rows.getOrNull(index) ?: return 0
        val tiles = section.tiles.orEmpty()
        val (at, title) = columns[section.id] ?: return 0
        if (tiles.getOrNull(at)?.title == title) return at
        val moved = tiles.indexOfFirst { it.title == title }
        return if (moved >= 0) moved else at.coerceIn(0, maxOf(tiles.lastIndex, 0))
    }

    val column: Int get() = columnOf(row)

    private fun keep(at: Int) {
        val section = rows.getOrNull(row) ?: return
        columns[section.id] = at to section.tiles.orEmpty().getOrNull(at)?.title
    }

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
                where == Where.TOP && tab == SEARCH_TAB -> tab = WHO_TAB
                where == Where.HERO -> button = 0
                where == Where.ROW && column > 0 -> keep(column - 1)
                else -> return false
            }
            Word.RIGHT -> when {
                where == Where.TOP && tab == WHO_TAB -> tab = SEARCH_TAB
                where == Where.TOP && tab == SEARCH_TAB -> tab = 0
                where == Where.TOP && tab < tabs - 1 -> tab += 1
                where == Where.HERO -> button = 1
                where == Where.ROW && column < lastColumn() -> keep(column + 1)
                else -> return false
            }
            Word.BACK -> when {
                where == Where.ROW -> where = above
                where == Where.HERO -> where = Where.TOP
                // Focus wandered along the bar without choosing: Back brings it home to this page's tab
                // first, and only then leaves the page.
                tab != active -> tab = active
                else -> return false
            }
            Word.OK -> return false
        }
        return true
    }
}

/** A page of rows. The parts draw, and this file only knows where the focus is. */
@Composable
fun Browse(shell: Shell, seen: Seen) {
    val data = seen.screen ?: return
    val current = seen.route?.screen?.value ?: "home"
    // Games has no rows yet, and the kept page behind it belongs to the tab before.
    val sections = if (current == "games") emptyList() else data.sections.filter { !it.tiles.isNullOrEmpty() }
    val rows = sections.filter { it.kind in RowKinds }
    val lead = sections.firstOrNull { it.kind == SectionKind.HERO }?.tiles?.first()
    val tabs = tabsOf(seen.nav)
    val active = tabs.indexOfFirst { it.screen.value == current }.coerceAtLeast(0)
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    // A page is drawn from what was kept and then from the fresh answer. The focus belongs to the page.
    // Each page keeps where the viewer was for the run of the app, so coming back from a title lands
    // on the tile that opened it.
    val memory = LocalPages.current
    val focus = remember(current, seen.route?.genre) {
        memory.getOrPut("${seen.active} $current ${seen.route?.genre.orEmpty()}") { PageFocus(rows, tabs.size, active, lead != null) } as PageFocus
    }
    focus.rows = rows
    focus.tabs = tabs.size
    focus.hero = lead != null
    val store = rememberDetails()
    val rise = with(LocalDensity.current) { RISE.ref.roundToPx() }
    var menu by remember { mutableStateOf(false) }

    fun choose() {
        when {
            focus.where == Where.TOP && focus.tab == WHO_TAB -> menu = true
            focus.where == Where.TOP && focus.tab == SEARCH_TAB -> shell.go(Address.SEARCH)
            focus.where == Where.TOP -> tabs.getOrNull(focus.tab)?.let { shell.go(tabAddress(it.screen.value)) }
            focus.where == Where.ROW -> rows.getOrNull(focus.row)?.tiles?.getOrNull(focus.column)?.let {
                shell.go(titleAddress(it.title))
            }
            lead == null -> Unit
            focus.button == 1 || lead.isSeries() -> shell.go(titleAddress(lead.title))
            else -> shell.go(watchAddress(lead.title))
        }
    }

    fun press(word: Word): Boolean = when {
        word == Word.OK -> {
            choose()
            true
        }
        focus.press(word) -> true
        // Back from the top of any page but Home goes Home. From Home it leaves the app.
        word == Word.BACK && current != "home" -> {
            shell.go(Address.HOME)
            true
        }
        else -> false
    }

    Box(Modifier.fillMaxSize().remote(!menu, @DontMemoize { press(it) })) {
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
                    // A row sliding away keeps the tile it was left on.
                    OpenRow(shell, seen.words, store, it, focus.columnOf(row))
                }
            }
        } else if (lead != null) {
            HeroCard(shell, seen, store, lead, if (focus.where == Where.HERO) focus.button else -1)
        } else if (rows.isEmpty()) {
            Text(
                seen.words[if (current == "games") "games.soon" else "mine.empty"].orEmpty(),
                Modifier.align(Alignment.Center),
                fontSize = 32.refType,
                color = Muted,
            )
        }
        val below = if (inRow) focus.row + 1 else 0
        rows.getOrNull(below)?.let { NextRow(it, !inRow, focus.columnOf(below)) }
    }
    if (menu) ProfileMenu(shell, seen, person) { menu = false }
}
