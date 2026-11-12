package streams.tv.views;

import android.content.Context;
import android.view.View;

/**
 * The half-dp reference scale, in Java. The twin of
 * {@code apps/android/surfaces/shared/src/main/kotlin/streams/shared/Measure.kt:12} and {@code :15},
 * and of {@code apps/apple/Sources/SurfacesTelevision/Measure.swift}.
 *
 * Every television lays out at 960 by 540 dp, and the screen references measure 1920 by 1080, so a
 * reference pixel is half a dp. These keep the numbers in the code the same as the measurements.
 *
 * <h2>Why this is a class and not a generated dimension file</h2>
 * XML does hold a fractional dimension, so {@code 311} reference pixels has a literal form:
 * {@code 155.5dp}. Note 210 rejects that form and this class is what it accepted instead. The
 * reason is not the arithmetic, it is the reading: pre-dividing every measured number makes the
 * measurement disappear from the source, so the next person to open the layout cannot find the
 * number that was measured off the capture. Every call site here keeps its measurement as an
 * integer literal on the line it is on today -- {@code Ref.px(view, 311)}, not {@code 155.5} -- and
 * the halving lives in exactly one place, as it does in Kotlin and in Swift.
 *
 * <h2>The arithmetic, stated because two toolkits have to agree</h2>
 * Compose converts a {@code Dp} to pixels as {@code (value * density)} rounded to the nearest
 * integer, and {@code Math.round(float)} is the same rounding for a positive value (both are
 * {@code floor(x + 0.5)}). So {@link #px(int, float)} lands on the pixel Compose lands on, and a
 * screen rewritten from Kotlin to Java does not move by one pixel on the way.
 *
 * A sum is converted once, never summed after converting: {@code px(72 + 904 + 19)} is the
 * three-part derivation at {@code tv/browse/Rows.kt:137}, and summing three roundings instead can
 * differ from it by a pixel. Writing the sum is therefore not a style choice.
 *
 * <h2>No stored density</h2>
 * Note 210 describes reading the density once from the root view's {@code DisplayMetrics} and
 * keeping it. This holds none: a stored density is process-wide state that is wrong on the second
 * display and stale after a configuration change, and nothing here needs it. The density is asked
 * of whatever view is being measured, which is one call per layout pass in {@link RefLayout}.
 */
public final class Ref {
    /** The reference canvas: the width written out at {@code tv/browse/HeroCard.kt:54} and {@code tv/browse/TopBar.kt:117}. */
    public static final int W = 1920;

    /** And its height. */
    public static final int H = 1080;

    /**
     * The layout this surface really draws in, in dp. Not used in the arithmetic: it is here so the
     * derivation is readable from the code rather than only from the comment, and the test holds
     * {@code dp(W)} against it.
     */
    public static final int LAYOUT_W = 960;

    /** And its height in dp. */
    public static final int LAYOUT_H = 540;

    private Ref() {
    }

    /**
     * A reference measurement in dp: half of it, exactly, keeping the half.
     *
     * @param ref the measurement as it was taken off a 1920 by 1080 capture
     */
    public static float dp(int ref) {
        return ref / 2f;
    }

    /** A reference measurement in pixels on a screen of this density. */
    public static int px(int ref, float density) {
        return Math.round(dp(ref) * density);
    }

    /** The same, on the screen this view is on. */
    public static int px(View view, int ref) {
        return px(ref, density(view));
    }

    /** The same, from a context, for a view that is being built and is not attached yet. */
    public static int px(Context context, int ref) {
        return px(ref, context.getResources().getDisplayMetrics().density);
    }

    /** The density of the screen a view is on, read where it is needed instead of kept. */
    public static float density(View view) {
        return view.getResources().getDisplayMetrics().density;
    }

    /**
     * A reference font size, in sp. The twin of {@code Measure.kt:15}, for
     * {@code setTextSize(TypedValue.COMPLEX_UNIT_SP, Ref.sp(44))}.
     *
     * In sp and not pixels on purpose: a language may ask for larger text
     * ({@code tv/app/Television.kt:56}), and sp is the unit that carries it.
     */
    public static float sp(int ref) {
        return ref / 2f;
    }
}
