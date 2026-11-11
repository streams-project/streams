package streams.tv

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import streams.shared.Back
import streams.shared.ref
import streams.shared.refType

private const val CHANGE_MS = 120
private val Quiet = Color.White.copy(alpha = 0.16f)

/**
 * A large button. In focus it turns white.
 *
 * Its word is its label for a screen reader (`saysItsWords`), and the icon beside it is decoration:
 * it repeats the word. A pill with no word at all draws nothing rather than an empty white shape -
 * the dead end measured on the Streamer on 2026-11-11 was exactly that, a `Pill("")` on black with
 * every word around it missing (notes/206-what-each-surface-does-with-no-server-2026-11-11.md).
 */
@Composable
fun Pill(word: String, on: Boolean, icon: Int? = null) {
    if (word.isEmpty() && icon == null) return
    val back by animateColorAsState(if (on) Color.White else Quiet, tween(CHANGE_MS, easing = LinearEasing))
    val ink by animateColorAsState(if (on) Back else Color.White, tween(CHANGE_MS, easing = LinearEasing))
    Row(
        Modifier.height(84.ref).background(back, CircleShape).padding(horizontal = 46.ref).saysItsWords(on),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (icon != null) {
            Icon(painterResource(icon), null, Modifier.size(34.ref), tint = ink)
            Spacer(Modifier.width(12.ref))
        }
        Text(word, color = ink, fontSize = 32.refType, fontWeight = FontWeight.Bold)
    }
}
