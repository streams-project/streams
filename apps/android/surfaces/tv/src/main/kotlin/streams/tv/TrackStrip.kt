package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import streams.shared.Back
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

private const val STRIP_X = 476
private const val STRIP_Y = 958

/** The row under the bar. */
@Composable
fun TrackStrip(choices: List<Choice>, focus: Int) {
    Row(Modifier.offset(STRIP_X.ref, STRIP_Y.ref)) {
        choices.forEachIndexed { index, choice ->
            val on = index == focus
            val ink = if (on) Back else Color.White
            Row(
                Modifier
                    .padding(end = 14.ref)
                    .height(54.ref)
                    .background(if (on) Color.White else Color.White.copy(alpha = 0.16f), CircleShape)
                    .padding(horizontal = 24.ref),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (choice.ticked) {
                    Icon(painterResource(R.drawable.icon_check), null, Modifier.size(26.ref), tint = ink)
                    Spacer(Modifier.width(10.ref))
                }
                Text(choice.word, color = ink, fontSize = 24.refType, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}
