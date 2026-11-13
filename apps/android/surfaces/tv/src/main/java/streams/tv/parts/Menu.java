package streams.tv.parts;

import android.content.Context;
import android.graphics.Color;
import android.graphics.drawable.Drawable;
import android.view.Gravity;
import android.widget.FrameLayout;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import streams.shared.Ink;
import streams.shared.Says;
import streams.shared.Type;
import streams.tv.views.Ref;
import streams.tv.views.Round;

/**
 * Plain rows of words, one of them a white pill. Seven screens share it as it stands. The Views
 * twin of {@code parts/Menu.kt:40-75}.
 *
 * <h2>Two marks, not one</h2>
 * {@code focus} is where the remote is and {@code kept} is the line another part of the page
 * belongs to while the remote is away ({@code Menu.kt:36-37}) -- a settings page whose right hand
 * side is being read still has to show which section it is reading. They are different colours and
 * a port that collapsed them would lose the second silently.
 *
 * <h2>Where the words start</h2>
 * {@code Menu.kt:42}: a menu with icons puts its words past them, at 90; a menu without puts them
 * at {@link #WORDS}, near the pill's round end, and the page shifts the whole menu left by that
 * much so the words land on the page's column. That is why {@code MENU_WORDS} is read by seven
 * files and why it is public here.
 */
public final class Menu extends LinearLayout {
    /**
     * Where a menu's words start inside its pill when it has no icons, and so how far left of the
     * page's column the menu itself is placed. {@code Menu.kt:33}.
     */
    public static final int WORDS = 36;

    /** And where they start when it has icons, which is past them. {@code Menu.kt:42}. */
    static final int WORDS_PAST_ICONS = 90;

    /** A line's height. {@code Menu.kt:52}. */
    static final int HIGH = 63;

    /** The gap between two lines. {@code Menu.kt:43}. */
    static final int GAP = 6;

    /** How wide a menu is when its page does not say. {@code Menu.kt:40}. */
    public static final int WIDE = 395;

    /** An icon's place and size. {@code Menu.kt:67}. */
    static final int ICON_AT = 33;

    static final int ICON = 34;

    /** The size of a line's word. {@code Menu.kt:73}. */
    static final int TYPE = 30;

    /** How white a kept line's plate is, in hundredths. {@code Menu.kt:57}. */
    static final int KEPT = 14;

    /** One line of a menu. {@code MenuLine} at {@code Menu.kt:24}. */
    public static final class Line {
        public final String word;
        public final Drawable icon;

        public Line(String word, Drawable icon) {
            this.word = word == null ? "" : word;
            this.icon = icon;
        }

        public Line(String word) {
            this(word, null);
        }
    }

    private final int wide;
    private int focus = -1;
    private int kept = -1;

    public Menu(Context context) {
        this(context, WIDE);
    }

    public Menu(Context context, int wide) {
        super(context);
        setOrientation(VERTICAL);
        this.wide = wide;
    }

    /** Draws these lines. The words are the page's, already in the viewer's language. */
    public void show(Line[] lines) {
        removeAllViews();
        boolean icons = false;
        for (Line line : lines) {
            if (line != null && line.icon != null) {
                icons = true;
            }
        }
        int wordsAt = icons ? WORDS_PAST_ICONS : WORDS;
        Context context = getContext();
        for (int at = 0; at < lines.length; at++) {
            Line line = lines[at];
            if (line == null) {
                continue;
            }
            FrameLayout row = new FrameLayout(context);
            if (line.icon != null) {
                ImageView mark = new ImageView(context);
                mark.setImageDrawable(line.icon);
                Says.onto(mark, Says.decoration());
                FrameLayout.LayoutParams spot = new FrameLayout.LayoutParams(
                        Ref.px(context, ICON), Ref.px(context, ICON), Gravity.START | Gravity.CENTER_VERTICAL);
                spot.setMarginStart(Ref.px(context, ICON_AT));
                row.addView(mark, spot);
            }
            TextView word = Type.line(context, TYPE, Type.BOOK, Color.WHITE);
            word.setText(line.word);
            FrameLayout.LayoutParams spot = new FrameLayout.LayoutParams(
                    FrameLayout.LayoutParams.WRAP_CONTENT, FrameLayout.LayoutParams.WRAP_CONTENT,
                    Gravity.START | Gravity.CENTER_VERTICAL);
            spot.setMarginStart(Ref.px(context, wordsAt));
            row.addView(word, spot);
            LayoutParams place = new LayoutParams(Ref.px(context, wide), Ref.px(context, HIGH));
            if (getChildCount() > 0) {
                place.topMargin = Ref.px(context, GAP);
            }
            addView(row, place);
        }
        paint();
    }

    /** Where the remote is, and which line the rest of the page belongs to. */
    public void mark(int focus, int kept) {
        this.focus = focus;
        this.kept = kept;
        paint();
    }

    public void mark(int focus) {
        mark(focus, -1);
    }

    private void paint() {
        Context context = getContext();
        for (int at = 0; at < getChildCount(); at++) {
            FrameLayout row = (FrameLayout) getChildAt(at);
            boolean on = at == focus;
            int plate = on ? Color.WHITE : at == kept ? Ink.white(KEPT) : Color.TRANSPARENT;
            row.setBackground(Round.pill(context, HIGH, plate));
            int ink = on ? Ink.BACK : Color.WHITE;
            for (int inside = 0; inside < row.getChildCount(); inside++) {
                if (row.getChildAt(inside) instanceof TextView) {
                    ((TextView) row.getChildAt(inside)).setTextColor(ink);
                } else if (row.getChildAt(inside) instanceof ImageView) {
                    ((ImageView) row.getChildAt(inside)).setColorFilter(ink);
                }
            }
            Says.onto(row, Says.saysItsWords(on));
        }
    }
}
