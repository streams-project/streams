package streams.phone

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.ActionCommand
import contract.Address
import contract.CommandHide
import contract.CommandList
import contract.SectionKind
import contract.Target
import contract.Tile
import java.text.Collator
import java.util.Locale
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Corner
import streams.shared.R
import streams.shared.detailOf
import streams.shared.isSeries
import streams.shared.rememberDetails
import streams.shared.says
import streams.shared.saysItsWords
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
    val here = seen.route?.screen?.value ?: "home"
    val onHome = here == "home" || here == "kids"
    val pickedGenre = if (here == "genre") {
        seen.nav.firstOrNull { it.screen.value == "genre" && it.genre == seen.route?.genre }
    } else null
    val title = pickedGenre?.label
        ?: seen.nav.firstOrNull { it.screen.value == here }?.label
        ?: words["app.title"].orEmpty()

    val screenKey = if (onHome) "home" else "$here:${seen.route?.genre.orEmpty()}"
    val scrollStates = remember { mutableMapOf<String, LazyListState>() }
    val list = scrollStates.getOrPut(screenKey) { LazyListState() }

    // The rows this page has, which is more than the answer carried: twelve arrive and the server
    // has built up to fifty, so the page reads the rest through the screen's own cursor as the
    // viewer travels down it. The same for a row's depth, which is held per page so a row keeps it.
    // One stamp for both halves of the reading, so a rebuilt answer resets the page's extra rows
    // and each row's depth together rather than leaving the depths pointing into the old answer.
    val stamp = pageStamp(screenKey, seen.screen)
    val sections = rememberPage(shell, seen.screen, stamp, list)
    val depth = rememberDepth(shell, stamp)
    val lead = sections.firstOrNull { it.kind == SectionKind.HERO }?.tiles?.firstOrNull()
    val rows = sections.filter { it.kind in RowKinds }

    // Whether what is on screen is this route's page. Until it is, the page draws the placeholder
    // rows the reference draws, and goes on drawing them: the reference puts nothing else in an
    // empty body, however long it stays empty (`netflix-reference/00-gaps.md:128-132`).
    val wanted = when (seen.route?.target) {
        Target.HOME -> "home"
        Target.KIDS_HOME -> "kids"
        else -> seen.route?.screen?.value
    }
    val hasScreen = seen.screen != null && (wanted == null || seen.screen?.name?.value == wanted)

    var tint by remember { mutableStateOf(FirstTint) }
    val shownTint by animateColorAsState(tint, tween(600), label = "tint")
    var chipsHidden by remember { mutableStateOf(false) }
    val chipShift by animateFloatAsState(if (chipsHidden) 1f else 0f, tween(220), label = "chips")
    LaunchedEffect(screenKey) { chipsHidden = false }

    val atTop by remember(list) { derivedStateOf { list.firstVisibleItemIndex == 0 && list.firstVisibleItemScrollOffset < 24 } }
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

    BackHandler(enabled = categories || !onHome) {
        if (categories) {
            categories = false
        } else {
            shell.back()
        }
    }

    fun open(tile: Tile) = shell.go(titleAddress(tile.title))

    val collator = remember(seen.language) {
        Collator.getInstance(Locale.forLanguageTag(seen.language.ifEmpty { "en" }))
    }
    val genres = remember(seen.nav, collator) {
        seen.nav.filter { it.screen.value == "genre" }
            .sortedWith { a, b -> collator.compare(a.label, b.label) }
    }

    Box(Modifier.fillMaxSize().background(Back).nestedScroll(scroll)) {
        // The billboard's colour behind the top of the page, fading to black.
        Box(
            Modifier
                .fillMaxWidth()
                .height(top + 640.dp)
                .background(Brush.verticalGradient(0f to shownTint.copy(alpha = 0.95f), 1f to Back)),
        )
        val headroom = PaddingValues(top = top + TOP_BAR.dp + CHIP_ROW.dp, bottom = 120.dp)
        if (!hasScreen && rows.isEmpty()) {
            WaitingRows(Modifier.padding(headroom))
        }
        LazyColumn(
            state = list,
            contentPadding = headroom,
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
            // The kind is the item's type, so the toolkit reuses a row of one kind for the next row
            // of that kind instead of building a new one in the middle of a fling.
            itemsIndexed(
                rows,
                key = { _, section -> section.id },
                contentType = { _, section -> section.kind.value },
            ) { _, section ->
                SectionRow(
                    section = section,
                    words = words,
                    depth = depth,
                    open = ::open,
                    openInfo = { shell.go(titleAddress(it.title)) },
                    onRemove = { shell.act(ActionCommand(CommandHide(it.title))) },
                )
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
                unreadCount = seen.unreadNotifications,
                words = words,
            )
            Box(
                Modifier
                    .height(CHIP_ROW.dp * (1f - chipShift))
                    .graphicsLayer { translationY = -chipShift * chipHigh },
            ) {
                ChipRow(chips(seen, onHome, here, pickedGenre?.label, { shell.go(tabAddress(it)) }) { categories = true }, 1f - chipShift)
            }
        }
        BottomBar(
            bar(shell, seen, onHome),
            Modifier.align(Alignment.BottomCenter).navigationBarsPadding().padding(bottom = 9.dp),
        )

        // Categories full-screen overlay (replacing DropdownMenu):
        AnimatedVisibility(
            visible = categories,
            enter = fadeIn(tween(250)),
            exit = fadeOut(tween(200)),
        ) {
            Box(
                Modifier
                    .fillMaxSize()
                    .background(Back.copy(alpha = 0.90f)),
            ) {
                LazyColumn(
                    Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(top = top + 40.dp, bottom = 120.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    items(genres, key = { it.id }) { item ->
                        val isChosen = here == "genre" && item.genre == seen.route?.genre
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .clickable {
                                    categories = false
                                    shell.go(Address.GENRE + item.genre.orEmpty())
                                }
                                .saysItsWords()
                                .padding(vertical = 12.dp, horizontal = 24.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                text = item.label,
                                color = if (isChosen) Color.White else Color(0xFFC0C0C0),
                                fontSize = 21.sp,
                                fontWeight = if (isChosen) FontWeight.Bold else FontWeight.Medium,
                                textAlign = TextAlign.Center,
                            )
                        }
                    }
                }
                // Round white/grey close button (X) at bottom center (56 dp round circle)
                Box(
                    Modifier
                        .align(Alignment.BottomCenter)
                        .navigationBarsPadding()
                        .padding(bottom = 24.dp)
                        .size(56.dp)
                        .clip(RoundedCornerShape(minOf(Corner.BUTTON, 28).dp))
                        .background(Color.White)
                        .clickable { categories = false }
                        // The cross is two lines on a canvas: there is nothing here for a reader to
                        // gather, so this control is given words.
                        .says(words["say.close"].orEmpty()),
                    contentAlignment = Alignment.Center,
                ) {
                    Canvas(Modifier.size(20.dp)) {
                        val strokeWidth = 2.5f.dp.toPx()
                        drawLine(
                            color = Back,
                            start = Offset(0f, 0f),
                            end = Offset(size.width, size.height),
                            strokeWidth = strokeWidth,
                            cap = StrokeCap.Round,
                        )
                        drawLine(
                            color = Back,
                            start = Offset(0f, size.height),
                            end = Offset(size.width, 0f),
                            strokeWidth = strokeWidth,
                            cap = StrokeCap.Round,
                        )
                    }
                }
            }
        }
    }
}

/** The chips: on Home the pages the reference lists, on a page the Categories menu alone. */
private fun chips(
    seen: Seen,
    onHome: Boolean,
    here: String,
    pickedGenreLabel: String?,
    go: (String) -> Unit,
    openCategories: () -> Unit,
): List<Chip> {
    if (onHome) {
        val pages = ChipScreens.mapNotNull { screen ->
            seen.nav.firstOrNull { it.screen.value == screen }?.let { item ->
                Chip(item.label, chosen = false) { go(screen) }
            }
        }
        return pages + Chip(seen.words["nav.categories"].orEmpty(), chosen = false, drops = true, go = openCategories)
    }
    val label = pickedGenreLabel ?: seen.words["nav.categories"].orEmpty()
    return listOf(Chip(label, chosen = false, drops = true, go = openCategories))
}

/** The bar: Home, Clips when the server offers it, Search and the personal tab with the profile's face. */
private fun bar(shell: Shell, seen: Seen, onHome: Boolean): List<BarItem> = buildList {
    add(BarItem(R.drawable.icon_home, seen.nav.firstOrNull { it.screen.value == "home" }?.label.orEmpty(), onHome) { shell.go(Address.HOME) })
    seen.nav.firstOrNull { it.screen.value == "clips" }?.let { add(BarItem(R.drawable.icon_clips, it.label, false) { shell.go(Address.CLIPS) }) }
    add(BarItem(R.drawable.icon_search, seen.words["nav.search"].orEmpty(), false) { shell.go(Address.SEARCH) })
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    seen.nav.firstOrNull { it.screen.value == "mine" }?.let {
        add(BarItem(R.drawable.icon_user, it.label, false, face = person, server = seen.server) { shell.go(Address.MINE) })
    }
}
