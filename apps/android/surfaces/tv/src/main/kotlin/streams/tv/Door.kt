package streams.tv

import android.view.View
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.unit.dp
import contract.Address
import kotlinx.coroutines.delay
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Join
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

private const val EMPTY_CARD_MS = 333L
private const val PEEK_AFTER_MS = 10_000L
private const val PEEK_FADE_MS = 583
private const val FRAME_MS = 42
private const val STAGE = 1920

// Share of the picture width the card still has to travel, one value per frame, as measured.
private val Slide = listOf(0.97f, 0.647f, 0.323f, 0.106f, 0.077f, 0.039f, 0.017f, 0.005f, 0.001f, 0f)

private val CardWash = Brush.verticalGradient(
    0f to Color(0xFF2B3A78),
    0.7f to Color(0xFF42549C),
    1f to Color(0xFF141730),
)
private val CardEdge = Color(0xFFC4E4FB)

/**
 * The violet light in the top left corner behind the card, as the approved opening renders it:
 * strength (1 - (x / 1500 + y / 620)) ^ 1.5 of 5F28EB, on a 1920 by 1080 page.
 */
private val Glow = Brush.linearGradient(
    0f to Color(0xFF5F28EB),
    0.25f to Color(0xFF5F28EB).copy(alpha = 0.65f),
    0.5f to Color(0xFF5F28EB).copy(alpha = 0.354f),
    0.75f to Color(0xFF5F28EB).copy(alpha = 0.125f),
    1f to Color.Transparent,
    start = Offset.Zero,
    end = Offset(219f, 530f),
)

private fun DrawScope.glow() {
    scale(size.width / STAGE, size.height / (STAGE * 9f / 16f), Offset.Zero) {
        drawRect(Glow, Offset.Zero, Size(STAGE.toFloat(), STAGE * 9f / 16f))
    }
}

private enum class Stop { HOME, TAB, HELP, SIGN_IN, JOIN }

/** Whether the opening has played. It plays once for each run of the app. */
@Stable
class DoorState {
    var opened by mutableStateOf(false)
}

/** A brand film or sound. It plays while `on`, and is handed back when it leaves the screen. */
@Composable
fun Film(
    shell: Shell,
    url: String,
    loop: Boolean,
    on: Boolean,
    modifier: Modifier,
    ready: () -> Unit = {},
    ended: () -> Unit,
) {
    val picture = remember { arrayOfNulls<Any>(1) }
    val readyNow by rememberUpdatedState(ready)
    val endedNow by rememberUpdatedState(ended)
    AndroidView(
        { context ->
            (shell.reel.picture(context, url, loop, { readyNow() }, { endedNow() }) as View).also { picture[0] = it }
        },
        modifier,
        update = { view -> shell.reel.play(view, on) },
    )
    DisposableEffect(Unit) { onDispose { picture[0]?.let(shell.reel::release) } }
}

@Composable
private fun Ask(word: String, on: Boolean, lit: Color, ink: Color) {
    Box(
        Modifier
            .scale(if (on) 1.04f else 1f)
            .size(268.ref, 80.ref)
            .background(if (on) lit else Color.White.copy(alpha = 0.22f), CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Text(word, color = if (on) ink else Color.White, fontSize = 28.refType, fontWeight = FontWeight.Bold)
    }
}

/**
 * The landing. After the opening it is already in place under it: the card does not slide, the
 * showreel is there from the first frame, says when it is `ready`, and plays once `roll` is true.
 * Coming back to it later, the card slides in and the showreel starts after it.
 */
@Composable
private fun Landing(shell: Shell, seen: Seen, opening: Boolean, roll: Boolean, keys: Boolean, ready: () -> Unit) {
    val words = seen.words
    var at by remember { mutableStateOf(Stop.SIGN_IN) }
    var film by remember { mutableStateOf(opening) }
    var peek by remember { mutableStateOf(false) }
    val slide = remember { Animatable(if (opening) 0f else Slide.first()) }
    val width = with(LocalDensity.current) { STAGE.ref.toPx() }
    val shown by animateFloatAsState(if (peek) 1f else 0f, tween(PEEK_FADE_MS, easing = LinearEasing), label = "peek")
    LaunchedEffect(Unit) {
        if (!opening) {
            slide.animateTo(
                0f,
                keyframes {
                    durationMillis = (Slide.size - 1) * FRAME_MS
                    Slide.forEachIndexed { index, left -> left at index * FRAME_MS }
                },
            )
            delay(EMPTY_CARD_MS)
            film = true
        }
    }
    LaunchedEffect(roll) {
        if (roll) {
            delay(PEEK_AFTER_MS)
            peek = true
        }
    }

    fun press(word: Word): Boolean {
        val stops = Stop.entries
        when (word) {
            Word.LEFT -> if (at.ordinal > 0) at = stops[at.ordinal - 1]
            Word.RIGHT -> if (at.ordinal < stops.lastIndex) at = stops[at.ordinal + 1]
            Word.UP -> if (at.ordinal >= Stop.HELP.ordinal) at = Stop.TAB
            Word.DOWN -> if (at.ordinal < Stop.HELP.ordinal) at = Stop.SIGN_IN
            Word.OK -> when (at) {
                Stop.SIGN_IN, Stop.JOIN -> shell.go(Address.SIGN_IN)
                Stop.HELP -> shell.go(Address.HELP)
                else -> Unit
            }
            Word.BACK -> return false
        }
        return true
    }

    val shape = RoundedCornerShape(24.ref)
    Box(Modifier.fillMaxSize().background(Back).drawBehind { glow() }.remote(keys, @DontMemoize { press(it) })) {
        Row(Modifier.offset(96.ref, 78.ref), verticalAlignment = Alignment.CenterVertically) {
            Box(
                Modifier
                    .size(82.ref)
                    .background(if (at == Stop.HOME) Color.White else Color.White.copy(alpha = 0.12f), CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painterResource(R.drawable.icon_home),
                    null,
                    Modifier.size(38.ref),
                    tint = if (at == Stop.HOME) Back else Color.White,
                )
            }
            Spacer(Modifier.width(40.ref))
            Text(
                words["door.trending"].orEmpty(),
                Modifier.alpha(if (at == Stop.TAB) 1f else 0.85f),
                fontSize = 30.refType,
                fontWeight = FontWeight.SemiBold,
            )
        }
        if (peek) {
            Box(
                Modifier
                    .offset(1636.ref, 167.ref)
                    .size(284.ref, 866.ref)
                    .alpha(shown)
                    .clip(RoundedCornerShape(topStart = 24.ref, bottomStart = 24.ref)),
            ) { Art("${seen.server}/tv/peek.jpg", Modifier.fillMaxSize()) }
        }
        Box(
            Modifier
                .offset(55.ref, 169.ref)
                .graphicsLayer { translationX = slide.value * width }
                .size(1542.ref, 864.ref)
                .border(3.ref, CardEdge, shape)
                .clip(shape)
                .background(CardWash),
        ) {
            if (film) Film(shell, "${seen.server}/tv/showreel.mp4", true, roll, Modifier.fillMaxSize(), ready) { ready() }
            Text(
                words["door.help"].orEmpty(),
                Modifier
                    .offset(96.ref, 770.ref)
                    .background(if (at == Stop.HELP) Color.White else Color.Transparent, RoundedCornerShape(12.ref)),
                color = if (at == Stop.HELP) Back else Color.White.copy(alpha = 0.85f),
                fontSize = 24.refType,
                fontWeight = FontWeight.SemiBold,
            )
            Row(Modifier.offset(915.ref, 720.ref)) {
                Ask(words["door.sign_in"].orEmpty(), at == Stop.SIGN_IN, Color.White, Back)
                Spacer(Modifier.width(32.ref))
                Ask(words["door.join"].orEmpty(), at == Stop.JOIN, Join, Color.White)
            }
        }
    }
}

/**
 * The way in: the opening, drawn live, once for each run of the app, over the landing it opens onto.
 * The landing is built under it from the first frame so its showreel can load while the mark rests,
 * and the opening's sounds live here so they play to their end.
 */
@Composable
fun Door(shell: Shell, seen: Seen, state: DoorState) {
    val opening = remember { !state.opened }
    var ready by remember { mutableStateOf(false) }
    var roll by remember { mutableStateOf(!opening) }
    // Under the opening's cover the landing loads but is not drawn until the first stage is over.
    var shown by remember { mutableStateOf(!opening) }
    var notes by remember { mutableStateOf(false) }
    var swell by remember { mutableStateOf(false) }
    Box(Modifier.fillMaxSize()) {
        Box(Modifier.fillMaxSize().graphicsLayer { alpha = if (shown) 1f else 0f }) {
            Landing(shell, seen, opening, roll, state.opened) { ready = true }
        }
        if (opening) {
            Film(shell, "${seen.server}/tv/opening-chime.m4a", false, notes, Modifier.size(0.dp)) {}
            Film(shell, "${seen.server}/tv/opening-swell.m4a", false, swell, Modifier.size(0.dp)) {}
        }
        if (!state.opened) {
            Opening(ready && seen.words.isNotEmpty()) { moment ->
                when (moment) {
                    Moment.NOTES -> notes = true
                    Moment.WARM -> shown = true
                    Moment.SECOND -> swell = true
                    Moment.ROLL -> roll = true
                    Moment.DONE -> {
                        shown = true
                        roll = true
                        state.opened = true
                    }
                }
            }
        }
    }
}
