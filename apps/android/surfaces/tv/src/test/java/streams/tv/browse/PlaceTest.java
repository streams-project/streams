package streams.tv.browse;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertTrue;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.junit.Test;
import streams.tv.views.Word;
import streams.tv.views.Words;

/**
 * Every row of note 187's key map that belongs to a page of rows, pressed.
 *
 * This is note 210's own named mitigation for the largest risk in the programme, made real. Its
 * risk table says: "A missed {@code invalidate()}. 164 snapshot state sites become explicit fields;
 * a forgotten redraw is a screen that silently does not move... Only found by pressing the key on a
 * device, on the one screen, in the one state. The build is green and the tests pass." And its
 * acceptance check: "The single funnel: {@code press} is the only writer and the only caller of
 * {@code changed()}. A unit test per screen that presses a scripted key sequence against the state
 * machine and asserts {@code changed()} fired exactly as often as {@code press} returned true. This
 * is testable precisely because {@code PageFocus} has no toolkit in it."
 *
 * {@link Place} has no toolkit in it either, so every one of these runs on this machine's JVM in
 * milliseconds with no device, no context and no resources.
 */
public class PlaceTest {
    private static Feed.Card card(String key, String name) {
        return new Feed.Card(key, name, "", "", new String[0]);
    }

    private static Feed.Row row(String id, String title, String... keys) {
        List<Feed.Card> cards = new ArrayList<>();
        for (int at = 0; at < keys.length; at++) {
            cards.add(card(keys[at], keys[at] + " name"));
        }
        return new Feed.Row(id, title, cards);
    }

    private static Feed feed(Feed.Card hero, Feed.Row... rows) {
        List<Feed.Tab> tabs = Arrays.asList(
                new Feed.Tab("Home", "home"),
                new Feed.Tab("Series", "series"),
                new Feed.Tab("Films", "films"));
        return new Feed(tabs, 0, new Feed.Who("Ibrahim", "", false), "http://box",
                Arrays.asList(rows), hero, "");
    }

    private static Place onRows() {
        return new Place(feed(null,
                row("r1", "Trending Now", "movie:1", "movie:2", "movie:3"),
                row("r2", "Because you watched", "tv:4", "movie:5"),
                row("r3", "New", "movie:6")), 0);
    }

    private static Place withHero() {
        return new Place(feed(card("tv:9", "Stranger Things"),
                row("r1", "Trending Now", "movie:1", "movie:2")), 0);
    }

    // --- note 187: a page keeps the line or card it opened -------------------------------------

    /**
     * The defect note 187 records as Android {@code fd7b104}, in the owner's words from the
     * simulator: after scrolling a row and moving up or down, the row showed from its first tile as
     * if it had never been scrolled.
     *
     * This is the test that the fix came across the port. It is the one behaviour in this file that
     * a reader would not think to check, because nothing about it is visible until the second visit.
     */
    @Test
    public void a_row_comes_back_to_the_tile_it_was_left_on() {
        Place place = onRows();
        assertEquals(0, place.column());
        assertTrue(place.press(Word.RIGHT));
        assertTrue(place.press(Word.RIGHT));
        assertEquals(2, place.column());

        assertTrue(place.press(Word.DOWN));
        assertEquals(1, place.row());
        assertEquals("a row the viewer has not been on opens on its first tile", 0, place.column());

        assertTrue(place.press(Word.UP));
        assertEquals(0, place.row());
        assertEquals("the row has to open where it was left", 2, place.column());
    }

    /**
     * And the row below shows where it was left too, which is the half of the same fix that is
     * visible without moving: {@code Browse.kt:298} draws the dim row at {@code columnOf(below)}.
     */
    @Test
    public void the_row_below_shows_where_it_was_left() {
        Place place = onRows();
        place.press(Word.DOWN);
        place.press(Word.RIGHT);
        assertEquals(1, place.column());
        place.press(Word.UP);
        assertEquals("the row below is drawn from its kept place", 1, place.columnOf(1));
    }

    /**
     * A fresh answer that reordered a row follows the title and not the number.
     *
     * Note 187's "found while checking": returning to a page fetches it again and the server may
     * reorder its rows and the tiles inside them, so a place kept by number alone opens a row on a
     * different title. {@link streams.tv.views.RowMemory} is what answers this, and this asserts
     * that {@link Place} really keeps the place under the row's own id so that it can.
     */
    @Test
    public void a_reordered_row_follows_the_title() {
        Place place = onRows();
        place.press(Word.RIGHT);
        place.press(Word.RIGHT);
        assertEquals("movie:3", place.open().key);

        // The same row, same id, with the titles shuffled: what was third is now first.
        place.arrived(feed(null,
                row("r1", "Trending Now", "movie:3", "movie:1", "movie:2"),
                row("r2", "Because you watched", "tv:4", "movie:5"),
                row("r3", "New", "movie:6")));
        assertEquals("the kept title moved, so the place follows it", 0, place.column());
        assertEquals("movie:3", place.open().key);
    }

    // --- note 187: Back walks out one level, and only root pages leave the app ------------------

    /**
     * Back walks out one level at a time: out of the rows to the bar, and then no further.
     * {@code Browse.kt:140-147}.
     */
    @Test
    public void back_walks_out_one_level_at_a_time() {
        Place place = onRows();
        assertEquals(Place.Where.ROW, place.where());
        assertTrue(place.press(Word.BACK));
        assertEquals(Place.Where.TOP, place.where());
        assertFalse("with nowhere left to go the page has to let Back through", place.press(Word.BACK));
    }

    /** With a hero, Back stops on it on the way out, because the hero is the level above the rows. */
    @Test
    public void back_stops_on_the_hero_on_the_way_out() {
        Place place = withHero();
        place.press(Word.DOWN);
        assertEquals(Place.Where.ROW, place.where());
        assertTrue(place.press(Word.BACK));
        assertEquals(Place.Where.HERO, place.where());
        assertTrue(place.press(Word.BACK));
        assertEquals(Place.Where.TOP, place.where());
        assertFalse(place.press(Word.BACK));
    }

    /**
     * Back on the top bar brings the focus home to this page's own tab first, and only then leaves
     * the page. Note 187's own row for Browse, and {@code Browse.kt:143-146}.
     */
    @Test
    public void back_on_the_bar_comes_home_to_this_pages_tab_first() {
        Place place = onRows();
        place.press(Word.BACK);
        assertEquals(Place.Where.TOP, place.where());
        assertTrue(place.press(Word.RIGHT));
        assertEquals("the focus wandered along the bar", 1, place.tab());
        assertTrue("Back has to come home before it leaves", place.press(Word.BACK));
        assertEquals(place.active(), place.tab());
        assertFalse("and only then let Back through", place.press(Word.BACK));
    }

    // --- note 187: Left at a left edge, and the bar's two negative places -----------------------

    /**
     * Going left along a row goes back along it, one tile at a time.
     *
     * Added after a mutation on 2026-11-12 found that nothing covered it: replacing
     * {@code keep(column() - 1)} with {@code keep(column())} in the LEFT branch left all 151 tests
     * green. Every other test moved along a row rightwards only, and the funnel script never
     * reached a LEFT with a tile to its left, so a row that could be scrolled forward and not back
     * would have shipped. This is the test that now fails for it.
     */
    @Test
    public void going_left_along_a_row_goes_back_along_it() {
        Place place = onRows();
        assertTrue(place.press(Word.RIGHT));
        assertTrue(place.press(Word.RIGHT));
        assertEquals(2, place.column());
        assertEquals("movie:3", place.open().key);
        assertTrue(place.press(Word.LEFT));
        assertEquals("one tile back, not none and not two", 1, place.column());
        assertEquals("movie:2", place.open().key);
        assertTrue(place.press(Word.LEFT));
        assertEquals(0, place.column());
        assertEquals("movie:1", place.open().key);
        assertFalse("and then it is at its edge", place.press(Word.LEFT));
        assertEquals(0, place.column());
    }

    /**
     * Left at the left edge of a row does not leave the page: this page's answer at its edge is to
     * refuse, which is what lets Back be the one way out of it. {@code Browse.kt:129}.
     */
    @Test
    public void left_at_the_left_edge_of_a_row_is_refused() {
        Place place = onRows();
        assertEquals(0, place.column());
        assertFalse(place.press(Word.LEFT));
        assertEquals(0, place.column());
    }

    /**
     * The bar runs leftwards past its first tab onto search and then onto the profile, and back
     * again, and stops at each end.
     *
     * The order of the branches is the behaviour here: {@code tab > 0} is tried before
     * {@code tab == 0}, so the two negative places are reached by falling past it
     * ({@code Browse.kt:124-139}). A reordered {@code when} wraps the bar the wrong way at its ends,
     * which is the one way this could be ported wrongly and still look right.
     */
    @Test
    public void the_bar_runs_onto_search_and_the_profile_and_stops() {
        Place place = onRows();
        place.press(Word.BACK);
        assertEquals(0, place.tab());
        assertTrue(place.press(Word.LEFT));
        assertEquals(Feed.SEARCH_TAB, place.tab());
        assertTrue(place.press(Word.LEFT));
        assertEquals(Feed.WHO_TAB, place.tab());
        assertFalse("the profile is the left end of the bar", place.press(Word.LEFT));

        assertTrue(place.press(Word.RIGHT));
        assertEquals(Feed.SEARCH_TAB, place.tab());
        assertTrue(place.press(Word.RIGHT));
        assertEquals(0, place.tab());
        assertTrue(place.press(Word.RIGHT));
        assertEquals(1, place.tab());
        assertTrue(place.press(Word.RIGHT));
        assertEquals(2, place.tab());
        assertFalse("and the last tab is the right end", place.press(Word.RIGHT));
    }

    // --- the funnel -----------------------------------------------------------------------------

    /**
     * {@code press} answers true exactly when something moved, and false exactly when nothing did.
     *
     * This is note 210's acceptance check for the whole redraw risk, stated as it states it: a
     * scripted key sequence, with the number of moves counted against the number of times
     * {@code press} answered true. {@code Funnel.key} calls {@code changed()} on true and on nothing
     * else ({@code Funnel.java:97-101}), so a true with no move is a redraw for nothing and a false
     * with a move is a screen that silently does not move -- the defect itself.
     */
    @Test
    public void press_answers_true_exactly_when_something_moved() {
        Place place = onRows();
        // Every branch of `press` is reached at least once, in both directions on both axes, and
        // then run past each of the four edges. The two LEFT presses after the three RIGHTs are
        // there because a mutation proved a script without them covered nothing: see
        // `going_left_along_a_row_goes_back_along_it`.
        Word[] script = {
            Word.RIGHT, Word.RIGHT, Word.RIGHT, Word.RIGHT, Word.LEFT, Word.LEFT, Word.LEFT,
            Word.RIGHT, Word.DOWN, Word.DOWN, Word.DOWN, Word.UP, Word.LEFT, Word.BACK,
            Word.RIGHT, Word.RIGHT, Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT,
            Word.BACK, Word.BACK, Word.BACK, Word.OK,
        };
        int used = 0;
        int moves = 0;
        for (int at = 0; at < script.length; at++) {
            String was = state(place);
            boolean answered = place.press(script[at]);
            boolean moved = !state(place).equals(was);
            if (answered) {
                used++;
            }
            if (moved) {
                moves++;
            }
            assertEquals(script[at] + " at step " + at + ": answered " + answered
                    + " and the state " + (moved ? "moved" : "did not move")
                    + " (" + was + " -> " + state(place) + ")", moved, answered);
        }
        assertEquals("the script has to move the page, or it is testing nothing", used, moves);
        assertTrue("and it has to move it more than once", moves > 5);
    }

    /** Everything this page's state is, as one string, for the funnel test to compare. */
    private static String state(Place place) {
        return place.where() + "/" + place.row() + "/" + place.column() + "/" + place.tab()
                + "/" + place.button();
    }

    /**
     * OK is not a move. {@code Browse.kt:148} answers false for it inside the state machine and
     * {@code :232-236} answers it separately, because choosing leaves the page: reporting it as a
     * move would redraw a page that is going away.
     */
    @Test
    public void ok_is_not_a_move() {
        Place place = onRows();
        String was = state(place);
        assertFalse(place.press(Word.OK));
        assertEquals(was, state(place));
    }

    // --- what OK chooses ------------------------------------------------------------------------

    /** The five things OK can choose from a page of rows. {@code Browse.kt:219-229}. */
    @Test
    public void ok_chooses_the_right_thing_from_each_stop() {
        Place place = onRows();
        assertEquals(Place.Chose.TITLE, place.choose());
        assertEquals("movie:1", place.chosenCard().key);

        place.press(Word.BACK);
        assertEquals(Place.Chose.TAB, place.choose());
        assertEquals("home", place.chosenTab().screen);

        place.press(Word.LEFT);
        assertEquals(Place.Chose.SEARCH, place.choose());
        assertNull(place.chosenTab());

        place.press(Word.LEFT);
        assertEquals(Place.Chose.MENU, place.choose());
    }

    /**
     * The hero's two buttons, and the one condition that decides between them.
     *
     * {@code Browse.kt:227}: More info opens the title's page, and so does Play on a series,
     * because a series has episodes to choose from. Play on anything else starts playing. The test
     * for a series is {@code title.value.startsWith("tv:")} at {@code shared/Addresses.kt:17}.
     */
    @Test
    public void the_heros_play_opens_a_series_and_plays_a_film() {
        Place series = withHero();
        assertEquals(Place.Where.HERO, series.where());
        assertEquals(0, series.button());
        assertEquals("Play on a series opens its page", Place.Chose.TITLE, series.choose());
        assertTrue(series.press(Word.RIGHT));
        assertEquals(1, series.button());
        assertEquals(Place.Chose.TITLE, series.choose());

        Place film = new Place(feed(card("movie:9", "Heat"), row("r1", "Row", "movie:1")), 0);
        assertEquals("and Play on a film plays it", Place.Chose.WATCH, film.choose());
        assertTrue(film.press(Word.RIGHT));
        assertEquals("while More info still opens its page", Place.Chose.TITLE, film.choose());
    }

    /** A page with no hero and no rows has nowhere for OK to go, and says so rather than guessing. */
    @Test
    public void a_page_with_nothing_on_it_chooses_nothing() {
        Place place = new Place(new Feed(new ArrayList<Feed.Tab>(), -1, null, "",
                new ArrayList<Feed.Row>(), null, "mine.empty"), -1);
        assertEquals(Place.Where.TOP, place.where());
        assertEquals(Place.Chose.NOTHING, place.choose());
        assertFalse("and no key moves it", place.press(Word.DOWN));
        assertFalse(place.press(Word.UP));
    }

    // --- arriving and the page above ------------------------------------------------------------

    /**
     * Where a page opens. {@code Browse.kt:78}: the hero or the bar when there is a hero or there
     * are no rows, and the first row otherwise.
     */
    @Test
    public void a_page_opens_where_it_has_something_to_open_on() {
        assertEquals(Place.Where.ROW, onRows().where());
        assertEquals(Place.Where.HERO, withHero().where());
        Place bare = new Place(feed(null), 0);
        assertEquals(Place.Where.TOP, bare.where());
    }

    /**
     * A fresh answer replaces the rows and keeps the focus. {@code Browse.kt:179-181}.
     *
     * Including the case a reader would get wrong: a page that shrank keeps its row number, and the
     * row is read through {@link Feed#row(int)}, which answers null, so the page draws nothing there
     * rather than crashing and finds the viewer where they were when it grows back.
     */
    @Test
    public void a_fresh_answer_keeps_the_focus_and_survives_a_page_that_shrank() {
        Place place = onRows();
        place.press(Word.DOWN);
        place.press(Word.DOWN);
        assertEquals(2, place.row());

        place.arrived(feed(null, row("r1", "Trending Now", "movie:1")));
        assertEquals("the focus is kept, not clamped", 2, place.row());
        assertNull("and the row it points at is simply not there", place.feed().row(2));
        assertEquals(0, place.columnOf(2));
        assertFalse("nothing below it to move to", place.press(Word.DOWN));

        place.arrived(feed(null,
                row("r1", "a", "movie:1"), row("r2", "b", "movie:2"), row("r3", "c", "movie:3")));
        assertEquals("and it is still there when the page grows back", 2, place.row());
    }

    /** A page that gains a hero gains a level above its rows, which is what {@code above} is. */
    @Test
    public void a_page_that_gains_a_hero_gains_a_level_above_its_rows() {
        Place place = onRows();
        assertTrue(place.press(Word.BACK));
        assertEquals(Place.Where.TOP, place.where());
        place.arrived(feed(card("tv:9", "Stranger Things"), row("r1", "Row", "movie:1")));
        assertTrue(place.press(Word.DOWN));
        assertEquals("the hero is now the first stop down", Place.Where.HERO, place.where());
        assertTrue(place.press(Word.DOWN));
        assertEquals(Place.Where.ROW, place.where());
        assertTrue(place.press(Word.UP));
        assertEquals("and the first stop up", Place.Where.HERO, place.where());
    }

    // --- reading ahead --------------------------------------------------------------------------

    /**
     * The page asks for its next rows as the focus comes within {@code REACH} of the last row it
     * holds, and a page with no rows at all asks straight away. {@code Browse.kt:203} and
     * {@code :45-53}.
     */
    @Test
    public void the_page_asks_for_more_rows_within_reach_of_its_last() {
        assertEquals("the derivation at Browse.kt:45-53", 2, Place.REACH);
        List<Feed.Row> many = new ArrayList<>();
        for (int at = 0; at < 8; at++) {
            many.add(row("r" + at, "Row " + at, "movie:" + at));
        }
        Place place = new Place(new Feed(new ArrayList<Feed.Tab>(), 0, null, "", many, null, ""), 0);
        assertFalse("row 0 of 8 is nowhere near the end", place.wantsMore());
        for (int at = 0; at < 5; at++) {
            place.press(Word.DOWN);
        }
        assertEquals(5, place.row());
        assertTrue("row 5 of 8 is within two of the last", place.wantsMore());

        Place bare = new Place(new Feed(new ArrayList<Feed.Tab>(), 0, null, "",
                new ArrayList<Feed.Row>(), null, ""), 0);
        assertTrue("a page with no rows can only grow by asking", bare.wantsMore());
    }

    // --- what a person hears --------------------------------------------------------------------

    /**
     * What a screen reader is told as the remote moves, at each stop. {@code Browse.kt:257-266}.
     *
     * A television draws its own focus, so this live region is the only thing that tells a person
     * where the remote has arrived ({@code tv/app/Spoken.kt:26-32}). The row's answer is the title's
     * name and the row's name read as one label, joined by a line end so that no language's
     * punctuation is decided in the code ({@code Says.sayBoth}).
     */
    @Test
    public void what_a_person_hears_at_each_stop() {
        Words words = new Words();
        java.util.Map<String, String> said = new java.util.HashMap<>();
        said.put("nav.search", "Search");
        said.put("common.play", "Play");
        said.put("common.more_info", "More info");
        words.read(said);

        Place place = withHero();
        assertEquals("Play", place.says(words));
        place.press(Word.RIGHT);
        assertEquals("More info", place.says(words));

        place.press(Word.DOWN);
        assertEquals("movie:1 name\nTrending Now", place.says(words));

        place.press(Word.BACK);
        place.press(Word.BACK);
        assertEquals("Home", place.says(words));
        place.press(Word.LEFT);
        assertEquals("Search", place.says(words));
        place.press(Word.LEFT);
        assertEquals("the profile says whose it is", "Ibrahim", place.says(words));
    }

    /**
     * A word the strings do not hold is heard as nothing, and never as English or as the key.
     *
     * {@code Says.kt:46-52}'s rule, and {@code Words.of}'s behaviour: {@code seen.words} is empty for
     * as long as a run has not read its language, and a part labelled with a placeholder in that
     * window is read out by a screen reader as a placeholder.
     */
    @Test
    public void a_stop_with_no_words_for_it_is_heard_as_nothing() {
        Words empty = new Words();
        Place place = withHero();
        assertEquals("", place.says(empty));
        place.press(Word.DOWN);
        place.press(Word.BACK);
        place.press(Word.BACK);
        place.press(Word.LEFT);
        assertEquals("", place.says(empty));
    }

    /** The page's feed is the one it was last given, and the focus is not part of it. */
    @Test
    public void the_feed_is_replaced_and_the_focus_is_not() {
        Place place = onRows();
        Feed fresh = feed(null, row("r1", "Trending Now", "movie:1"));
        place.arrived(fresh);
        assertSame(fresh, place.feed());
    }
}
