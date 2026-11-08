package streams.tv

import androidx.compose.foundation.shape.RoundedCornerShape
import streams.shared.Corner
import streams.shared.ref
import kotlin.math.roundToInt

/** How much larger a corner is on a television than on a phone, in reference pixels per brand unit. */
private const val TV_SCALE = 1.5f

private fun tv(brand: Int): Int = (brand * TV_SCALE).roundToInt()

/**
 * The brand's corners on a television, by role, in reference pixels. Every rounded part takes one of
 * these, so brand.json changes them all at once.
 */
object Round {
    /** Posters, stills, episode pictures, the picture grid. */
    val card = tv(Corner.CARD)

    /** The hero, the landing cards, menus, panels. */
    val panel = tv(Corner.PANEL)

    /** Fields, boxes of text, the code plate, PIN boxes. */
    val field = tv(Corner.FIELD)

    /** Keys and letters: smaller than a field. */
    val key = field * 2 / 3

    /** Badges and small labels. */
    val chip = field / 2

    /** A corner that sits inside another, `inset` from its edge, so the two curves share a centre. */
    fun inside(outer: Int, inset: Int): Int = (outer - inset).coerceAtLeast(chip)

    /** A profile's face, its focus ring and the add tile: the corner is a share of the size, so it grows with it. */
    val face = RoundedCornerShape(card * 100 / 130)
}

/** The page margin on a television, left and right, in reference pixels. Pages line up their edges on it. */
const val PAGE_EDGE = 72

/** Pages without the top bar (settings, help, profile edits) keep their words on this column. */
const val SUB_EDGE = 130

fun shapeOf(corner: Int) = RoundedCornerShape(corner.ref)
