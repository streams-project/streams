package streams.tv.views;

import android.content.Context;
import android.view.View;
import streams.shared.Scale;

/**
 * The television's reference canvas, and the names its screens measure in.
 *
 * The canvas is 1920 by 1080 because that is what the captures this surface was built from measure;
 * the dp it is laid out in and the halving that gets it there are {@link Scale}, beside the phone,
 * because the conversion is not the television's. This file is what stayed behind: four constants
 * that are this surface's alone, and the names its 123 call sites already write.
 *
 * <h2>Why the arithmetic is delegated and not kept</h2>
 * Note 242 found {@code Type} and {@code WordText} unable to leave this package while they called
 * {@code Ref.sp} and {@code Ref.px}, since a shared module cannot depend on the television. The
 * methods below are one line each and hold no number, so the call sites did not move and the two
 * classes did. What is <b>not</b> here is as much the point: no divisor, and no second statement of
 * the derivation. Both are in {@link Scale}, and a test reads this file to keep them from coming
 * back.
 */
public final class Ref {
    /**
     * The reference canvas: the width written out at {@code tv/browse/HeroCard.kt:54} and
     * {@code tv/browse/TopBar.kt:117}. The same number as {@link Scale#CANVAS_W}, read from there
     * rather than typed again, and held against the other television's own statement of it by
     * {@code NumbersTest}.
     */
    public static final int W = Scale.CANVAS_W;

    /** And its height, which is this surface's alone: nothing shared is derived from it. */
    public static final int H = 1080;

    /**
     * The layout this surface really draws in, in dp. Not used in the arithmetic: it is here so the
     * derivation is readable from the code rather than only from the comment.
     */
    public static final int LAYOUT_W = Scale.LAYOUT_W;

    /** And its height in dp, which the test holds against {@code dp(H)}. */
    public static final int LAYOUT_H = 540;

    private Ref() {
    }

    /** A reference measurement in dp. {@link Scale#dp}. */
    public static float dp(int ref) {
        return Scale.dp(ref);
    }

    /** A reference measurement in pixels on a screen of this density. {@link Scale#px(int, float)}. */
    public static int px(int ref, float density) {
        return Scale.px(ref, density);
    }

    /** The same, on the screen this view is on. */
    public static int px(View view, int ref) {
        return Scale.px(view, ref);
    }

    /** The same, from a context, for a view that is being built and is not attached yet. */
    public static int px(Context context, int ref) {
        return Scale.px(context, ref);
    }

    /** The density of the screen a view is on, read where it is needed instead of kept. */
    public static float density(View view) {
        return Scale.density(view);
    }

    /** A reference font size, in sp. {@link Scale#sp}. */
    public static float sp(int ref) {
        return Scale.sp(ref);
    }
}
