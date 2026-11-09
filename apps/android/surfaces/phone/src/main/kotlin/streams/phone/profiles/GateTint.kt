package streams.phone.profiles

import android.graphics.Bitmap
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.palette.graphics.Palette

/**
 * The extracted hue and saturation from a title's art, applying the measured brightness model.
 */
data class GateTint(
    val h: Float,
    val s: Float,
) {
    companion object {
        val Neutral = GateTint(0f, 0f)

        fun hsv(h: Float, s: Float, v: Float): Color {
            val rgb = android.graphics.Color.HSVToColor(floatArrayOf(h, s, v))
            return Color(rgb)
        }

        fun fromBitmap(bitmap: Bitmap): GateTint {
            val palette = Palette.from(bitmap).generate()
            val swatch = palette.darkMutedSwatch ?: palette.dominantSwatch ?: return Neutral
            val hsv = FloatArray(3)
            android.graphics.Color.colorToHSV(swatch.rgb, hsv)
            return GateTint(h = hsv[0], s = maxOf(hsv[1], 0.20f))
        }
    }

    /** Bottom fade target and strip between art and arc: HSV(h, s, 0.13). */
    val bandColor: Color
        get() = if (this == Neutral) Color(0xFF1A1A1A) else hsv(h, s, 0.13f)

    /** Edit tile: HSV(h, s, 0.44) with 20% white overlay. */
    val editTileColor: Color
        get() {
            if (this == Neutral) return Color(0xFF333333)
            val base = hsv(h, s, 0.44f)
            return Color(
                red = base.red * 0.8f + 0.2f,
                green = base.green * 0.8f + 0.2f,
                blue = base.blue * 0.8f + 0.2f,
                alpha = 1f,
            )
        }

    /**
     * Sheet gradient: V starts at 0.13 from apex down to y 702.4 dp, then rises linearly to 0.40 at bottom.
     */
    fun sheetBrush(sheetHeightPx: Float): Brush {
        if (this == Neutral) {
            return Brush.verticalGradient(
                colors = listOf(Color(0xFF1A1A1A), Color(0xFF1A1A1A)),
            )
        }
        val stopLocation = if (sheetHeightPx > 0f) {
            (40f / sheetHeightPx).coerceIn(0.04f, 0.25f)
        } else {
            0.08f
        }
        val c0 = hsv(h, s, 0.13f)
        val c1 = hsv(h, s, 0.40f)
        return Brush.verticalGradient(
            0.0f to c0,
            stopLocation to c0,
            1.0f to c1,
        )
    }
}
