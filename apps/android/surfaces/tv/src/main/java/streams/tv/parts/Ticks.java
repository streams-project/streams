package streams.tv.parts;

import android.content.Context;
import android.graphics.Color;
import android.graphics.drawable.Drawable;
import android.view.Gravity;
import android.widget.FrameLayout;
import android.widget.ImageView;
import android.widget.TextView;
import streams.shared.Ink;
import streams.shared.Says;
import streams.shared.Type;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.Round;

/**
 * Words in four columns with one ticked. The language and subtitle pages share it. The Views twin
 * of {@code parts/TickGrid.kt:28-57}.
 *
 * <h2>Two marks that are not the same thing</h2>
 * {@code ticked} is the choice the viewer has already made and {@code focus} is where the remote
 * is. A port that drew one would lose the other, and the one it would lose is the tick.
 *
 * <h2>What is not said, and why it is left alone</h2>
 * {@code TickGrid.kt:35-36} carries an owner note: which line is ticked is drawn as a tick and is
 * not read out, because saying it needs a word for "chosen" in {@code strings/} and this surface
 * may not invent one. That is still true in Java -- inventing it here would be English in a program
 * file, which {@code check_strings.py} exists to stop -- so the gap is carried across unchanged and
 * stays on the handoff list rather than being quietly closed with a word nobody translated.
 *
 * <h2>The word lands on its column</h2>
 * {@code TickGrid.kt:40} places the plate {@link #BACK} to the left of the column and
 * {@code :54} draws the word {@link #BACK} back to the right inside it, so the word sits exactly on
 * the column and the plate reaches around it. The two numbers are one number and are written once.
 */
public final class Ticks extends RefLayout {
    /** How many lines a column holds. {@code TickGrid.kt:24}. */
    public static final int LINES = 8;

    /** Where the four columns' words sit. {@code TickGrid.kt:26}. */
    static final int[] COLUMNS = {212, 612, 1015, 1425};

    /** The middle of the first line. {@code TickGrid.kt:27}. */
    static final int FIRST_MIDDLE = 293;

    /** And the drop to the next. {@code TickGrid.kt:28}. */
    static final int LINE = 75;

    /** How far the plate reaches back from the word, which is also the word's place in it. */
    static final int BACK = 90;

    /** The plate. {@code TickGrid.kt:41-42}. */
    static final int WIDE = 390;

    static final int HIGH = 63;

    /** Where the tick sits in it, and how large. {@code TickGrid.kt:49-50}. */
    static final int TICK_AT = 32;

    static final int TICK = 34;

    /** The size of a word. {@code TickGrid.kt:54}. */
    static final int TYPE = 30;

    private final Drawable tick;
    private int ticked = -1;
    private int focus = -1;
    private String[] said = new String[0];

    public Ticks(Context context, Drawable tick) {
        super(context);
        this.tick = tick;
    }

    /** Where a word's plate goes: half its height above the line's middle. */
    static int topOf(int at) {
        return FIRST_MIDDLE + (at % LINES) * LINE - HIGH / 2;
    }

    /**
     * Which column a word falls in. {@code TickGrid.kt:30}: past the fourth column's worth of
     * lines the words stack on the last column rather than running off the screen, which is what
     * {@code coerceAtMost} does and what a language list longer than 32 would otherwise do.
     */
    static int columnOf(int at) {
        int column = at / LINES;
        return COLUMNS[Math.min(column, COLUMNS.length - 1)];
    }

    /** Draws these words. They are the catalogue's or the words', already in the right language. */
    public void show(String[] said, int ticked, int focus) {
        this.said = said == null ? new String[0] : said;
        this.ticked = ticked;
        this.focus = focus;
        removeAllViews();
        Context context = getContext();
        for (int at = 0; at < this.said.length; at++) {
            boolean on = at == focus;
            int ink = on ? Ink.BACK : Color.WHITE;
            FrameLayout line = new FrameLayout(context);
            line.setBackground(Round.pill(context, HIGH, on ? Color.WHITE : Color.TRANSPARENT));
            if (at == ticked && tick != null) {
                ImageView mark = new ImageView(context);
                mark.setImageDrawable(tick);
                mark.setColorFilter(ink);
                Says.onto(mark, Says.decoration());
                FrameLayout.LayoutParams spot = new FrameLayout.LayoutParams(
                        Ref.px(context, TICK), Ref.px(context, TICK),
                        Gravity.START | Gravity.CENTER_VERTICAL);
                spot.setMarginStart(Ref.px(context, TICK_AT));
                line.addView(mark, spot);
            }
            TextView word = Type.line(context, TYPE, Type.BOOK, ink);
            word.setText(this.said[at]);
            FrameLayout.LayoutParams spot = new FrameLayout.LayoutParams(
                    FrameLayout.LayoutParams.WRAP_CONTENT, FrameLayout.LayoutParams.WRAP_CONTENT,
                    Gravity.START | Gravity.CENTER_VERTICAL);
            spot.setMarginStart(Ref.px(context, BACK));
            line.addView(word, spot);
            Says.onto(line, Says.saysItsWords(on));
            addView(line, RefLayout.at(columnOf(at) - BACK, topOf(at), WIDE, HIGH));
        }
    }

    /** Where the remote is, and which line carries the tick. */
    public void mark(int ticked, int focus) {
        show(said, ticked, focus);
    }

    public int ticked() {
        return ticked;
    }

    public int focus() {
        return focus;
    }
}
