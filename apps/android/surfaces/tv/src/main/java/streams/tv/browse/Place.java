package streams.tv.browse;

import streams.tv.views.RowMemory;
import streams.tv.views.Says;
import streams.tv.views.Word;
import streams.tv.views.Words;

/**
 * Where the viewer is on a page of rows. It knows nothing about drawing.
 *
 * A port of {@code PageFocus} at {@code tv/browse/Browse.kt:76-152}, with {@code choose}
 * ({@code :218-230}) and the spoken {@code here} ({@code :257-266}) brought into it. The four
 * {@code mutableStateOf} and {@code mutableIntStateOf} delegates at {@code Browse.kt:78-81} become
 * four plain fields, which is note 210's "what is harder in Views" item 1: 164 snapshot state sites
 * across the tracked Kotlin become fields plus an explicit redraw, and a missed redraw is the
 * largest source of new defects in the whole programme.
 *
 * <h2>Why {@code choose} and {@code says} moved in here, which is a change from the Kotlin</h2>
 * In Compose they were a local function and a local {@code val} inside the {@code Browse} composable
 * ({@code Browse.kt:218-230}, {@code :257-266}), so neither could be reached by a test and both read
 * the shell and the words directly. Here the state machine answers three questions about itself --
 * what moved, what the viewer chose, and what a screen reader should hear -- and none of the three
 * touches a toolkit, a shell or a context. That is what makes note 210's named mitigation a real
 * test: {@code PlaceTest} presses a scripted key sequence and asserts both what moved and that
 * {@code press} answered true exactly when something did.
 *
 * The shell stays out: {@link #chose()} answers <em>what</em> was chosen and the seam turns it into
 * an address, because {@code titleAddress} and {@code tabAddress}
 * ({@code shared/Addresses.kt:8-14}) take contract value types this language cannot name.
 */
public final class Place {
    /** The three places the remote can be on a page of rows. {@code Browse.kt:55}. */
    public enum Where {
        TOP,
        HERO,
        ROW
    }

    /** What OK did, for the seam to turn into an address. {@code Browse.kt:218-230}. */
    public enum Chose {
        /** Nothing: a page with no hero and no rows, where OK has nowhere to go. */
        NOTHING,
        /** The face: the profile menu opens over the page. */
        MENU,
        /** The search button. */
        SEARCH,
        /** A tab of the bar. {@link #chosenTab()} is which. */
        TAB,
        /** A title in a row, or the hero's More info, or the hero's Play on a series. */
        TITLE,
        /** The hero's Play on something that is not a series: it starts playing. */
        WATCH
    }

    private Feed feed;
    private int tabs;
    private final int active;
    private boolean hero;

    private Where where;
    private int row;
    private int tab;
    private int button;

    /**
     * Where each row of this page was left. Wave 1's class, unchanged: the place and the title that
     * was on it, so a fresh answer that reordered a row follows the title rather than the number.
     * {@code Browse.kt:86-103}, which note 187 records as Android {@code fd7b104}.
     */
    private final RowMemory columns = new RowMemory();

    /**
     * A page the viewer has just arrived on.
     *
     * The opening {@code where} is {@code Browse.kt:78}'s exactly: the hero or the bar when there is
     * a hero or there are no rows, and the first row otherwise. Its {@code tab} starts on the page's
     * own tab ({@code :80}), which is what makes Back's "come home to this tab first" branch mean
     * something.
     */
    public Place(Feed feed, int active) {
        this.feed = feed == null ? Feed.NOTHING : feed;
        this.tabs = this.feed.tabs.size();
        this.active = active;
        this.hero = this.feed.hero != null;
        this.where = hero || this.feed.rows.isEmpty() ? above() : Where.ROW;
        this.row = 0;
        this.tab = active;
        this.button = 0;
    }

    /**
     * A fresh answer for the same page. {@code Browse.kt:179-181}: the rows, the tab count and
     * whether there is a hero are replaced, and where the viewer is, is not.
     *
     * The three fields are re-read and {@code where}, {@code row}, {@code tab} and {@code button}
     * are left alone on purpose -- that is what "a page keeps the line or card it opened" means in
     * note 187's key map. {@code row} is not clamped here either, for the same reason
     * {@code Browse.kt} does not clamp it: {@link #row()} is read through {@link Feed#row(int)},
     * which answers null past the end, so a page that shrank draws nothing rather than crashing,
     * and a page that grew back finds the viewer where they were.
     */
    public void arrived(Feed feed) {
        this.feed = feed == null ? Feed.NOTHING : feed;
        this.tabs = this.feed.tabs.size();
        this.hero = this.feed.hero != null;
    }

    /** The page above the rows: the hero when there is one, the bar when there is not. {@code Browse.kt:77}. */
    private Where above() {
        return hero ? Where.HERO : Where.TOP;
    }

    public Where where() {
        return where;
    }

    public int row() {
        return row;
    }

    public int tab() {
        return tab;
    }

    public int button() {
        return button;
    }

    /** The page's own tab, which Back comes home to. */
    public int active() {
        return active;
    }

    public Feed feed() {
        return feed;
    }

    /** The tile a row was left on, or its first. {@code Browse.kt:89-96}, through wave 1's kernel. */
    public int columnOf(int index) {
        Feed.Row found = feed.row(index);
        if (found == null) {
            return 0;
        }
        return columns.columnOf(found.id, found.keys());
    }

    /** And for the row the remote is on. {@code Browse.kt:98}. */
    public int column() {
        return columnOf(row);
    }

    /** {@code Browse.kt:100-103}. */
    private void keep(int at) {
        Feed.Row found = feed.row(row);
        if (found == null) {
            return;
        }
        columns.keep(found.id, found.keys(), at);
    }

    /** {@code Browse.kt:105}. */
    private int lastColumn() {
        Feed.Row found = feed.row(row);
        return found == null ? -1 : found.cards.size() - 1;
    }

    /**
     * One word from the remote. True when something moved, which is the one thing that redraws.
     *
     * A translation of {@code Browse.kt:107-151} branch for branch and in the same order, because
     * in one place the order <em>is</em> the behaviour. Going right along the bar, the two negative
     * places ({@link Feed#WHO_TAB} at -3 and {@link Feed#SEARCH_TAB} at -2) have to be tried before
     * {@code tab < tabs - 1}, because both of them satisfy that test as well: a reordered
     * {@code when} would increment -3 to -2 and then -2 to -1, which is no place at all, and the
     * profile would walk sideways into nothing instead of onto search. Going left the branches are
     * disjoint and the order does not matter, which is worth saying because it is the half a reader
     * would assume was the dangerous one. Proved both ways by mutation on 2026-11-12.
     *
     * {@code OK} answers false here and is answered by {@link #choose()}, exactly as
     * {@code Browse.kt:148} and {@code :232-236} split it: choosing leaves the page, so it is not a
     * move and must not be reported as one.
     */
    public boolean press(Word word) {
        switch (word) {
            case UP:
                if (where == Where.ROW && row > 0) {
                    row -= 1;
                } else if (where == Where.ROW) {
                    where = above();
                } else if (where == Where.HERO) {
                    where = Where.TOP;
                } else {
                    return false;
                }
                return true;
            case DOWN:
                if (where == Where.TOP && hero) {
                    where = Where.HERO;
                } else if ((where == Where.HERO || where == Where.TOP) && !feed.rows.isEmpty()) {
                    where = Where.ROW;
                    row = 0;
                } else if (where == Where.ROW && row < feed.lastRow()) {
                    row += 1;
                } else {
                    return false;
                }
                return true;
            case LEFT:
                if (where == Where.TOP && tab > 0) {
                    tab -= 1;
                } else if (where == Where.TOP && tab == 0) {
                    tab = Feed.SEARCH_TAB;
                } else if (where == Where.TOP && tab == Feed.SEARCH_TAB) {
                    tab = Feed.WHO_TAB;
                } else if (where == Where.HERO) {
                    button = 0;
                } else if (where == Where.ROW && column() > 0) {
                    keep(column() - 1);
                } else {
                    return false;
                }
                return true;
            case RIGHT:
                if (where == Where.TOP && tab == Feed.WHO_TAB) {
                    tab = Feed.SEARCH_TAB;
                } else if (where == Where.TOP && tab == Feed.SEARCH_TAB) {
                    tab = 0;
                } else if (where == Where.TOP && tab < tabs - 1) {
                    tab += 1;
                } else if (where == Where.HERO) {
                    button = 1;
                } else if (where == Where.ROW && column() < lastColumn()) {
                    keep(column() + 1);
                } else {
                    return false;
                }
                return true;
            case BACK:
                if (where == Where.ROW) {
                    where = above();
                } else if (where == Where.HERO) {
                    where = Where.TOP;
                } else if (tab != active) {
                    // Focus wandered along the bar without choosing: Back brings it home to this
                    // page's tab first, and only then leaves the page. Note 187's own row, and
                    // `Browse.kt:143-146`.
                    tab = active;
                } else {
                    return false;
                }
                return true;
            case OK:
            default:
                return false;
        }
    }

    /**
     * What OK chooses from here. The five branches of {@code Browse.kt:219-229}, in order.
     *
     * The last two are the hero's, and the condition is the one at {@code :227}: More info, and Play
     * on a series, both open the title's page; Play on anything else starts playing.
     */
    public Chose choose() {
        if (where == Where.TOP && tab == Feed.WHO_TAB) {
            return Chose.MENU;
        }
        if (where == Where.TOP && tab == Feed.SEARCH_TAB) {
            return Chose.SEARCH;
        }
        if (where == Where.TOP) {
            return feed.tab(tab) == null ? Chose.NOTHING : Chose.TAB;
        }
        if (where == Where.ROW) {
            return chosenCard() == null ? Chose.NOTHING : Chose.TITLE;
        }
        if (feed.hero == null) {
            return Chose.NOTHING;
        }
        return button == 1 || feed.hero.series() ? Chose.TITLE : Chose.WATCH;
    }

    /** Which tab OK chose, or null when it chose something else. */
    public Feed.Tab chosenTab() {
        return where == Where.TOP ? feed.tab(tab) : null;
    }

    /**
     * Which title OK chose, or null. The tile under the remote in a row, or the hero.
     *
     * Named {@code chosen} and not {@code card} because the hero is the answer when the remote is on
     * it, and the hero is not in any row.
     */
    public Feed.Card chosenCard() {
        if (where == Where.ROW) {
            Feed.Row found = feed.row(row);
            return found == null ? null : found.at(column());
        }
        if (where == Where.HERO) {
            return feed.hero;
        }
        return null;
    }

    /** The card the remote is on inside a row, for the open card and for what is said. */
    public Feed.Card open() {
        Feed.Row found = feed.row(row);
        return found == null ? null : found.at(column());
    }

    /**
     * What a person hears as the remote moves. {@code Browse.kt:257-266}, with the words read
     * through wave 1's {@link Words} instead of out of a map.
     *
     * A television draws its own focus, so without this nothing announces where the remote has
     * arrived ({@code tv/app/Spoken.kt:26-32}). It is handed to {@code RemoteHost.setSays}, which
     * makes it a polite live region, so it is spoken again each time it changes -- which is each
     * time the remote moves.
     *
     * No key here has an English fallback: a word the strings do not hold answers empty and the part
     * is left unlabelled, which is {@code Says.kt:46-52}'s rule and {@link Words#of}'s behaviour.
     */
    public String says(Words words) {
        if (where == Where.TOP && tab == Feed.WHO_TAB) {
            return feed.who == null ? "" : feed.who.name;
        }
        if (where == Where.TOP && tab == Feed.SEARCH_TAB) {
            return words.of("nav.search");
        }
        if (where == Where.TOP) {
            Feed.Tab found = feed.tab(tab);
            return found == null ? "" : found.label;
        }
        if (where == Where.HERO) {
            return words.of(button == 1 ? "common.more_info" : "common.play");
        }
        Feed.Row found = feed.row(row);
        Feed.Card card = found == null ? null : found.at(column());
        return Says.sayBoth(card == null ? null : card.name, found == null ? null : found.title);
    }

    /**
     * Whether the page should read its next rows now: the focus is within {@code REACH} of the last
     * row the page holds, or the page has no rows at all and can only grow by asking.
     *
     * {@code Browse.kt:203}, and {@code REACH = 2} with its derivation at {@code :45-53}: one row
     * below the focused one is already partly on screen -- 82 px of its tiles
     * ({@code netflix-reference/tv/raw/tv-geometry-1920x1080.json},
     * {@code row_screen.vertical_layout_px.next_row_tile_visible_height}) -- so the second row down
     * is the first the viewer cannot see at all. Holding that one as well is this surface's own
     * prefetch margin and not a Netflix measurement: Netflix prefetches nothing on a television and
     * fetches on arrival instead
     * ({@code netflix-reference/tv/00-netflix-on-the-tv-measured.md}, section 7).
     */
    public boolean wantsMore() {
        return feed.rows.isEmpty() || (where == Where.ROW && row >= feed.lastRow() - REACH);
    }

    /** {@code Browse.kt:53}. */
    public static final int REACH = 2;
}
