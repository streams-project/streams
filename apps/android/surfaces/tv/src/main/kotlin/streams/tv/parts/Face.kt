package streams.tv

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import contract.Profile
import streams.shared.Face as SharedFace
import streams.shared.refType

/** A profile's picture, or its first letter when it has none. Every screen draws this one part. */
@Composable
fun Face(person: Profile?, size: Int, server: String, modifier: Modifier = Modifier) {
    SharedFace(
        person = person,
        server = server,
        modifier = modifier,
        shape = Round.face,
        fontSize = (size * 46 / 100).refType,
    )
}
