package streams.tv.views;

import android.content.Context;
import android.graphics.drawable.GradientDrawable;
import streams.shared.Corner;

/**
 * The brand's corners on a television, and the page margin every screen lines its edges up on.
 *
 * The Java twin of {@code tv/app/Round.kt:17-55}. The numbers are the same numbers and they are
 * taken from that file: {@code TV_SCALE} at {@code :18}, the five roles at {@code :26-43},
 * {@code PAGE_EDGE} at {@code :50} and {@code SUB_EDGE} at {@code :53}.
 *
 * <h2>Why the corners are read and not restated</h2>
 * {@code Corner.CARD} and its four siblings are {@code const val} on a Kotlin object
 * ({@code shared/Theme.kt:51-60}) reading {@code BuildConfig} integers that
 * {@code shared/build.gradle.kts:14-19} writes from {@code brand.json}. A {@code const val} is a
 * static field on the class, so Java calls it, and generated {@code BuildConfig} is Java already.
 * So the brand's shape has one home on both sides of this migration and this class only scales it.
 * The colours are not so lucky -- see {@link Ink}.
 *
 * <h2>Why a {@link GradientDrawable} and not a {@code Path}</h2>
 * {@code shapeOf(corner)} ({@code Round.kt:55}) is a {@code RoundedCornerShape} handed to
 * {@code clip} and {@code background}. The Views twin of both is one background drawable with a
 * corner radius: {@code setBackground} draws the fill and the border, and the view clips its own
 * children to it only if it is asked to, which is the same split Compose has. Drawing a rounded
 * rectangle into {@code onDraw} by hand would be a second statement of the same shape, and
 * {@code GradientDrawable} is in {@code android.jar} and older than either toolkit.
 */
public final class Round {
    /**
     * How much larger a corner is on a television than on a phone, in reference pixels per brand
     * unit. {@code Round.kt:18}.
     */
    public static final float TV_SCALE = 1.5f;

    /** Posters, stills, episode pictures, the picture grid. {@code Round.kt:28}. */
    public static final int CARD = tv(Corner.CARD);

    /** The hero, the landing cards, menus, panels. {@code Round.kt:31}. */
    public static final int PANEL = tv(Corner.PANEL);

    /** Fields, boxes of text, the code plate, PIN boxes. {@code Round.kt:34}. */
    public static final int FIELD = tv(Corner.FIELD);

    /** Keys and letters: smaller than a field. {@code Round.kt:37}. */
    public static final int KEY = FIELD * 2 / 3;

    /** Badges and small labels. {@code Round.kt:40}. */
    public static final int CHIP = FIELD / 2;

    /**
     * A profile's face and its focus ring: the corner is a share of the size, so it grows with it.
     * {@code Round.kt:46} holds the same share as a {@code RoundedCornerShape}; here it is the
     * share itself, applied to the size the caller draws the face at.
     */
    public static final int FACE_SHARE_OF = 130;

    /** And the share taken. {@code Round.kt:46}'s {@code card * 100 / 130}. */
    public static final int FACE_SHARE = 100;

    /**
     * The page margin on a television, left and right, in reference pixels. {@code Round.kt:50}.
     *
     * No page writes this number again -- {@code NumbersTest.kt}'s last test scans the whole surface
     * for a bare margin literal and finds none -- which is what makes it changeable at all.
     */
    public static final int PAGE_EDGE = 72;

    /** Pages without the top bar keep their words on this column. {@code Round.kt:53}. */
    public static final int SUB_EDGE = 130;

    private Round() {
    }

    /** A brand corner at television size, rounded as {@code Round.kt:20} rounds it. */
    public static int tv(int brand) {
        return Math.round(brand * TV_SCALE);
    }

    /** A corner that sits inside another, {@code inset} from its edge. {@code Round.kt:43}. */
    public static int inside(int outer, int inset) {
        return Math.max(outer - inset, CHIP);
    }

    /** A filled rounded rectangle: the Views form of {@code background(colour, shapeOf(corner))}. */
    public static GradientDrawable plate(Context context, int corner, int color) {
        GradientDrawable shape = new GradientDrawable();
        shape.setShape(GradientDrawable.RECTANGLE);
        shape.setCornerRadius(Ref.px(context, corner));
        shape.setColor(color);
        return shape;
    }

    /**
     * The same with a border drawn on it: {@code background(...).border(stroke, colour, shape)}.
     *
     * The stroke is in reference pixels like every other measurement, and it is drawn inside the
     * view's own bounds. Compose's {@code border} draws inside the bounds too, so the open card's
     * white line ({@code tv/browse/Rows.kt:110}) lands in the same place either way.
     */
    public static GradientDrawable edged(Context context, int corner, int color, int refStroke, int edge) {
        GradientDrawable shape = plate(context, corner, color);
        shape.setStroke(Ref.px(context, refStroke), edge);
        return shape;
    }

    /** A circle of one colour: a tab's plate, the search button, the dot between two facts. */
    public static GradientDrawable round(int color) {
        GradientDrawable shape = new GradientDrawable();
        shape.setShape(GradientDrawable.OVAL);
        shape.setColor(color);
        return shape;
    }

    /**
     * A pill: a rounded rectangle whose corner is past half its height, so it reads as round
     * whatever it is filled with. The Views twin of {@code CircleShape} on a part that is wider than
     * it is tall, which is what {@code TopBar.kt:93} and {@code parts/Pill.kt:44} ask for.
     */
    public static GradientDrawable pill(Context context, int refHigh, int color) {
        return plate(context, refHigh, color);
    }
}
