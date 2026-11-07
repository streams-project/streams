package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import contract.ImageRole
import contract.Tile
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Plate
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

private val Side = Brush.horizontalGradient(0f to Color.Black.copy(alpha = 0.8f), 0.62f to Color.Transparent)
private val Foot = Brush.verticalGradient(0.55f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.55f))

/** The rounded card at the top of a page. */
@Composable
fun HeroCard(shell: Shell, seen: Seen, store: DetailStore, lead: Tile, button: Int) {
    val detail by detailOf(shell, store, lead.title)
    val shape = RoundedCornerShape(24.ref)
    Box(
        Modifier
            .offset(75.ref, 143.ref)
            .size(1789.ref, 767.ref)
            .clip(shape)
            .background(Plate)
            .border(2.ref, Color.White.copy(alpha = 0.28f), shape),
    ) {
        Art(wide(lead.images), Modifier.fillMaxSize())
        Box(Modifier.fillMaxSize().background(Side))
        Box(Modifier.fillMaxSize().background(Foot))
        Column(Modifier.align(Alignment.BottomStart).padding(start = 30.ref, bottom = 34.ref).width(760.ref)) {
            val logo = (lead.images.orEmpty() + detail?.images.orEmpty())
                .firstOrNull { it.role == ImageRole.LOGO }
                ?.url
            if (logo != null) {
                AsyncImage(
                    logo,
                    lead.name,
                    Modifier.sizeIn(maxWidth = 600.ref, maxHeight = 200.ref),
                    alignment = Alignment.BottomStart,
                    contentScale = ContentScale.Fit,
                )
            } else {
                Text(lead.name, fontSize = 76.refType, lineHeight = 80.refType, fontWeight = FontWeight.ExtraBold)
            }
            Spacer(Modifier.height(16.ref))
            detail?.let { Facts(it, seen.words) }
            Spacer(Modifier.height(16.ref))
            Text(
                detail?.overview.orEmpty(),
                color = Color.White.copy(alpha = 0.8f),
                fontSize = 30.refType,
                lineHeight = 39.refType,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(30.ref))
            Row(horizontalArrangement = Arrangement.spacedBy(16.ref)) {
                Pill(seen.words["common.play"].orEmpty(), button == 0, R.drawable.icon_play)
                Pill(seen.words["common.more_info"].orEmpty(), button == 1)
            }
        }
    }
}
