package streams.tv

import streams.shared.fill
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import contract.Address
import contract.Episode
import contract.Season
import contract.SeasonSummary
import contract.TitleDetail
import contract.contractJson
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Muted
import streams.shared.Raised
import streams.shared.ref
import streams.shared.refType

private const val CARD_X = 776
private const val CARD_Y = 140
private const val CARD_STEP = 284
private const val WORDS_X = 1268
private const val MOVE_MS = 240
private const val NEAR = 4

private val CardFoot = Brush.verticalGradient(0.5f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.7f))

@Composable
private fun EpisodeCard(episode: Episode, label: String, on: Boolean, modifier: Modifier) {
    val shape = shapeOf(Round.card)
    Box(modifier.fillMaxWidth().height(256.ref)) {
        Box(Modifier.offset(CARD_X.ref, 0.ref).size(456.ref, 256.ref).clip(shape).background(Raised)) {
            Art(wide(episode.images), Modifier.fillMaxSize())
            Box(Modifier.fillMaxSize().background(CardFoot))
            Text(
                label,
                Modifier.align(Alignment.BottomStart).padding(start = 20.ref, bottom = 14.ref),
                fontSize = 24.refType,
                fontWeight = FontWeight.Medium,
            )
            if (on) Box(Modifier.fillMaxSize().border(4.ref, Color.White, shape))
        }
        Column(Modifier.offset(WORDS_X.ref, 44.ref).width(590.ref)) {
            Text(episode.name, fontSize = 30.refType, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(
                episode.overview.orEmpty(),
                Modifier.padding(top = 8.ref),
                fontSize = 24.refType,
                lineHeight = 32.refType,
                color = Color.White.copy(alpha = 0.8f),
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
                style = storyStyle(),
            )
            episode.runtimeMin?.takeIf { it > 0 }?.let {
                Text("(${words.counted("title.length.minutes", it.toInt())})", Modifier.padding(top = 8.ref), fontSize = 24.refType, color = Muted)
            }
        }
    }
}

@Composable
private fun SeasonLine(words: Map<String, String>, season: SeasonSummary, chosen: Boolean, on: Boolean) {
    val back = when {
        on -> Color.White
        chosen -> Color.White.copy(alpha = 0.16f)
        else -> Color.Transparent
    }
    val ink = if (on) Back else Color.White
    Box(Modifier.width(546.ref).height(62.ref).background(back, CircleShape).padding(horizontal = 32.ref)) {
        Text(
            words["title.season.number"].orEmpty().fill("n", season.number),
            Modifier.align(Alignment.CenterStart),
            color = ink,
            fontSize = 30.refType,
        )
        Text(
            words["title.episodes.count"].orEmpty().fill("n", season.episodeCount),
            Modifier.align(Alignment.CenterEnd),
            color = ink,
            fontSize = 24.refType,
        )
    }
}

/** A series' seasons on the left, and that season's episodes as cards on the right. */
@Composable
fun Episodes(shell: Shell, seen: Seen, data: TitleDetail, leave: () -> Unit) {
    val words = seen.words
    val seasons = data.seasons.orEmpty()
    val path = data.title.value.replace(':', '/')
    // The page opens on the season and episode the viewer is on, so Play next is one press away.
    val onNow = data.resume?.episode
    var chosen by remember { mutableIntStateOf(seasons.indexOfFirst { it.number == onNow?.season }.coerceAtLeast(0)) }
    var inList by remember { mutableStateOf(false) }
    var line by remember { mutableIntStateOf(chosen) }
    var card by remember { mutableIntStateOf(0) }
    var placed by remember { mutableStateOf(onNow == null) }
    val number = seasons.getOrNull(chosen)?.number ?: 1
    val episodes by produceState(emptyList<Episode>(), number) {
        value = shell.read("/v1/titles/$path/seasons/$number")
            ?.let { contractJson.decodeFromString(Season.serializer(), it).episodes }
            .orEmpty()
    }
    LaunchedEffect(episodes) {
        if (!placed && episodes.isNotEmpty() && seasons.getOrNull(chosen)?.number == onNow?.season) {
            card = episodes.indexOfFirst { it.number == onNow?.episode }.coerceAtLeast(0)
            placed = true
        }
    }
    val step = with(LocalDensity.current) { CARD_STEP.ref.toPx() }
    val moved by animateFloatAsState(card.toFloat(), tween(MOVE_MS, easing = Glide), label = "cards")

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> if (inList) line = maxOf(0, line - 1) else card = maxOf(0, card - 1)
            Word.DOWN -> if (inList) line = minOf(seasons.lastIndex, line + 1) else card = minOf(episodes.lastIndex, card + 1)
            // From the cards Left opens the seasons at the one shown; from the seasons it goes back.
            Word.LEFT -> if (inList) {
                leave()
            } else {
                inList = true
                line = chosen
            }
            Word.RIGHT -> inList = false
            Word.OK -> if (inList) {
                chosen = line
                card = 0
                inList = false
            } else {
                episodes.getOrNull(card)?.let { shell.go("${Address.WATCH}$path/$number/${it.number}") }
            }
            Word.BACK -> leave()
        }
        return true
    }

    Box(Modifier.fillMaxSize().background(Color.Black).remote(@DontMemoize { press(it) })) {
        // The facts follow the name, however many lines it takes; the seasons keep their place.
        Column(Modifier.offset(PAGE_EDGE.ref, 70.ref).width(640.ref), verticalArrangement = Arrangement.spacedBy(20.ref)) {
            Text(
                data.name,
                fontSize = 56.refType,
                lineHeight = 62.refType,
                fontWeight = FontWeight.ExtraBold,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                style = storyStyle(),
            )
            Facts(data, words)
        }
        Column(Modifier.offset(PAGE_EDGE.ref, 346.ref)) {
            seasons.forEachIndexed { index, season ->
                SeasonLine(words, season, index == chosen, inList && index == line)
            }
        }
        Box(Modifier.offset(0.ref, CARD_Y.ref).graphicsLayer { translationY = -moved * step }) {
            episodes.forEachIndexed { index, episode ->
                if (index in card - 1..card + NEAR) {
                    key(episode.number) {
                        EpisodeCard(
                            episode,
                            words["title.episode"].orEmpty().fill("n", episode.number),
                            !inList && index == card,
                            Modifier.offset(0.ref, (index * CARD_STEP).ref),
                        )
                    }
                }
            }
        }
    }
}
