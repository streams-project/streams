package streams.tv

import streams.shared.Art
import streams.shared.wide
import streams.shared.fill
import streams.shared.counted
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
private fun EpisodeCard(words: Map<String, String>, episode: Episode, label: String, on: Boolean, modifier: Modifier) {
    val shape = shapeOf(Round.card)
    Box(modifier.fillMaxWidth().height(256.ref).saysItsWords(on)) {
        Box(Modifier.offset(CARD_X.ref, 0.ref).size(456.ref, 256.ref).clip(shape).background(Raised)) {
            // The still is the episode that is named and described beside it.
            Art(wide(episode.images), Modifier.fillMaxSize().decoration())
            Box(Modifier.fillMaxSize().background(CardFoot).decoration())
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
    Box(
        Modifier
            .width(546.ref)
            .height(62.ref)
            .background(back, CircleShape)
            .padding(horizontal = 32.ref)
            .saysItsWords(on),
    ) {
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

/**
 * The episodes in hand, and the season the answer that brought them was for.
 *
 * The two travel together because they are read together, and because the page used to read them
 * apart. `produceState` keeps the value it has until the new season answers, so for the whole of a
 * read -- which is as long as the link is slow -- the cards on screen were the *previous* season's
 * names, stills and descriptions, under a column that already marked the new season as chosen, with
 * nothing on the page saying so. Every viewer saw that on every season change. The rare tail of it
 * was an address: a held Down walks the stale list's bound, and OK then asked for
 * `/watch/<title>/2/7` out of season 1's seven episodes, which the server answers with no source
 * and the viewer reads as "this title cannot play right now" on a title that plays fine.
 *
 * So the season travels with its episodes. [cardsOf] is what the page reads, and it hands back
 * nothing at all while the chosen season is being read; the address OK asks for names [season], the
 * season the list in hand came from, so the number in the address and the card it was read off can
 * never be two different seasons. [season] is 0 before anything has arrived, which no season number
 * is, and [episodes] is empty there.
 */
private data class Arrived(val season: Long, val episodes: List<Episode>)

private val NothingArrived = Arrived(0, emptyList())

/**
 * The cards a season shows: the ones in hand when the answer that brought them was for this season,
 * and none while this season is being read.
 *
 * This is the whole of the half that reached every viewer. A read takes as long as the link is
 * slow, and for all of it the page drew the *previous* season's names, stills and descriptions
 * under a column already marking the new season as chosen, because the pair in hand is only
 * replaced when the new answer lands. What is drawn now is what this surface draws for any page
 * with nothing to show yet -- `browse/Waiting.kt` takes it from the measured reference: the
 * destination already lit, the body empty, no skeleton, no shimmer, no old content held.
 *
 * Everything else on the page reads this and not the pair, so nothing can be walked to, lit or
 * played that is not drawn.
 */
private fun cardsOf(arrived: Arrived, season: Long) =
    if (arrived.season == season) arrived.episodes else emptyList()

/**
 * Where the remote is left on a column of [count] cards: never off either end of it, and on 0 when
 * there are none.
 *
 * The floor is the whole of it. A season being read is an empty list, whose last index is -1, and
 * a card at -1 is a page drawn a card low with nothing on it lit.
 */
internal fun cardIn(card: Int, count: Int) = card.coerceIn(0, maxOf(0, count - 1))

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
    val arrived by produceState(NothingArrived, number) {
        value = Arrived(
            number,
            shell.read("/v1/titles/$path/seasons/$number")
                ?.let { contractJson.decodeFromString(Season.serializer(), it).episodes }
                .orEmpty(),
        )
    }
    // The cards of the season the page is on, which is none of them until it answers.
    val episodes = cardsOf(arrived, number)
    LaunchedEffect(arrived) {
        // Once, and only for the season the viewer is on: the season that answered is the one the
        // card is placed in, so a season chosen later starts at its first episode.
        val on = onNow
        if (!placed && episodes.isNotEmpty() && on != null && arrived.season == on.season) {
            card = episodes.indexOfFirst { it.number == on.episode }.coerceAtLeast(0)
            placed = true
        } else {
            // Whatever a season answers with, the remote is left on a card that list has. A card
            // the arrival does not reach is drawn by nothing and lit by nothing: the page scrolls
            // to a slot that is not there and no card carries the border that says where the remote
            // is. The floor below keeps the card off -1 in the first place; this is what catches a
            // card an arrival leaves out of range, whatever put it there.
            card = cardIn(card, episodes.size)
        }
    }
    val step = with(LocalDensity.current) { CARD_STEP.ref.toPx() }
    val moved by animateFloatAsState(card.toFloat(), tween(MOVE_MS, easing = Glide), label = "cards")

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> if (inList) line = maxOf(0, line - 1) else card = maxOf(0, card - 1)
            // The floor is the season still being read: the last index of a list with nothing in it
            // is -1, so Down before the first answer lands put the card at -1, where nothing
            // corrects it. The column then drew a full card low, with an empty slot where episode 1
            // belongs and no card carrying the border that says where the remote is. One press on a
            // slow link was enough, on a plain first open of a series nobody has started.
            Word.DOWN -> if (inList) line = minOf(seasons.lastIndex, line + 1) else card = cardIn(card + 1, episodes.size)
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
                // The season the card came from, which `cardsOf` has already made the chosen one:
                // naming it here is what keeps the two from being able to disagree.
                episodes.getOrNull(card)?.let { shell.go("${Address.WATCH}$path/${arrived.season}/${it.number}") }
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
                            words,
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
