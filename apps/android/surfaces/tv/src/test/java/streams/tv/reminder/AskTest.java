package streams.tv.reminder;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import org.junit.Test;
import streams.shared.Words;
import streams.tv.notifications.Reading;
import streams.tv.views.Word;

/**
 * Every row of the reminder screen's key map, pressed.
 *
 * <p>{@link Ask} has no toolkit in it, so all of this runs on this machine's JVM with no device.
 * That the screen really redraws only when {@code press} answered true is held by
 * {@code ReminderHeardTest}, where there is a view to press keys at.
 *
 * <p>The words come from {@code strings/en.json} through {@code Reading.words()} and not from a map
 * written here: a test with its own words proves the screen can read a map and proves nothing about
 * whether the key it asks for exists.
 */
public class AskTest {
    private static final Coming DUNE =
            new Coming("movie:7", "Dune", "http://box/p7.jpg", "2026-12-25", false);

    private static Words words() {
        return Reading.words();
    }

    // --- which line the remote opens on ---------------------------------------------------------

    /**
     * The second, the one that leaves, which is {@code settings/Settings.kt:234}'s
     * {@code confirmAt = 1}.
     *
     * A panel that opens on the line that does the thing turns one stray OK into an action nobody
     * asked for. This surface already decided that for its other two line panel.
     */
    @Test
    public void the_remote_opens_on_the_line_that_leaves() {
        assertEquals(Ask.CLOSE, new Ask(DUNE).at());
        assertEquals(Ask.Chose.CLOSE, new Ask(DUNE).choose());
    }

    @Test
    public void up_and_down_walk_the_two_lines() {
        Ask ask = new Ask(DUNE);
        assertTrue(ask.press(Word.UP));
        assertEquals(Ask.TURN, ask.at());
        assertTrue(ask.press(Word.DOWN));
        assertEquals(Ask.CLOSE, ask.at());
    }

    @Test
    public void the_ends_stop_and_say_nothing_moved() {
        Ask ask = new Ask(DUNE);
        assertFalse("there is nothing below the second line", ask.press(Word.DOWN));
        assertEquals(Ask.CLOSE, ask.at());
        ask.press(Word.UP);
        assertFalse("nor above the first", ask.press(Word.UP));
        assertEquals(Ask.TURN, ask.at());
    }

    @Test
    public void leaving_and_choosing_are_not_moves() {
        Ask ask = new Ask(DUNE);
        assertFalse(ask.press(Word.LEFT));
        assertFalse(ask.press(Word.BACK));
        assertFalse(ask.press(Word.OK));
        assertFalse("there is no second column", ask.press(Word.RIGHT));
        assertEquals(Ask.CLOSE, ask.at());
    }

    @Test
    public void left_and_back_leave_and_nothing_else_does() {
        Ask ask = new Ask(DUNE);
        assertTrue(ask.leaves(Word.LEFT));
        assertTrue(ask.leaves(Word.BACK));
        assertFalse(ask.leaves(Word.RIGHT));
        assertFalse(ask.leaves(Word.UP));
        assertFalse(ask.leaves(Word.DOWN));
        assertFalse(ask.leaves(Word.OK));
    }

    /**
     * The whole map is the settings confirm panel's, read off that file rather than described here.
     *
     * Two statements of one key map drift. This holds the four branches of
     * {@code settings/Settings.kt}'s confirm against the four here, so a change to the surface's
     * grammar for a two line panel that is not made to this screen fails.
     */
    @Test
    public void the_map_is_the_one_the_settings_confirm_panel_already_uses() {
        String kotlin = Reading.text(
                "apps/android/surfaces/tv/src/main/kotlin/streams/tv/settings/Settings.kt");
        int from = kotlin.indexOf("val asking = pendingConfirm");
        assertTrue("the settings confirm panel has gone", from > 0);
        String panel = kotlin.substring(from, from + 420);
        assertTrue("Up walks towards the first line",
                panel.contains("Word.UP -> if (confirmAt > 0) confirmAt -= 1"));
        assertTrue("Down walks towards the second",
                panel.contains("Word.DOWN -> if (confirmAt < 1) confirmAt += 1"));
        assertTrue("OK on the second line leaves",
                panel.contains("Word.OK -> if (confirmAt == 0)"));
        assertTrue("and Back and Left both dismiss",
                panel.contains("Word.BACK, Word.LEFT -> pendingConfirm = null"));
    }

    // --- what OK chooses ------------------------------------------------------------------------

    @Test
    public void ok_on_the_first_line_turns_the_reminder_and_on_the_second_leaves() {
        Ask ask = new Ask(DUNE);
        assertEquals(Ask.Chose.CLOSE, ask.choose());
        ask.press(Word.UP);
        assertEquals(Ask.Chose.TURN, ask.choose());
        assertTrue("with no reminder set, OK sets one", ask.turningOn());
    }

    @Test
    public void a_title_already_reminded_turns_the_other_way() {
        Ask ask = new Ask(DUNE.turned(true));
        ask.press(Word.UP);
        assertEquals(Ask.Chose.TURN, ask.choose());
        assertFalse(ask.turningOn());
        assertEquals("remind.set", ask.turnKey());
    }

    @Test
    public void a_title_this_screen_knows_nothing_about_offers_nothing() {
        Ask ask = new Ask(Coming.NOTHING);
        ask.press(Word.UP);
        assertEquals(Ask.Chose.NOTHING, ask.choose());
        assertEquals("", ask.says(words()));
    }

    /**
     * A title that has already arrived is not something to be reminded of, which is the condition
     * the phone's button hangs on at {@code Title.kt:300}.
     */
    @Test
    public void a_title_that_is_already_here_offers_nothing() {
        Ask ask = new Ask(new Coming("movie:7", "Dune", "", "", false));
        ask.press(Word.UP);
        assertEquals(Ask.Chose.NOTHING, ask.choose());
    }

    // --- the words ------------------------------------------------------------------------------

    @Test
    public void the_line_says_what_pressing_ok_would_leave_behind() {
        Ask ask = new Ask(DUNE);
        assertEquals("Remind Me", words().of(ask.turnKey()));
        assertEquals("Reminder Set", words().of(new Ask(DUNE.turned(true)).turnKey()));
        assertEquals("Cancel", words().of(ask.closeKey()));
    }

    /**
     * The day is filled through {@code shared/Theme.kt:66}'s {@code fill}, which wraps it in the
     * bidirectional isolates U+2068 and U+2069.
     *
     * Not decoration: a date dropped into an Arabic sentence without them comes out in the wrong
     * order on screen. The isolates are asserted here because a {@code String.replace} written in
     * this package instead would pass every other test in this file.
     */
    @Test
    public void when_it_arrives_is_filled_in_the_way_every_other_counted_word_is() {
        assertEquals("Coming on ⁨2026-12-25⁩", new Ask(DUNE).comingWords(words()));
    }

    @Test
    public void a_run_that_has_not_read_its_language_draws_no_line_rather_than_a_bare_date() {
        assertEquals("", new Ask(DUNE).comingWords(new Words()));
        assertEquals("", new Ask(new Coming("movie:7", "Dune", "", "", false))
                .comingWords(words()));
    }

    // --- what a reader is handed ------------------------------------------------------------------

    /**
     * The title is repeated at both stops on purpose: a reader that moved from one line to the
     * other and heard only "Cancel" would have lost what was being cancelled.
     */
    @Test
    public void a_reader_is_handed_the_title_when_it_arrives_and_the_line_the_remote_is_on() {
        Ask ask = new Ask(DUNE);
        assertEquals("Dune\nComing on ⁨2026-12-25⁩\nCancel", ask.says(words()));
        ask.press(Word.UP);
        assertEquals("Dune\nComing on ⁨2026-12-25⁩\nRemind Me", ask.says(words()));
    }

    @Test
    public void a_reader_hears_the_reminder_is_set_once_it_is() {
        Ask ask = new Ask(DUNE);
        ask.press(Word.UP);
        ask.arrived(ask.title().turned(true));
        assertEquals("Dune\nComing on ⁨2026-12-25⁩\nReminder Set", ask.says(words()));
        assertEquals("and the focus did not move under the viewer's hand", Ask.TURN, ask.at());
    }
}
