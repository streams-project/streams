package streams.tv.views;

import android.content.Context;
import android.graphics.Typeface;
import android.os.Build;
import android.text.TextUtils;
import android.util.TypedValue;
import android.widget.TextView;
import streams.shared.R;

/**
 * The brand's text, in Views.
 *
 * {@code shared/Theme.kt:48} builds the family from one variable font at five weights, and
 * {@code tv/app/Television.kt:44} hands it to every screen through {@code ProvideTextStyle}. Views
 * has no such local, so a line of text is given its typeface where it is built, and this is the one
 * place that knows which file and which weights.
 *
 * <h2>The weights are the five the brand declares</h2>
 * 400, 500, 600, 700 and 800, from {@code Theme.kt:48}. They are named here as integers rather than
 * as {@code Typeface.BOLD} because a variable font takes the number: {@code FontWeight.ExtraBold} is
 * 800 and {@code Typeface.BOLD} is whatever the platform picks, and the difference is visible on the
 * open card's name, which is drawn at 800 ({@code tv/browse/Rows.kt:105}).
 *
 * <h2>Below API 28 the weight is approximated, and that is stated rather than hidden</h2>
 * {@code Typeface.create(Typeface, int, boolean)} arrived at API 28 and this app's minSdk is 26, so
 * on two releases the five weights collapse to normal and bold. {@code Resources.getFont} is API 26,
 * so the brand's own font is still used there -- it is the weight that is approximated and not the
 * face. Compose on those releases picks a weight from the same variable font and so does better;
 * this is a real and small loss on Android 8.0 and 8.1, named here so nobody re-finds it.
 *
 * <h2>Why the size is sp and not pixels</h2>
 * {@code Measure.kt:15}'s {@code refType} is half an sp, and {@code tv/app/Television.kt:56} lets a
 * language ask for larger text -- Arabic letters read small beside Latin at one size. Views carries
 * that through the font scale, which only applies to sp. Note 210 records the right answer for the
 * per language multiplier as {@code createConfigurationContext} with an adjusted
 * {@code Configuration.fontScale} on the activity's base context, and names the wrong answer as
 * multiplying at every call site, of which there would be 114 on this surface. Nothing here
 * multiplies: it asks for sp and the context decides.
 */
public final class Type {
    public static final int BOOK = 400;
    public static final int MIDDLE = 500;
    public static final int STRONG = 600;
    public static final int BOLD = 700;
    public static final int HEAVY = 800;

    private Type() {
    }

    /** The brand's face at a weight. */
    public static Typeface face(Context context, int weight) {
        Typeface font = context.getResources().getFont(R.font.inter);
        if (Build.VERSION.SDK_INT >= 28) {
            return Typeface.create(font, weight, false);
        }
        return Typeface.create(font, weight >= BOLD ? Typeface.BOLD : Typeface.NORMAL);
    }

    /**
     * A line of catalogue text: a title's name, a row's name, a story. Its words come from the
     * answer and are already in the viewer's language, so it holds no key -- {@link WordText} is for
     * the words of the interface, and the difference is which file the words live in.
     */
    public static TextView of(Context context, int refType, int weight, int color) {
        TextView text = new TextView(context);
        text.setTextSize(TypedValue.COMPLEX_UNIT_SP, Ref.sp(refType));
        text.setTypeface(face(context, weight));
        text.setTextColor(color);
        text.setIncludeFontPadding(false);
        return text;
    }

    /**
     * One line that ends in an ellipsis when it does not fit, which is
     * {@code maxLines = 1, overflow = TextOverflow.Ellipsis} at {@code tv/browse/Rows.kt:106-107}.
     */
    public static TextView line(Context context, int refType, int weight, int color) {
        TextView text = of(context, refType, weight, color);
        text.setMaxLines(1);
        text.setEllipsize(TextUtils.TruncateAt.END);
        return text;
    }

    /**
     * A story: up to {@code lines} lines, ellipsised, with the line height the capture gives.
     *
     * {@code lineHeight} is a measured number on this surface -- 39 against a 30 size at
     * {@code tv/browse/Rows.kt:149-150} and {@code tv/browse/HeroCard.kt:86-87} -- and Views sets it
     * in pixels rather than in sp, so it is converted here and not left to a multiplier. A line
     * height expressed as a multiple would lose the measurement, which is section 6's whole rule.
     */
    public static TextView story(Context context, int refType, int refLine, int lines, int color) {
        TextView text = of(context, refType, BOOK, color);
        text.setMaxLines(lines);
        text.setEllipsize(TextUtils.TruncateAt.END);
        int wanted = Ref.px(context, refLine);
        if (Build.VERSION.SDK_INT >= 28) {
            text.setLineHeight(wanted);
        } else {
            // `setLineHeight` arrived at API 28 and this app's minSdk is 26, so on two releases the
            // same number is reached by adding what the font does not already give. This is what
            // `setLineHeight` itself does: it measures the paint's own line and adds the remainder.
            android.graphics.Paint.FontMetricsInt metrics = text.getPaint().getFontMetricsInt();
            text.setLineSpacing(Math.max(wanted - (metrics.descent - metrics.ascent), 0), 1f);
        }
        return text;
    }
}
