package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
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
import streams.shared.Back
import streams.shared.ref
import streams.shared.refType

/** One line of a menu. */
data class MenuLine(val word: String, val icon: Int? = null)

/**
 * Plain rows of words. The one in focus is a white pill, and `kept` marks the line another part of
 * the page belongs to while the focus is there. Pages share this as it stands.
 */
@Composable
fun Menu(lines: List<MenuLine>, focus: Int, modifier: Modifier = Modifier, width: Int = 395, kept: Int = -1) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(6.ref)) {
        lines.forEachIndexed { index, line ->
            val on = index == focus
            val ink = if (on) Back else Color.White
            Box(
                Modifier
                    .width(width.ref)
                    .height(63.ref)
                    .background(
                        when {
                            on -> Color.White
                            index == kept -> Color.White.copy(alpha = 0.14f)
                            else -> Color.Transparent
                        },
                        CircleShape,
                    ),
            ) {
                line.icon?.let {
                    Icon(
                        painterResource(it),
                        null,
                        Modifier.align(Alignment.CenterStart).offset(33.ref).size(34.ref),
                        tint = ink,
                    )
                }
                Text(
                    line.word,
                    Modifier.align(Alignment.CenterStart).offset(90.ref),
                    color = ink,
                    fontSize = 30.refType,
                )
            }
        }
    }
}
