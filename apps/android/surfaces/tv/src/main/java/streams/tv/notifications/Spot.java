package streams.tv.notifications;

import streams.shared.Says;
import streams.shared.Words;
import streams.tv.views.Ref;
import streams.tv.views.Word;

/**
 * Where the viewer is on the notifications list, and what the remote does from there. It knows
 * nothing about drawing.
 *
 * <p>The twin of {@code browse/Place.java} for a screen with one column: four fields, an explicit
 * answer to every word a remote can say, and no toolkit in it at all, so every row of the key map
 * below is pressed by {@code SpotTest} on this machine's JVM with no device.
 *
 * <h2>The key map, designed for this screen and not inherited</h2>
 * The owner's rule is that a television screen's key map is designed per screen. This one is a
 * single column, so every word has to be answered here rather than left to fall through:
 *
 * <table>
 * <tr><th>word</th><th>what it does</th><th>at the edge</th></tr>
 * <tr><td>Up</td><td>the line above</td><td>on the first line, nothing moves</td></tr>
 * <tr><td>Down</td><td>the line below</td><td>on the last line, nothing moves</td></tr>
 * <tr><td>Left</td><td>leaves the screen</td><td>every line is the left edge of this page</td></tr>
 * <tr><td>Right</td><td>nothing. There is no second column</td><td>--</td></tr>
 * <tr><td>OK</td><td>opens the title, and the notice is read</td><td>--</td></tr>
 * <tr><td>Back</td><td>leaves the screen</td><td>--</td></tr>
 * </table>
 *
 * <p><b>Left.</b> This surface already decided what Left does at the left edge of a page that is
 * not a row screen: {@code settings/Settings.kt:257-258}, "On the sections, the left edge of the
 * page, Left goes back". A one column list is all left edge, so Left leaves from any line. The
 * alternative -- Left doing nothing -- was rejected because a remote's left arrow is how a person
 * backs out of a pane on this surface everywhere else, and a dead arrow on one screen is the kind
 * of difference a viewer feels without being able to name.
 *
 * <p><b>Up and Down at the ends stop.</b> They do not wrap. Nothing in this surface wraps: the row
 * screen's {@code Place} stops at {@code row > 0} and {@code row < lastRow()}, and a list that
 * jumped from the last notice to the first would be the only place on the television that did.
 *
 * <p><b>Left and Back are not moves.</b> {@link #press} answers false for both, exactly as
 * {@code Place.press} answers false for OK: leaving the page is not something that moved on it, so
 * it must not report a redraw. {@link #leaves(Word)} is what the screen asks instead.
 *
 * <h2>Focus on return</h2>
 * {@link #arrived(Notices)} keeps the focus on the notice it was on, found again <em>by its id</em>
 * and not by its place. The feed is the one list in this app that grows at the top: a notice that
 * arrives while the screen is open pushes every other one down, and a focus kept by number would
 * silently slide onto the notice above the one the viewer was reading. The same reasoning, and the
 * same shape, as {@code RowMemory} keeping a row's place under the title that was on it (note 187).
 */
public final class Spot {
    /**
     * The still on a line: <b>459 x 261</b>, measured.
     *
     * {@code reference/netflix-on-the-tv.md}, "Episodes screen": each episode card is a "459 x 261
     * (16:9) still with the episode label burnt in bottom left, then title, synopsis and duration
     * beside it". That is this screen -- a picture with words beside it, stacked down a page -- so
     * the measured card is taken whole rather than a new size being invented for it.
     */
    public static final int STILL_WIDE = 459;

    /** And its height. Same measurement. */
    public static final int STILL_HIGH = 261;

    /**
     * The gap above, below and beside: <b>25 px</b>, measured.
     *
     * {@code reference/netflix-on-the-tv.md}, "Vertical rhythm of a row screen": "tiles 199-708,
     * gap 25, facts line 733-755". 25 is the measured distance between a picture and the words
     * underneath it on this television, so it is the distance between a picture and the words
     * beside it here, and between one line and the next.
     */
    public static final int GAP = 25;

    /**
     * Where the page head ends: {@code parts/PageHead.under()}, which is {@code HELP_AT 148} plus
     * {@code HELP_TYPE 30}.
     *
     * Written as a number here and not as the call, because this class holds no toolkit and
     * {@code PageHead} is a {@code View}. {@code NoticesHeardTest} holds the two together, where
     * Android is real, so the two cannot drift.
     */
    public static final int HEAD = 178;

    /** The first line's top. */
    public static final int FIRST = HEAD + GAP;

    /** One line to the next: the still plus the gap, as the row screen's pitch is tile plus gutter. */
    public static final int STEP = STILL_HIGH + GAP;

    /**
     * How many lines are wholly on the screen: <b>3</b>.
     *
     * {@code (1080 - 203) / 286}. A derivation and not a measurement, from {@link Ref#H} and the
     * three numbers above, so a change to any of them moves it.
     */
    public static final int VISIBLE = (Ref.H - FIRST) / STEP;

    /**
     * How close to the end the focus comes before the next page is asked for: one screenful.
     *
     * The phone asks at {@code lastVisible >= items.size - 3}
     * ({@code phone/notifications/Notifications.kt:106}) and arrives at the same 3 from a different
     * screen, which is a coincidence worth saying out loud rather than a shared constant: this one
     * follows the television's own geometry and would move if the still did.
     */
    public static final int REACH = VISIBLE;

    /** What OK chooses from here. */
    public enum Chose {
        /** Nothing: an empty feed, where OK has nowhere to go. */
        NOTHING,
        /** A notice. {@link #chosen()} is which. */
        NOTICE
    }

    private Notices feed;
    private int at;

    /** The id of the notice the remote is on, so a fresh feed can find it again. */
    private String kept = "";

    public Spot(Notices feed) {
        this.feed = feed == null ? Notices.NOTHING : feed;
        this.at = 0;
        this.kept = idAt(0);
    }

    /**
     * A fresh feed for the same screen. The drawing is replaced and where the viewer is, is not:
     * the notice that was under the remote is found again by id, and the focus follows it.
     *
     * A notice that is gone leaves the focus at the same place in the new list, clamped to its end,
     * which is the nearest thing to where the viewer was looking.
     */
    public void arrived(Notices feed) {
        this.feed = feed == null ? Notices.NOTHING : feed;
        int found = this.feed.placeOf(kept);
        if (found >= 0) {
            at = found;
        } else if (at > this.feed.last()) {
            at = Math.max(0, this.feed.last());
        }
        kept = idAt(at);
    }

    public Notices feed() {
        return feed;
    }

    /** Which line the remote is on. */
    public int at() {
        return at;
    }

    /**
     * How far the list has slid up, in lines.
     *
     * The focus walks down to the last line that is wholly on the screen and the list scrolls from
     * there, which is {@code settings/Settings.kt:238}'s {@code maxOf(0, line - 7)} shape for this
     * screen's three lines. The row screen's rule is the other one -- the reference measures the
     * focused row pinned to {@code y 199-708} in 24 of 24 frames and the content scrolling under it
     * -- and it is not taken here: that band was measured on a screen of tall tiles where one row is
     * the whole of the content, and pinning a three line list to its first line would scroll the
     * page on the very first press of Down with two lines still unread below.
     */
    public int scrolled() {
        return Math.max(0, at - (VISIBLE - 1));
    }

    /**
     * One word from the remote. True when something moved, which is the one thing that redraws.
     *
     * False for Left, Back and OK: none of the three moves anything on this page, and a redraw
     * reported for them would be a redraw the screen did not need. What they do instead is
     * {@link #leaves(Word)} and {@link #choose()}.
     */
    public boolean press(Word word) {
        switch (word) {
            case UP:
                if (at > 0) {
                    at -= 1;
                    kept = idAt(at);
                    return true;
                }
                return false;
            case DOWN:
                if (at < feed.last()) {
                    at += 1;
                    kept = idAt(at);
                    return true;
                }
                return false;
            case LEFT:
            case RIGHT:
            case OK:
            case BACK:
            default:
                return false;
        }
    }

    /**
     * Whether this word leaves the screen. Left and Back, and nothing else.
     *
     * Asked by the screen after {@link #press} has answered false, in the same order
     * {@code BrowseView.press} asks it: a word that moved something is never a word that leaves.
     */
    public boolean leaves(Word word) {
        return word == Word.LEFT || word == Word.BACK;
    }

    /** What OK chooses from here. */
    public Chose choose() {
        return chosen() == null ? Chose.NOTHING : Chose.NOTICE;
    }

    /** Which notice OK chose, or null on an empty feed. */
    public Notices.Notice chosen() {
        return feed.at(at);
    }

    /**
     * Whether the screen should read its next page now: the focus is within {@link #REACH} of the
     * last notice held, or nothing has arrived at all and the screen can only fill by asking.
     *
     * {@code Place.wantsMore}'s shape, with {@code more} added because this feed has an end the
     * server names and a page of rows does not.
     */
    public boolean wantsMore() {
        if (feed.items.isEmpty()) {
            return true;
        }
        return feed.more && at >= feed.last() - REACH;
    }

    /**
     * What a person hears as the remote moves.
     *
     * A television draws its own focus and this screen is one focusable node, so without this
     * nothing announces where the remote has arrived. It is handed to {@code RemoteHost.setSays},
     * which makes it a polite live region, so it is said again each time it changes.
     *
     * <p>Four things, in the order a person wants them: what kind of notice it is, the title's
     * name, the day, and whether it has been read. They are joined by {@link Says#sayBoth}, so the
     * break is a line end and no language's punctuation or word order is decided here. A kind this
     * build has no key for, and a word the run has not read yet, are both empty and drop out, which
     * is {@code Words.of}'s rule: never English, never the key.
     *
     * <p>An empty feed says the screen's own name and that there is nothing on it, because a reader
     * that arrives on a screen with no stops would otherwise be handed silence.
     */
    public String says(Words words) {
        Notices.Notice notice = feed.at(at);
        if (notice == null) {
            return Says.sayBoth(words.of("notifications.title"), words.of("notifications.empty"));
        }
        String what = Says.sayBoth(words.of(notice.kind), notice.name);
        String when = Says.sayBoth(notice.day, notice.unread ? words.of("say.unread") : "");
        return Says.sayBoth(what, when);
    }

    private String idAt(int index) {
        Notices.Notice notice = feed.at(index);
        return notice == null ? "" : notice.id;
    }
}
