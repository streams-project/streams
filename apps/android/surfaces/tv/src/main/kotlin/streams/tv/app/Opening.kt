package streams.tv

import android.content.res.Resources
import androidx.compose.foundation.Canvas
import kotlin.math.roundToInt
import streams.shared.LOCK_W
import streams.shared.MARK_H
import streams.shared.OPENING_SECOND
import streams.shared.OpeningArt
import streams.shared.Stage
import streams.shared.lockupAtRest
import streams.shared.openingArt
import streams.shared.openingFirst
import streams.shared.openingSecond
import streams.shared.ref
import streams.shared.smooth
import streams.shared.span
import streams.ports.Shell
import streams.ports.Seen
import contract.Target
import androidx.compose.ui.unit.dp
import androidx.compose.runtime.setValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.Stable
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import kotlin.math.max
import kotlin.math.min

// The opening, drawn live. Its drawing and its first and second stages are shared with the phone
// (streams.shared.Opening); the television adds its timing, ring, sounds, showreel and remote.

private const val WIDE = 1920f
private const val HIGH = 1080f
private const val MIDDLE = HIGH / 2

/** When the first stage ends and the mark and text rest. */
private const val REST = 3.2f
/** The shortest rest when everything is ready before the first stage ends. */
private const val SHORT_REST = 0.6f
/** Longest the opening waits for the landing before it goes on anyway. */
private const val WAIT_AT_MOST = 8f
private const val RING_FADE = 0.35f
private const val SECOND = OPENING_SECOND
private const val NOTES_AT = 2.05f
/** How long before the window opens the showreel starts, under the cover, past its fade from black. */
const val FILM_LEAD = 1.0f

private val Night = Color(0xFF06070F)
private val Ring = Color(0xFF2F6BFF)

/** The television's stage: the screen references' 1920 by 1080, the lockup on the middle line. */
private val TV = Stage(WIDE, HIGH)

private fun DrawScope.ring(t: Float, alpha: Float) {
    if (alpha <= 0f) return
    val r = 33f
    drawArc(
        Ring.copy(alpha = alpha),
        startAngle = (t * 347) % 360,
        sweepAngle = 280f,
        useCenter = false,
        topLeft = Offset(0.498f * WIDE - r, 0.804f * HIGH - r),
        size = Size(2 * r, 2 * r),
        style = Stroke(8f, cap = StrokeCap.Round),
    )
}

/** The moments of the opening that the page around it acts on. */
enum class Moment {
    /** The two soft notes. */
    NOTES,
    /** The first stage is over; the landing may draw under the cover, so it is ready to be seen. */
    WARM,
    /** The second stage begins, with its swell. */
    SECOND,
    /** The showreel should start. */
    ROLL,
    /** The opening is over. */
    DONE,
}

/** Where the opening is, for the whole app. It plays once for each run of the app. */
@Stable
class OpeningState {
    /** Over: the screens take keys again. */
    var opened by mutableStateOf(false)
    /** The first stage is over, so the screen under the cover may draw. */
    var shown by mutableStateOf(false)
    var notes by mutableStateOf(false)
    var swell by mutableStateOf(false)
    /** The landing's showreel may play. */
    var roll by mutableStateOf(false)
    /** The landing's showreel can play. */
    var landingReady by mutableStateOf(false)
    var soundsOver by mutableStateOf(false)

    /** Ends the opening, however it ends: the screens show, take keys, and the showreel may play. */
    fun over() {
        shown = true
        roll = true
        opened = true
    }
}

/** True while the opening covers the screens: they keep their state but take no keys. */
val LocalCovered = compositionLocalOf { false }

/**
 * The opening, over whatever the first screen is: the landing for a visitor, profiles or home for a
 * member. It waits until that screen has what it needs, and on the landing for the showreel too.
 */
@Composable
fun OpeningLayer(shell: Shell, seen: Seen, state: OpeningState) {
    if (!state.soundsOver) {
        Film(shell, "${seen.server}/tv/opening-chime.m4a", false, state.notes, Modifier.size(0.dp)) {}
        Film(shell, "${seen.server}/tv/opening-swell.m4a", false, state.swell, Modifier.size(0.dp)) {
            state.soundsOver = true
        }
    }
    if (state.opened) return
    val target = seen.route?.target
    val landing = target == Target.START
    val ready = seen.words.isNotEmpty() && target != null && target != Target.WAIT && (!landing || state.landingReady)
    Opening(ready, if (landing) FILM_LEAD else 0f) { moment ->
        when (moment) {
            Moment.NOTES -> state.notes = true
            Moment.WARM -> state.shown = true
            Moment.SECOND -> state.swell = true
            Moment.ROLL -> state.roll = true
            Moment.DONE -> state.over()
        }
    }
}

/** The brand's mark and text, `markHigh` reference pixels tall, as the opening leaves them at rest. */
@Composable
fun BrandLockup(markHigh: Float, modifier: Modifier = Modifier) {
    val art = artOf(LocalContext.current.resources)
    val k = markHigh / MARK_H
    Canvas(modifier.size((LOCK_W * k).roundToInt().ref, (MARK_H * k).roundToInt().ref)) {
        val s = size.height / MARK_H
        withTransform({
            scale(s, s, Offset.Zero)
            translate(-(WIDE - LOCK_W) / 2, -(MIDDLE - MARK_H / 2))
        }) { lockupAtRest(art, TV) }
    }
}

/** The pictures are made once for the run of the app and shared by the opening and the lockup. */
private fun artOf(resources: Resources): OpeningArt = openingArt(resources)

/** The opening itself. `ready` turns true when the screen under it can show; the rest waits for it. */
@Composable
fun Opening(ready: Boolean, lead: Float, moment: (Moment) -> Unit) {
    val resources = LocalContext.current.resources
    val art = remember { artOf(resources) }
    val leadNow by rememberUpdatedState(lead)
    val clock = remember { mutableFloatStateOf(0f) }
    val readyNow by rememberUpdatedState(ready)
    val tell by rememberUpdatedState(moment)
    // When the second stage starts. Unknown until the landing is ready.
    val second = remember { mutableFloatStateOf(Float.NaN) }
    val ringFrom = remember { mutableFloatStateOf(Float.NaN) }

    LaunchedEffect(Unit) {
        val start = withFrameNanos { it }
        val told = mutableSetOf<Moment>()
        fun once(at: Boolean, which: Moment) {
            if (at && told.add(which)) tell(which)
        }
        var readyAt = Float.NaN
        while (true) {
            val t = withFrameNanos { (it - start) / 1e9f }
            clock.floatValue = t
            if (readyAt.isNaN() && (readyNow || t >= WAIT_AT_MOST)) readyAt = t
            once(t >= NOTES_AT, Moment.NOTES)
            once(t >= REST, Moment.WARM)
            if (second.floatValue.isNaN() && t >= REST) {
                val ring = ringFrom.floatValue
                when {
                    // The showreel starts a little before the window, so the arrow opens onto the film.
                    !readyAt.isNaN() && ring.isNaN() -> second.floatValue = max(REST + SHORT_REST, readyAt + leadNow)
                    !readyAt.isNaN() -> second.floatValue = max(ring + 2 * RING_FADE, readyAt + leadNow)
                    ring.isNaN() && t >= REST + SHORT_REST -> ringFrom.floatValue = t
                }
            }
            val s = second.floatValue
            once(!s.isNaN() && t >= s - leadNow, Moment.ROLL)
            once(!s.isNaN() && t >= s, Moment.SECOND)
            if (!s.isNaN() && t >= s + SECOND) {
                tell(Moment.DONE)
                break
            }
        }
    }

    Box(Modifier.fillMaxSize().remote { tell(Moment.DONE); true }) {
        // Its own layer, so a new frame of the opening redraws nothing under it.
        Canvas(Modifier.fillMaxSize().graphicsLayer()) {
            val t = clock.floatValue
            val s = second.floatValue
            withTransform({ scale(size.width / WIDE, size.height / HIGH, Offset.Zero) }) {
                if (!s.isNaN() && t >= s) {
                    openingSecond(art, TV, t - s, Night)
                } else {
                    drawRect(Night, Offset.Zero, Size(WIDE, HIGH))
                    openingFirst(art, TV, min(t, REST))
                    val from = ringFrom.floatValue
                    if (!from.isNaN()) {
                        val fadeOut = if (s.isNaN()) 1f else 1 - smooth(span(t, s - RING_FADE, s))
                        ring(t, smooth(span(t, from, from + RING_FADE)) * fadeOut)
                    }
                }
            }
        }
    }
}
