package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import streams.shared.Accent
import streams.shared.Corner
import streams.shared.R
import streams.shared.decoration
import streams.shared.fill
import streams.shared.says
import streams.shared.saysItsWords

/** The top bar's height (reference: 51.9 dp) and the chip row's (73.4 dp, chips 49.2 tall). */
const val TOP_BAR = 52
const val CHIP_ROW = 73
private const val CHIP_HIGH = 49
private const val OUTLINE_HIGH = 43

/**
 * The top of a main page, pinned: the brand's mark and the page's name on the left, the bell on the
 * right (reference: mark box 48 x 44 at 3.9 dp, name from 51.9 dp, 44 dp buttons; 48 dp Android touch targets).
 * On a page reached from a chip (Series, Films) the mark gives way to a back arrow, as the reference's does.
 */
@Composable
fun TopBar(
    server: String,
    name: String,
    back: (() -> Unit)?,
    bell: (() -> Unit)?,
    unreadCount: Int = 0,
    words: Map<String, String> = emptyMap(),
) {
    val layoutDirection = LocalLayoutDirection.current
    Row(
        Modifier.fillMaxWidth().height(TOP_BAR.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // Either a back arrow, which is a control and needs words, or the brand's mark, which is
        // decoration: the page's own name is drawn next to it and says the same thing.
        Box(
            Modifier.padding(start = 4.dp).size(48.dp, 48.dp)
                .let { if (back != null) it.clickable(onClick = back).says(words["common.back"].orEmpty()) else it.decoration() },
            contentAlignment = Alignment.Center,
        ) {
            if (back != null) {
                Icon(
                    painterResource(R.drawable.icon_back),
                    null,
                    Modifier.size(24.dp).graphicsLayer {
                        if (layoutDirection == LayoutDirection.Rtl) scaleX = -1f
                    },
                    tint = Color.White,
                )
            } else {
                AsyncImage("$server/mark.svg", null, Modifier.size(30.dp))
            }
        }
        Text(
            name,
            Modifier.weight(1f).then(if (bell == null) Modifier.padding(end = 8.dp) else Modifier),
            color = Color.White,
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (bell != null) {
            // The bell says how many notices are waiting, because the count is drawn on it.
            val bellSays = if (unreadCount > 0) {
                words["say.notifications_unread"].orEmpty().fill("n", unreadCount)
            } else {
                words["notifications.title"].orEmpty()
            }
            Box(
                Modifier.size(48.dp).clickable(onClick = bell).says(bellSays),
                contentAlignment = Alignment.Center,
            ) {
                Icon(painterResource(R.drawable.icon_bell), null, Modifier.size(24.dp), tint = Color.White)
                if (unreadCount > 0) {
                    val countText = if (unreadCount > 9) "9+" else unreadCount.toString()
                    Box(
                        Modifier
                            .align(Alignment.TopEnd)
                            .padding(top = 8.dp, end = 6.dp)
                            .background(Accent, RoundedCornerShape(Corner.BUTTON.dp))
                            .padding(horizontal = 4.dp, vertical = 1.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = countText,
                            color = Color.White,
                            fontSize = 10.sp,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                        )
                    }
                }
            }
            Spacer(Modifier.width(4.dp))
        }
    }
}

/** The chips' words, the reference's grey (191, 191, 190); black on a chosen chip. */
private val ChipInk = Color(0xFFBFBFBE)

/** One chip of the row: what it says, whether it is chosen, and where it goes. */
data class Chip(val word: String, val chosen: Boolean, val drops: Boolean = false, val go: () -> Unit)

/**
 * The chip row under the top bar, as the reference's: 49 dp chips from 16 dp in, 6 dp apart, 16 dp
 * padding, a 10 % white fill under a 1 dp edge of 10 % white, words 15 sp medium. The row reads as
 * one group: the first chip's outer side and the last chip's are round (brand.json
 * `chip_end_radius`), every other corner takes `chip_radius`. Start and end follow the reading
 * direction, so the group mirrors in Arabic and Hebrew.
 */
@Composable
fun ChipRow(chips: List<Chip>, shown: Float) {
    val outline = chips.size == 1 && chips[0].drops
    val high = if (outline) OUTLINE_HIGH else CHIP_HIGH
    val inner = Corner.CHIP.dp
    val outer = minOf(Corner.CHIP_END, high / 2).dp
    LazyRow(
        Modifier.fillMaxWidth().height(CHIP_ROW.dp).alpha(shown),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
        contentPadding = PaddingValues(horizontal = 16.dp),
    ) {
        // By place, not by word: the row always opens at its first chip, however the menu arrives.
        // The word is the key -- two chips of one row never carry the same one -- and whether a chip
        // drops a menu is its type, so the row's chips are reused rather than rebuilt.
        itemsIndexed(
            chips,
            key = { _, chip -> chip.word },
            contentType = { _, chip -> chip.drops },
        ) { index, chip ->
            val shape = if (outline) {
                RoundedCornerShape(inner)
            } else {
                val first = index == 0
                val last = index == chips.lastIndex
                RoundedCornerShape(
                    topStart = if (first) outer else inner,
                    bottomStart = if (first) outer else inner,
                    topEnd = if (last) outer else inner,
                    bottomEnd = if (last) outer else inner,
                )
            }
            val fill = when {
                chip.chosen -> Color.White
                outline -> Color.Transparent
                else -> Color.White.copy(alpha = 0.10f)
            }
            val edge = when {
                chip.chosen -> Color.Transparent
                outline -> Color.White.copy(alpha = 0.40f)
                else -> Color.White.copy(alpha = 0.10f)
            }
            val ink = if (chip.chosen) Color.Black else ChipInk
            Row(
                Modifier
                    .height(high.dp)
                    .clip(shape)
                    .background(fill)
                    .border(1.dp, edge, shape)
                    .clickable(onClick = chip.go)
                    // A chip draws its own word, and the caret beside it is drawn inside this node.
                    .saysItsWords()
                    .padding(horizontal = 16.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    chip.word,
                    color = ink,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    softWrap = false,
                )
                if (chip.drops) {
                    Spacer(Modifier.width(4.dp))
                    Icon(painterResource(R.drawable.icon_caret_down), null, Modifier.size(18.dp), tint = ink)
                }
            }
        }
    }
}

/** The header block: top bar and chip row together, the chips sliding away as the page scrolls down. */
@Composable
fun Header(top: @Composable () -> Unit, chips: @Composable () -> Unit) {
    Column { top(); chips() }
}
