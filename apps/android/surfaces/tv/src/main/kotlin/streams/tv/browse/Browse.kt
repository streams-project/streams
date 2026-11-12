package streams.tv

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import contract.Address
import contract.More
import contract.Section
import contract.Tile
import contract.contractJson
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.rememberDetails
import streams.shared.tabAddress
import streams.shared.titleAddress
import streams.shared.watchAddress
import streams.tv.browse.BrowseView
import streams.tv.browse.Feed
import streams.tv.browse.Place

/** Where the viewer was on each page of rows, for the run of the app. */
val LocalPages = staticCompositionLocalOf { HashMap<String, Any>() }

/**
 * The rows a page has past its first answer. The server builds up to 50 rows and hands a screen 12
 * with a cursor to read the rest; until this, nothing outside `apps/web` followed that cursor and 38
 * built rows could not be reached from a television (notes/netflix-reference/00-gaps.md, gap 1).
 */
private class PageGrowth {
    /** The cursor the first answer came with. A fresh answer carries a new one and starts the page again. */
    var from: String? = null

    /** The next cursor to read, or nothing when the page has given everything it has. */
    var next by mutableStateOf<String?>(null)
    var busy = false
    var extra by mutableStateOf<List<Section>>(emptyList())
}

/**
 * A page of rows.
 *
 * ## What is left in this file, and what moved
 * The page itself is `browse/BrowseView.java`: the layout, the key map, the focus memory, the
 * measured geometry, what a screen reader is handed and the drawing. What is left here is the data
 * seam -- reading the contract, which Java cannot (`browse/Reading.kt` says why, with the `javap`
 * output), and the two pieces of suspending work this page does.
 *
 * The screen is reached through one `AndroidView`. Note 210's "what is easier in Views" item 1
 * already names four such wrappers in this surface that exist only to hold a View which already is
 * one, and says they become `addView` in wave 3, when `app/Main.kt`'s `setContent` becomes
 * `setContentView` of the television root. This is the fifth, with the same stated end. Keeping it
 * for the length of wave 2 is the point of note 210's risk row for diverging surfaces: the APK
 * carries both toolkits and is installable and testable at every commit.
 *
 * ## The two effects, and why they did not move
 * `LaunchedEffect` is a coroutine with cancellation, and both of these need one. The paging read
 * (`:198-216` of the Compose original) is the one note 210 names as the dangerous one, because it
 * cancels and restarts when the focused row changes; the detail read goes through
 * `shared/Details.kt`'s `produceState`, which settles for 240 ms before asking so that holding an
 * arrow along a row does not ask for twenty titles. Rewriting either as explicit lifecycle code is
 * wave 3's work on the shell, where the activity's scope lives, and doing it here would mean this
 * package owning a cancellation model that no other screen has yet.
 *
 * What replaces recomposition in the meantime is one integer: the Java screen calls `moved()` when
 * the remote has gone somewhere, this reads it, and both effects re-key off what the page now holds.
 * That is the explicit redraw note 210 asks for, in the one direction Compose cannot be told about.
 */
@Composable
fun Browse(shell: Shell, seen: Seen) {
    // No answer in hand yet: the page waits with its destination lit, rather than drawing nothing.
    val data = seen.screen ?: return Waiting(shell, seen)
    val current = seen.route?.screen?.value ?: "home"
    // A page is drawn from what was kept and then from the fresh answer. The focus belongs to the page.
    // Each page keeps where the viewer was for the run of the app, so coming back from a title lands
    // on the tile that opened it, and keeps the rows it has already read past the first twelve.
    val memory = LocalPages.current
    val page = "${seen.active} $current ${seen.route?.genre.orEmpty()}"
    val growth = remember(current, seen.route?.genre) { memory.getOrPut("more $page") { PageGrowth() } as PageGrowth }
    // Which rows this page may draw, and whether the answer in hand is even its own: Games has no
    // rows yet, and the kept page behind it belongs to the tab before. Both answers are in Page.kt,
    // apart from the drawing, because `src/test/kotlin/streams/tv/PageTest.kt` checks them, and both
    // read `SectionKind`, which is one of the 184 getters Java cannot call.
    val asked = seen.route?.screen?.value
    val mine = answerIsThisPage(current, asked, data.name.value)
    val rows = rowsOn(current, asked, data.name.value, data.sections, growth.extra)
    val lead = heroOn(current, data.sections)
    val tabs = tabsOf(seen.nav)
    val active = tabs.indexOfFirst { it.screen.value == current }.coerceAtLeast(0)
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    val store = rememberDetails()
    var menu by remember { mutableStateOf(false) }
    // Bumped by the Java screen when the remote has moved, so the two reads below follow it.
    var moved by remember { mutableIntStateOf(0) }
    val pictures = remember(shell) { ShellArt(shell) }
    // `parts/Badge.kt:21`'s `keptWords` was assigned from inside `OpenRow` (`Rows.kt:117`), and
    // `TileBadges` takes it as the default for its words, so `title/PosterGrid.kt:64` drew the
    // title page's badges with whatever words the last browse page had left behind. Converting
    // `OpenRow` out of the drawing path removed the only writer a viewer reaches before a title
    // page, which would have left those badges wordless until the viewer happened to open a help
    // screen (`help/Problem.kt:118` is the other writer). Kept here, one line, until T3 and T4
    // convert those two packages and the global goes with them.
    keptWords = seen.words

    // A page with no hero and no rows of its own says so, in the viewer's language. Which of the two
    // words it is, is this page's business and not the screen's (`Browse.kt:291` as it stood).
    val emptyKey = if (rows.isEmpty() && lead == null) {
        if (current == "games") "games.soon" else "mine.empty"
    } else {
        ""
    }
    val feed = feedOf(tabs, active, person, seen.server, rows, lead, seen.words, emptyKey)
    val place = remember(current, seen.route?.genre) {
        memory.getOrPut(page) { Place(feed, active) } as Place
    }

    // `moved` is a key here and nothing else, and that is the whole of what replaces a
    // recomposition on this page: the Java screen bumps it when the remote has gone somewhere, this
    // reads it, and the detail read below follows the remote. Without `moved` as a key the page
    // would draw correctly and never read another title's details again -- the exact silent failure
    // note 210 names as the largest source of new defects, in the one place it can still happen.
    val onTile = remember(moved, place, rows, lead) {
        if (place.where() == Place.Where.ROW) {
            rows.getOrNull(place.row())?.tiles.orEmpty().getOrNull(place.columnOf(place.row()))
        } else {
            lead
        }
    }

    val screen = remember { mutableStateOf<BrowseView?>(null) }

    // A fresh answer is a fresh page: what was read past its first rows belongs to the answer it came from.
    LaunchedEffect(data.cursor, mine) {
        if (!mine) return@LaunchedEffect
        if (growth.from != data.cursor) {
            growth.from = data.cursor
            growth.next = data.cursor
            growth.extra = emptyList()
            growth.busy = false
        }
    }
    // The page reads its next rows as the focus comes within reach of the last one it holds. A remote
    // cannot flick past twelve rows, so the rows arrive before they are wanted rather than on a gesture.
    LaunchedEffect(place.wantsMore(), growth.extra.size, growth.next, mine) {
        if (!mine) return@LaunchedEffect
        val cursor = growth.next ?: return@LaunchedEffect
        // A page whose first answer carried no rows at all can only grow by asking; any other page
        // waits until the focus is within reach of the last row it already holds. That answer is
        // `Place.wantsMore`, which carries `REACH = 2` and its derivation.
        if (growth.busy || !place.wantsMore()) return@LaunchedEffect
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

    Box(Modifier.fillMaxSize()) {
        AndroidView(
            factory = { context -> BrowseView(context, pictures).also { screen.value = it } },
            modifier = Modifier.fillMaxSize(),
        ) { view ->
            view.mirror(seen.rtl)
            view.speak(seen.words)
            view.asks(asking(shell, current, rows, lead, { menu = it }, { moved += 1 }))
            view.show(place, feed, emptyKey)
            // The remote goes to the overlay and comes back, and the page keeps its place meanwhile.
            view.setLive(!menu)
        }
    }
    if (onTile != null) {
        val detail by streams.shared.detailOf(shell, store, onTile.title)
        LaunchedEffect(detail, screen.value) {
            screen.value?.told(storyOf(detail, lead, seen.words))
        }
    } else {
        LaunchedEffect(screen.value) { screen.value?.told(null) }
    }
    if (menu) ProfileMenu(shell, seen, person) { menu = false }
}

/**
 * What the screen asks of the shell: where OK goes, and what Back does with nowhere left to go.
 *
 * The five branches are `Browse.kt:219-229`'s as it stood, and the address builders are
 * `shared/Addresses.kt`'s own -- `titleAddress`, `watchAddress` and `tabAddress` take contract value
 * types, so the tile is found again from its key here rather than the address being spelled out in
 * Java. One statement of how an address is built, which is what `check_patterns.py` protects.
 */
private fun asking(
    shell: Shell,
    current: String,
    rows: List<Section>,
    lead: Tile?,
    menu: (Boolean) -> Unit,
    onMoved: () -> Unit,
): BrowseView.Asked = object : BrowseView.Asked {
    override fun chose(what: Place.Chose, tab: Feed.Tab?, card: Feed.Card?) {
        when (what) {
            Place.Chose.MENU -> menu(true)
            Place.Chose.SEARCH -> shell.go(Address.SEARCH)
            Place.Chose.TAB -> tab?.let { shell.go(tabAddress(it.screen)) }
            Place.Chose.TITLE -> tileOf(rows, lead, card)?.let { shell.go(titleAddress(it.title)) }
            Place.Chose.WATCH -> tileOf(rows, lead, card)?.let { shell.go(watchAddress(it.title)) }
            Place.Chose.NOTHING -> Unit
        }
    }

    override fun leave() {
        // Back from the top of any page but Home goes Home, and on Home it stops here: the app stays
        // open. What is measured is the general fact - "Back never exits the app. Exit is a menu item"
        // (netflix-reference/tv/00-netflix-on-the-tv-measured.md, section 10 item 8, with note 184's
        // Exit Netflix line). The four BACK presses in raw/key-probes.csv were on the screen reached
        // after profile selection, not on Home; section 8 of that note says Home was never reached, so
        // nothing here is claimed about Back on Netflix's own Home. The way out is the Exit line at the
        // foot of the profile menu behind the face.
        // OWNER: 00-gaps.md gap 7 keeps "whether Back on Home should exit" as his decision, and this
        // branch also swallows Back on the kids home (Target.KIDS_HOME leaves route.screen null, so
        // `current` falls back to "home"), which no capture covers. Left as it stands for him.
        if (current != "home") shell.go(Address.HOME)
    }

    override fun moved() = onMoved()
}

/** The tile a flat card came from, so an address is built from the contract's own key. */
private fun tileOf(rows: List<Section>, lead: Tile?, card: Feed.Card?): Tile? {
    val key = card?.key ?: return null
    rows.forEach { section ->
        section.tiles.orEmpty().forEach { if (it.title.value == key) return it }
    }
    return lead?.takeIf { it.title.value == key }
}
