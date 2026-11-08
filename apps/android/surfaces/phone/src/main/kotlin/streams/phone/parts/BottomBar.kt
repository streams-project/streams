package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import contract.Profile
import streams.shared.Accent
import streams.shared.Art
import streams.shared.Face
import streams.shared.Join

/**
 * One place on the bar: its icon, its word, whether it is the page in view, and where it goes. My
 * The bar shows the profile's face in place of an icon, as the reference shows the viewer's.
 */
data class BarItem(val icon: Int, val word: String, val here: Boolean, val face: Profile? = null, val server: String = "", val go: () -> Unit)

/** The bar's plate, the chosen item's plate and the idle words, as the reference's. */
private val BarFill = Color(0xFF2D2D2D)
private val ChosenFill = Color(0xFF383838)
private val IdleInk = Color(0xFFC0C0C0)

/**
 * The bar at the foot of every main page, floating as the reference's does. It sits in the reading
 * direction, so it turns with a right to left language.
 */
@Composable
fun BottomBar(items: List<BarItem>, modifier: Modifier = Modifier) {
    Row(
        modifier
            .height(63.dp)
            .background(BarFill, CircleShape)
            .border(1.dp, Color.White.copy(alpha = 0.1f), CircleShape)
            .padding(horizontal = 7.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        items.forEach { item ->
            Box(
                Modifier
                    .size(width = 76.dp, height = 48.dp)
                    .background(if (item.here) ChosenFill else Color.Transparent, CircleShape)
                    .clickable(onClick = item.go),
                contentAlignment = Alignment.Center,
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    if (item.face != null) {
                        BarFace(item.face, item.server)
                    } else {
                        Icon(painterResource(item.icon), item.word, Modifier.size(24.dp), tint = if (item.here) Color.White else IdleInk)
                    }
                    Text(item.word, color = if (item.here) Color.White else IdleInk, fontSize = 11.sp, maxLines = 1)
                }
            }
        }
    }
}

/** The profile's face at the bar's icon size: its picture, or its first letter on its colour. */
@Composable
private fun BarFace(person: Profile, server: String) {
    Face(
        person = person,
        server = server,
        modifier = Modifier.size(24.dp),
        shape = RoundedCornerShape(6.dp),
        fontSize = 13.sp,
    )
}
