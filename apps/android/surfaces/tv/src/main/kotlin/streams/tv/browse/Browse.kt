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
import androidx.compose.runtime.LaunchedEffect
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
import contract.More
import contract.Section
import contract.SectionKind
import contract.Tile
import contract.TitleKey
import contract.contractJson
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Muted
import streams.shared.ref
import streams.shared.refType

private const val RISE = 810
private const val RISE_MS = 320

/**
 * How many rows past the focused one the page keeps in hand. A television screen holds one open row
 * and the title plus 82 px of the next (netflix-reference/tv geometry, `row_screen.vertical_layout_px`),
 * so the second row below focus is the first one the viewer cannot already see.
 */
private const val REACH = 2

private enum class Where { TOP, HERO, ROW }

/** Where the viewer was on each page of rows, for the run of the app. */
val LocalPages = staticCompositionLocalOf { HashMap<String, Any>() }

private val RowKinds = setOf(SectionKind.ROW, SectionKind.RANKED, SectionKind.CONTINUE, SectionKind.GRID)

/**
 * The rows a page has past its first answer. The server builds up to 50 rows and hands a screen 12
 * with a cursor to read the rest; until this, nothing outside `apps/web` followed that cursor and 38
 * built rows could not be reached from a television (notes/netflix-reference/00-gaps.md, gap 1).
 */
private class PageGrowth {
    /** The cursor the first answer came with. A fresh answer carries a new one and starts the page again. */
    var from: String? = null

    /** The next cursor to read, or nothing when the page has given everything it has. */
    var next: String? = null
    var busy = false
    var extra by mutableStateOf<List<Section>>(emptyList())
}

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
    // A page is drawn from what was kept and then from the fresh answer. The focus belongs to the page.
    // Each page keeps where the viewer was for the run of the app, so coming back from a title lands
    // on the tile that opened it, and keeps the rows it has already read past the first twelve.
    val memory = LocalPages.current
    val page = "${seen.active} $current ${seen.route?.genre.orEmpty()}"
    val growth = remember(current, seen.route?.genre) { memory.getOrPut("more $page") { PageGrowth() } as PageGrowth }
    // Games has no rows yet, and the kept page behind it belongs to the tab before.
    val sections = if (current == "games") emptyList() else data.sections.filter { !it.tiles.isNullOrEmpty() }
    val rows = (sections + growth.extra).filter { it.kind in RowKinds }
    val lead = sections.firstOrNull { it.kind == SectionKind.HERO }?.tiles?.first()
    val tabs = tabsOf(seen.nav)
    val active = tabs.indexOfFirst { it.screen.value == current }.coerceAtLeast(0)
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    val focus = remember(current, seen.route?.genre) {
        memory.getOrPut(page) { PageFocus(rows, tabs.size, active, lead != null) } as PageFocus
    }
    focus.rows = rows
    focus.tabs = tabs.size
    focus.hero = lead != null
    val store = rememberDetails()
    val rise = with(LocalDensity.current) { RISE.ref.roundToPx() }
    var menu by remember { mutableStateOf(false) }

    // A fresh answer is a fresh page: what was read past its first rows belongs to the answer it came from.
    LaunchedEffect(data.cursor) {
        if (growth.from != data.cursor) {
            growth.from = data.cursor
            growth.next = data.cursor
            growth.extra = emptyList()
            growth.busy = false
        }
    }
    // The page reads its next rows as the focus comes within reach of the last one it holds. A remote
    // cannot flick past twelve rows, so the rows arrive before they are wanted rather than on a gesture.
    LaunchedEffect(focus.row, focus.where, growth.extra.size, current) {
        val cursor = growth.next ?: return@LaunchedEffect
        // A page whose first answer carried no rows at all can only grow by asking; any other page
        // waits until the focus is within reach of the last row it already holds.
        val within = rows.isEmpty() || (focus.where == Where.ROW && focus.row >= rows.lastIndex - REACH)
        if (growth.busy || !within) return@LaunchedEffect
        growth.busy = true
        try {
            val body = shell.read("/v1/more/$cursor") ?: return@LaunchedEffect
            val more = runCatching { contractJson.decodeFromString(More.serializer(), body) }.getOrNull()
                ?: return@LaunchedEffect
            // An answer with no rows left is the end, whatever cursor came with it.
            growth.next = if (more.sections.isNullOrEmpty()) null else more.cursor
            growth.extra = growth.extra + more.sections.orEmpty().filter { !it.tiles.isNullOrEmpty() }
        } finally {
            growth.busy = false
        }
    }

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
        // Back from the top of any page but Home goes Home, and on Home it stops here: the app stays
        // open. Netflix on a television answered four Back presses without leaving its activity
        // (netflix-reference/tv/00-netflix-on-the-tv-measured.md, section 7, n=4); the way out is the
        // Exit line at the foot of the profile menu behind the face.
        word == Word.BACK -> {
            if (current != "home") shell.go(Address.HOME)
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
