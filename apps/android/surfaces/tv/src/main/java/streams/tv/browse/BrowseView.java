package streams.tv.browse;

import android.content.Context;
import android.view.Gravity;
import java.util.Map;
import streams.tv.views.Ink;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.RemoteHost;
import streams.tv.views.Round;
import streams.tv.views.Word;
import streams.tv.views.WordText;
import streams.tv.views.Words;

/**
 * A page of rows: the television's home, its genre pages and its lists.
 *
 * The Views form of {@code tv/browse/Browse.kt:155-301}. The parts draw and this class only knows
 * where the focus is, which is that file's own sentence for itself ({@code :154}).
 *
 * <h2>The one funnel, which is the point of the whole shape</h2>
 * Note 210 names a missed redraw as the largest source of new defects in this programme: 164
 * snapshot state sites become plain fields, the build is green, the tests pass, and one screen
 * silently does not move. The mitigation it states is a single funnel, and this is it end to end:
 * a key arrives at {@code RemoteHost.dispatchKeyEvent}, goes through {@link streams.tv.views.Funnel}
 * which filters a key down and turns the arrows round for a right to left page, reaches
 * {@link #press} here, which writes {@link Place} and nothing else, and {@code Funnel} then calls
 * {@link #changed()} exactly when {@code press} answered true. There is no other writer of this
 * screen's state and no other caller of {@code changed()}.
 *
 * {@link #changed()} is overridden because this screen's parts move: {@code RemoteHost}'s own
 * {@code invalidate()} is right for a screen that paints its focus and lays out the same, and this
 * one slides a row's posters and swaps which row is open. So {@code changed()} re-applies the state
 * to the children, and the children decide what that costs -- a poster already pointed at a title
 * does not re-read its picture, so a key press is a few {@code setText} calls and one animator.
 *
 * <h2>Why there is no row sliding in and out yet</h2>
 * {@code Browse.kt:271-286}'s {@code AnimatedContent} slides the row leaving upwards and the row
 * arriving from below by {@code RISE = 810} over {@code RISE_MS = 320}, crossfading, and handles
 * interruption. That is two row views and a pair of animators, which this holds: {@link #rising} is
 * the second one. What it does not hold is a third, so a viewer holding the down arrow through three
 * rows sees the third move start from the second's end rather than from a third view -- which is
 * what Compose does too, because {@code AnimatedContent} keeps one outgoing child.
 */
public class BrowseView extends RemoteHost implements Words.Bound {
    /** How far a row rises as the page moves between rows. {@code tv/browse/Browse.kt:42}. */
    public static final int RISE = 810;

    /** And how long it takes. {@code tv/browse/Browse.kt:43}. */
    public static final int RISE_MS = 320;

    /** What this screen asks of whoever owns the shell. */
    public interface Asked {
        /** OK was pressed on something. The seam turns it into an address. */
        void chose(Place.Chose what, Feed.Tab tab, Feed.Card card);

        /** Back was pressed with nowhere left to go on this page. {@code Browse.kt:248-251}. */
        void leave();

        /**
         * Something moved. The seam reads {@link Place#wantsMore()} and the card under the remote,
         * because the paging read and the detail read are suspending work and belong to it.
         *
         * This is what replaces a recomposition: Compose re-ran {@code LaunchedEffect(focus.row,
         * focus.where, ...)} at {@code Browse.kt:198} when the focus changed, and in Views the
         * screen has to say so.
         */
        void moved();
    }

    private final Art.Source pictures;
    private final BarView bar;
    private final HeroView hero;
    private final RowsView.Open open;
    private final RowsView.Open rising;
    private final RowsView.Next next;
    private final WordText empty;

    private final Words words = new Words();
    private Place place = new Place(Feed.NOTHING, -1);
    private Asked asked;
    private Feed.Story story;

    /** Which of the two row views is the one on screen now. */
    private boolean risingIsOpen;

    public BrowseView(Context context, Art.Source pictures) {
        super(context);
        this.pictures = pictures;
        setBackgroundColor(Ink.BACK);

        bar = new BarView(context, pictures);
        addView(bar, RefLayout.at(0, 0, Ref.W, Ref.H));

        hero = new HeroView(context, pictures);
        // `HeroCard.kt:53-54`: the page margin, 143 down, and the page less its two margins wide.
        // The width is written as that derivation and not as the 888 dp it comes to, which is what
        // `NumbersTest.kt`'s sixth test is for.
        addView(hero, RefLayout.at(Round.PAGE_EDGE, HeroView.Y,
                Ref.W - 2 * Round.PAGE_EDGE, HeroView.HIGH));

        open = new RowsView.Open(context, pictures);
        addView(open, RefLayout.at(0, 0, Ref.W, Ref.H));
        rising = new RowsView.Open(context, pictures);
        addView(rising, RefLayout.at(0, 0, Ref.W, Ref.H));
        rising.setVisibility(GONE);

        next = new RowsView.Next(context, pictures);
        addView(next, RefLayout.at(0, 0, Ref.W, Ref.H));

        // `Browse.kt:290-295`: a page with no hero and no rows says so in the middle of the screen,
        // in the viewer's language. The key is chosen by the seam, because which of the two words it
        // is depends on which page this is.
        empty = WordText.of(context, "", 32);
        empty.setTextColor(Ink.MUTED);
        empty.setGravity(Gravity.CENTER);
        addView(empty, RefLayout.at(0, 0, Ref.W, Ref.H));
    }

    public void asks(Asked asked) {
        this.asked = asked;
    }

    /** The words this page draws, re-read when the language changes. */
    public void speak(Map<String, String> said) {
        if (words.read(said)) {
            words.into(this);
        }
    }

    /**
     * A fresh answer. The focus is kept and the drawing is replaced, which is
     * {@code Browse.kt:176-181}: the page's {@code PageFocus} is remembered for the run of the app
     * and only its rows, its tab count and its hero are refreshed.
     *
     * @param place the page's own focus, held by the caller in a {@code PageMemory} so that coming
     *              back to this page lands on the tile that left it
     */
    public void show(Place place, Feed feed, String emptyKey) {
        this.place = place;
        place.arrived(feed);
        empty.setKey(emptyKey);
        empty.words(words);
        paint(false);
    }

    /**
     * True when this page runs right to left: the arrows turn and every child mirrors.
     *
     * One line fed from {@code seen.rtl}, as {@code tv/app/Television.kt:52} feeds it today, and
     * note 210's own warning applies -- a wrong sign here mirrors every screen at once. It is not
     * an override because {@code RemoteHost.setRtl} and {@code RefLayout.setMirrored} are both
     * final: the mirror is one switch and nothing may intercept it. What this adds is that the
     * parts which are layouts of their own place their own children, so each has to be told.
     */
    public void mirror(boolean rtl) {
        setRtl(rtl);
        bar.setMirrored(rtl);
        hero.mirror(rtl);
        open.setMirrored(rtl);
        rising.setMirrored(rtl);
        next.setMirrored(rtl);
    }

    /** What is known about the title the remote is on. {@code Browse.kt:143-153}. */
    public void told(Feed.Story story) {
        this.story = story;
        if (place.where() == Place.Where.ROW) {
            held().tell(story);
        } else {
            hero.show(place.feed().hero, story, place.where() == Place.Where.HERO
                    ? place.button() : -1);
        }
    }

    /** The page's own focus, for the seam to read what the remote is on. */
    public Place place() {
        return place;
    }

    @Override
    protected boolean press(Word word) {
        if (word == Word.OK) {
            if (asked != null) {
                asked.chose(place.choose(), place.chosenTab(), place.chosenCard());
            }
            // True, as `Browse.kt:232-236` answers true: the key was used, whether or not the page
            // it opens exists yet.
            return true;
        }
        int wasRow = place.row();
        Place.Where wasWhere = place.where();
        if (place.press(word)) {
            boolean rowChanged = place.where() == Place.Where.ROW
                    && (wasWhere != Place.Where.ROW || wasRow != place.row());
            // Which way the rise goes is which way the focus went, which is what
            // `targetState > initialState` answers at `Browse.kt:273`. Arriving in a row from above
            // comes up from below, as moving down a row does.
            goingDown = wasWhere != Place.Where.ROW || place.row() > wasRow;
            paint(rowChanged);
            if (asked != null) {
                asked.moved();
            }
            return true;
        }
        if (word == Word.BACK) {
            // Back from the top of any page but Home goes Home, and on Home it stops here: the app
            // stays open. `Browse.kt:238-251` carries the whole of the reasoning and the owner's
            // open question with it, which this does not reopen.
            if (asked != null) {
                asked.leave();
            }
            return true;
        }
        return false;
    }

    /**
     * Something moved, so draw. Called from {@link streams.tv.views.Funnel} and from nowhere else.
     *
     * {@code RemoteHost.changed()} invalidates, which is right for a screen whose parts do not move.
     * This one's do, and the state has already been applied by {@link #press}, so this only has to
     * ask for the layout the moved parts need.
     */
    @Override
    public void changed() {
        requestLayout();
        invalidate();
    }

    /** The row view that is on screen now. */
    private RowsView.Open held() {
        return risingIsOpen ? rising : open;
    }

    private RowsView.Open spare() {
        return risingIsOpen ? open : rising;
    }

    /**
     * Puts the state on the children. The one place that reads {@link Place} and writes a view.
     *
     * @param rose true when the page has moved to a different row, which is the only case the rise
     *             animation runs for
     */
    private void paint(boolean rose) {
        Feed feed = place.feed();
        Place.Where where = place.where();
        boolean inRow = where == Place.Where.ROW;

        bar.show(feed, where == Place.Where.TOP ? place.tab() : -1);

        boolean showHero = !inRow && feed.hero != null;
        hero.setVisibility(showHero ? VISIBLE : GONE);
        if (showHero) {
            hero.show(feed.hero, story, where == Place.Where.HERO ? place.button() : -1);
        }

        // `Browse.kt:289-295`: the words in the middle only when there is no hero and no row.
        empty.setVisibility(!inRow && feed.hero == null && feed.rows.isEmpty() ? VISIBLE : GONE);

        if (inRow) {
            Feed.Row row = feed.row(place.row());
            if (rose) {
                rise(row);
            } else {
                held().setVisibility(VISIBLE);
                held().show(row, place.columnOf(place.row()), true);
                held().tell(story);
            }
        } else {
            open.setVisibility(GONE);
            rising.setVisibility(GONE);
        }

        // `Browse.kt:297-298`: the dim row below is the next one when the viewer is in a row, and
        // the first one when they are above the rows. It shows where it was left, which is the
        // second half of note 187's "a page keeps the line or card it opened".
        int below = inRow ? place.row() + 1 : 0;
        next.show(feed.row(below), !inRow, place.columnOf(below));

        // What a person hears as the remote moves. A television draws its own focus, so without
        // this nothing announces where the remote has arrived (`tv/app/Spoken.kt:26-32`).
        // `RemoteHost.setSays` makes it a polite live region, so it is said again each time it
        // changes -- which is each time this runs.
        setSays(place.says(words));
    }

    /**
     * The row rise. {@code Browse.kt:271-286}: the row arriving slides up from {@code RISE} below
     * and fades in from 0.4, the row leaving slides the same distance the other way and fades out,
     * both on {@link RowsView#glide()} over {@link #RISE_MS}.
     *
     * The direction is the direction the focus moved, which is what {@code targetState >
     * initialState} answers at {@code :273}.
     */
    private void rise(Feed.Row row) {
        RowsView.Open going = held();
        RowsView.Open coming = spare();
        risingIsOpen = !risingIsOpen;

        int way = goingDown ? 1 : -1;
        float rise = Ref.px(this, RISE);

        coming.setVisibility(VISIBLE);
        coming.show(row, place.columnOf(place.row()), false);
        coming.tell(story);
        coming.animate().cancel();
        coming.setTranslationY(way * rise);
        coming.setAlpha(0.4f);
        coming.animate().translationY(0f).alpha(1f)
                .setDuration(RISE_MS).setInterpolator(RowsView.glide()).start();

        going.animate().cancel();
        going.animate().translationY(-way * rise).alpha(0f)
                .setDuration(RISE_MS).setInterpolator(RowsView.glide())
                .withEndAction(new Runnable() {
                    @Override
                    public void run() {
                        going.setVisibility(GONE);
                        going.setAlpha(1f);
                        going.setTranslationY(0f);
                    }
                }).start();
    }

    /** Which way the last move went, for the rise. Written only by {@link #press}. */
    private boolean goingDown = true;

    /**
     * Every word on this page, read again.
     *
     * {@code Words.into} walks the view tree for anything that holds a key, and this screen's own
     * three parts that hold one are reached that way; the override is here because the bar's search
     * reading and the hero's two pills need the word and their focus together, which the walk alone
     * cannot give them.
     */
    @Override
    public void words(Words said) {
        bar.words(said);
        hero.words(said);
        empty.words(said);
        setSays(place.says(said));
    }
}
