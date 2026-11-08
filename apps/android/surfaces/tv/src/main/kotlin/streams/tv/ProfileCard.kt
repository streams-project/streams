package streams.tv

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import contract.Profile
import streams.shared.ref
import streams.shared.refType

private const val LARGE = 300
private const val SMALL = 122

/** The profile large on the right of a page, with what is being edited under it. */
@Composable
fun ProfileCard(person: Profile, server: String, name: @Composable () -> Unit, under: String?) {
    Box {
        Face(person, LARGE, server, Modifier.offset(1237.ref, 336.ref).size(LARGE.ref))
        Box(Modifier.offset(1085.ref, 676.ref).width(600.ref), contentAlignment = Alignment.Center) { name() }
        if (under != null) {
            Text(
                under,
                Modifier.offset(1085.ref, 748.ref).width(600.ref),
                fontSize = 30.refType,
                textAlign = TextAlign.Center,
            )
        }
    }
}

/** A profile's name as the card shows it. */
@Composable
fun CardName(name: String) {
    Text(name, fontSize = 40.refType, fontWeight = FontWeight.Medium, maxLines = 1)
}

/** The profile small at the top right, when the page needs the room. */
@Composable
fun ProfileCorner(person: Profile, server: String) {
    Row(
        Modifier.offset(1085.ref, 70.ref).width(702.ref),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            person.name,
            Modifier.weight(1f),
            fontSize = 34.refType,
            fontWeight = FontWeight.Medium,
            textAlign = TextAlign.End,
            maxLines = 1,
        )
        Spacer(Modifier.width(30.ref))
        Face(person, SMALL, server, Modifier.size(SMALL.ref))
    }
}
