package streams.tv.notifications;

import android.content.Context;
import android.graphics.Color;
import android.view.Gravity;
import android.view.View;
import android.widget.TextView;
import java.util.Map;
import streams.shared.Ink;
import streams.shared.Says;
import streams.shared.Type;
import streams.shared.WordText;
import streams.shared.Words;
import streams.tv.browse.Art;
import streams.tv.parts.PageHead;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.RemoteHost;
import streams.tv.views.Round;
import streams.tv.views.Word;

/**
 * The notifications screen: what has arrived for this viewer, newest first.
 *
 * <p>The television's answer to {@code phone/notifications/Notifications.kt}. The phone had this
 * screen and the television had none, so a viewer who set a reminder on their phone was told when
 * it arrived and the same person on the television was told nothing. The server has served
 * {@code GET /v1/notifications} to both all along.
 *
 * <h2>The one funnel</h2>
 * The same shape as {@code browse/BrowseView.java}, for the same reason note 210 gives: a key
 * arrives at {@code RemoteHost.dispatchKeyEvent}, goes through {@code Funnel}, reaches {@link
 * #press} here, which writes {@link Spot} and nothing else, and {@code Funnel} calls {@link
 * #changed()} exactly when {@code press} answered true. There is no other writer of this screen's
 * state and no other caller of {@code changed()}.
 *
 * <h2>Written in Java against the contract, with no Kotlin seam</h2>
 * {@link Notices#read(String)} reads {@code GET /v1/notifications} itself, which {@code Feed.java}
 * could not do on 2026-11-12 and note 240 made possible on 2026-11-13. So the whole screen -- the
 * reading, the geometry, the key map, what a reader is handed and the drawing -- is in this
 * package. What is still asked of a host is the shell: an address to go to, a body to post, a
 * picture to fetch. Those are {@link Asked} and {@link Art.Source}.
 */
public class NoticesView extends RemoteHost implements Words.Bound {
    /**
     * The focus outline: <b>3 px</b>, measured.
     *
     * {@code reference/netflix-on-the-tv.md}, "A row screen, exact": focus outline {@code #FBFFFA},
     * 3 px. The colour taken is plain white, which is what {@code browse/Cards.java}'s open card
     * already draws its edge in, rather than a second near-white in the surface.
     */
    public static final int EDGE = 3;

    /** What this screen asks of whoever owns the shell. */
    public interface Asked {
        /** OK was pressed on a notice. The host turns its key into an address. */
        void chose(Notices.Notice notice);

        /**
         * A notice was opened while unread. The host posts {@code /v1/notifications/read} with this
         * id and then {@code shell.refreshNotifications()}, which is the one call every surface
         * already holds on the {@code Shell} port.
         */
        void read(Notices.Notice notice);

        /** Left or Back with nowhere left to go on this screen. */
        void leave();

        /**
         * Something moved. The host reads {@link Spot#wantsMore()} and {@link Notices#cursor},
         * because the paging read is suspending work and belongs to it.
         *
         * This is what replaces a recomposition: in Compose the effect re-ran when the focus
         * changed, and in Views the screen has to say so.
         */
        void moved();
    }

    private final Art.Source pictures;
    private final PageHead head;
    private final RefLayout holder;
    private final RefLayout track;
    private final WordText empty;
    private final Words words = new Words();

    private Spot spot = new Spot(Notices.NOTHING);
    private Asked asked;

    public NoticesView(Context context, Art.Source pictures) {
        super(context);
        this.pictures = pictures;
        setBackgroundColor(Ink.BACK);

        head = new PageHead(context, "notifications.title", "");
        addView(head, RefLayout.at(0, 0, Ref.W, Ref.H));

        // Two boxes and not one, which is the only subtle thing in this layout. The holder stands
        // still and clips, so a line that has slid up disappears under the head instead of drawing
        // over it; the track inside it is what moves. Sliding the clipping box would take its own
        // clip with it and clip nothing.
        holder = new RefLayout(context);
        holder.setClipChildren(true);
        addView(holder, RefLayout.at(0, Spot.FIRST, Ref.W, Ref.H - Spot.FIRST));

        track = new RefLayout(context);
        holder.addView(track, RefLayout.at(0, 0, Ref.W, Ref.H - Spot.FIRST));

        // `Browse.kt:290-295`'s shape: a screen with nothing on it says so in the viewer's own
        // language, in the middle of the screen, rather than drawing an empty page.
        empty = WordText.of(context, "notifications.empty", Line.SMALL_TYPE);
        empty.setTextColor(Ink.MUTED);
        empty.setGravity(Gravity.CENTER);
        addView(empty, RefLayout.at(0, Spot.FIRST, Ref.W, Ref.H - Spot.FIRST));
    }

    public void asks(Asked asked) {
        this.asked = asked;
    }

    /** The words this screen draws, re-read when the language changes. */
    public void speak(Map<String, String> said) {
        if (words.read(said)) {
            words.into(this);
        }
    }

    /**
     * True when this screen runs right to left: the arrows turn and every child mirrors.
     *
     * Not an override, for the reason {@code BrowseView.mirror} gives: {@code RemoteHost.setRtl}
     * and {@code RefLayout.setMirrored} are both final because the mirror is one switch and nothing
     * may intercept it. What this adds is the parts that lay out their own children.
     */
    public void mirror(boolean rtl) {
        setRtl(rtl);
        head.setMirrored(rtl);
        holder.setMirrored(rtl);
        track.setMirrored(rtl);
        for (int at = 0; at < track.getChildCount(); at++) {
            ((Line) track.getChildAt(at)).setMirrored(rtl);
        }
    }

    /**
     * A fresh feed. The focus is kept and the drawing is replaced, which is {@link
     * Spot#arrived(Notices)}: the notice the remote was on is found again by its id.
     *
     * @param spot the screen's own focus, held by the caller so that coming back to this screen
     *             lands on the notice that left it
     */
    public void show(Spot spot, Notices feed) {
        this.spot = spot;
        spot.arrived(feed);
        build();
        paint();
    }

    /** The screen's own focus, for the host to read what the remote is on and whether to page. */
    public Spot spot() {
        return spot;
    }

    /**
     * One word from the remote.
     *
     * The order is the one {@code BrowseView.press} settled: OK first because choosing leaves the
     * screen and is not a move; then what moved; then what leaves. A word that moved something can
     * never also be a word that leaves.
     */
    @Override
    protected boolean press(Word word) {
        if (word == Word.OK) {
            Notices.Notice chosen = spot.chosen();
            if (spot.choose() == Spot.Chose.NOTICE && asked != null) {
                if (chosen.unread) {
                    // Drawn read at once and told to the server after, which is what the phone does
                    // (`Notifications.kt:189-200`): the dot must not wait on the network.
                    asked.read(chosen);
                    spot.arrived(spot.feed().readTo(chosen.id));
                    build();
                    paint();
                }
                asked.chose(chosen);
            }
            // True as `Browse.kt:232-236` answers true: the key was used, whether or not the page
            // it opens exists yet.
            return true;
        }
        if (spot.press(word)) {
            paint();
            if (asked != null) {
                asked.moved();
            }
            return true;
        }
        if (spot.leaves(word)) {
            if (asked != null) {
                asked.leave();
            }
            return true;
        }
        return false;
    }

    /**
     * Something moved, so draw. Called from {@code Funnel} and from nowhere else.
     *
     * The state has already been applied by {@link #press}; this asks for the layout the moved
     * parts need, as {@code BrowseView.changed()} does and for the same reason.
     */
    @Override
    public void changed() {
        requestLayout();
        invalidate();
    }

    /** One line view per notice, as {@code parts/Menu.show} rebuilds its rows for a fresh list. */
    private void build() {
        track.removeAllViews();
        Notices feed = spot.feed();
        for (int at = 0; at < feed.size(); at++) {
            Line line = new Line(getContext(), pictures);
            line.setMirrored(track.isMirrored());
            track.addView(line, RefLayout.at(Round.SUB_EDGE, at * Spot.STEP,
                    Line.WIDE, Spot.STILL_HIGH));
        }
    }

    /** Puts the state on the children. The one place that reads {@link Spot} and writes a view. */
    private void paint() {
        Notices feed = spot.feed();
        boolean bare = feed.items.isEmpty();
        empty.setVisibility(bare ? VISIBLE : GONE);
        holder.setVisibility(bare ? GONE : VISIBLE);
        track.setTranslationY(-Ref.px(this, spot.scrolled() * Spot.STEP));
        for (int at = 0; at < track.getChildCount(); at++) {
            ((Line) track.getChildAt(at)).show(feed.at(at), at == spot.at(), words);
        }
        // What a person hears as the remote moves. A television draws its own focus, so without
        // this nothing announces where the remote has arrived. `RemoteHost.setSays` makes it a
        // polite live region, so it is said again each time it changes -- which is each time this
        // runs.
        setSays(spot.says(words));
    }

    @Override
    public void words(Words said) {
        paint();
    }

    /**
     * One notice: a still, and beside it what kind of notice it is, the title's name and the day.
     *
     * <h2>What a reader is handed, and why it is one stop and not four</h2>
     * The line carries the whole reading -- kind, name, day, and unread as the state after them --
     * and the four things it draws are marked as decoration. A reader therefore stops once per
     * notice and hears all of it, instead of stopping four times and hearing a fragment each time.
     * That is {@code Says.says(label)}'s own bargain: it gathers, so the node is the stop.
     *
     * <p>The alternative -- {@code saysItsWords(on)} on the line, as {@code parts/Menu} does for its
     * rows -- was rejected here because it cannot be checked. {@code screenReaderFocusable} tells a
     * reader to gather the children's words, and the gathering is the reader's work, not the
     * toolkit's: {@code createAccessibilityNodeInfo} on a group returns a node with no text at all,
     * so a test that walks the real node tree would see this line as silent whether it was labelled
     * or not. A description is on the node, and a test can read it.
     */
    static final class Line extends RefLayout {
        /** The line spans the page between its two margins. */
        static final int WIDE = Ref.W - 2 * Round.SUB_EDGE;

        /** The words start past the still and the measured gap. */
        static final int WORDS_AT = Spot.STILL_WIDE + Spot.GAP;

        static final int WORDS_WIDE = WIDE - WORDS_AT;

        /**
         * The title's name, at the size this surface draws a title's name: {@code browse/Cards.java}
         * draws the open card's name at 44.
         */
        static final int NAME_TYPE = 44;

        /**
         * The kind and the day, at this surface's small-word size: {@code parts/Menu.TYPE} and
         * {@code parts/PageHead.HELP_TYPE} are both 30.
         */
        static final int SMALL_TYPE = 30;

        /** The three lines of words and the two measured gaps between them. */
        static final int BLOCK = SMALL_TYPE + Spot.GAP + NAME_TYPE + Spot.GAP + SMALL_TYPE;

        /** Centred against the still, so the two read as one line. */
        static final int WORDS_TOP = (Spot.STILL_HIGH - BLOCK) / 2;

        private final Art still;
        private final TextView kind;
        private final TextView name;
        private final TextView day;
        private final View edge;

        Line(Context context, Art.Source pictures) {
            super(context);
            setBackground(Round.plate(context, Round.CARD, Ink.RAISED));

            still = new Art(context, pictures, null);
            addView(still, RefLayout.at(0, 0, Spot.STILL_WIDE, Spot.STILL_HIGH));

            kind = Type.line(context, SMALL_TYPE, Type.STRONG, Ink.ACCENT);
            addView(kind, RefLayout.at(WORDS_AT, WORDS_TOP, WORDS_WIDE, RefLayout.WRAP));

            name = Type.line(context, NAME_TYPE, Type.HEAVY, Color.WHITE);
            addView(name, RefLayout.at(WORDS_AT, WORDS_TOP + SMALL_TYPE + Spot.GAP,
                    WORDS_WIDE, RefLayout.WRAP));

            day = Type.line(context, SMALL_TYPE, Type.BOOK, Ink.MUTED);
            addView(day, RefLayout.at(WORDS_AT,
                    WORDS_TOP + SMALL_TYPE + Spot.GAP + NAME_TYPE + Spot.GAP,
                    WORDS_WIDE, RefLayout.WRAP));

            edge = new View(context);
            edge.setBackground(Round.edged(context, Round.CARD, Color.TRANSPARENT, EDGE, Color.WHITE));
            addView(edge, RefLayout.at(0, 0, WIDE, Spot.STILL_HIGH));

            // Everything drawn inside is said by the line itself, so none of it is a stop of its own.
            Says.onto(still, Says.decoration());
            Says.onto(kind, Says.decoration());
            Says.onto(name, Says.decoration());
            Says.onto(day, Says.decoration());
            Says.onto(edge, Says.decoration());
        }

        void show(Notices.Notice notice, boolean on, Words words) {
            String kindWords = notice == null ? "" : words.of(notice.kind);
            kind.setText(kindWords);
            kind.setVisibility(kindWords.isEmpty() ? GONE : VISIBLE);
            name.setText(notice == null ? "" : notice.name);
            // Read and unread differ in weight of ink and not in a mark: a dot small enough to sit
            // in a line of text is a dot nobody sees from three metres away.
            name.setTextColor(notice != null && notice.unread ? Color.WHITE : Ink.MUTED);
            day.setText(notice == null ? "" : notice.day);
            edge.setVisibility(on ? VISIBLE : GONE);

            String label = Says.sayBoth(Says.sayBoth(kindWords, notice == null ? "" : notice.name),
                    notice == null ? "" : notice.day);
            String state = notice != null && notice.unread ? words.of("say.unread") : "";
            Says.onto(this, Says.says(label).and(state).on(on));
        }
    }
}
