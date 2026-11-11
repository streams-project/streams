package streams.phone

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.palette.graphics.Palette
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.allowHardware
import coil3.toBitmap
import contract.ImageRole
import contract.Tile
import streams.shared.Corner
import streams.shared.R
import streams.shared.isSeries
import streams.shared.tall

/** The card's shape on the reference: 0.712 wide for every 1 high, 19.9 dp in from each side. */
private const val CARD_RATIO = 0.712f
const val CARD_SIDE = 20

/**
 * How much of the screen's height the card takes. On the reference phone the card is
 * 381.7 x 536.2 dp on a 936.6 dp screen, which is 536.2 / 936.6 = 0.572, and that is what leaves
 * the billboard and exactly one row on the first screen
 * (`netflix-reference/phone/00-netflix-phone-measurements.md:111`).
 *
 * A phone reaches this share by filling its width; a wide screen does not. Measured on a
 * 1280 x 800 dp tablet on 2026-11-11, filling the width makes the card 1240 x 1742 dp -- 218
 * percent of the screen, with the first row's name two and a fifth screens down
 * (`netflix-reference/tablet/00-tablet-surface-measurements.md`, section 3). So above the
 * expanded breakpoint the height is held to this share and the width follows the ratio.
 *
 * The share is the phone's. No Netflix tablet has been measured by this project, and this is the
 * one number here that is carried across rather than read off the reference.
 */
private const val CARD_SHARE = 0.572f

/** The tag line's grey, as the reference's. */
private val TagInk = Color(0xFFD4D4D1)

/** The My List button's fill over the tinted page, as the reference's. */
private val SecondFill = Color(0xFF3A3F47).copy(alpha = 0.9f)

private val Fade = Brush.verticalGradient(0.45f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.88f))

/**
 * The billboard: a tall card of a title's picture with its logo, a line of tags and two buttons
 * (reference: 381.7 x 536.2 dp, logo about 70 percent wide within 108 dp, tags 14 sp with the kind in
 * bold, two 48 dp buttons 16 dp apart, 16 dp in, 13.7 dp from the bottom). Our brand rounds the card
 * and corners both buttons from brand.json. `tint` gets the picture's main colour, for the page behind.
 */
@Composable
fun Billboard(
    tile: Tile,
    words: Map<String, String>,
    saved: Boolean,
    play: () -> Unit,
    list: () -> Unit,
    open: () -> Unit,
    tint: (Color) -> Unit,
) {
    val context = LocalContext.current
    val shape = RoundedCornerShape(Corner.PANEL.dp)
    BoxWithConstraints(Modifier.fillMaxWidth().padding(horizontal = CARD_SIDE.dp)) {
        // On a phone the card fills the width and lands on the reference's share of the screen.
        // On a screen this wide, filling the width would be two screens of billboard, so the
        // height is capped at that share and the width follows the measured ratio.
        val filled = maxWidth
        val high = if (widthClass() == WidthClass.EXPANDED) {
            minOf(filled / CARD_RATIO, LocalConfiguration.current.screenHeightDp.dp * CARD_SHARE)
        } else {
            filled / CARD_RATIO
        }
        val wide = minOf(filled, high * CARD_RATIO)
        Box(
            Modifier
                .align(Alignment.TopCenter)
                .width(wide)
                .height(high)
                .clip(shape)
                .border(1.dp, Color.White.copy(alpha = 0.28f), shape)
                .background(Color(0xFF16161D))
                .clickable(onClick = open),
        ) {
            val picture = tall(tile.images)
            if (picture != null) {
                AsyncImage(
                    ImageRequest.Builder(context).data(picture).allowHardware(false).build(),
                    tile.name,
                    Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                    onSuccess = { state ->
                        val found = Palette.from(state.result.image.toBitmap()).generate()
                        (found.darkMutedSwatch ?: found.dominantSwatch)?.let { tint(Color(it.rgb)) }
                    },
                )
            }
            Box(Modifier.fillMaxSize().background(Fade))
            Column(
                Modifier.align(Alignment.BottomCenter).fillMaxWidth().padding(start = 16.dp, end = 16.dp, bottom = 14.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                val logo = tile.images?.firstOrNull { it.role == ImageRole.LOGO }?.url
                if (logo != null) {
                    AsyncImage(
                        logo,
                        tile.name,
                        Modifier.sizeIn(maxWidth = wide * 0.7f, maxHeight = 108.dp),
                        contentScale = ContentScale.Fit,
                    )
                } else {
                    Text(
                        tile.name,
                        color = Color.White,
                        fontSize = 34.sp,
                        fontWeight = FontWeight.ExtraBold,
                        textAlign = TextAlign.Center,
                        maxLines = 2,
                    )
                }
                Spacer(Modifier.height(16.dp))
                Tags(tile, words)
                Spacer(Modifier.height(14.dp))
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                    Wide(words["common.play"].orEmpty(), R.drawable.icon_play, Color.White, Color.Black, play, Modifier.weight(1f))
                    Wide(
                        words["list.add"].orEmpty(),
                        if (saved) R.drawable.icon_check else R.drawable.icon_plus,
                        SecondFill,
                        Color.White,
                        list,
                        Modifier.weight(1f),
                    )
                }
            }
        }
    }
}

/** The line of tags: the title's kind in bold, then its tags, each apart by a dot. */
@Composable
private fun Tags(tile: Tile, words: Map<String, String>) {
    val kind = words[if (tile.isSeries()) "title.kind.series" else "title.kind.film"].orEmpty()
    Text(
        buildAnnotatedString {
            withStyle(SpanStyle(fontWeight = FontWeight.Bold, color = Color.White)) { append(kind) }
            tile.tags.orEmpty().take(4).forEach { append("  •  "); append(it) }
        },
        modifier = Modifier.padding(horizontal = 8.dp),
        color = TagInk,
        fontSize = 14.sp,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        textAlign = TextAlign.Center,
    )
}

/** One of the billboard's two buttons: 48 dp high, cornered by brand.json `button_radius`. */
@Composable
private fun Wide(word: String, icon: Int, fill: Color, ink: Color, go: () -> Unit, modifier: Modifier) {
    Row(
        modifier
            .height(48.dp)
            .clip(RoundedCornerShape(minOf(Corner.BUTTON, 24).dp))
            .background(fill)
            .clickable(onClick = go)
            .padding(horizontal = 8.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(painterResource(icon), null, Modifier.size(22.dp), tint = ink)
        Spacer(Modifier.width(6.dp))
        Text(
            word,
            color = ink,
            fontSize = 15.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
