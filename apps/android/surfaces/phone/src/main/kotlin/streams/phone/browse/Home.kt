package streams.phone

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.ActionCommand
import contract.Address
import contract.CommandList
import contract.SectionKind
import contract.Tile
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.R
import streams.shared.detailOf
import streams.shared.isSeries
import streams.shared.rememberDetails
import streams.shared.tabAddress
import streams.shared.titleAddress
import streams.shared.watchAddress

/** Before the billboard's colours are read, the page wears our blue, darkened. */
private val FirstTint = Color(0xFF1A2A55)

/** The pages the chips offer, in the reference's order; the rest of the menu lives in the bar. */
private val ChipScreens = listOf("series", "films", "games", "new")

/**
 * A main page (Home, Series, Films): the pinned header, the billboard, the rows, and the floating
 * bar, laid out and behaving as the reference's. The page behind the header takes the billboard's
 * colour at the top and turns black as it scrolls; the chips slide away while the page scrolls down
 * and come back as soon as it scrolls up.
 */
@Composable
fun Home(shell: Shell, seen: Seen) {
    val words = seen.words
    val sections = seen.screen?.sections.orEmpty()
    val lead = sections.firstOrNull { it.kind == SectionKind.HERO }?.tiles?.firstOrNull()
    val rows = sections.filter { it.kind in RowKinds }
    val here = seen.route?.screen?.value ?: "home"
    val onHome = here == "home" || here == "kids"
    val title = seen.nav.firstOrNull { it.screen.value == here && it.genre == seen.route?.genre }?.label
        ?: seen.nav.firstOrNull { it.screen.value == here }?.label.orEmpty()

    var tint by remember { mutableStateOf(FirstTint) }
    val shownTint by animateColorAsState(tint, tween(600), label = "tint")
    var chipsHidden by remember { mutableStateOf(false) }
    val chipShift by animateFloatAsState(if (chipsHidden) 1f else 0f, tween(220), label = "chips")
    val list = rememberLazyListState()
    val atTop by remember { derivedStateOf { list.firstVisibleItemIndex == 0 && list.firstVisibleItemScrollOffset < 24 } }
    val scroll = remember {
        object : NestedScrollConnection {
            override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset {
                if (available.y < -2f) chipsHidden = true
                if (available.y > 2f) chipsHidden = false
                return Offset.Zero
            }
        }
    }
    val store = rememberDetails()
    val detail = lead?.let { detailOf(shell, store, it.title).value }
    var saved by remember(lead?.title) { mutableStateOf<Boolean?>(null) }
    val inList = saved ?: detail?.inList ?: false
    var categories by remember { mutableStateOf(false) }
    val top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
    val chipHigh = with(LocalDensity.current) { CHIP_ROW.dp.toPx() }

    fun open(tile: Tile) = shell.go(titleAddress(tile.title))

    Box(Modifier.fillMaxSize().background(Back).nestedScroll(scroll)) {
        // The billboard's colour behind the top of the page, fading to black.
        Box(
            Modifier
                .fillMaxWidth()
                .height(top + 640.dp)
                .background(Brush.verticalGradient(0f to shownTint.copy(alpha = 0.95f), 1f to Back)),
        )
        LazyColumn(
            state = list,
            contentPadding = PaddingValues(top = top + TOP_BAR.dp + CHIP_ROW.dp, bottom = 120.dp),
        ) {
            if (lead != null) {
                item(key = "billboard") {
                    Billboard(
                        lead,
                        words,
                        inList,
                        play = { shell.go(if (lead.isSeries()) titleAddress(lead.title) else watchAddress(lead.title)) },
                        list = {
                            saved = !inList
                            shell.act(ActionCommand(CommandList(inList = !inList, title = lead.title)))
                        },
                        open = { open(lead) },
                        tint = { tint = it },
                    )
                }
                item(key = "billboard-gap") { Box(Modifier.height(16.dp)) }
            }
            itemsIndexed(rows, key = { _, section -> section.id }) { index, section ->
                SectionRow(section, words, tallTiles = index == 0 && section.kind == SectionKind.ROW, open = ::open)
            }
        }
        // The pinned header: clear over the colour at the top, black once the page moves.
        Column(
            Modifier
                .fillMaxWidth()
                .background(if (atTop) Color.Transparent else Back.copy(alpha = 0.97f))
                .statusBarsPadding(),
        ) {
            TopBar(
                seen.server,
                title,
                back = if (onHome) null else ({ shell.back() }),
                bell = { shell.go(Address.NOTIFICATIONS) },
            )
            Box(
                Modifier
                    .height(CHIP_ROW.dp * (1f - chipShift))
                    .graphicsLayer { translationY = -chipShift * chipHigh },
            ) {
                ChipRow(chips(seen, onHome, { shell.go(tabAddress(it)) }) { categories = true }, 1f - chipShift)
                DropdownMenu(categories, { categories = false }) {
                    seen.nav.filter { it.screen.value == "genre" }.sortedBy { it.label }.forEach { item ->
                        DropdownMenuItem(
                            text = { Text(item.label, fontSize = 16.sp) },
                            onClick = {
                                categories = false
                                shell.go(Address.GENRE + item.genre.orEmpty())
                            },
                            modifier = Modifier.height(56.dp),
                        )
                    }
                }
            }
        }
        BottomBar(
            bar(shell, seen, onHome),
            Modifier.align(Alignment.BottomCenter).navigationBarsPadding().padding(bottom = 9.dp),
        )
    }
}

/** The chips: on Home the pages the reference lists, on a page the Categories menu alone. */
private fun chips(seen: Seen, onHome: Boolean, go: (String) -> Unit, openCategories: () -> Unit): List<Chip> {
    val pages = if (onHome) {
        ChipScreens.mapNotNull { screen ->
            seen.nav.firstOrNull { it.screen.value == screen }?.let { item -> Chip(item.label, chosen = false) { go(screen) } }
        }
    } else {
        emptyList()
    }
    return pages + Chip(seen.words["nav.categories"].orEmpty(), chosen = false, drops = true, go = openCategories)
}

/** The bar: Home, Clips when the server offers it, Search and My Streams with the profile's face. */
private fun bar(shell: Shell, seen: Seen, onHome: Boolean): List<BarItem> = buildList {
    add(BarItem(R.drawable.icon_home, seen.nav.firstOrNull { it.screen.value == "home" }?.label.orEmpty(), onHome) { shell.go(Address.HOME) })
    seen.nav.firstOrNull { it.screen.value == "clips" }?.let { add(BarItem(R.drawable.icon_clips, it.label, false) { shell.go(Address.CLIPS) }) }
    add(BarItem(R.drawable.icon_search, seen.words["nav.search"].orEmpty(), false) { shell.go(Address.SEARCH) })
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    seen.nav.firstOrNull { it.screen.value == "mine" }?.let {
        add(BarItem(R.drawable.icon_user, it.label, false, face = person, server = seen.server) { shell.go(Address.MINE) })
    }
}
