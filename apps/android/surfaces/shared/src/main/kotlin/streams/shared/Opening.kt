package streams.shared

import android.content.res.Resources
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Canvas
import androidx.compose.ui.graphics.ClipOp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Matrix
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.drawscope.CanvasDrawScope
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.LayoutDirection
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.PI
import kotlin.math.pow
import kotlin.math.sin

// The brand's opening, drawn live: the mark and text from opening.json, and the motion of direction
// C as the owner approved it, read from the renderer that made the old video
// (streams-studio/showreel/opening.py). Each family lays it on its own stage and adds its own
// parts: the television its films, sounds and window, the phone its short timing and its spinner.

/** The lockup's width at rest, in stage units. */
const val LOCK_W = 760f

/** The mark's height at rest, which is also the lockup's height. */
const val MARK_H = LOCK_W / 3.922f

/** How large the mark grows in the first stage before it settles into the lockup. */
const val BIG = 1.4f

/**
 * Where an opening is drawn, in stage units: `wide` by `high`, the lockup resting centred across
 * it on the line `middle`. A screen draws a stage at one scale on both axes, so the lockup keeps
 * its shape; the television's stage is 1920 by 1080 with the lockup on the middle line.
 */
class Stage(val wide: Float, val high: Float, val middle: Float = high / 2)

private val LineFrom = Color(0xFF5A78FF)
private val LineTo = Color(0xFF9696FF)

/** The approved curve of every move: smooth at both ends. */
fun smooth(x: Float): Float {
    val c = x.coerceIn(0f, 1f)
    return c * c * c * (c * (6 * c - 15) + 10)
}

/** How far `t` is from `a` to `b`, from 0 to 1. */
fun span(t: Float, a: Float, b: Float) = ((t - a) / (b - a)).coerceIn(0f, 1f)

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

/** A picture drawn once, so a frame only moves and scales it. */
private fun bake(width: Int, height: Int, draw: DrawScope.() -> Unit): ImageBitmap {
    val image = ImageBitmap(width, height)
    CanvasDrawScope().draw(Density(1f), LayoutDirection.Ltr, Canvas(image), Size(width.toFloat(), height.toFloat()), draw)
    return image
}

/**
 * The brand's mark and text, from opening.json, made by platform/tools/make_opening.py. The parts
 * are pictures made once: `bake` picture pixels for one unit of the mark's art, `textBake` for one
 * unit of the text's. A screen picks them near the largest size it shows each part at; the
 * television keeps 0.9 and 0.5, as its mark is never shown above 0.86 pixels a unit.
 */
class OpeningArt(resources: Resources, val bake: Float = 0.9f, val textBake: Float = 0.5f) {
    private val json = JSONObject(resources.openRawResource(R.raw.opening).bufferedReader().readText())
    private val mark = json.getJSONObject("mark")
    private val box = mark.getJSONArray("box").floats()

    /** The path of the arrow, for the window it opens onto the screen under it. */
    val arrowPath = path(mark.getJSONObject("arrow").getString("path"))
    private val word = json.getJSONObject("word")
    private val view = word.getJSONArray("view").floats()

    /** The middle of the mark, which every move is anchored to. */
    val centre = Offset((box[0] + box[2]) / 2, (box[1] + box[3]) / 2)

    /** Stage units for one unit of the art, at lockup size. */
    val unit = MARK_H / (box[3] - box[1])
    val markWide = MARK_H * (box[2] - box[0]) / (box[3] - box[1])
    private val cap = MARK_H / 1.42f
    val textUnit = cap * 1.08f / view[3]
    val textWide = view[2] * textUnit
    val textHigh = view[3] * textUnit

    /**
     * The lockup's inked width at rest, in stage units: from the mark's left edge to the last
     * letter's right edge, which stops short of the wordmark's view box.
     */
    val inkWide = markWide + cap * 0.2f + (path(word.getString("path")).getBounds().right - view[0]) * textUnit

    /** Where the mark's middle rests on a stage. */
    fun markX(stage: Stage) = (stage.wide - LOCK_W) / 2 + markWide / 2

    /** Where the text's left edge rests on a stage. */
    fun textX(stage: Stage) = (stage.wide - LOCK_W) / 2 + markWide + cap * 0.2f

    /** Where the pictures of the two parts start, in art units: the mark's box with room around it. */
    val corner = Offset(box[0] - 8, box[1] - 8)
    private val pictureWide = ((box[2] - box[0] + 16) * bake).toInt()
    private val pictureHigh = ((box[3] - box[1] + 16) * bake).toInt()

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

    val text = bake((view[2] * textBake).toInt(), (view[3] * textBake).toInt()) {
        withTransform({
            scale(textBake, textBake, Offset.Zero)
            translate(-view[0], -view[1])
        }) { drawPath(path(word.getString("path")), Color.White) }
    }

    private fun bakePart(draw: DrawScope.() -> Unit) = bake(pictureWide, pictureHigh) {
        withTransform({
            scale(bake, bake, Offset.Zero)
            translate(-corner.x, -corner.y)
        }) { draw() }
    }

    private fun fill(json: JSONObject) = Fill(json).brush

    private fun path(d: String): Path = PathParser().parsePathString(d).toPath()
}

private val kept = HashMap<Pair<Float, Float>, OpeningArt>()

/** The pictures are made once for the run of the app, for each size a screen asks for. */
fun openingArt(resources: Resources, bake: Float = 0.9f, textBake: Float = 0.5f): OpeningArt =
    kept.getOrPut(bake to textBake) { OpeningArt(resources, bake, textBake) }

/**
 * The pictures made sharp for a screen that shows `pixels` screen pixels for one stage unit: the
 * mark at the largest it grows to in the first stage (BIG, with its settle), the text at rest.
 */
fun openingArtAt(resources: Resources, pixels: Float): OpeningArt {
    val json = JSONObject(resources.openRawResource(R.raw.opening).bufferedReader().readText())
    val box = json.getJSONObject("mark").getJSONArray("box").floats()
    val view = json.getJSONObject("word").getJSONArray("view").floats()
    val unit = MARK_H / (box[3] - box[1])
    val textUnit = MARK_H / 1.42f * 1.08f / view[3]
    fun up(x: Float) = kotlin.math.ceil(x * 100) / 100
    return openingArt(resources, up(pixels * unit * BIG * 1.03f), up(pixels * textUnit))
}

/** One part of the mark at a place and size on the stage. */
private fun DrawScope.part(art: OpeningArt, stage: Stage, image: ImageBitmap, x: Float, scale: Float, alpha: Float) {
    if (alpha <= 0f) return
    withTransform({
        translate(x, stage.middle)
        scale(scale * art.unit / art.bake, scale * art.unit / art.bake, Offset.Zero)
        translate((art.corner.x - art.centre.x) * art.bake, (art.corner.y - art.centre.y) * art.bake)
    }) {
        drawImage(image, dstSize = IntSize(image.width, image.height), alpha = alpha, filterQuality = FilterQuality.High)
    }
}

private fun DrawScope.ribbon(art: OpeningArt, stage: Stage, x: Float, scale: Float, alpha: Float) =
    part(art, stage, art.ribbon, x, scale, alpha)

private fun DrawScope.arrow(art: OpeningArt, stage: Stage, x: Float, scale: Float, alpha: Float) =
    part(art, stage, art.arrow, x, scale, alpha)

/** The text, coming out from behind the mark: nothing left of `edge` shows. */
private fun DrawScope.text(art: OpeningArt, stage: Stage, x: Float, edge: Float, alpha: Float) {
    if (alpha <= 0f) return
    clipRect(left = edge, top = 0f, right = stage.wide, bottom = stage.high) {
        withTransform({
            translate(x, stage.middle - art.textHigh / 2)
            scale(art.textUnit / art.textBake, art.textUnit / art.textBake, Offset.Zero)
        }) {
            drawImage(art.text, dstSize = IntSize(art.text.width, art.text.height), alpha = alpha, filterQuality = FilterQuality.High)
        }
    }
}

/** Mark and text: `q` 0 is the mark alone in the middle, 1 is both at rest. */
fun DrawScope.lockup(art: OpeningArt, stage: Stage, q: Float) {
    val markX = art.markX(stage)
    val x = stage.wide / 2 + (markX - stage.wide / 2) * q
    if (q > 0.002f) text(art, stage, art.textX(stage) + (x - markX) - (art.textWide + 60) * (1 - q), x, 1f)
    ribbon(art, stage, x, BIG + (1 - BIG) * q, 1f)
    arrow(art, stage, x, BIG + (1 - BIG) * q, 1f)
}

/** The lockup as the opening leaves it at rest, `alpha` opaque. */
fun DrawScope.lockupAtRest(art: OpeningArt, stage: Stage, alpha: Float = 1f) {
    val markX = art.markX(stage)
    text(art, stage, art.textX(stage), markX, alpha)
    ribbon(art, stage, markX, 1f, alpha)
    arrow(art, stage, markX, 1f, alpha)
}

/**
 * The first stage, `t` seconds in (0 to 3.15, at rest after): a line pulls back into the arrow,
 * the ribbon slides in, the text comes out.
 */
fun DrawScope.openingFirst(art: OpeningArt, stage: Stage, t: Float) {
    if (t >= 2.25f) {
        lockup(art, stage, smooth(span(t, 2.25f, 3.15f)))
        return
    }
    val middle = stage.middle
    val grow = smooth(span(t, 0.2f, 0.95f))
    val back = smooth(span(t, 0.95f, 1.7f))
    val from = stage.wide / 2 - 40
    val reach = 560 * grow * (1 - back)
    if (back < 1 && reach > 0) {
        val line = Brush.horizontalGradient(listOf(LineFrom, LineTo), from, from + reach)
        drawRect(
            Brush.verticalGradient(
                listOf(Color.Transparent, LineFrom.copy(alpha = 0.35f), Color.Transparent),
                middle - 40,
                middle + 40,
            ),
            Offset(from - 20, middle - 40),
            Size(reach + 40, 80f),
        )
        drawRect(line, Offset(from, middle - 4), Size(reach, 8f))
    }
    val tri = smooth(span(t, 0.25f, 0.75f))
    val open = smooth(span(t, 1.05f, 2.0f))
    val settle = 0.025f * sin(PI.toFloat() * span(t, 1.8f, 2.25f))
    val size = (0.16f + 0.02f * tri + (BIG - 0.18f) * open) * (1 + settle)
    val rib = smooth(span(t, 1.5f, 2.1f))
    ribbon(art, stage, stage.wide / 2 + 54 * (1 - rib), BIG * (0.92f + 0.08f * rib), rib)
    arrow(art, stage, from + 40 * open, size, tri)
}

/** How long the second stage takes, in seconds. */
const val OPENING_SECOND = 2.0f

/**
 * The second stage, `u` seconds in: the text and ribbon go, and the arrow grows into a window onto
 * the screen already drawn under this layer, through a cover of colour `cover`.
 */
fun DrawScope.openingSecond(art: OpeningArt, stage: Stage, u: Float, cover: Color) {
    val markX = art.markX(stage)
    val p = span(u, 0.3f, OPENING_SECOND)
    val grown = 1 + 90 * smooth(p).pow(2.7f)
    val x = markX + (stage.wide / 2 - markX) * smooth(span(p, 0f, 0.55f))
    val see = smooth(span(p, 0.02f, 0.36f))
    val window = Path().apply { addPath(art.arrowPath) }
    window.transform(
        Matrix().apply {
            translate(x, stage.middle)
            scale(grown * art.unit, grown * art.unit)
            translate(-art.centre.x, -art.centre.y)
        },
    )
    // The cover has the arrow cut out of it, and the screen under this layer shows through.
    val sheet = Path().apply {
        fillType = PathFillType.EvenOdd
        addRect(Rect(0f, 0f, stage.wide, stage.high))
        addPath(window)
    }
    drawPath(sheet, cover)
    val words = 1 - smooth(span(u, 0.15f, 0.65f))
    val band = 1 - smooth(span(u, 0.1f, 0.5f))
    if (words > 0f || band > 0f) {
        clipPath(window, ClipOp.Difference) {
            text(art, stage, art.textX(stage), markX, words)
            ribbon(art, stage, markX, 1f, band)
        }
    }
    arrow(art, stage, x, grown, 1 - see)
}
