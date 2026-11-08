package streams.tv

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
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

/** A picture filling its box. */
@Composable
fun Art(url: String?, modifier: Modifier = Modifier) {
    AsyncImage(url, null, modifier, contentScale = ContentScale.Crop)
}
