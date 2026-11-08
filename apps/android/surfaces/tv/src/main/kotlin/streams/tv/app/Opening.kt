package streams.tv

import streams.shared.Art
import android.content.res.Resources
import androidx.compose.foundation.Canvas
import kotlin.math.roundToInt
import streams.shared.ref
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
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.ClipOp
import androidx.compose.ui.graphics.Matrix
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.Canvas
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.drawscope.CanvasDrawScope
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.platform.LocalContext
import org.json.JSONArray
import org.json.JSONObject
import streams.shared.R
import kotlin.math.PI
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sin

// The opening, drawn live. Its motion is direction C as the owner approved it, read from the
// renderer that made the old video (streams-studio/showreel/opening.py).

private const val WIDE = 1920f
private const val HIGH = 1080f
private const val MIDDLE = HIGH / 2
private const val LOCK_W = 760f
private const val MARK_H = LOCK_W / 3.922f
private const val BIG = 1.4f

/** When the first stage ends and the mark and text rest. */
private const val REST = 3.2f
/** The shortest rest when everything is ready before the first stage ends. */
private const val SHORT_REST = 0.6f
/** Longest the opening waits for the landing before it goes on anyway. */
private const val WAIT_AT_MOST = 8f
private const val RING_FADE = 0.35f
private const val SECOND = 2.0f
private const val NOTES_AT = 2.05f
/** How long before the window opens the showreel starts, under the cover, past its fade from black. */
const val FILM_LEAD = 1.0f

private val Night = Color(0xFF06070F)
private val LineFrom = Color(0xFF5A78FF)
private val LineTo = Color(0xFF9696FF)
private val Ring = Color(0xFF2F6BFF)

private fun smooth(x: Float): Float {
    val c = x.coerceIn(0f, 1f)
    return c * c * c * (c * (6 * c - 15) + 10)
}

private fun span(t: Float, a: Float, b: Float) = ((t - a) / (b - a)).coerceIn(0f, 1f)

private fun color(hex: String): Color {
    val v = hex.removePrefix("#").toLong(16)
    return if (hex.length > 7) {
        Color(((v and 0xFF) shl 24 or (v shr 8)).toInt())
    } else {
        Color((0xFF000000 or v).toInt())
    }
}

private fun JSONArray.floats() = FloatArray(length()) { getDouble(it).toFloat() }

/** A gradient laid on the art's own 1024 grid. */
private class Fill(json: JSONObject) {
    private val from = json.getJSONArray("from").floats()
    private val to = json.getJSONArray("to").floats()
    private val stops = json.getJSONArray("stops").let { list ->
        Array(list.length()) { list.getJSONArray(it).let { s -> s.getDouble(0).toFloat() to color(s.getString(1)) } }
    }

    val brush = Brush.linearGradient(*stops, start = Offset(from[0], from[1]), end = Offset(to[0], to[1]))
}

/** Picture pixels for one unit of the art. The mark is never shown above 0.86 pixels a unit. */
private const val BAKE = 0.9f
private const val TEXT_BAKE = 0.5f

/** A picture drawn once, so a frame only moves and scales it. */
private fun bake(width: Int, height: Int, draw: DrawScope.() -> Unit): ImageBitmap {
    val image = ImageBitmap(width, height)
    CanvasDrawScope().draw(Density(1f), LayoutDirection.Ltr, Canvas(image), Size(width.toFloat(), height.toFloat()), draw)
    return image
}

/** The brand's mark and text, from opening.json, made by platform/tools/make_opening.py. */
private class Art(resources: Resources) {
    private val json = JSONObject(resources.openRawResource(R.raw.opening).bufferedReader().readText())
    private val mark = json.getJSONObject("mark")
    private val box = mark.getJSONArray("box").floats()
    /** The path of the arrow, for the window it opens onto the landing. */
    val arrowPath = path(mark.getJSONObject("arrow").getString("path"))
    private val word = json.getJSONObject("word")
    private val view = word.getJSONArray("view").floats()

    /** The middle of the mark, which every move is anchored to. */
    val centre = Offset((box[0] + box[2]) / 2, (box[1] + box[3]) / 2)
    /** Screen pixels for one unit of the art, at lockup size. */
    val unit = MARK_H / (box[3] - box[1])
    val markWide = MARK_H * (box[2] - box[0]) / (box[3] - box[1])
    private val cap = MARK_H / 1.42f
    val markX = (WIDE - LOCK_W) / 2 + markWide / 2
    val textX = (WIDE - LOCK_W) / 2 + markWide + cap * 0.2f
    val textUnit = cap * 1.08f / view[3]
    val textWide = view[2] * textUnit
    val textHigh = view[3] * textUnit

    /** Where the pictures of the two parts start, in art units: the mark's box with room around it. */
    val corner = Offset(box[0] - 8, box[1] - 8)
    private val pictureWide = ((box[2] - box[0] + 16) * BAKE).toInt()
    private val pictureHigh = ((box[3] - box[1] + 16) * BAKE).toInt()

    val ribbon = bakePart { drawPath(path(mark.getJSONObject("ribbon").getString("path")), fill(mark.getJSONObject("ribbon").getJSONObject("fill"))) }

    val arrow = bakePart {
        drawPath(arrowPath, fill(mark.getJSONObject("arrow").getJSONObject("fill")))
        clipPath(arrowPath) {
            val shine = mark.getJSONObject("shine")
            drawRect(fill(shine.getJSONObject("fill")), Offset.Zero, Size(1024f, 1024f), shine.getDouble("opacity").toFloat())
            // The glow is a blurred ellipse: a soft radial fall off on a circle stretched to its shape.
            val glow = mark.getJSONObject("glow")
            val blur = glow.getDouble("blur").toFloat()
            val b = path(glow.getString("path")).getBounds()
            val r = Rect(b.left - 2 * blur, b.top - 2 * blur, b.right + 2 * blur, b.bottom + 2 * blur)
            val tone = color(glow.getString("color"))
            withTransform({ scale(1f, r.height / r.width, r.center) }) {
                drawCircle(
                    Brush.radialGradient(
                        0f to tone,
                        (b.width / r.width) to tone.copy(alpha = 0.55f),
                        1f to tone.copy(alpha = 0f),
                        center = r.center,
                        radius = r.width / 2,
                    ),
                    r.width / 2,
                    r.center,
                    glow.getDouble("opacity").toFloat(),
                )
            }
        }
    }

    val text = bake((view[2] * TEXT_BAKE).toInt(), (view[3] * TEXT_BAKE).toInt()) {
        withTransform({
            scale(TEXT_BAKE, TEXT_BAKE, Offset.Zero)
            translate(-view[0], -view[1])
        }) { drawPath(path(word.getString("path")), Color.White) }
    }

    private fun bakePart(draw: DrawScope.() -> Unit) = bake(pictureWide, pictureHigh) {
        withTransform({
            scale(BAKE, BAKE, Offset.Zero)
            translate(-corner.x, -corner.y)
        }) { draw() }
    }

    private fun fill(json: JSONObject) = Fill(json).brush

    private fun path(d: String): Path = PathParser().parsePathString(d).toPath()
}

/** One part of the mark at a place and size on the screen. */
private fun DrawScope.part(art: Art, image: ImageBitmap, x: Float, scale: Float, alpha: Float) {
    if (alpha <= 0f) return
    withTransform({
        translate(x, MIDDLE)
        scale(scale * art.unit / BAKE, scale * art.unit / BAKE, Offset.Zero)
        translate((art.corner.x - art.centre.x) * BAKE, (art.corner.y - art.centre.y) * BAKE)
    }) {
        drawImage(image, dstSize = IntSize(image.width, image.height), alpha = alpha, filterQuality = FilterQuality.High)
    }
}

private fun DrawScope.ribbon(art: Art, x: Float, scale: Float, alpha: Float) = part(art, art.ribbon, x, scale, alpha)

private fun DrawScope.arrow(art: Art, x: Float, scale: Float, alpha: Float) = part(art, art.arrow, x, scale, alpha)

/** The text, coming out from behind the mark: nothing left of `edge` shows. */
private fun DrawScope.text(art: Art, x: Float, edge: Float, alpha: Float) {
    if (alpha <= 0f) return
    clipRect(left = edge, top = 0f, right = WIDE, bottom = HIGH) {
        withTransform({
            translate(x, MIDDLE - art.textHigh / 2)
            scale(art.textUnit / TEXT_BAKE, art.textUnit / TEXT_BAKE, Offset.Zero)
        }) {
            drawImage(art.text, dstSize = IntSize(art.text.width, art.text.height), alpha = alpha, filterQuality = FilterQuality.High)
        }
    }
}

/** Mark and text: `q` 0 is the mark alone in the middle, 1 is both at rest. */
private fun DrawScope.lockup(art: Art, q: Float) {
    val x = WIDE / 2 + (art.markX - WIDE / 2) * q
    if (q > 0.002f) text(art, art.textX + (x - art.markX) - (art.textWide + 60) * (1 - q), x, 1f)
    ribbon(art, x, BIG + (1 - BIG) * q, 1f)
    arrow(art, x, BIG + (1 - BIG) * q, 1f)
}

/** The first stage: a line pulls back into the arrow, the ribbon slides in, the text comes out. */
private fun DrawScope.first(art: Art, t: Float) {
    if (t >= 2.25f) {
        lockup(art, smooth(span(t, 2.25f, 3.15f)))
        return
    }
    val grow = smooth(span(t, 0.2f, 0.95f))
    val back = smooth(span(t, 0.95f, 1.7f))
    val from = WIDE / 2 - 40
    val reach = 560 * grow * (1 - back)
    if (back < 1 && reach > 0) {
        val line = Brush.horizontalGradient(listOf(LineFrom, LineTo), from, from + reach)
        drawRect(
            Brush.verticalGradient(
                listOf(Color.Transparent, LineFrom.copy(alpha = 0.35f), Color.Transparent),
                MIDDLE - 40,
                MIDDLE + 40,
            ),
            Offset(from - 20, MIDDLE - 40),
            Size(reach + 40, 80f),
        )
        drawRect(line, Offset(from, MIDDLE - 4), Size(reach, 8f))
    }
    val tri = smooth(span(t, 0.25f, 0.75f))
    val open = smooth(span(t, 1.05f, 2.0f))
    val settle = 0.025f * sin(PI.toFloat() * span(t, 1.8f, 2.25f))
    val size = (0.16f + 0.02f * tri + (BIG - 0.18f) * open) * (1 + settle)
    val rib = smooth(span(t, 1.5f, 2.1f))
    ribbon(art, WIDE / 2 + 54 * (1 - rib), BIG * (0.92f + 0.08f * rib), rib)
    arrow(art, from + 40 * open, size, tri)
}

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

/**
 * The second stage, `u` seconds in: the text and ribbon go, and the arrow grows into a window onto
 * the landing, which is already drawn under this cover.
 */
private fun DrawScope.second(art: Art, u: Float) {
    val p = span(u, 0.3f, SECOND)
    val grown = 1 + 90 * smooth(p).pow(2.7f)
    val x = art.markX + (WIDE / 2 - art.markX) * smooth(span(p, 0f, 0.55f))
    val see = smooth(span(p, 0.02f, 0.36f))
    val window = Path().apply { addPath(art.arrowPath) }
    window.transform(
        Matrix().apply {
            translate(x, MIDDLE)
            scale(grown * art.unit, grown * art.unit)
            translate(-art.centre.x, -art.centre.y)
        },
    )
    // The cover has the arrow cut out of it, and the landing under this layer shows through.
    val cover = Path().apply {
        fillType = PathFillType.EvenOdd
        addRect(Rect(0f, 0f, WIDE, HIGH))
        addPath(window)
    }
    drawPath(cover, Night)
    val words = 1 - smooth(span(u, 0.15f, 0.65f))
    val band = 1 - smooth(span(u, 0.1f, 0.5f))
    if (words > 0f || band > 0f) {
        clipPath(window, ClipOp.Difference) {
            text(art, art.textX, art.markX, words)
            ribbon(art, art.markX, 1f, band)
        }
    }
    arrow(art, x, grown, 1 - see)
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
        }) {
            text(art, art.textX, art.markX, 1f)
            ribbon(art, art.markX, 1f, 1f)
            arrow(art, art.markX, 1f, 1f)
        }
    }
}

private var keptArt: Art? = null

/** The pictures are made once for the run of the app and shared by the opening and the lockup. */
private fun artOf(resources: Resources): Art = keptArt ?: Art(resources).also { keptArt = it }

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
                    second(art, t - s)
                } else {
                    drawRect(Night, Offset.Zero, Size(WIDE, HIGH))
                    first(art, min(t, REST))
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
