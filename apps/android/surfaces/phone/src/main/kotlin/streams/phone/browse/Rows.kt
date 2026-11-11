package streams.phone

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalTextStyle
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import contract.BadgeKind
import contract.Section
import contract.SectionKind
import contract.Tile
import streams.shared.Accent
import streams.shared.Art
import streams.shared.Back
import streams.shared.Corner
import streams.shared.R
import streams.shared.Raised
import streams.shared.fill
import streams.shared.number
import streams.shared.tall
import streams.shared.warm

/** The rows a page shows, in the server's order. */
val RowKinds = setOf(SectionKind.ROW, SectionKind.RANKED, SectionKind.CONTINUE, SectionKind.GRID)

/**
 * Tiles on the reference: 119 x 166.6 dp (5 : 7), every row the same. Measured on 2026-11-11 with
 * `uiautomator`, where row 1 reads 305 x 427 px like every row after it: "the first row is not
 * taller". This replaces the 184.6 x 369.2 dp panel the first row used to draw, which came from the
 * 2026-10-04 reading in `netflix-reference/00-design-tokens.md:86`.
 */
internal val TileWide = 119.dp
internal val TileHigh = 166.6.dp

/** Between tiles, and the row's start from the screen's edge (reference: 5.9 and 12.1 dp). */
internal val TileGap = 6.dp
internal val RowStart = 12.dp

/** The picture of a Continue Watching tile, which carries its controls under the art. */
private val ContinueHigh = 138.dp

/** Pictures read ahead of the last tile on screen, as the television surface reads its next posters. */
private const val WARM_AHEAD = 6

/**
 * One row: its name over a sideways list of tiles (reference: the name 34.3 dp high, bold, 18 sp; 4 dp
 * to the tiles; 12 dp to the next row).
 * Top 10 row shows large outlined rank numerals beside each tile. Continue Watching shows progress
 * and action buttons. Row title tap opens the row as a full page grid when it has more tiles.
 *
 * The row is as deep as the page has read it, and reads further as the viewer flings: the reference's
 * standard row runs to 70 titles and this project's server holds 120, against the 15 one answer
 * carries.
 */
@Composable
fun SectionRow(
    section: Section,
    words: Map<String, String>,
    depth: Depth,
    open: (Tile) -> Unit,
    openInfo: ((Tile) -> Unit)? = null,
    onRemove: ((Tile) -> Unit)? = null,
    openGrid: ((Section) -> Unit)? = null,
) {
    val rawTiles = depth.tiles(section)
    var hiddenTitles by remember { mutableStateOf(setOf<String>()) }
    val tiles = rawTiles.filter { it.title.value !in hiddenTitles }
    if (tiles.isEmpty()) return

    val hasMore = section.cursor != null || rawTiles.size > 10
    var gridOpen by remember { mutableStateOf(false) }

    fun onTitleTap() {
        if (hasMore) {
            if (openGrid != null) openGrid(section) else gridOpen = true
        }
    }

    // How far the viewer has flung this row. It decides two things: when the next part of the row is
    // read, and which pictures are fetched before anything draws them.
    val row = rememberLazyListState()
    val context = LocalContext.current
    val tileBox = with(LocalDensity.current) { TileWide.roundToPx() to TileHigh.roundToPx() }
    val reached by remember(row) {
        derivedStateOf { row.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0 }
    }
    LaunchedEffect(section.id, tiles.size, reached) {
        depth.reach(section, tiles.size, reached)
        for (place in reached + 1..reached + WARM_AHEAD) {
            warm(context, tall(tiles.getOrNull(place)?.images), tileBox.first, tileBox.second)
        }
    }

    Column(Modifier.fillMaxWidth().padding(bottom = 12.dp)) {
        Box(
            modifier = Modifier
                .defaultMinSize(minHeight = 34.dp)
                .padding(horizontal = RowStart)
                .then(if (hasMore) Modifier.clickable(onClick = ::onTitleTap) else Modifier),
            contentAlignment = Alignment.CenterStart,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = section.title.orEmpty(),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f, fill = false),
                )
                if (hasMore) {
                    Spacer(Modifier.width(6.dp))
                    Text(
                        text = if (LocalLayoutDirection.current == LayoutDirection.Rtl) "‹" else "›",
                        color = Color.White.copy(alpha = 0.7f),
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }
        }
        Spacer(Modifier.height(4.dp))
        LazyRow(
            state = row,
            contentPadding = PaddingValues(horizontal = RowStart),
            horizontalArrangement = Arrangement.spacedBy(TileGap),
        ) {
            // Every tile of one row is the same kind, so the toolkit can hand a tile that has left
            // the screen to the next row of that kind instead of building another one mid fling.
            itemsIndexed(
                tiles,
                key = { _, tile -> tile.title.value },
                contentType = { _, _ -> section.kind.value },
            ) { index, tile ->
                when (section.kind) {
                    SectionKind.RANKED -> {
                        val rank = tile.badges.orEmpty().firstOrNull { it.rank != null }?.rank ?: (index + 1).toLong()
                        RankedTileCard(
                            rank = rank,
                            tile = tile,
                            words = words,
                            open = { open(tile) },
                        )
                    }
                    SectionKind.CONTINUE -> {
                        ContinueCard(
                            tile = tile,
                            words = words,
                            open = { open(tile) },
                            openInfo = { openInfo?.invoke(tile) ?: open(tile) },
                            onRemove = {
                                hiddenTitles = hiddenTitles + tile.title.value
                                onRemove?.invoke(tile)
                            },
                        )
                    }
                    else -> {
                        val rank = tile.badges.orEmpty().firstOrNull { it.rank != null }?.rank
                        TileCard(
                            tile = tile,
                            rank = rank,
                            percent = null,
                            words = words,
                            open = { open(tile) },
                        )
                    }
                }
            }
        }
    }

    if (gridOpen) {
        FullPageGrid(
            title = section.title.orEmpty(),
            tiles = tiles,
            kind = section.kind.value,
            words = words,
            reach = { held, place -> depth.reach(section, held, place) },
            onClose = { gridOpen = false },
            open = open,
        )
    }
}

/**
 * Top 10 row item: draws a large outlined rank numeral (80-100 dp high) beside the tile,
 * with the poster tile overlapping the numeral on top. Digits follow the language.
 */
@Composable
private fun RankedTileCard(
    rank: Long,
    tile: Tile,
    words: Map<String, String>,
    open: () -> Unit,
) {
    val rankText = number(rank)
    val isDoubleDigit = rank >= 10
    val extraWide = if (isDoubleDigit) 34.dp else 26.dp
    val itemWide = TileWide + extraWide
    val numeralWide = if (isDoubleDigit) 68.dp else 48.dp
    val numeralHigh = 92.dp

    Box(
        modifier = Modifier.size(itemWide, TileHigh),
        contentAlignment = Alignment.BottomStart,
    ) {
        // Large outlined rank numeral at start
        Box(
            modifier = Modifier
                .width(numeralWide)
                .height(numeralHigh),
            contentAlignment = Alignment.CenterStart,
        ) {
            val strokeWidthPx = with(LocalDensity.current) { 3.dp.toPx() }
            Text(
                text = rankText,
                fontSize = 90.sp,
                fontWeight = FontWeight.Black,
                color = Color(0xFF1E1E1E),
                maxLines = 1,
            )
            Text(
                text = rankText,
                fontSize = 90.sp,
                fontWeight = FontWeight.Black,
                style = LocalTextStyle.current.copy(
                    drawStyle = Stroke(width = strokeWidthPx),
                ),
                color = Color(0xFF9E9E9E),
                maxLines = 1,
            )
        }

        // Poster Tile at end, on top of numeral
        Box(modifier = Modifier.align(Alignment.BottomEnd)) {
            TileCard(
                tile = tile,
                rank = null,
                percent = null,
                words = words,
                open = open,
            )
        }
    }
}

/**
 * Continue Watching tile: art with progress bar, plus info and menu buttons under the art.
 */
@Composable
private fun ContinueCard(
    tile: Tile,
    words: Map<String, String>,
    open: () -> Unit,
    openInfo: () -> Unit,
    onRemove: () -> Unit,
) {
    var menuOpen by remember { mutableStateOf(false) }
    val shape = RoundedCornerShape(Corner.CARD.dp)
    Column(
        modifier = Modifier
            .width(TileWide)
            .clip(shape)
            .background(Raised),
    ) {
        // Art with progress bar
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(ContinueHigh)
                .clickable(onClick = open),
        ) {
            Art(tall(tile.images), Modifier.fillMaxSize(), TileWide, ContinueHigh)
            val percent = tile.percent ?: 0L
            Box(
                Modifier
                    .align(Alignment.BottomStart)
                    .fillMaxWidth()
                    .height(3.dp)
                    .background(Color.White.copy(alpha = 0.3f)),
            ) {
                Box(
                    Modifier
                        .fillMaxWidth(percent.coerceIn(0L, 100L) / 100f)
                        .height(3.dp)
                        .background(Accent),
                )
            }
        }
        // Control strip under the art
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(38.dp)
                .background(Raised)
                .padding(horizontal = 4.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(34.dp)
                    .clip(RoundedCornerShape(17.dp))
                    .clickable(onClick = openInfo),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painter = painterResource(R.drawable.icon_info),
                    contentDescription = words["title.about"] ?: "Info",
                    modifier = Modifier.size(20.dp),
                    tint = Color.White,
                )
            }
            Box(
                modifier = Modifier
                    .size(34.dp)
                    .clip(RoundedCornerShape(17.dp))
                    .clickable { menuOpen = true },
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painter = painterResource(R.drawable.icon_more),
                    contentDescription = null,
                    modifier = Modifier.size(20.dp),
                    tint = Color.White,
                )
                DropdownMenu(
                    expanded = menuOpen,
                    onDismissRequest = { menuOpen = false },
                ) {
                    DropdownMenuItem(
                        text = {
                            Text(
                                words["row.continue.remove"] ?: "Remove from row",
                                fontSize = 14.sp,
                                color = Color.White,
                            )
                        },
                        onClick = {
                            menuOpen = false
                            onRemove()
                        },
                    )
                }
            }
        }
    }
}

/**
 * One tile: the title's picture at 119 x 166.6 dp, rounded by the brand (reference about 6 dp).
 *
 * There was a second shape here, a 184.6 x 369.2 dp panel for the first row carrying a "TOP 10"
 * corner badge and a caption chip reading "New Episode" or "Watch Now". The 2026-11-11 reading of
 * the reference has no tall first row and no text on a tile at all, so both are gone.
 */
@Composable
private fun TileCard(
    tile: Tile,
    rank: Long?,
    percent: Long?,
    words: Map<String, String>,
    open: () -> Unit,
) {
    val shape = RoundedCornerShape(Corner.CARD.dp)
    Box(Modifier.size(TileWide, TileHigh).clip(shape).background(Raised).clickable(onClick = open)) {
        Art(tall(tile.images), Modifier.fillMaxSize(), TileWide, TileHigh)

        if (rank != null) {
            Text(
                words["badge.ranked"].orEmpty().fill("rank", rank),
                Modifier
                    .padding(6.dp)
                    .background(Accent, RoundedCornerShape(6.dp))
                    .padding(horizontal = 6.dp, vertical = 2.dp),
                color = Color.White,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
            )
        }
        if (tile.badges.orEmpty().any { it.kind == BadgeKind.NEW }) {
            Text(
                words["badge.new"] ?: "New",
                Modifier
                    .align(Alignment.TopStart)
                    .padding(6.dp)
                    .background(Accent, RoundedCornerShape(4.dp))
                    .padding(horizontal = 5.dp, vertical = 2.dp),
                color = Color.White,
                fontSize = 10.sp,
                fontWeight = FontWeight.Bold,
            )
        }
        if (percent != null) {
            Box(
                Modifier
                    .align(Alignment.BottomStart)
                    .fillMaxWidth()
                    .height(3.dp)
                    .background(Color.White.copy(alpha = 0.3f)),
            ) {
                Box(Modifier.fillMaxWidth(percent.coerceIn(0L, 100L) / 100f).height(3.dp).background(Accent))
            }
        }
    }
}

/**
 * Full-page grid opened when row title is tapped. It reads the rest of the row as the viewer
 * scrolls, through the same cursor the row itself uses, so the grid reaches the whole row and not
 * only the part the page arrived with.
 */
@Composable
private fun FullPageGrid(
    title: String,
    tiles: List<Tile>,
    kind: String,
    words: Map<String, String>,
    reach: (held: Int, place: Int) -> Unit,
    onClose: () -> Unit,
    open: (Tile) -> Unit,
) {
    Dialog(
        onDismissRequest = onClose,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        BackHandler(onBack = onClose)
        val layoutDir = LocalLayoutDirection.current
        Column(
            modifier = Modifier
                .fillMaxSize()
                .background(Back)
                .statusBarsPadding(),
        ) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(52.dp)
                    .padding(horizontal = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    modifier = Modifier
                        .size(48.dp, 48.dp)
                        .clickable(onClick = onClose),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painter = painterResource(R.drawable.icon_back),
                        contentDescription = words["common.back"],
                        modifier = Modifier.size(24.dp).graphicsLayer {
                            if (layoutDir == LayoutDirection.Rtl) scaleX = -1f
                        },
                        tint = Color.White,
                    )
                }
                Spacer(Modifier.width(8.dp))
                Text(
                    text = title,
                    color = Color.White,
                    fontSize = 20.sp,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f),
                )
            }
            val grid = rememberLazyGridState()
            val reached by remember(grid) {
                derivedStateOf { grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0 }
            }
            LaunchedEffect(tiles.size, reached) { reach(tiles.size, reached) }
            LazyVerticalGrid(
                columns = GridCells.Fixed(3),
                state = grid,
                contentPadding = PaddingValues(horizontal = RowStart, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(TileGap),
                verticalArrangement = Arrangement.spacedBy(TileGap),
                modifier = Modifier.fillMaxSize(),
            ) {
                items(tiles, key = { it.title.value }, contentType = { kind }) { tile ->
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(TileHigh)
                            .clip(RoundedCornerShape(Corner.CARD.dp))
                            .background(Raised)
                            .clickable {
                                onClose()
                                open(tile)
                            },
                    ) {
                        Art(tall(tile.images), Modifier.fillMaxSize(), TileWide, TileHigh)
                    }
                }
            }
        }
    }
}
