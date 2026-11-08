package streams.tv

import streams.shared.Art
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import contract.Profile
import streams.shared.Accent
import streams.shared.Join
import streams.shared.ref
import streams.shared.refType

/** A profile's picture, or its first letter when it has none. Every screen draws this one part. */
@Composable
fun Face(person: Profile?, size: Int, server: String, modifier: Modifier = Modifier) {
    val shape = Round.face
    val avatar = person?.avatar?.takeIf { it.startsWith("/") || it.startsWith("http") }
    val plate = if (person?.kids == true) Join else Accent
    Box(modifier.clip(shape).background(plate), contentAlignment = Alignment.Center) {
        if (avatar != null) {
            Art(if (avatar.startsWith("/")) server + avatar else avatar, Modifier.fillMaxSize())
        } else {
            Text(
                person?.name?.take(1)?.uppercase().orEmpty(),
                fontSize = (size * 46 / 100).refType,
                fontWeight = FontWeight.ExtraBold,
                maxLines = 1,
                overflow = TextOverflow.Clip,
            )
        }
    }
}
