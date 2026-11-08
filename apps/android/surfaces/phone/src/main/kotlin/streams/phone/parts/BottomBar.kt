package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.Profile
import streams.shared.Accent
import streams.shared.Art
import streams.shared.Corner
import streams.shared.Join

/**
 * One place on the bar: its icon, its word, whether it is the page in view, and where it goes.
 * The bar shows the profile's face in place of an icon, as the reference shows the viewer's.
 */
data class BarItem(val icon: Int, val word: String, val here: Boolean, val face: Profile? = null, val server: String = "", val go: () -> Unit)

/** The bar's plate, the chosen item's plate and the idle words, as the reference's. */
private val BarFill = Color(0xFF2D2D2D)
private val ChosenFill = Color(0xFF383838)
private val IdleInk = Color(0xFFC0C0C0)

/**
 * The bar at the foot of every main page, floating as the reference's does. It sits in the reading
 * direction, so it turns with a right to left language. All shapes and corners come from brand.json
 * (`Corner.BUTTON`), never hardcoded CircleShape.
 */
@Composable
fun BottomBar(items: List<BarItem>, modifier: Modifier = Modifier) {
    val barShape = RoundedCornerShape(minOf(Corner.BUTTON, 32).dp)
    val itemShape = RoundedCornerShape(minOf(Corner.BUTTON, 24).dp)
    val minItemWidth = if (items.size > 4) 58.dp else 66.dp

    Row(
        modifier
            .height(63.dp)
            .background(BarFill, barShape)
            .border(1.dp, Color.White.copy(alpha = 0.1f), barShape)
            .padding(horizontal = 7.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Center,
    ) {
        items.forEach { item ->
            Box(
                Modifier
                    .defaultMinSize(minWidth = minItemWidth)
                    .height(48.dp)
                    .padding(horizontal = 2.dp)
                    .clip(itemShape)
                    .background(if (item.here) ChosenFill else Color.Transparent)
                    .clickable(onClick = item.go)
                    .padding(horizontal = 6.dp),
                contentAlignment = Alignment.Center,
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center,
                ) {
                    if (item.face != null) {
                        BarFace(item.face, item.server)
                    } else {
                        Icon(painterResource(item.icon), item.word, Modifier.size(24.dp), tint = if (item.here) Color.White else IdleInk)
                    }
                    Spacer(Modifier.height(2.dp))
                    Text(
                        item.word,
                        color = if (item.here) Color.White else IdleInk,
                        fontSize = 11.sp,
                        maxLines = 1,
                        softWrap = false,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

/** The profile's face at the bar's icon size: its picture, or its first letter on its colour. */
@Composable
private fun BarFace(person: Profile, server: String) {
    val avatar = person.avatar?.takeIf { it.startsWith("/") || it.startsWith("http") }
    Box(
        Modifier.size(24.dp).clip(RoundedCornerShape(6.dp)).background(if (person.kids) Join else Accent),
        contentAlignment = Alignment.Center,
    ) {
        if (avatar != null) {
            Art(if (avatar.startsWith("/")) server + avatar else avatar, Modifier.fillMaxSize())
        } else {
            Text(person.name.take(1).uppercase(), color = Color.White, fontSize = 13.sp, fontWeight = FontWeight.ExtraBold)
        }
    }
}
