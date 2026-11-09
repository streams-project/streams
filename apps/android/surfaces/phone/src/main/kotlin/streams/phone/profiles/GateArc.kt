package streams.phone.profiles

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import kotlin.math.asin
import kotlin.math.min

/**
 * The circular arc top edge of the chooser sheet: apex at center, sag at edges, and
 * radius ~1465 dp, measured from the Pixel 8 and iPhone reference frames.
 */
class ArcSheetShape(
    val sagDp: Dp = 15.2.dp,
    val radiusDp: Dp = 1465.dp,
) : Shape {
    override fun createOutline(
        size: Size,
        layoutDirection: LayoutDirection,
        density: Density,
    ): Outline {
        val w = size.width
        val h = size.height
        val halfW = w / 2f
        val sag = with(density) { sagDp.toPx() }
        val r = with(density) { radiusDp.toPx() }

        val path = Path().apply {
            if (r > 0f && halfW > 0f) {
                val clampedHalfW = min(halfW, r)
                val thetaRad = asin((clampedHalfW / r).toDouble()).toFloat()
                val thetaDeg = Math.toDegrees(thetaRad.toDouble()).toFloat()

                // Center is at (halfW, r). At x=halfW, y=0 (the apex).
                // At x=0 and x=w, y = r - sqrt(r^2 - halfW^2) ≈ sag.
                val circleRect = Rect(
                    center = Offset(halfW, r),
                    radius = r,
                )
                arcTo(
                    rect = circleRect,
                    startAngleDegrees = 270f - thetaDeg,
                    sweepAngleDegrees = 2f * thetaDeg,
                    forceMoveTo = true,
                )
            } else {
                moveTo(0f, 0f)
                lineTo(w, 0f)
            }
            lineTo(w, h)
            lineTo(0f, h)
            close()
        }
        return Outline.Generic(path)
    }
}
