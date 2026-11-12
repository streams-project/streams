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
import streams.shared.R;
import streams.tv.views.Ink;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.Round;
import streams.tv.views.Says;
import streams.tv.views.Type;
import streams.tv.views.WordText;
import streams.tv.views.Words;

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
 * <h2>Its three readings</h2>
 * {@code SpokenTest.kt}'s census records this file at 3 calls and these are the three: the profile
 * says the person's name and whether the remote is on it ({@code TopBar.kt:83}); search says its
 * word and the same ({@code :101}); a tab's own drawn words are its label ({@code :56}). The face
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

    private final RefLayout who;
    private final Parts.Face face;
    private final ImageView caret;
    private final View search;
    private final ImageView glass;
    private final LinearLayout tabs;
    private final List<TextView> labels = new ArrayList<>();
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

        while (labels.size() < feed.tabs.size()) {
            TextView label = Type.line(getContext(), 26, Type.MIDDLE, Color.WHITE);
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
        said = words.of("nav.search");
        Says.onto(search, Says.stop(said, searchLit));
    }

    private String said = "";
    private boolean searchLit;

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
