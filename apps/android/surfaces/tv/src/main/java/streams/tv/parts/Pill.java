package streams.tv.parts;

import android.animation.ArgbEvaluator;
import android.animation.ValueAnimator;
import android.content.Context;
import android.graphics.Color;
import android.graphics.drawable.Drawable;
import android.view.Gravity;
import android.view.animation.LinearInterpolator;
import android.widget.ImageView;
import android.widget.LinearLayout;
import streams.shared.Ink;
import streams.shared.Says;
import streams.shared.Type;
import streams.shared.WordText;
import streams.shared.Words;
import streams.tv.views.Ref;
import streams.tv.views.Round;

/**
 * A large button. In focus it turns white. The Views twin of {@code parts/Pill.kt:38-52}.
 *
 * <h2>What it says and what it does not</h2>
 * Its word is its label ({@code saysItsWords}), and the icon beside it is decoration because the
 * icon repeats the word ({@code Pill.kt:33-34}).
 *
 * <h2>A pill with nothing in it draws nothing</h2>
 * {@code Pill.kt:35-36} and {@code :40} record why, and it is the one dead end on this surface that
 * was found on hardware rather than in a review: on the Streamer on 2026-11-11, with no server, a
 * {@code Pill("")} drew an empty white shape on black with every word around it missing
 * (notes/206). So the view's visibility follows its words, re-read every time the words change,
 * and an icon on its own is enough to keep it -- which is what the hero's first button relies on.
 *
 * <h2>The change is 120 ms and it is kept</h2>
 * {@code Pill.kt:28} and {@code :41-42} animate both colours over {@code CHANGE_MS} with
 * {@code LinearEasing}. A Views port that set the colours outright would lose a measured number
 * silently and look different on a device while every test stayed green, so the same two colours
 * run through one {@link ValueAnimator} at the same duration with a {@link LinearInterpolator}.
 * {@code browse/Parts.Pill} does set them outright; holding the two equal is the lead's
 * reconciliation and is named in this package's report, not papered over here.
 */
public final class Pill extends LinearLayout implements Words.Bound {
    /** How long the colour takes to change. {@code Pill.kt:28}. */
    static final int CHANGE_MS = 120;

    /** The pill's height, which is also its corner. {@code Pill.kt:44}. */
    static final int HIGH = 84;

    /** The room each side of its word. {@code Pill.kt:44}. */
    static final int PAD = 46;

    /** The icon beside the word, and the gap after it. {@code Pill.kt:48-49}. */
    static final int ICON = 34;

    static final int GAP = 12;

    /** The size of its word. {@code Pill.kt:51}. */
    static final int TYPE = 32;

    /** How white the plate is when the remote is elsewhere, in hundredths. {@code Pill.kt:29}. */
    static final int QUIET = 16;

    private final WordText word;
    private final ImageView mark;
    private final ArgbEvaluator between = new ArgbEvaluator();
    private ValueAnimator change;
    private float lit;
    private boolean on;

    public Pill(Context context, String key, Drawable icon) {
        super(context);
        setOrientation(HORIZONTAL);
        setGravity(Gravity.CENTER_VERTICAL);
        setPadding(Ref.px(context, PAD), 0, Ref.px(context, PAD), 0);
        setMinimumHeight(Ref.px(context, HIGH));
        if (icon != null) {
            mark = new ImageView(context);
            mark.setImageDrawable(icon);
            Says.onto(mark, Says.decoration());
            LayoutParams size = new LayoutParams(Ref.px(context, ICON), Ref.px(context, ICON));
            size.rightMargin = Ref.px(context, GAP);
            addView(mark, size);
        } else {
            mark = null;
        }
        word = WordText.of(context, key, TYPE);
        word.setTypeface(Type.face(context, Type.BOLD));
        word.setMaxLines(1);
        addView(word);
        paint(0f);
        Says.onto(this, Says.saysItsWords(false));
    }

    /** Words that are not the interface's: a word the answer sent, or one already resolved. */
    public void say(String said) {
        word.setKey("");
        word.setText(said == null ? "" : said);
        showOrHide();
    }

    /**
     * Where the remote is. {@code Pill.kt:41-42} is the whole of its focus: two colours and the
     * reading that says the remote is on it.
     *
     * @param settled true to land on the new colours rather than travel to them, for a pill that is
     *                built already in focus and has nothing to animate from
     */
    public void lit(boolean on, boolean settled) {
        this.on = on;
        Says.onto(this, Says.saysItsWords(on));
        float wanted = on ? 1f : 0f;
        if (change != null) {
            change.cancel();
            change = null;
        }
        if (settled || lit == wanted) {
            paint(wanted);
            return;
        }
        change = ValueAnimator.ofFloat(lit, wanted);
        change.setDuration(CHANGE_MS);
        change.setInterpolator(new LinearInterpolator());
        change.addUpdateListener(step -> paint(((Float) step.getAnimatedValue()).floatValue()));
        change.start();
    }

    public void lit(boolean on) {
        lit(on, false);
    }

    public boolean lit() {
        return on;
    }

    /** The two colours at a point in the change, which is what the 120 ms travels through. */
    private void paint(float through) {
        lit = through;
        int back = ((Integer) between.evaluate(through, Ink.white(QUIET), Color.WHITE)).intValue();
        int ink = ((Integer) between.evaluate(through, Color.WHITE, Ink.BACK)).intValue();
        setBackground(Round.pill(getContext(), HIGH, back));
        word.setTextColor(ink);
        if (mark != null) {
            mark.setColorFilter(ink);
        }
    }

    @Override
    public void words(Words words) {
        word.words(words);
        showOrHide();
    }

    private void showOrHide() {
        boolean has = word.getText().length() > 0 || mark != null;
        setVisibility(has ? VISIBLE : GONE);
    }
}
