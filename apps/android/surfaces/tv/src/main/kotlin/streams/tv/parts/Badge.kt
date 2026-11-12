package streams.tv

import streams.shared.fill
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import contract.Badge
import contract.BadgeKind
import contract.Tile
import streams.shared.ref
import streams.shared.refType

internal var keptWords: Map<String, String> = emptyMap()

internal fun badgeLabel(badge: Badge, words: Map<String, String>): String = when (badge.kind) {
    BadgeKind.NEW -> words["badge.new"].orEmpty()
    BadgeKind.RANKED -> {
        val rank = badge.rank ?: return ""
        words["badge.ranked"].orEmpty().fill("rank", rank)
    }
    else -> ""
}

/** Draws `Tile.badges` (`BadgeKind.NEW`, `BadgeKind.RANKED`) as small pills inside the tile at bottom left. */
@Composable
fun TileBadges(tile: Tile, modifier: Modifier = Modifier, words: Map<String, String> = keptWords) {
    val badges = tile.badges.orEmpty()
    if (badges.isEmpty()) return
    val shape = shapeOf(Round.chip)
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(8.ref),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        for (badge in badges) {
            val text = badgeLabel(badge, words)
            Box(
                Modifier
                    .background(Color.Black.copy(alpha = 0.72f), shape)
                    .padding(horizontal = 12.ref, vertical = 4.ref),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text,
                    color = Color.White,
                    fontSize = 22.refType,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                )
            }
        }
    }
}
