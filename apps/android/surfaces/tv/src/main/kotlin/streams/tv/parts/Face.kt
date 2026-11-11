package streams.tv

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import contract.Profile
import streams.shared.Face as SharedFace
import streams.shared.refType

/**
 * A profile's picture, or its first letter when it has none. Every screen draws this one part.
 *
 * `says` is the words for a face that stands on its own, with no name drawn beside it: the person's
 * name, which is already in the viewer's language because they typed it. A face that sits next to
 * its own name leaves this out, so a screen reader does not say the name twice.
 */
@Composable
fun Face(person: Profile?, size: Int, server: String, modifier: Modifier = Modifier, says: String? = null) {
    SharedFace(
        person = person,
        server = server,
        modifier = modifier.says(says),
        shape = Round.face,
        fontSize = (size * 46 / 100).refType,
    )
}
