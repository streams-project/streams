package streams.tv.parts;

import android.content.Context;
import android.graphics.Color;
import android.graphics.drawable.Drawable;
import android.view.Gravity;
import android.view.View;
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
 * The letter grid a remote types on. The Views twin of {@code Keyboard} and {@code CapView} at
 * {@code parts/Keyboard.kt:78-124}. What it draws is read whole out of {@link Keys} after every
 * press; it holds no state of its own.
 *
 * <h2>It does not mirror</h2>
 * {@code setMirrored} is left false on purpose and the panel is nested, not the page.
 * {@code RefLayout.leftOf} mirrors a sized child as {@code Ref.W - refX - refW}, and {@code Ref.W}
 * is the screen's 1920 -- right for a layout that is the page and wrong by 1198 reference pixels
 * for a 722 wide panel. A keyboard's letters run left to right in every language this surface
 * speaks anyway: a right to left page places the whole panel where it wants it and the keys inside
 * keep their order, which is what the Compose form does by drawing into a fixed {@code Box}.
 *
 * <h2>The panel holds its rows, checked rather than assumed</h2>
 * {@link #fits()} states the two containments the measured numbers have to satisfy, so a key size
 * or a margin changed on its own fails a test instead of drawing a row off the edge of the panel.
 */
public final class KeysView extends RefLayout {
    /** The panel. {@code Keyboard.kt:113}. */
    static final int WIDE = 722;

    static final int HIGH = 476;

    /** Where the keys start inside it. {@code Keyboard.kt:118}. */
    static final int INSET_X = 30;

    static final int INSET_Y = 29;

    /** The two buttons at the foot. {@code Keyboard.kt:83-86}. */
    static final int BUTTON_TOP = 356;

    static final int BUTTON_HIGH = 62;

    static final int BUTTON_WIDE = 327;

    static final int BUTTON_RIGHT_AT = 331;

    /** The size of a key's word. {@code Keyboard.kt:96}. */
    static final int TYPE = 26;

    /** And of the mark a key wears instead. {@code Keyboard.kt:98}. */
    static final int MARK = 34;

    /** How white the panel is, its edge, and a key at rest, in hundredths. {@code Keyboard.kt:114-115}, {@code :92}. */
    static final int PANEL = 8;

    static final int EDGE = 25;

    static final int QUIET = 16;

    /** The panel's edge, in reference pixels. {@code Keyboard.kt:115}. */
    static final int EDGE_WIDE = 1;

    private final RefLayout inside;
    private final Drawable shift;
    private final Drawable space;
    private final Drawable erase;
    private View[][] faces = new View[0][];
    private String shape = "";

    public KeysView(Context context, Drawable shift, Drawable space, Drawable erase) {
        super(context);
        this.shift = shift;
        this.space = space;
        this.erase = erase;
        setBackground(Round.edged(context, Round.PANEL, Ink.white(PANEL), EDGE_WIDE, Ink.white(EDGE)));
        inside = new RefLayout(context);
        addView(inside, RefLayout.at(INSET_X, INSET_Y));
    }

    /** The panel's own size, for a page placing it: {@code RefLayout.at(x, y, WIDE, HIGH)}. */
    public static RefLayout.Params at(int refX, int refY) {
        return RefLayout.at(refX, refY, WIDE, HIGH);
    }

    /**
     * Draws the keys as they stand. Called after every press, which is the explicit redraw that
     * replaces Compose's snapshot: there is one call site and it is unconditional, so a key that
     * changed and did not repaint is not a case this surface has.
     */
    public void show(Keys keys, String cancel, String done) {
        Keys.Cap[][] rows = keys.caps(cancel, done);
        String wanted = keys.page() + ":" + keys.upper() + ":" + cancel + ":" + done;
        if (!wanted.equals(shape)) {
            build(rows);
            shape = wanted;
        }
        for (int row = 0; row < rows.length; row++) {
            int lit = keys.lit(row);
            for (int at = 0; at < rows[row].length; at++) {
                paint(faces[row][at], rows[row][at], at == lit);
            }
        }
    }

    private void build(Keys.Cap[][] rows) {
        inside.removeAllViews();
        faces = new View[rows.length][];
        Context context = getContext();
        for (int row = 0; row < rows.length; row++) {
            faces[row] = new View[rows[row].length];
            for (int at = 0; at < rows[row].length; at++) {
                Keys.Cap cap = rows[row][at];
                FrameLayout key = new FrameLayout(context);
                if (cap.word != null) {
                    TextView word = Type.line(context, TYPE,
                            pill(cap) ? Type.BOLD : Type.MIDDLE, Color.WHITE);
                    word.setText(cap.word);
                    key.addView(word, new FrameLayout.LayoutParams(
                            FrameLayout.LayoutParams.WRAP_CONTENT,
                            FrameLayout.LayoutParams.WRAP_CONTENT, Gravity.CENTER));
                }
                Drawable drawn = markOf(cap.mark);
                if (drawn != null) {
                    ImageView mark = new ImageView(context);
                    mark.setImageDrawable(drawn);
                    Says.onto(mark, Says.decoration());
                    key.addView(mark, new FrameLayout.LayoutParams(
                            Ref.px(context, MARK), Ref.px(context, MARK), Gravity.CENTER));
                }
                inside.addView(key, RefLayout.at(leftOf(cap), topOf(row, cap), wideOf(cap), highOf(cap)));
                faces[row][at] = key;
            }
        }
    }

    private void paint(View key, Keys.Cap cap, boolean on) {
        int ink = on ? Ink.BACK : Color.WHITE;
        int plate = on ? Color.WHITE : Ink.white(QUIET);
        key.setBackground(pill(cap)
                ? Round.pill(getContext(), BUTTON_HIGH, plate)
                : Round.plate(getContext(), Round.KEY, plate));
        FrameLayout group = (FrameLayout) key;
        for (int at = 0; at < group.getChildCount(); at++) {
            View child = group.getChildAt(at);
            if (child instanceof TextView) {
                ((TextView) child).setTextColor(ink);
            } else if (child instanceof ImageView) {
                ((ImageView) child).setColorFilter(ink);
            }
        }
        Says.onto(key, Says.saysItsWords(on));
    }

    private Drawable markOf(Keys.Mark mark) {
        switch (mark) {
            case SHIFT:
                return shift;
            case SPACE:
                return space;
            case ERASE:
                return erase;
            default:
                return null;
        }
    }

    /** A key at the foot is a button and not a key: a pill, not a rounded square. {@code Keyboard.kt:81}. */
    static boolean pill(Keys.Cap cap) {
        return cap.press == Keys.Press.CANCEL || cap.press == Keys.Press.DONE;
    }

    /** {@code Keyboard.kt:84}. */
    static int leftOf(Keys.Cap cap) {
        if (pill(cap)) {
            return cap.press == Keys.Press.CANCEL ? 0 : BUTTON_RIGHT_AT;
        }
        return cap.column * Keys.STEP;
    }

    /** {@code Keyboard.kt:82}. */
    static int topOf(int row, Keys.Cap cap) {
        return pill(cap) ? BUTTON_TOP : row * Keys.STEP;
    }

    /**
     * {@code Keyboard.kt:85}. A key spanning two columns is two steps wide less the one gap it
     * swallows, which is why the gap is written as {@code STEP - KEY} and not as a 3 of its own.
     */
    static int wideOf(Keys.Cap cap) {
        return pill(cap) ? BUTTON_WIDE : cap.span * Keys.STEP - (Keys.STEP - Keys.KEY);
    }

    /** {@code Keyboard.kt:83}. */
    static int highOf(Keys.Cap cap) {
        return pill(cap) ? BUTTON_HIGH : Keys.KEY;
    }

    /**
     * Whether the measured numbers still hold the keys inside the panel, across and down.
     *
     * Across: the two buttons are the widest thing on it, and they plus the two insets have to fit
     * the panel. Down: the buttons' own row has to end above the panel's foot.
     */
    static boolean fits() {
        int widest = BUTTON_RIGHT_AT + BUTTON_WIDE;
        boolean across = INSET_X + widest + INSET_X <= WIDE;
        boolean down = INSET_Y + BUTTON_TOP + BUTTON_HIGH <= HIGH;
        return across && down;
    }
}
