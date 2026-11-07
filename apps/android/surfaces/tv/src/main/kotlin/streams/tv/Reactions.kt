package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import contract.Rating
import streams.shared.Back
import streams.shared.R
import streams.shared.ref

private class Face(val rating: Rating, val plain: Int, val filled: Int)

private val Faces = listOf(
    Face(Rating.DOWN, R.drawable.icon_thumb_down, R.drawable.icon_thumb_down_filled),
    Face(Rating.UP, R.drawable.icon_thumb_up, R.drawable.icon_thumb_up_filled),
    Face(Rating.LOVE, R.drawable.icon_love, R.drawable.icon_love_filled),
)

/** How many ratings the row offers. */
val ReactionCount = Faces.size

/** The rating at a place in the row. */
fun reactionAt(index: Int): Rating = Faces[index].rating

/** Three ways to rate a title. `focus` is -1 when the row has no focus. */
@Composable
fun Reactions(chosen: Rating?, focus: Int, left: Int, top: Int) {
    Faces.forEachIndexed { index, face ->
        val on = index == focus
        Box(
            Modifier
                .offset((left + index * 90).ref, top.ref)
                .size(64.ref)
                .background(if (on) Color.White else Color.Transparent, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painterResource(if (chosen == face.rating) face.filled else face.plain),
                null,
                Modifier.size(38.ref),
                tint = if (on) Back else Color.White,
            )
        }
    }
}
