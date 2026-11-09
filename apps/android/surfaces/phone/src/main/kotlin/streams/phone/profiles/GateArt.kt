package streams.phone.profiles

import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import contract.GateLine
import contract.GateLineKind
import contract.GateTitle
import contract.Image
import contract.ImageRole
import contract.Tile
import streams.ports.Seen
import streams.shared.fill
import streams.shared.number
import streams.shared.tall
import streams.shared.wide
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.util.Locale

fun gateArtUrl(tile: Tile?): String? {
    if (tile == null) return null
    return tile.images?.firstOrNull { it.role == ImageRole.ART }?.url
        ?: tall(tile.images)
        ?: wide(tile.images)
}

fun gateLogoUrl(tile: Tile?): String? {
    return tile?.images?.firstOrNull { it.role == ImageRole.LOGO }?.url
}

fun formatGateDate(dateStr: String?, lang: String): String {
    if (dateStr.isNullOrEmpty()) return ""
    return try {
        val date = LocalDate.parse(dateStr)
        val formatter = DateTimeFormatter.ofPattern("d MMMM", Locale.forLanguageTag(lang))
        number(date.format(formatter))
    } catch (_: Exception) {
        dateStr
    }
}

fun makeGateLine(line: GateLine?, seen: Seen): String? {
    if (line == null) return null
    val words = seen.words
    return when (line.kind) {
        GateLineKind.COMING -> {
            val day = line.date?.value ?: return null
            val dateText = formatGateDate(day, seen.language.ifEmpty { "en" })
            val pattern = words["gate.coming"] ?: "Coming on {date}"
            pattern.fill("date", dateText)
        }
        GateLineKind.SEASONS_COMING -> {
            val day = line.date?.value ?: return null
            val dateText = formatGateDate(day, seen.language.ifEmpty { "en" })
            val pattern = words["gate.seasons_coming"] ?: "Seasons coming on {date}"
            pattern.fill("date", dateText)
        }
        GateLineKind.RANKED -> {
            val rank = line.rank ?: return null
            val rankStr = number(rank)
            val isTV = line.media?.value == "tv"
            val key = if (isTV) "gate.ranked.series" else "gate.ranked.films"
            val defaultText = if (isTV) "No. {rank} in Series Today" else "No. {rank} in Films Today"
            val pattern = words[key] ?: defaultText
            pattern.fill("rank", rankStr)
        }
        else -> null
    }
}

/**
 * The art area with Ken Burns zoom and bottom fade into band color.
 */
@Composable
fun GateArtArea(
    artUrl: String?,
    tint: GateTint,
    zoomScale: Float,
    height: Dp,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(height)
            .background(tint.bandColor),
    ) {
        if (!artUrl.isNullOrEmpty()) {
            AsyncImage(
                model = artUrl,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier
                    .fillMaxSize()
                    .graphicsLayer {
                        scaleX = zoomScale
                        scaleY = zoomScale
                    },
            )

            // Vertical gradient fading to band colour: full strength to ~0.67, fading to 0.98
            Box(
                Modifier
                    .fillMaxSize()
                    .background(
                        Brush.verticalGradient(
                            0.0f to Color.Transparent,
                            0.67f to Color.Transparent,
                            0.88f to tint.bandColor.copy(alpha = 0.85f),
                            0.98f to tint.bandColor,
                            1.0f to tint.bandColor,
                        )
                    )
            )
        }
    }
}

/**
 * The logo and gate line block positioned above the choose/manage label.
 */
@Composable
fun GateTitleBlock(
    title: GateTitle?,
    seen: Seen,
    modifier: Modifier = Modifier,
) {
    if (title == null) return
    val logoUrl = gateLogoUrl(title.tile)
    val lineText = makeGateLine(title.line, seen)

    Column(
        modifier = modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        if (!logoUrl.isNullOrEmpty()) {
            AsyncImage(
                model = logoUrl,
                contentDescription = title.tile.name,
                contentScale = ContentScale.Fit,
                modifier = Modifier
                    .widthIn(max = 167.4.dp)
                    .heightIn(max = 72.2.dp),
            )
        } else {
            Text(
                text = title.tile.name,
                color = Color.White,
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                textAlign = TextAlign.Center,
                modifier = Modifier.widthIn(max = 220.dp),
            )
        }

        if (!lineText.isNullOrEmpty()) {
            Text(
                text = lineText,
                color = Color.White,
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(top = 8.2.dp),
            )
        }
    }
}
