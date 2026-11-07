package streams.tv

import android.view.View
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.viewinterop.AndroidView
import contract.ActionCommand
import contract.ActionPause
import contract.ActionSeek
import contract.ActionSelectAudio
import contract.ActionSelectText
import contract.ActionStop
import contract.ActionUnpause
import contract.CommandRate
import contract.PlayState
import contract.ProfileSettings
import contract.Rating
import contract.TitleDetail
import contract.WindowKind
import contract.contractJson
import kotlinx.coroutines.delay
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Accent
import streams.shared.Back
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

private const val SHOWN_MS = 3000L
private const val SEEK_MS = 10_000L

/** How long the bar waits for the next press before it seeks, and for the picture to catch up. */
private const val SEEK_SETTLE_MS = 350L
private const val SEEK_CATCH_UP_MS = 1500L

/** How long the end waits, so the next episode can start before the page goes back. */
private const val ENDED_WAIT_MS = 800L

/** How far subtitles rise, as a share of the picture's height. */
private const val WORDS_LOW = 0f
private const val WORDS_HIGH = 0.2f

private enum class Reach { TOP, RATE, BAR, TRACKS }
private enum class Control { BACK, RESTART, NEXT }

private val Shade = Brush.verticalGradient(
    0f to Color.Black.copy(alpha = 0.55f),
    0.3f to Color.Transparent,
    0.6f to Color.Transparent,
    1f to Color.Black.copy(alpha = 0.7f),
)

private class RateChoice(val rating: Rating, val word: String, val icon: Int)

private val Rates = listOf(
    RateChoice(Rating.DOWN, "player.down", R.drawable.icon_thumb_down),
    RateChoice(Rating.UP, "player.up", R.drawable.icon_thumb_up),
    RateChoice(Rating.LOVE, "player.love", R.drawable.icon_love),
)

private fun clock(ms: Long): String {
    val seconds = ms / 1000
    val body = "%d:%02d".format(seconds / 60 % 60, seconds % 60)
    return if (seconds >= 3600) "${seconds / 3600}:" + "%02d:%02d".format(seconds / 60 % 60, seconds % 60) else body
}

@Composable
private fun TopIcons(controls: List<Control>, focus: Int, words: Map<String, String>) {
    controls.forEachIndexed { index, control ->
        val on = index == focus
        val x = 120 + index * 124
        Box(
            Modifier
                .offset(x.ref, 112.ref)
                .size(72.ref)
                .background(if (on) Color.White else Color.Transparent, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painterResource(
                    when (control) {
                        Control.BACK -> R.drawable.icon_back
                        Control.RESTART -> R.drawable.icon_restart
                        Control.NEXT -> R.drawable.icon_next
                    },
                ),
                null,
                Modifier.size(46.ref),
                tint = if (on) Back else Color.White,
            )
        }
        if (on) {
            Text(
                words[
                    when (control) {
                        Control.BACK -> "player.back"
                        Control.RESTART -> "player.restart"
                        Control.NEXT -> "player.next"
                    },
                ].orEmpty(),
                Modifier.offset((x - 90).ref, 202.ref).width(252.ref),
                fontSize = 28.refType,
                textAlign = TextAlign.Center,
            )
        }
    }
}

@Composable
private fun RatePills(focus: Int, chosen: Rating?, words: Map<String, String>) {
    Row(Modifier.offset(122.ref, 596.ref)) {
        Rates.forEachIndexed { index, rate ->
            val on = index == focus
            val ink = if (on) Back else Color.White
            Row(
                Modifier
                    .padding(end = 22.ref)
                    .height(62.ref)
                    .background(if (on) Color.White else Color.White.copy(alpha = if (chosen == rate.rating) 0.4f else 0.2f), CircleShape)
                    .padding(horizontal = 22.ref),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(painterResource(rate.icon), null, Modifier.size(34.ref), tint = ink)
                Spacer(Modifier.width(12.ref))
                Text(words[rate.word].orEmpty(), color = ink, fontSize = 28.refType, fontWeight = FontWeight.Bold)
            }
        }
    }
    Text(words["player.hint"].orEmpty(), Modifier.offset(122.ref, 694.ref), fontSize = 28.refType)
}

@Composable
private fun Bar(seen: Seen, at: Long, playing: Boolean, on: Boolean) {
    val done = if (seen.lengthMs > 0) (at.toFloat() / seen.lengthMs).coerceIn(0f, 1f) else 0f
    Box(
        Modifier
            .offset(262.ref, 874.ref)
            .size(78.ref)
            .background(if (on) Color.White else Color.Transparent, CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            painterResource(if (playing) R.drawable.icon_pause else R.drawable.icon_play),
            null,
            Modifier.size(48.ref),
            tint = if (on) Back else Color.White,
        )
    }
    Text(clock(at), Modifier.offset(360.ref, 894.ref).width(100.ref), fontSize = 30.refType, textAlign = TextAlign.End)
    Box(Modifier.offset(476.ref, 910.ref).size(1052.ref, 6.ref).background(Color.White.copy(alpha = 0.4f), CircleShape)) {
        Box(Modifier.fillMaxWidth(done).fillMaxHeight().background(Accent, CircleShape))
    }
    Text(clock(seen.lengthMs), Modifier.offset(1548.ref, 894.ref), fontSize = 30.refType)
}

/** The picture, and the controls that rise over it when the viewer reaches for the remote. */
@Composable
fun Watch(shell: Shell, seen: Seen) {
    val play = seen.play
    val words = seen.words
    val paused = play?.state == PlayState.PAUSED
    val playing = play?.state == PlayState.PLAYING
    val failed = play?.state == PlayState.FAILED
    var awake by remember { mutableIntStateOf(1) }
    var shown by remember { mutableStateOf(true) }
    var zone by remember { mutableStateOf(Reach.BAR) }
    var top by remember { mutableIntStateOf(0) }
    var rate by remember { mutableIntStateOf(1) }
    val detail by produceState(seen.detail?.takeIf { it.title == play?.title }, play?.title) {
        val key = play?.title ?: return@produceState
        if (value == null) {
            value = shell.read("/v1/titles/" + key.value.replace(':', '/'))
                ?.let { contractJson.decodeFromString(TitleDetail.serializer(), it) }
        }
    }
    var rated by remember(detail?.rating) { mutableStateOf(detail?.rating) }
    val asking = play?.stillAsking == true
    var stay by remember { mutableIntStateOf(0) }
    var track by remember { mutableIntStateOf(0) }
    var aim by remember { mutableStateOf<Long?>(null) }
    val panel = remember { TrackPanelState() }
    val choices = choicesOf(seen)
    val part = seen.windowNow(setOf(WindowKind.INTRO, WindowKind.RECAP))
    val closing = seen.windowNow(setOf(WindowKind.CREDITS))?.takeIf { play?.nextEpisode != null }
    val age = seen.windowNow(setOf(WindowKind.AGE_CARD))?.text
    val next = play?.nextEpisode
    val controls = buildList {
        add(Control.BACK)
        add(Control.RESTART)
        if (next != null) add(Control.NEXT)
    }
    LaunchedEffect(awake, paused) {
        shown = true
        if (!paused) {
            delay(SHOWN_MS)
            shown = false
            zone = Reach.BAR
        }
    }
    LaunchedEffect(shown) { shell.player.lift(if (shown) WORDS_HIGH else WORDS_LOW) }
    LaunchedEffect(Unit) {
        val kept = shell.read("/v1/profiles/me/settings")
            ?.let { contractJson.decodeFromString(ProfileSettings.serializer(), it) }
            ?.subtitleStyle.orEmpty()
        shell.player.wordsLook(
            kept["size"]?.toIntOrNull() ?: 1,
            kept["color"] ?: "#ffffff",
            kept["background"] ?: "none",
        )
    }
    // When the next episode is starting, the state moves on and this never reaches the back.
    LaunchedEffect(play?.state, asking) {
        if (play?.state == PlayState.ENDED && !asking) {
            delay(ENDED_WAIT_MS)
            shell.back()
        }
    }
    LaunchedEffect(aim) {
        val target = aim ?: return@LaunchedEffect
        delay(SEEK_SETTLE_MS)
        shell.act(ActionSeek(toMs = target))
        delay(SEEK_CATCH_UP_MS)
        aim = null
    }
    val title = play?.title

    fun toggle() = shell.act(if (paused) ActionUnpause else ActionPause)

    /** Presses add up from where the last one aimed, as the position only moves once a second. */
    fun seek(by: Long) {
        val end = if (seen.lengthMs > 0) seen.lengthMs else Long.MAX_VALUE
        aim = ((aim ?: seen.positionMs) + by).coerceIn(0, end)
    }

    fun goNext() {
        val episode = next ?: return
        shell.swap("/watch/" + play.title.value.replace(':', '/') + "/${episode.season}/${episode.episode}")
    }

    fun choose(choice: Choice) {
        if (choice.other) {
            panel.show(seen)
            return
        }
        val sound = choice.audio
        if (sound != null && !sound.playing) shell.act(ActionSelectAudio(track = sound.id))
        shell.act(ActionSelectText(track = choice.text?.id))
    }

    fun press(word: Word): Boolean {
        if (panel.open) {
            panel.press(word, shell, seen)
            return true
        }
        if (word == Word.BACK) return false
        if (asking) {
            when (word) {
                Word.LEFT -> stay = 0
                Word.RIGHT -> stay = 1
                Word.OK -> if (stay == 0) {
                    goNext()
                } else {
                    shell.act(ActionStop)
                    shell.back()
                }
                else -> Unit
            }
            return true
        }
        val wasHidden = !shown
        awake += 1
        if (failed) return true
        if (word == Word.OK && zone == Reach.BAR && wasHidden) {
            when {
                part != null -> shell.act(ActionSeek(toMs = part.seekToMs ?: part.endMs))
                closing != null -> goNext()
                else -> toggle()
            }
            return true
        }
        when (zone) {
            Reach.BAR -> when (word) {
                Word.OK -> toggle()
                Word.LEFT -> seek(-SEEK_MS)
                Word.RIGHT -> seek(SEEK_MS)
                Word.UP -> if (!wasHidden) zone = if (paused) Reach.RATE else Reach.TOP
                Word.DOWN -> if (!wasHidden && choices.isNotEmpty()) {
                    zone = Reach.TRACKS
                    track = choices.indexOfFirst { it.ticked }.coerceAtLeast(0)
                }
                else -> Unit
            }
            Reach.TRACKS -> when (word) {
                Word.LEFT -> if (track > 0) track -= 1
                Word.RIGHT -> if (track < choices.lastIndex) track += 1
                Word.UP -> zone = Reach.BAR
                Word.OK -> choices.getOrNull(track)?.let(::choose)
                else -> Unit
            }
            Reach.RATE -> when (word) {
                Word.LEFT -> if (rate > 0) rate -= 1
                Word.RIGHT -> if (rate < Rates.lastIndex) rate += 1
                Word.UP -> zone = Reach.TOP
                Word.DOWN -> zone = Reach.BAR
                Word.OK -> title?.let {
                    val asked = Rates[rate].rating
                    rated = if (rated == asked) null else asked
                    shell.act(ActionCommand(CommandRate(rating = rated, title = it)))
                }
                else -> Unit
            }
            Reach.TOP -> when (word) {
                Word.LEFT -> if (top > 0) top -= 1
                Word.RIGHT -> if (top < controls.lastIndex) top += 1
                Word.DOWN -> zone = if (paused) Reach.RATE else Reach.BAR
                Word.OK -> when (controls[top]) {
                    Control.BACK -> return false
                    Control.RESTART -> shell.act(ActionSeek(toMs = 0))
                    Control.NEXT -> goNext()
                }
                else -> Unit
            }
        }
        return true
    }

    Box(Modifier.fillMaxSize().background(Color.Black).remote(@DontMemoize { press(it) })) {
        AndroidView({ shell.player.picture(it) as View }, Modifier.fillMaxSize())
        when (play?.state) {
            null, PlayState.RESOLVING, PlayState.STARTING, PlayState.STALLED, PlayState.SWITCHING ->
                CircularProgressIndicator(Modifier.align(Alignment.Center).size(96.ref), color = Color.White)
            else -> Unit
        }
        if (failed) {
            Text(
                seen.notice ?: words["error.no_source"].orEmpty(),
                Modifier.align(Alignment.Center).width(1100.ref),
                fontSize = 40.refType,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
            )
        } else if (shown) {
            Box(Modifier.fillMaxSize().background(Shade))
            TopIcons(controls, if (zone == Reach.TOP) top else -1, words)
            Column(Modifier.align(Alignment.TopEnd).padding(top = 96.ref, end = 124.ref), horizontalAlignment = Alignment.End) {
                Text(detail?.name.orEmpty(), fontSize = 34.refType, fontWeight = FontWeight.Bold)
                play?.episode?.let {
                    Text("S${it.season}: E${it.episode}", fontSize = 30.refType, color = Color.White.copy(alpha = 0.75f))
                }
            }
            if (paused) RatePills(if (zone == Reach.RATE) rate else -1, rated, words)
            Bar(seen, aim ?: seen.positionMs, !paused, zone == Reach.BAR)
            TrackStrip(choices, if (zone == Reach.TRACKS) track else -1)
        }
        if (!failed) {
            age?.let { AgeNote(it) }
            val moment = when {
                part?.kind == WindowKind.INTRO -> words["player.skip_intro"]
                part != null -> words["player.skip_recap"]
                closing != null -> words["player.next"]
                else -> null
            }
            moment?.let { MomentPill(it, !shown, shown) }
        }
        if (panel.open) TrackPanel(panel, seen)
        if (asking) StillAsking(words, stay)
    }
}
