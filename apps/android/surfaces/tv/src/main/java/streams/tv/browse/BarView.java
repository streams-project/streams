package streams.tv.browse;

import android.content.Context;
import android.graphics.Color;
import android.view.Gravity;
import android.view.View;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import java.util.ArrayList;
import java.util.List;
import streams.shared.Ink;
import streams.shared.R;
import streams.shared.Says;
import streams.shared.ThemeKt;
import streams.shared.Type;
import streams.shared.WordText;
import streams.shared.Words;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.Round;

/**
 * Who is watching, search, the tabs and the mark, at the top of every signed in page.
 * {@code tv/browse/TopBar.kt:63-118}.
 *
 * <h2>Why the bar takes words at all</h2>
 * {@code TopBar.kt:66-69} says it: two of its parts are a picture and nothing else -- the profile
 * and search -- and a picture with no words is unreachable from a remote. The tabs carry the names
 * the server sent with them, and the mark is decoration, because it is the same on every page and
 * says nothing a viewer needs.
 *
 * <h2>Its four readings</h2>
 * {@code SpokenTest.kt}'s census records the Kotlin at 3 calls and these are the three: the profile
 * says the person's name and whether the remote is on it ({@code TopBar.kt:83}); search says its
 * word and the same ({@code :101}); a tab's own drawn words are its label ({@code :56}). The fourth
 * is this file's own and has no Kotlin line: the bell, which says how many notices are waiting,
 * because the count is drawn on it and a person who cannot see it has nowhere else to get it.
 * {@code SaidTest} is where the Java's are counted. The face
 * inside the profile part says nothing of its own, so a reader does not hear the name twice.
 */
final class BarView extends RefLayout implements Words.Bound {
    /** The bar's centre line. {@code tv/browse/TopBar.kt:37}. */
    static final int BAR_MID = 68;

    /** The height of its round parts. {@code tv/browse/TopBar.kt:38}. */
    static final int BAR_HIGH = 64;

    /** And the face's size. {@code tv/browse/TopBar.kt:39}. */
    static final int FACE = 48;

    /** Where search sits. {@code tv/browse/TopBar.kt:91}. */
    static final int SEARCH_X = 539;

    /** And where the tabs start. {@code tv/browse/TopBar.kt:111}. */
    static final int TABS_X = 609;

    /** The gap between two tabs. {@code tv/browse/TopBar.kt:114}. */
    static final int TAB_GAP = 8;

    /** How many tabs fit the bar. {@code tv/browse/TopBar.kt:42}. */
    static final int TABS = 6;

    /** The screen a tab must not be, because genres live on their own pages. The same line. */
    static final String NOT_A_TAB = "genre";

    /**
     * Where the bell sits: one round part and one tab gap in from the mark, at the far end.
     *
     * A derivation and not a transcription. The mark is drawn at
     * {@code Ref.W - PAGE_EDGE - BAR_HIGH}; the bell is the round part before it, and the distance
     * between two round parts of this bar is already measured -- {@link #TAB_GAP}, the gap between
     * two tabs. So the bell is a bar height and a tab gap further in, and no number is invented for
     * it. Moving the mark moves the bell with it.
     */
    static final int BELL_X = Ref.W - Round.PAGE_EDGE - BAR_HIGH - TAB_GAP - BAR_HIGH;

    /**
     * The bell glyph inside its circle: <b>34 px</b>, which is the search glass's size.
     *
     * The two are the same kind of thing -- a picture alone in a round part of this bar -- so the
     * bell is drawn at the size this bar already draws that at, rather than at a size of its own.
     */
    static final int GLYPH = 34;

    /**
     * The plate that carries the count, at the bell's top right: half a round part high.
     *
     * The phone draws its count in a badge 18 dp wide and so caps it at "9+"
     * ({@code phone/browse/Header.kt:112}). That cap is a width this bar does not have: there is a
     * whole bar height to draw in and a television is read from three metres, where a number small
     * enough to need capping is a number nobody can read at all. So the count is drawn as the
     * server sent it, in the language's own digits, and the phone's 9 is not copied here -- it
     * would be a third home for a number that is a phone's badge width and nothing else.
     */
    static final int COUNT_HIGH = BAR_HIGH / 2;

    /**
     * The size this bar writes in: a tab's label and the count on the bell.
     *
     * {@code tv/browse/TopBar.kt:59}. It was a literal in {@link #show} until the bell wanted the
     * same size and would have made it a second one.
     */
    static final int TAB_TYPE = 26;

    private final RefLayout who;
    private final Parts.Face face;
    private final ImageView caret;
    private final View search;
    private final ImageView glass;
    private final LinearLayout tabs;
    private final List<TextView> labels = new ArrayList<>();
    private final View bell;
    private final ImageView chime;
    private final TextView count;
    private final Art mark;

    BarView(Context context, Art.Source pictures) {
        super(context);

        // `TopBar.kt:83-88`: the face and the caret beside it, as one part the remote stops on.
        who = new RefLayout(context);
        face = new Parts.Face(context, pictures, FACE);
        who.addView(face, RefLayout.at(0, 0, FACE, FACE));
        caret = new ImageView(context);
        caret.setImageDrawable(context.getDrawable(R.drawable.icon_caret_down));
        caret.setColorFilter(Color.WHITE);
        Says.onto(caret, Says.decoration());
        who.addView(caret, RefLayout.at(FACE, (FACE - 22) / 2, 22, 22));
        addView(who, RefLayout.at(Round.PAGE_EDGE, BAR_MID - FACE / 2, FACE + 22, FACE));

        search = new View(context);
        glass = new ImageView(context);
        glass.setImageDrawable(context.getDrawable(R.drawable.icon_search));
        Says.onto(glass, Says.decoration());
        addView(search, RefLayout.at(SEARCH_X, BAR_MID - BAR_HIGH / 2, BAR_HIGH, BAR_HIGH));
        addView(glass, RefLayout.at(SEARCH_X + (BAR_HIGH - 34) / 2,
                BAR_MID - BAR_HIGH / 2 + (BAR_HIGH - 34) / 2, 34, 34));

        tabs = new LinearLayout(context);
        tabs.setOrientation(LinearLayout.HORIZONTAL);
        addView(tabs, RefLayout.at(TABS_X, BAR_MID - BAR_HIGH / 2, RefLayout.WRAP, BAR_HIGH));

        // The bell, built exactly as search above it is: a round plate the remote stops on, a
        // picture inside it that says nothing of its own, and -- this one's alone -- the count.
        bell = new View(context);
        chime = new ImageView(context);
        chime.setImageDrawable(context.getDrawable(R.drawable.icon_bell));
        Says.onto(chime, Says.decoration());
        addView(bell, RefLayout.at(BELL_X, BAR_MID - BAR_HIGH / 2, BAR_HIGH, BAR_HIGH));
        addView(chime, RefLayout.at(BELL_X + (BAR_HIGH - GLYPH) / 2,
                BAR_MID - BAR_HIGH / 2 + (BAR_HIGH - GLYPH) / 2, GLYPH, GLYPH));

        // The count sits on the bell's top right, as the phone's badge sits on its bell's. It is
        // decoration: the bell itself says how many are waiting, in words, so a reader that stopped
        // on the number as well would hear it twice and the second time without a noun.
        count = Type.line(context, TAB_TYPE, Type.HEAVY, Color.WHITE);
        count.setGravity(Gravity.CENTER);
        count.setPadding(Ref.px(context, TAB_GAP), 0, Ref.px(context, TAB_GAP), 0);
        count.setBackground(Round.pill(context, COUNT_HIGH, Ink.ACCENT));
        Says.onto(count, Says.decoration());
        addView(count, RefLayout.at(BELL_X + BAR_HIGH - COUNT_HIGH, BAR_MID - BAR_HIGH / 2,
                RefLayout.WRAP, COUNT_HIGH));

        // `TopBar.kt:117`: the brand's mark at the far end of the bar.
        mark = new Art(context, pictures, null);
        mark.setScaleType(ImageView.ScaleType.FIT_CENTER);
        addView(mark, RefLayout.at(Ref.W - Round.PAGE_EDGE - BAR_HIGH, BAR_MID - BAR_HIGH / 2,
                BAR_HIGH, BAR_HIGH));
    }

    /**
     * Draws the bar.
     *
     * @param focus which place the remote is on, or -1 when it is somewhere else on the page, which
     *              is what {@code Browse.kt:269} and {@code Waiting.kt:42} both pass
     */
    void show(Feed feed, int focus) {
        boolean searchOn = focus == Feed.SEARCH_TAB;
        boolean whoOn = focus == Feed.WHO_TAB;
        boolean searchHere = feed.active == Feed.SEARCH_TAB;

        face.show(feed.who, Round.CARD * Round.FACE_SHARE / Round.FACE_SHARE_OF, whoOn);
        Says.onto(who, Says.stop(feed.who == null ? "" : feed.who.name, whoOn));

        search.setBackground(Round.round(searchOn
                ? Color.WHITE
                : searchHere ? Ink.white(16) : Color.TRANSPARENT));
        glass.setColorFilter(searchOn ? Ink.BACK : Color.WHITE);
        searchLit = searchOn;
        Says.onto(search, Says.stop(said, searchOn));

        boolean bellOn = focus == Feed.BELL_TAB;
        bell.setBackground(Round.round(bellOn ? Color.WHITE : Color.TRANSPARENT));
        chime.setColorFilter(bellOn ? Ink.BACK : Color.WHITE);
        // No "the page the viewer is on" plate under the bell, which is the one way it differs from
        // search: the notifications screen is not one of this bar's pages. It is reached from the
        // bar and draws no bar of its own, so there is no state in which the bell is "here".
        count.setVisibility(feed.unread > 0 ? VISIBLE : GONE);
        count.setText(ThemeKt.number(Integer.valueOf(feed.unread)));
        bellUnread = feed.unread;
        bellLit = bellOn;
        Says.onto(bell, Says.stop(Feed.bellSays(book, bellUnread), bellOn));

        while (labels.size() < feed.tabs.size()) {
            TextView label = Type.line(getContext(), TAB_TYPE, Type.MIDDLE, Color.WHITE);
            label.setGravity(Gravity.CENTER);
            label.setPadding(Ref.px(getContext(), 32), 0, Ref.px(getContext(), 32), 0);
            LinearLayout.LayoutParams spot = new LinearLayout.LayoutParams(
                    LinearLayout.LayoutParams.WRAP_CONTENT, Ref.px(getContext(), BAR_HIGH));
            if (!labels.isEmpty()) {
                spot.leftMargin = Ref.px(getContext(), TAB_GAP);
            }
            tabs.addView(label, spot);
            labels.add(label);
        }
        for (int at = 0; at < labels.size(); at++) {
            TextView label = labels.get(at);
            Feed.Tab tab = feed.tab(at);
            label.setVisibility(tab == null ? View.GONE : View.VISIBLE);
            if (tab == null) {
                continue;
            }
            boolean on = at == focus;
            boolean here = at == feed.active;
            label.setText(tab.label);
            // `TopBar.kt:46-49`: white under the remote, a quarter white for the page the viewer is
            // on, and nothing for the rest.
            label.setBackground(Round.pill(getContext(), BAR_HIGH,
                    on ? Color.WHITE : here ? Ink.white(16) : Color.TRANSPARENT));
            label.setTextColor(on ? Ink.BACK : Color.WHITE);
            Says.onto(label, Says.saysItsWords(on));
        }

        mark.show(feed.server.isEmpty() ? "" : feed.server + "/mark.svg", BAR_HIGH, BAR_HIGH);
    }

    /**
     * Search is a picture, so its words are the only thing that makes it reachable at all.
     * {@code TopBar.kt:101}.
     *
     * Its reading needs the word and whether the remote is on it, and the two arrive from different
     * places: the word when a run reads its language, the focus on every key. So both are kept and
     * the reading is re-applied from whichever arrives second, which is the one place either is
     * written.
     */
    @Override
    public void words(Words words) {
        book = words;
        said = words.of("nav.search");
        Says.onto(search, Says.stop(said, searchLit));
        // The bell's reading is built from three things that never arrive together: the words, the
        // count and where the remote is. The words arrive here, the other two in `show`, so both
        // places write it and whichever runs second is the one a reader hears.
        Says.onto(bell, Says.stop(Feed.bellSays(book, bellUnread), bellLit));
    }

    private String said = "";
    private boolean searchLit;

    /**
     * The words this run has read, kept because the bell's reading needs them and the count at the
     * same time, and the two never arrive in the same call.
     *
     * Search needs no such thing: its reading is one key, so the resolved string is enough. The
     * bell's is a key, a number and a plural-free template, so what has to be kept is the book.
     */
    private Words book = new Words();

    /** How many notices were waiting at the last answer, so {@link #words} can say it again. */
    private int bellUnread;

    private boolean bellLit;

    /** The tabs that fit the bar, which is the Java twin of {@code tabsOf} at {@code TopBar.kt:42}. */
    static List<Feed.Tab> fitting(List<Feed.Tab> all) {
        List<Feed.Tab> kept = new ArrayList<>();
        for (int at = 0; at < all.size() && kept.size() < TABS; at++) {
            if (!NOT_A_TAB.equals(all.get(at).screen)) {
                kept.add(all.get(at));
            }
        }
        return kept;
    }
}
