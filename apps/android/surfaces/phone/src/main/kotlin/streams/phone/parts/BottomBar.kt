package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import streams.shared.Plate
import streams.shared.R

/** One place on the bar: its icon, its word, and whether it is the page in view. */
data class BarItem(val icon: Int, val word: String, val here: Boolean, val go: () -> Unit)

/**
 * The bar at the foot of every main page, floating as the reference's does: a rounded plate over the
 * page, icons above short words, the page in view in white and the others dimmed. It sits in the
 * reading direction, so it turns with a right to left language.
 */
@Composable
fun BottomBar(items: List<BarItem>, modifier: Modifier = Modifier) {
    Row(
        modifier
            .widthIn(max = 420.dp)
            .height(64.dp)
            .background(Plate.copy(alpha = 0.92f), CircleShape)
            .border(1.dp, Color.White.copy(alpha = 0.12f), CircleShape)
            .padding(horizontal = 8.dp),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        items.forEach { item ->
            val ink = if (item.here) Color.White else Color.White.copy(alpha = 0.6f)
            Box(
                Modifier
                    .size(width = 84.dp, height = 56.dp)
                    .background(if (item.here) Color.White.copy(alpha = 0.12f) else Color.Transparent, CircleShape)
                    .clickable(onClick = item.go),
                contentAlignment = Alignment.Center,
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(painterResource(item.icon), item.word, Modifier.size(24.dp), tint = ink)
                    Text(item.word, color = ink, fontSize = 11.sp, maxLines = 1)
                }
            }
        }
    }
}
