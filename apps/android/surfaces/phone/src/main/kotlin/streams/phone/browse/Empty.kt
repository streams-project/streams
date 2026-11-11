package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import streams.shared.Corner
import streams.shared.Muted
import streams.shared.Raised

/**
 * What a page shows before it has anything, and what it says when it never will.
 *
 * Until today this surface drew its two bars and nothing else: no message, no retry, no sign that it
 * was waiting. The reference, measured on 2026-11-11, draws a row's name and its tiles as flat
 * rounded rectangles at the final 119 x 166.6 dp, with no shimmer, and for a row further down a
 * plain bar where the name goes. That is what is here. The words for the state after the wait are
 * the ones the television surface of this same family already uses.
 */

/** Rows a phone screen holds: 936.6 dp of screen over a 217.0 dp row pitch is 4.3. */
private const val WAITING_ROWS = 4

/** A placeholder name bar: the reference's row name box is 30 dp high inside a 34.3 dp block. */
private val BarHigh = 18.dp
private val BarWide = 148.dp

/** How long a page may stay empty before the screen says so. The television surface waits the same. */
const val OFFLINE_WAIT_MS = 8_000L

/**
 * A page with nothing in it yet: names as bars, tiles flat at their final size. Nothing moves, as
 * nothing moves on the reference.
 */
@Composable
fun WaitingRows(modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth()) {
        repeat(WAITING_ROWS) {
            Column(Modifier.fillMaxWidth().padding(bottom = 12.dp)) {
                Box(
                    Modifier
                        .padding(horizontal = RowStart)
                        .height(34.dp),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    Box(
                        Modifier
                            .size(BarWide, BarHigh)
                            .clip(RoundedCornerShape(Corner.CARD.dp))
                            .background(Raised),
                    )
                }
                Spacer(Modifier.height(4.dp))
                Row(
                    Modifier.fillMaxWidth().padding(start = RowStart),
                    horizontalArrangement = Arrangement.spacedBy(TileGap),
                ) {
                    // Four tiles, so the fourth is cut by the screen's edge as the reference's is.
                    repeat(4) {
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

/**
 * A page that cannot be reached: what is wrong, what to do about it, and the one button that tries
 * again. The heading, the line and the button word are `problem.offline`, `problem.offline.help`
 * and `problem.retry`, already written in every language this app speaks.
 */
@Composable
fun Unreachable(words: Map<String, String>, notice: String?, retry: () -> Unit) {
    Box(Modifier.fillMaxSize().padding(horizontal = 32.dp), contentAlignment = Alignment.Center) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                words["problem.offline"].orEmpty(),
                color = Color.White,
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
            )
            Spacer(Modifier.height(8.dp))
            Text(
                notice?.takeIf { it.isNotEmpty() } ?: words["problem.offline.help"].orEmpty(),
                color = Muted,
                fontSize = 15.sp,
                textAlign = TextAlign.Center,
            )
            Spacer(Modifier.height(20.dp))
            // The phone's button, as the billboard's: 48 dp high, the brand's corner, white on black.
            Row(
                Modifier
                    .height(48.dp)
                    .clip(RoundedCornerShape(minOf(Corner.BUTTON, 24).dp))
                    .background(Color.White)
                    .clickable(onClick = retry)
                    .padding(horizontal = 24.dp),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    words["problem.retry"].orEmpty(),
                    color = Color.Black,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                )
            }
        }
    }
}
