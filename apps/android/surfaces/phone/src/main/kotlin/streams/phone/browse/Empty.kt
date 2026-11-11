package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.unit.dp
import streams.shared.Corner
import streams.shared.Raised
import kotlin.math.ceil

/**
 * What a page shows before it has anything.
 *
 * Until today this surface drew its two bars and nothing else. The reference, measured on
 * 2026-11-11, draws a row's name and its tiles as flat rounded rectangles at the final
 * 119 x 166.6 dp, with no shimmer, and for a row further down a plain bar where the name goes
 * (`netflix-reference/00-gaps.md:128-132`). That is what is here, and that is all that is here:
 * the reference shows no message, no retry and no sign of waiting while a page is empty, so
 * neither does this.
 */

/** Rows a phone screen holds: 936.6 dp of screen over a 217.0 dp row pitch is 4.3. */
private const val WAITING_ROWS = 4

/**
 * The placeholder name bar, measured off the reference capture itself
 * (`netflix-reference/phone/02-home-scrolled-placeholders.png`, 1080 x 2400 at 2.5625 px/dp): the
 * bar runs y 1543-1621, which is 79 px, and x 31-1048, which starts and ends on the row's own
 * 12.1 dp gutter. So it is 30.8 dp high -- the measured row name box is 30 high inside a 34.3 dp
 * block -- and as wide as the row.
 */
private val BarHigh = 30.8.dp

/**
 * A page with nothing in it yet: names as bars, tiles flat at their final size. Nothing moves, as
 * nothing moves on the reference.
 */
@Composable
fun WaitingRows(modifier: Modifier = Modifier) {
    val screenWide = LocalConfiguration.current.screenWidthDp
    val across = ceil((screenWide - RowStart.value) / (TileWide.value + TileGap.value)).toInt().coerceAtLeast(1)
    Column(modifier.fillMaxWidth()) {
        repeat(WAITING_ROWS) {
            Column(Modifier.fillMaxWidth().padding(bottom = 12.dp)) {
                Box(
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = RowStart)
                        .height(34.dp),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    Box(
                        Modifier
                            .fillMaxWidth()
                            .height(BarHigh)
                            .clip(RoundedCornerShape(Corner.CARD.dp))
                            .background(Raised),
                    )
                }
                Spacer(Modifier.height(4.dp))
                Row(
                    Modifier.fillMaxWidth().padding(start = RowStart),
                    horizontalArrangement = Arrangement.spacedBy(TileGap),
                ) {
                    // As many tiles as the row holds, the last one cut by the edge as the
                    // reference's is: the screen less the row's 12.1 dp start, over the 124.9 dp
                    // tile pitch, rounded up. On the reference's 421.5 dp phone that is four --
                    // three full and a cut fourth, which is the measured 3.29 visible.
                    repeat(across) {
                        Box(
                            Modifier
                                .size(TileWide, TileHigh)
                                .clip(RoundedCornerShape(Corner.CARD.dp))
                                .background(Raised),
                        )
                    }
                }
            }
        }
    }
}
