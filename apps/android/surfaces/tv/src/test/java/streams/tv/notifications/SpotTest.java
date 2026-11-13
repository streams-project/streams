package streams.tv.notifications;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertTrue;

import java.util.ArrayList;
import java.util.List;
import org.junit.Test;
import streams.shared.Words;
import streams.tv.views.Word;

/**
 * Every row of the notifications screen's key map, pressed.
 *
 * <p>Note 210's named mitigation for the largest risk in this programme, made real for this screen:
 * "a unit test per screen that presses a scripted key sequence against the state machine and
 * asserts {@code changed()} fired exactly as often as {@code press} returned true". {@link Spot}
 * has no toolkit in it, so every one of these runs on this machine's JVM in milliseconds with no
 * device, no context and no resources. That {@code press} really is the only thing the funnel
 * redraws on is held by {@code NoticesHeardTest}, where there is a real view to press keys at.
 */
public class SpotTest {
    private static Notices.Notice notice(String id, String name, boolean unread) {
        return new Notices.Notice(id, "movie:1", name, "", "notification.coming", "2026-12-01",
                unread);
    }

    private static Notices feed(int howMany, boolean more) {
        List<Notices.Notice> items = new ArrayList<Notices.Notice>();
        for (int at = 0; at < howMany; at++) {
            items.add(notice("n" + at, "Title " + at, at == 0));
        }
        return new Notices(items, more ? "next" : "", more);
    }

    private static Words words() {
        return Reading.words();
    }

    // --- where the remote opens, and what Up and Down do ---------------------------------------

    @Test
    public void the_remote_opens_on_the_newest_notice() {
        assertEquals(0, new Spot(feed(5, false)).at());
    }

    @Test
    public void down_and_up_walk_the_list() {
        Spot spot = new Spot(feed(5, false));
        assertTrue(spot.press(Word.DOWN));
        assertTrue(spot.press(Word.DOWN));
        assertEquals(2, spot.at());
        assertTrue(spot.press(Word.UP));
        assertEquals(1, spot.at());
    }

    /**
     * The ends stop and do not wrap, and nothing moved means nothing is redrawn.
     *
     * Nothing in this surface wraps -- {@code Place.press} stops at {@code row > 0} and
     * {@code row < lastRow()} -- and a list that jumped from the last notice to the first would be
     * the only place on the television that did.
     */
    @Test
    public void the_ends_stop_and_say_nothing_moved() {
        Spot spot = new Spot(feed(2, false));
        assertFalse("there is nothing above the newest", spot.press(Word.UP));
        assertEquals(0, spot.at());
        assertTrue(spot.press(Word.DOWN));
        assertFalse("and nothing below the oldest", spot.press(Word.DOWN));
        assertEquals(1, spot.at());
    }

    @Test
    public void an_empty_feed_moves_nowhere() {
        Spot spot = new Spot(Notices.NOTHING);
        assertFalse(spot.press(Word.DOWN));
        assertFalse(spot.press(Word.UP));
        assertEquals(0, spot.at());
    }

    // --- the words that are not moves ----------------------------------------------------------

    /**
     * Left, Back and OK report nothing moved, exactly as {@code Place.press} answers false for OK.
     *
     * Leaving a page is not something that moved on it. If any of the three answered true the
     * funnel would redraw a screen the viewer is walking away from.
     */
    @Test
    public void leaving_and_choosing_are_not_moves() {
        Spot spot = new Spot(feed(5, false));
        assertFalse(spot.press(Word.LEFT));
        assertFalse(spot.press(Word.BACK));
        assertFalse(spot.press(Word.OK));
        assertFalse("there is no second column, so Right does nothing either",
                spot.press(Word.RIGHT));
        assertEquals("and none of the four moved the focus", 0, spot.at());
    }

    /**
     * Left leaves, which is this surface's own answer for the left edge of a page that is not a row
     * screen: {@code settings/Settings.kt:257-258}. A one column list is all left edge.
     */
    @Test
    public void left_and_back_leave_and_right_does_not() {
        Spot spot = new Spot(feed(5, false));
        assertTrue(spot.leaves(Word.LEFT));
        assertTrue(spot.leaves(Word.BACK));
        assertFalse(spot.leaves(Word.RIGHT));
        assertFalse(spot.leaves(Word.UP));
        assertFalse(spot.leaves(Word.DOWN));
        assertFalse(spot.leaves(Word.OK));
    }

    // --- what OK chooses -----------------------------------------------------------------------

    @Test
    public void ok_chooses_the_notice_under_the_remote() {
        Spot spot = new Spot(feed(5, false));
        spot.press(Word.DOWN);
        spot.press(Word.DOWN);
        assertEquals(Spot.Chose.NOTICE, spot.choose());
        assertEquals("n2", spot.chosen().id);
    }

    @Test
    public void ok_on_an_empty_feed_chooses_nothing() {
        Spot spot = new Spot(Notices.NOTHING);
        assertEquals(Spot.Chose.NOTHING, spot.choose());
        assertEquals(null, spot.chosen());
    }

    // --- focus on return -----------------------------------------------------------------------

    /**
     * The one behaviour here a reader would not think to check, and the reason the focus is kept by
     * id and not by number.
     *
     * The feed is the one list in this app that grows at the <em>top</em>. A notice arriving while
     * the screen is open pushes everything down one, and a focus kept by number would slide onto
     * the notice above the one the viewer was reading, silently, between two key presses.
     */
    @Test
    public void a_notice_arriving_at_the_top_does_not_move_the_focus() {
        Spot spot = new Spot(feed(3, false));
        spot.press(Word.DOWN);
        assertEquals("n1", spot.chosen().id);

        List<Notices.Notice> grown = new ArrayList<Notices.Notice>();
        grown.add(notice("fresh", "Just in", true));
        grown.addAll(feed(3, false).items);
        spot.arrived(new Notices(grown, "", false));

        assertEquals("the focus followed the notice, not the place", "n1", spot.chosen().id);
        assertEquals(2, spot.at());
    }

    @Test
    public void a_notice_that_has_gone_leaves_the_focus_inside_the_list() {
        Spot spot = new Spot(feed(4, false));
        spot.press(Word.DOWN);
        spot.press(Word.DOWN);
        spot.press(Word.DOWN);
        assertEquals(3, spot.at());
        spot.arrived(feed(2, false));
        assertEquals("clamped to the end of the shorter list", 1, spot.at());
        assertEquals("n1", spot.chosen().id);
    }

    @Test
    public void a_fresh_feed_is_the_one_the_screen_draws() {
        Spot spot = new Spot(feed(2, false));
        Notices later = feed(9, true);
        spot.arrived(later);
        assertSame(later, spot.feed());
    }

    // --- the list sliding under the head -------------------------------------------------------

    /**
     * The focus walks down to the last line wholly on the screen and the list scrolls from there,
     * which is {@code settings/Settings.kt:238}'s shape.
     */
    @Test
    public void the_list_holds_still_until_the_focus_reaches_the_last_line_on_screen() {
        Spot spot = new Spot(feed(9, false));
        for (int at = 0; at < Spot.VISIBLE; at++) {
            assertEquals("line " + at + " is on screen already", 0, spot.scrolled());
            spot.press(Word.DOWN);
        }
        assertEquals("the first press past the last visible line slides it one", 1, spot.scrolled());
        spot.press(Word.DOWN);
        assertEquals(2, spot.scrolled());
        spot.press(Word.UP);
        assertEquals("and it slides back", 1, spot.scrolled());
    }

    /**
     * How many lines fit is a derivation and not a number written down: the canvas less the head
     * and the gap, over the still plus the gap.
     *
     * {@code assertEquals(3, VISIBLE)} would pass forever. This holds the field against the
     * arithmetic it is made of, so a change to the still, the gap or the head moves both sides.
     */
    @Test
    public void how_many_lines_fit_is_the_canvas_divided_by_the_pitch() {
        assertEquals(Spot.STILL_HIGH + Spot.GAP, Spot.STEP);
        assertEquals(Spot.HEAD + Spot.GAP, Spot.FIRST);
        assertEquals((streams.tv.views.Ref.H - Spot.FIRST) / Spot.STEP, Spot.VISIBLE);
        assertEquals("the screen reads ahead by one screenful", Spot.VISIBLE, Spot.REACH);
    }

    /**
     * And the still and the gap are the reference's own measurements, read out of the file the code
     * cites rather than copied into this test.
     */
    @Test
    public void the_still_and_the_gap_are_the_measured_ones() {
        String measured = Reading.text("reference/netflix-on-the-tv.md");
        assertTrue("the episode card's still is " + Spot.STILL_WIDE + " x " + Spot.STILL_HIGH,
                measured.contains("**" + Spot.STILL_WIDE + " x " + Spot.STILL_HIGH + " (16:9)**"));
        assertTrue("the measured gap between a picture and the words under it",
                measured.contains("gap " + Spot.GAP + ", facts line"));
    }

    // --- reading ahead -------------------------------------------------------------------------

    @Test
    public void a_screen_with_nothing_on_it_asks_for_a_page() {
        assertTrue(new Spot(Notices.NOTHING).wantsMore());
    }

    @Test
    public void a_feed_the_server_says_is_whole_asks_for_nothing() {
        Spot spot = new Spot(feed(9, false));
        for (int at = 0; at < 8; at++) {
            spot.press(Word.DOWN);
        }
        assertFalse(spot.wantsMore());
    }

    @Test
    public void the_next_page_is_asked_for_a_screenful_before_the_end() {
        Spot spot = new Spot(feed(9, true));
        int last = spot.feed().last();
        assertFalse("not from the top of nine", spot.wantsMore());
        for (int at = 0; at < last - Spot.REACH - 1; at++) {
            spot.press(Word.DOWN);
        }
        assertFalse("still one line short", spot.wantsMore());
        spot.press(Word.DOWN);
        assertEquals(last - Spot.REACH, spot.at());
        assertTrue("within a screenful of the end", spot.wantsMore());
    }

    // --- what a reader is handed ----------------------------------------------------------------

    /**
     * Four things in the order a person wants them, in the words the server really ships.
     *
     * The words come from {@code strings/en.json} and not from a map written here: a test with its
     * own words proves the screen can read a map and proves nothing about whether the key it asks
     * for exists.
     */
    @Test
    public void a_reader_is_handed_the_kind_the_name_the_day_and_whether_it_is_read() {
        Spot spot = new Spot(feed(3, false));
        assertEquals("Coming Soon\nTitle 0\n2026-12-01\nNot read yet", spot.says(words()));
        spot.press(Word.DOWN);
        assertEquals("and the one it moved to, which has been read",
                "Coming Soon\nTitle 1\n2026-12-01", spot.says(words()));
    }

    @Test
    public void a_reader_arriving_on_an_empty_screen_is_handed_its_name_and_that_it_is_empty() {
        assertEquals("Notifications\nNo notifications yet.",
                new Spot(Notices.NOTHING).says(words()));
    }

    /**
     * A run that has not read its language yet hands a reader nothing, rather than English or a
     * key. {@code Words.of}'s rule, and the reason {@code Says.kt:46-52} exists.
     */
    @Test
    public void before_a_language_is_read_a_reader_is_handed_only_what_the_server_sent() {
        Spot spot = new Spot(feed(1, false));
        assertEquals("Title 0\n2026-12-01", spot.says(new Words()));
    }

    /** And an unknown kind drops out of the reading instead of saying its wire value. */
    @Test
    public void a_kind_this_build_has_no_word_for_is_not_read_out() {
        List<Notices.Notice> items = new ArrayList<Notices.Notice>();
        items.add(new Notices.Notice("n0", "movie:1", "Title 0", "", "", "2026-12-01", false));
        Spot spot = new Spot(new Notices(items, "", false));
        assertEquals("Title 0\n2026-12-01", spot.says(words()));
    }
}
