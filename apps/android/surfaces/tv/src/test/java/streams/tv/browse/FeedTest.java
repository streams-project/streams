package streams.tv.browse;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

import java.util.ArrayList;
import java.util.Arrays;
import org.junit.Test;

/**
 * What the flat model answers at its edges.
 *
 * {@link Feed} is the type that exists because Java cannot read the contract, so the thing worth
 * checking about it is the part a screen would otherwise have to check for itself: what it answers
 * past the end of a row, past the end of a page, and for a page that has nothing at all. Every one
 * of those is a state a browse page is really in -- the first seconds of a run, a page whose answer
 * belongs to the tab before, the Games tab -- and in Compose each was a {@code getOrNull} at the
 * call site. Here it is answered once, so a screen that asks for the fourteenth row of a row of
 * three gets null rather than a crash.
 */
public class FeedTest {
    private static Feed.Card card(String key) {
        return new Feed.Card(key, key + " name", "/tall.jpg", "/wide.jpg", new String[] {"New"});
    }

    private static Feed.Row row(String id, String... keys) {
        ArrayList<Feed.Card> cards = new ArrayList<Feed.Card>();
        for (int at = 0; at < keys.length; at++) {
            cards.add(card(keys[at]));
        }
        return new Feed.Row(id, "Trending Now", cards);
    }

    /** A row hands out its title keys in the order it draws them, which is what the memory keys on. */
    @Test
    public void a_row_hands_out_its_keys_in_order() {
        assertArrayEquals(new String[] {"movie:1", "tv:2", "movie:3"},
                row("r1", "movie:1", "tv:2", "movie:3").keys());
        assertArrayEquals(new String[0], row("empty").keys());
    }

    /** Asking past either end of a row answers null and not an exception. */
    @Test
    public void asking_past_the_end_of_a_row_answers_nothing() {
        Feed.Row one = row("r1", "movie:1", "movie:2");
        assertEquals("movie:1", one.at(0).key);
        assertEquals("movie:2", one.at(1).key);
        assertNull(one.at(2));
        assertNull(one.at(-1));
        assertNull(one.at(Integer.MAX_VALUE));
    }

    /** And past either end of a page. */
    @Test
    public void asking_past_the_end_of_a_page_answers_nothing() {
        Feed feed = new Feed(new ArrayList<Feed.Tab>(), 0, null, "", Arrays.asList(row("r1", "movie:1")),
                null, "");
        assertEquals("r1", feed.row(0).id);
        assertNull(feed.row(1));
        assertNull(feed.row(-1));
        assertEquals(0, feed.lastRow());
        assertNull(feed.tab(0));
    }

    /** A page with no rows has a last row of -1, which is what the read-ahead and DOWN both read. */
    @Test
    public void a_page_with_no_rows_has_no_last_row() {
        assertEquals(-1, Feed.NOTHING.lastRow());
        assertTrue(Feed.NOTHING.rows.isEmpty());
        assertTrue(Feed.NOTHING.tabs.isEmpty());
        assertNull(Feed.NOTHING.hero);
        assertNull(Feed.NOTHING.who);
        assertEquals("", Feed.NOTHING.server);
        assertEquals("", Feed.NOTHING.empty);
    }

    /**
     * Nothing in this model is ever null where a string is expected.
     *
     * The reason is the one {@code Says} and {@code Words} both give: a part handed null draws
     * nothing and says nothing, which is what is wanted, and a part handed null that then calls
     * {@code startsWith} on it is a crash on the first row of the first page. So every string is
     * emptied at the door, once.
     */
    @Test
    public void nothing_in_the_model_is_null_where_a_string_belongs() {
        Feed.Card bare = new Feed.Card(null, null, null, null, null);
        assertEquals("", bare.key);
        assertEquals("", bare.name);
        assertEquals("", bare.tall);
        assertEquals("", bare.wide);
        assertEquals(0, bare.badges.length);
        assertFalse("and a card with no key is not a series", bare.series());

        Feed.Row none = new Feed.Row(null, null, null);
        assertEquals("", none.id);
        assertEquals("", none.title);
        assertTrue(none.cards.isEmpty());

        Feed.Tab tab = new Feed.Tab(null, null);
        assertEquals("", tab.label);
        assertEquals("", tab.screen);

        Feed.Who who = new Feed.Who(null, null, false);
        assertEquals("", who.name);
        assertEquals("", who.picture);

        Feed.Story told = new Feed.Story(null, null, null);
        assertEquals(0, told.facts.length);
        assertEquals("", told.overview);
        assertEquals("", told.logo);
    }

    /** The two negative places on the bar are not row numbers and never collide with one. */
    @Test
    public void the_bars_two_negative_places_are_not_row_numbers() {
        assertTrue(Feed.SEARCH_TAB < 0);
        assertTrue(Feed.WHO_TAB < Feed.SEARCH_TAB);
        Feed feed = new Feed(Arrays.asList(new Feed.Tab("Home", "home")), 0, null, "",
                new ArrayList<Feed.Row>(), null, "");
        assertNull("no tab sits at a negative place", feed.tab(Feed.SEARCH_TAB));
        assertNull(feed.tab(Feed.WHO_TAB));
    }
}
