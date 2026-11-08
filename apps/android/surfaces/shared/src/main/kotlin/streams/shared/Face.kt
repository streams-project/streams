package streams.shared

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.TextUnit
import contract.Profile

/** A profile's picture, or its first letter on Accent (Join for Kids) when it has none. */
@Composable
fun Face(
    person: Profile?,
    server: String,
    modifier: Modifier = Modifier,
    shape: Shape? = null,
    fontSize: TextUnit = TextUnit.Unspecified,
) {
    val avatar = person?.avatar?.takeIf { it.startsWith("/") || it.startsWith("http") }
    val plate = if (person?.kids == true) Join else Accent
    val boxMod = if (shape != null) modifier.clip(shape) else modifier
    Box(boxMod.background(plate), contentAlignment = Alignment.Center) {
        if (avatar != null) {
            Art(if (avatar.startsWith("/")) server + avatar else avatar, Modifier.fillMaxSize())
        } else {
            Text(
                person?.name?.take(1)?.uppercase().orEmpty(),
                color = Color.White,
                fontSize = fontSize,
                fontWeight = FontWeight.ExtraBold,
                maxLines = 1,
                overflow = TextOverflow.Clip,
            )
        }
    }
}
