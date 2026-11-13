package streams.tv.parts;

import android.content.Context;
import streams.shared.Type;
import streams.shared.WordText;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.Round;

/**
 * The title and one line of help at the top left of a settings style page. Six files and eleven
 * sites draw it, which makes it the most reused part on the surface. The Views twin of
 * {@code parts/PageHead.kt:14-18}.
 *
 * <h2>It spans the canvas</h2>
 * The Kotlin is a {@code Box} holding two offset lines, drawn inside a page that fills the screen,
 * so both coordinates are in the page's own space. This is the same thing: a transparent
 * {@link RefLayout} added at {@code RefLayout.at(0, 0, Ref.W, Ref.H)}, which is also what makes the
 * mirror right -- {@code RefLayout.leftOf} takes a mirrored child's position as
 * {@code Ref.W - refX - refW}, and that is the page's width only when the layout is the page.
 *
 * <h2>Its words are keys</h2>
 * Both lines are the interface's own words, so they are {@link WordText} and read their key when
 * the words arrive. No call site hands this English, which is what {@code check_strings.py} is for.
 */
public final class PageHead extends RefLayout {
    /** The title's line. {@code PageHead.kt:16}. */
    static final int TITLE_AT = 74;

    static final int TITLE_TYPE = 44;

    /** And the help under it. {@code PageHead.kt:17}. */
    static final int HELP_AT = 148;

    static final int HELP_TYPE = 30;

    private final WordText title;
    private final WordText help;

    public PageHead(Context context, String titleKey, String helpKey) {
        super(context);
        title = WordText.of(context, titleKey, TITLE_TYPE);
        title.setTypeface(Type.face(context, Type.BOLD));
        addView(title, RefLayout.at(Round.SUB_EDGE, TITLE_AT));
        help = WordText.of(context, helpKey, HELP_TYPE);
        addView(help, RefLayout.at(Round.SUB_EDGE, HELP_AT));
    }

    /** The line of help, when a page changes which one it shows between its steps. */
    public void setHelpKey(String key) {
        help.setKey(key);
    }

    /** And the title, for the same reason. */
    public void setTitleKey(String key) {
        title.setKey(key);
    }

    /** How far down the page the head reaches, for a part placed under it. */
    public static int under() {
        return HELP_AT + HELP_TYPE;
    }

    /** Guards the one assumption this part makes about its parent: it is the page. */
    public boolean spansTheCanvas() {
        return getLayoutParams() instanceof RefLayout.Params
                && ((RefLayout.Params) getLayoutParams()).refW == Ref.W;
    }
}
