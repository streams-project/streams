package streams.phone

import android.os.Process
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import contract.Target
import kotlinx.coroutines.delay
import streams.ports.Seen
import streams.shared.Accent
import streams.shared.Back
import streams.shared.LOCK_W
import streams.shared.MARK_H
import streams.shared.OpeningArt
import streams.shared.Stage
import streams.shared.lockupAtRest
import streams.shared.openingArtAt
import streams.shared.openingFirst
import streams.shared.smooth
import streams.shared.span
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt

// The phone's opening: the approved first stage, short and silent, in the reference's cold start
// rhythm: dark, the lockup builds out, holds, goes, and the first screen fades in. Times are seconds
// from the moment the screen goes dark.

/** The lockup starts to build. */
private const val BUILD_FROM = 0.75f

/**
 * The approved curves end slowly, so the lockup looks whole about 0.06 s before its last move ends:
 * this puts the look of it whole at the reference's 2.25 s.
 */
private const val BUILD_TO = 2.3f

/** It holds whole until here, then goes once the first screen is ready. */
private const val HELD = 2.5f

/** The lockup goes this fast: the reference cuts. */
private const val GO = 0.05f

/** Dark between the lockup and the first screen. */
private const val GAP = 0.1f

/** The first screen fades in over this, evenly, so its first frame already shows. */
private const val REVEAL = 0.25f

/** The spinner fades in over this when the app is still waiting at HELD. */
private const val SPIN_IN = 0.25f

/** Longest the opening waits for words (and for the profiles on the chooser) once the route is known. */
private const val WAIT_AT_MOST = 8f

/** The approved first stage: its line starts at 0.2 s and the lockup rests at 3.15 s. */
private const val FIRST_FROM = 0.2f
private const val FIRST_TO = 3.15f

/**
 * Seconds from the process starting to the screen going dark under the system's launch animation.
 * Measured on the Pixel 8 launching by intent: the process started 0.08 s before the first fully
 * dark frame of a screen recording.
 */
private const val DARK_AFTER_START = 0.08

/** A first frame later than this after the process started means the process was up before the launch. */
private const val LATE = 3f

/** Later waits show the lockup and the spinner only when they last longer than this, in ms. */
private const val LATER_WAIT_MS = 300L

/** The lockup's width at rest as a share of the screen's shorter side (the reference: 71.3 percent). */
private const val SHARE = 0.713f

/** How far the lockup's middle sits above the screen's middle. */
private val LIFT = 6.dp

/** The spinner: its gap below the lockup, its size across and its stroke (the reference: 67.5, 40, 4). */
private val SPIN_GAP = 67.5.dp
private val SPIN = 40.dp
private val SPIN_STROKE = 4.dp

/** This run's opening. A process has one: a cold start plays it, and nothing later does. */
@Stable
class OpeningRun internal constructor() {
    /** Over: the screens show in full and take touches. */
    var over by mutableStateOf(false)
        private set

    /**
     * The first screen draws under the opening's cover from the moment the lockup rests, so its
     * first, slow frame falls where nothing moves.
     */
    var under by mutableStateOf(false)
        private set

    /** How far the first screen has come in, from 0 to 1: the cover over it is the rest. */
    var shown by mutableFloatStateOf(0f)
        private set

    internal fun draw() {
        under = true
    }

    internal fun show(amount: Float) {
        shown = amount
    }

    internal fun finish() {
        under = true
        shown = 1f
        over = true
    }
}

/** The opening of this process. */
internal val thisRun = OpeningRun()

/**
 * Where the lockup and the spinner sit on a screen `w` by `h` pixels, `dp` pixels a dp: the
 * lockup's ink SHARE of the shorter side wide and centred across, its middle LIFT above the
 * screen's.
 */
private class Place(w: Float, h: Float, dp: Float, art: OpeningArt) {
    /** Screen pixels for one stage unit. */
    val k = SHARE * min(w, h) / art.inkWide
    private val middle = h / 2 - LIFT.value * dp
    val stage = Stage(w / k, h / k, middle / k)

    /** How far right the drawing moves, in pixels, so the ink rather than its box is centred. */
    val shift = (LOCK_W - art.inkWide) / 2 * k

    /** The spinner's top edge, in pixels. */
    val spinTop = (middle + MARK_H * k / 2 + SPIN_GAP.value * dp).roundToInt()
}

/** The art, sharp at this screen's size. */
@Composable
private fun phoneArt(): OpeningArt {
    val resources = LocalContext.current.resources
    return remember {
        val metrics = resources.displayMetrics
        openingArtAt(resources, SHARE * min(metrics.widthPixels, metrics.heightPixels) / LOCK_W)
    }
}

/** The spinner, the reference's size, in the brand's accent. */
@Composable
private fun Spinner(modifier: Modifier) {
    CircularProgressIndicator(
        modifier.size(SPIN),
        color = Accent,
        strokeWidth = SPIN_STROKE,
        trackColor = Color.Transparent,
        strokeCap = StrokeCap.Butt,
    )
}

/**
 * The opening, over the first screen, which is built under it and drawn under its cover once
 * `run.under`; `run.shown` lifts the cover. It plays from the first frame of the run, holds the
 * lockup with a spinner while the app still waits, and hands over to the first screen once that is
 * known: the chooser, home, or the way in.
 */
@Composable
internal fun Opening(seen: Seen, run: OpeningRun) {
    val art = phoneArt()
    val target = seen.route?.target
    val known = target != null && target != Target.WAIT
    val full = known && seen.words.isNotEmpty() && (target != Target.PROFILES || seen.profiles.isNotEmpty())
    val knownNow by rememberUpdatedState(known)
    val fullNow by rememberUpdatedState(full)
    val clock = remember { mutableFloatStateOf(0f) }
    // When the lockup starts to go. Unknown while the app waits.
    val goneAt = remember { mutableFloatStateOf(Float.NaN) }

    LaunchedEffect(Unit) {
        val first = withFrameNanos { it } / 1e9
        val started = Process.getStartUptimeMillis() / 1e3
        var dark = if (first - started > LATE) first else started + DARK_AFTER_START
        // A first frame later than the build's start delays the whole opening, so none of it is cut.
        dark = max(dark, first - BUILD_FROM)
        while (true) {
            val t = withFrameNanos { (it / 1e9 - dark).toFloat() }
            clock.floatValue = t
            if (t >= BUILD_TO && !run.under) run.draw()
            if (goneAt.floatValue.isNaN() && t >= HELD && (fullNow || (knownNow && t >= WAIT_AT_MOST))) {
                goneAt.floatValue = t
            }
            val g = goneAt.floatValue
            if (!g.isNaN()) {
                val from = g + GO + GAP
                run.show(span(t, from, from + REVEAL))
                if (t >= from + REVEAL) {
                    run.finish()
                    break
                }
            }
        }
    }

    val spinning by remember {
        derivedStateOf {
            val g = goneAt.floatValue
            clock.floatValue >= HELD && (g.isNaN() || clock.floatValue < g + GO) && (g.isNaN() || g > HELD)
        }
    }
    val dp = LocalDensity.current.density
    BoxWithConstraints(
        Modifier.fillMaxSize().pointerInput(Unit) {
            // Nothing under the opening takes a touch until it hands over.
            awaitPointerEventScope {
                while (true) awaitPointerEvent().changes.forEach { it.consume() }
            }
        },
    ) {
        val place = remember(constraints.maxWidth, constraints.maxHeight, dp) {
            Place(constraints.maxWidth.toFloat(), constraints.maxHeight.toFloat(), dp, art)
        }
        // The cover: the brand's background over the first screen, lifted as it comes in.
        Canvas(Modifier.fillMaxSize().graphicsLayer { alpha = 1 - run.shown }) { drawRect(Back) }
        // Its own layer, so a new frame of the opening redraws nothing under it.
        Canvas(
            Modifier.fillMaxSize().graphicsLayer {
                val g = goneAt.floatValue
                alpha = if (g.isNaN()) 1f else 1 - span(clock.floatValue, g, g + GO)
            },
        ) {
            val t = clock.floatValue
            val first = FIRST_FROM + (t - BUILD_FROM) * (FIRST_TO - FIRST_FROM) / (BUILD_TO - BUILD_FROM)
            withTransform({
                translate(place.shift, 0f)
                scale(place.k, place.k, Offset.Zero)
            }) {
                openingFirst(art, place.stage, first.coerceAtMost(FIRST_TO))
            }
        }
        if (spinning) {
            Spinner(
                Modifier.align(Alignment.TopCenter).offset { IntOffset(0, place.spinTop) }.graphicsLayer {
                    val t = clock.floatValue
                    val g = goneAt.floatValue
                    val until = if (g.isNaN()) t else min(t, g)
                    alpha = smooth(span(until, HELD, HELD + SPIN_IN)) * (if (g.isNaN()) 1f else 1 - span(t, g, g + GO))
                },
            )
        }
    }
}

/**
 * A wait after the opening (a new window in the same run, before the session is known): the lockup
 * at rest with the spinner, as the opening leaves them, shown only if the wait lasts.
 */
@Composable
internal fun Waiting() {
    val art = phoneArt()
    var long by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) {
        delay(LATER_WAIT_MS)
        long = true
    }
    val alpha by animateFloatAsState(if (long) 1f else 0f, tween(200), label = "waiting")
    val dp = LocalDensity.current.density
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val place = remember(constraints.maxWidth, constraints.maxHeight, dp) {
            Place(constraints.maxWidth.toFloat(), constraints.maxHeight.toFloat(), dp, art)
        }
        if (long) {
            Canvas(Modifier.fillMaxSize().graphicsLayer { this.alpha = alpha }) {
                withTransform({
                    translate(place.shift, 0f)
                    scale(place.k, place.k, Offset.Zero)
                }) { lockupAtRest(art, place.stage) }
            }
            Spinner(Modifier.align(Alignment.TopCenter).offset { IntOffset(0, place.spinTop) }.graphicsLayer { this.alpha = alpha })
        }
    }
}
