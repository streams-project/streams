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
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
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
import streams.shared.isSeries
import streams.shared.number
import streams.shared.tall

/** The rows a page shows, in the server's order. */
val RowKinds = setOf(SectionKind.ROW, SectionKind.RANKED, SectionKind.CONTINUE, SectionKind.GRID)

/** Tiles on the reference: standard 119 x 166.6 dp (5 : 7), the first row's tall panels 184.6 x 369.2. */
private val TileWide = 119.dp
private val TileHigh = 166.6.dp
private val TallWide = 184.6.dp
private val TallHigh = 369.2.dp

/** Between tiles, and the row's start from the screen's edge (reference: 5.9 and 12.1 dp). */
private val TileGap = 6.dp
private val RowStart = 12.dp

/**
 * One row: its name over a sideways list of tiles (reference: the name 34.3 dp high, bold, 18 sp; 4 dp
 * to the tiles; 12 dp to the next row). The page's first row shows tall panels, as the reference's.
 * Top 10 row shows large outlined rank numerals beside each tile. Continue Watching shows progress
 * and action buttons. Row title tap opens the row as a full page grid when it has more tiles.
 */
@Composable
fun SectionRow(
    section: Section,
    words: Map<String, String>,
    tallTiles: Boolean,
    open: (Tile) -> Unit,
    openInfo: ((Tile) -> Unit)? = null,
    onRemove: ((Tile) -> Unit)? = null,
    openGrid: ((Section) -> Unit)? = null,
) {
    val rawTiles = section.tiles.orEmpty()
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

    Column(Modifier.fillMaxWidth().padding(bottom = 12.dp)) {
        Box(
            modifier = Modifier
                .height(34.dp)
                .padding(start = RowStart)
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
                )
                if (hasMore) {
                    Spacer(Modifier.width(6.dp))
                    Text(
                        text = "›",
                        color = Color.White.copy(alpha = 0.7f),
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }
        }
        Spacer(Modifier.height(4.dp))
        LazyRow(
            contentPadding = PaddingValues(horizontal = RowStart),
            horizontalArrangement = Arrangement.spacedBy(TileGap),
        ) {
            itemsIndexed(tiles, key = { _, tile -> tile.title.value }) { index, tile ->
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
                            wide = if (tallTiles) TallWide else TileWide,
                            high = if (tallTiles) TallHigh else TileHigh,
                            rank = rank,
                            percent = null,
                            tall = tallTiles,
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
            words = words,
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
                wide = TileWide,
                high = TileHigh,
                rank = null,
                percent = null,
                tall = false,
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
                .height(138.dp)
                .clickable(onClick = open),
        ) {
            Art(tall(tile.images), Modifier.fillMaxSize())
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
 * One tile: the title's picture, rounded by the brand (reference about 6 dp).
 * Tall tiles in the first row display the TOP 10 corner badge and lower caption chip.
 */
@Composable
private fun TileCard(
    tile: Tile,
    wide: Dp,
    high: Dp,
    rank: Long?,
    percent: Long?,
    tall: Boolean,
    words: Map<String, String>,
    open: () -> Unit,
) {
    val shape = RoundedCornerShape(Corner.CARD.dp)
    Box(Modifier.size(wide, high).clip(shape).background(Raised).clickable(onClick = open)) {
        Art(tall(tile.images), Modifier.fillMaxSize())

        if (tall) {
            // "TOP 10" corner badge on tall first row tile
            if (rank != null || tile.badges.orEmpty().any { it.rank != null || it.kind == BadgeKind.RANKED }) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopStart)
                        .background(
                            Accent,
                            RoundedCornerShape(
                                topStart = Corner.CARD.dp,
                                bottomEnd = 6.dp,
                            ),
                        )
                        .padding(horizontal = 7.dp, vertical = 4.dp),
                ) {
                    Text(
                        text = words["badge.top_10"] ?: "TOP 10",
                        color = Color.White,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }
            // Caption chip on lower edge ("New Episode", "Watch Now")
            val caption = if (tile.isSeries() || tile.badges.orEmpty().any { it.kind == BadgeKind.NEW }) {
                words["badge.new_episode"] ?: words["badge.new"] ?: "New Episode"
            } else {
                words["badge.watch_now"] ?: "Watch Now"
            }
            Box(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = 10.dp)
                    .background(Color.Black.copy(alpha = 0.75f), RoundedCornerShape(4.dp))
                    .padding(horizontal = 8.dp, vertical = 3.dp),
            ) {
                Text(
                    text = caption,
                    color = Color.White,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold,
                )
            }
        } else {
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
}

/** Full-page grid opened when row title is tapped. */
@Composable
private fun FullPageGrid(
    title: String,
    tiles: List<Tile>,
    words: Map<String, String>,
    onClose: () -> Unit,
    open: (Tile) -> Unit,
) {
    Dialog(
        onDismissRequest = onClose,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        BackHandler(onBack = onClose)
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
                        .size(48.dp, 44.dp)
                        .clickable(onClick = onClose),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painter = painterResource(R.drawable.icon_back),
                        contentDescription = "Back",
                        modifier = Modifier.size(24.dp),
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
                )
            }
            LazyVerticalGrid(
                columns = GridCells.Fixed(3),
                contentPadding = PaddingValues(horizontal = RowStart, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(TileGap),
                verticalArrangement = Arrangement.spacedBy(TileGap),
                modifier = Modifier.fillMaxSize(),
            ) {
                items(tiles, key = { it.title.value }) { tile ->
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
                        Art(tall(tile.images), Modifier.fillMaxSize())
                    }
                }
            }
        }
    }
}
