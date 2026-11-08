package streams.tv

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
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
import contract.Profile
import streams.shared.Back
import streams.shared.Muted
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

const val FACE_LARGE = 194
private const val FACE_SMALL = 130
private const val GROW_MS = 200

/** One profile in the column. The one in focus grows in place, with its name and the pencil. */
@Composable
fun ProfileRow(person: Profile, on: Boolean, pencil: Boolean, server: String) {
    val time = tween<androidx.compose.ui.unit.Dp>(GROW_MS, easing = Glide)
    val size by animateDpAsState((if (on) FACE_LARGE else FACE_SMALL).ref, time, label = "size")
    val left by animateDpAsState((if (on) 193 else 225).ref, time, label = "left")
    Box(Modifier.fillMaxWidth().height(size)) {
        if (on) {
            val back = if (pencil) Color.White else Color.Transparent
            Box(
                Modifier.offset(111.ref, 67.ref).size(56.ref).background(back, CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painterResource(R.drawable.icon_pencil),
                    null,
                    Modifier.size(30.ref),
                    tint = if (pencil) Back else Muted,
                )
            }
        }
        Box(Modifier.offset(left, 0.ref).size(size)) {
            Face(person, if (on) FACE_LARGE else FACE_SMALL, server, Modifier.size(size))
            if (on && !pencil) {
                Box(Modifier.size(size).border(4.ref, Color.White, Round.face))
            }
        }
        if (on) {
            Row(
                Modifier.offset(420.ref, 74.ref),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(16.ref),
            ) {
                Text(
                    person.name,
                    fontSize = 40.refType,
                    fontWeight = FontWeight.Bold,
                )
                if (person.hasPin) {
                    Icon(
                        painterResource(R.drawable.icon_locked),
                        null,
                        Modifier.size(32.ref),
                        tint = Muted,
                    )
                }
            }
        }
    }
}
