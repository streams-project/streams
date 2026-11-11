package streams.shared

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import coil3.compose.AsyncImage
import contract.Image
import contract.ImageRole

private fun find(images: List<Image>?, role: ImageRole): String? =
    images?.firstOrNull { it.role == role }?.url

/** The picture for a wide place. */
fun wide(images: List<Image>?): String? =
    find(images, ImageRole.BACKDROP) ?: find(images, ImageRole.STILL) ?: find(images, ImageRole.POSTER)

/** The picture for a tall place. */
fun tall(images: List<Image>?): String? = find(images, ImageRole.POSTER) ?: wide(images)

/**
 * A picture filling its box.
 *
 * Given the box's size it asks for that size, which is what lets a tile read ahead of the viewer be
 * the same tile that is drawn. Nothing fades in: the reference paints a flat tile and then the
 * artwork, with no shimmer and no crossfade.
 *
 * [says] is the words a screen reader is given for it. The default is none, which is the right
 * answer for most of them: the art inside a tile is the tile, and the tile says the title's name,
 * so the picture is decoration and marking it so keeps a reader from stopping twice on one thing
 * (`Says.kt`). A picture that is the only thing carrying its meaning passes its words here.
 */
@Composable
fun Art(
    url: String?,
    modifier: Modifier = Modifier,
    wide: Dp = Dp.Unspecified,
    high: Dp = Dp.Unspecified,
    says: String? = null,
) {
    val context = LocalContext.current
    val density = LocalDensity.current
    val model: Any? = if (url == null || wide == Dp.Unspecified || high == Dp.Unspecified) {
        url
    } else {
        val box = with(density) { wide.roundToPx() to high.roundToPx() }
        remember(context, url, box) { sized(context, url, box.first, box.second) }
    }
    AsyncImage(model, says, modifier, contentScale = ContentScale.Crop)
}
