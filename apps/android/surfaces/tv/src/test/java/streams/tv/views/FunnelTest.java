package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import android.view.KeyEvent;
import java.util.ArrayList;
import java.util.List;
import org.junit.Test;

/**
 * The key funnel: one key in, one redraw out, and nothing else.
 *
 * This is the test note 210's risk table asks for by name. Its first row is "a missed
 * {@code invalidate()}" -- 164 snapshot state sites become plain fields, and a forgotten redraw is a
 * screen that silently does not move, found only by pressing the one key on the one screen in the
 * one state, with a green build and passing tests. The mitigation it states is a single funnel, and
 * the check it states is "a unit test per screen that presses a scripted key sequence against the
 * state machine and asserts {@code changed()} fired exactly as often as {@code press} returned
 * true". That is testable precisely because nothing in {@link Funnel} knows about drawing.
 */
public class FunnelTest {
    /** A screen that answers a scripted list of answers and records what it was asked. */
    private static final class Screen implements Funnel.Press, Funnel.Redraw {
        final List<Word> asked = new ArrayList<>();
        final boolean[] answers;
        int at;
        int redraws;

        Screen(boolean... answers) {
            this.answers = answers;
        }

        @Override
        public boolean press(Word word) {
            asked.add(word);
            return at < answers.length && answers[at++];
        }

        @Override
        public void changed() {
            redraws++;
        }
    }

    private static Funnel funnelOf(Screen screen) {
        return new Funnel(screen, screen);
    }

    /** A word the screen used is one redraw, and the key does not go on to the system. */
    @Test
    public void a_word_the_screen_used_is_one_redraw() {
        Screen screen = new Screen(true);
        assertTrue(funnelOf(screen).key(KeyEvent.KEYCODE_DPAD_DOWN, KeyEvent.ACTION_DOWN));
        assertEquals(1, screen.asked.size());
        assertEquals(Word.DOWN, screen.asked.get(0));
        assertEquals(1, screen.redraws);
    }

    /**
     * A word the screen did not use draws nothing and falls through, which is how Back leaves a page
     * and how the player's keys reach the player.
     */
    @Test
    public void a_word_the_screen_did_not_use_draws_nothing_and_falls_through() {
        Screen screen = new Screen(false);
        assertFalse(funnelOf(screen).key(KeyEvent.KEYCODE_BACK, KeyEvent.ACTION_DOWN));
        assertEquals(1, screen.asked.size());
        assertEquals(0, screen.redraws);
    }

    /**
     * The funnel's whole claim: the redraw count is the number of times the screen said something
     * moved. Not more (a redraw for a key that did nothing is a wasted frame on weak hardware) and
     * not fewer (a move with no redraw is the defect this exists to prevent).
     */
    @Test
    public void changed_fires_exactly_as_often_as_press_returned_true() {
        boolean[] answers = {true, false, true, true, false, false, true, false, true, true};
        int[] keys = {
                KeyEvent.KEYCODE_DPAD_DOWN,
                KeyEvent.KEYCODE_DPAD_DOWN,
                KeyEvent.KEYCODE_DPAD_RIGHT,
                KeyEvent.KEYCODE_DPAD_RIGHT,
                KeyEvent.KEYCODE_DPAD_LEFT,
                KeyEvent.KEYCODE_DPAD_UP,
                KeyEvent.KEYCODE_DPAD_CENTER,
                KeyEvent.KEYCODE_BACK,
                KeyEvent.KEYCODE_DPAD_DOWN,
                KeyEvent.KEYCODE_DPAD_RIGHT};
        Screen screen = new Screen(answers);
        Funnel funnel = funnelOf(screen);
        int used = 0;
        for (int key : keys) {
            if (funnel.key(key, KeyEvent.ACTION_DOWN)) {
                used++;
            }
        }
        int moved = 0;
        for (boolean answer : answers) {
            if (answer) {
                moved++;
            }
        }
        assertEquals("every key reached the screen", keys.length, screen.asked.size());
        assertEquals("the funnel said used exactly when the screen did", moved, used);
        assertEquals("and drew exactly that many times", moved, screen.redraws);
    }

    /**
     * A covered screen takes no keys and keeps its place, and takes them back after.
     *
     * {@code live = on && !LocalCovered.current} at {@code Remote.kt:76}: under the opening, or
     * under a profile menu or a PIN panel, the screen beneath keeps every field it holds and simply
     * stops being asked. The test for "keeps its place" is that the screen is never asked at all --
     * if it were asked and answered false, a page would be deciding what to do about a key it should
     * not have seen.
     */
    @Test
    public void a_covered_screen_is_not_asked_at_all_and_is_asked_again_after() {
        Screen screen = new Screen(true, true);
        Funnel funnel = funnelOf(screen);
        funnel.setLive(false);
        assertFalse(funnel.key(KeyEvent.KEYCODE_DPAD_DOWN, KeyEvent.ACTION_DOWN));
        assertEquals(0, screen.asked.size());
        assertEquals(0, screen.redraws);
        funnel.setLive(true);
        assertTrue(funnel.key(KeyEvent.KEYCODE_DPAD_DOWN, KeyEvent.ACTION_DOWN));
        assertEquals(1, screen.asked.size());
        assertEquals(1, screen.redraws);
    }

    /**
     * Only a key down moves anything, which is {@code Remote.kt:94}'s
     * {@code event.type == KeyEventType.KeyDown}.
     *
     * Without it every press moves twice: once on the way down and once on the way up, so one press
     * of Right walks two tiles along a row. That is a defect a viewer meets on the first key of the
     * first screen, which is why it is a test and not a comment.
     */
    @Test
    public void only_a_key_down_moves_anything() {
        Screen screen = new Screen(true, true);
        Funnel funnel = funnelOf(screen);
        assertFalse(funnel.key(KeyEvent.KEYCODE_DPAD_RIGHT, KeyEvent.ACTION_UP));
        assertEquals(0, screen.asked.size());
        assertTrue(funnel.key(KeyEvent.KEYCODE_DPAD_RIGHT, KeyEvent.ACTION_DOWN));
        assertEquals(1, screen.asked.size());
        assertEquals(1, screen.redraws);
    }

    /**
     * A right to left page swaps the arrows once, on the way in, so no screen has to know which way
     * the language runs. {@code Remote.kt:77-80} and {@code :93}.
     */
    @Test
    public void a_right_to_left_page_swaps_the_arrows_once_on_the_way_in() {
        Screen screen = new Screen(true, true, true, true);
        Funnel funnel = funnelOf(screen);
        funnel.setMirrored(true);
        funnel.key(KeyEvent.KEYCODE_DPAD_LEFT, KeyEvent.ACTION_DOWN);
        funnel.key(KeyEvent.KEYCODE_DPAD_RIGHT, KeyEvent.ACTION_DOWN);
        funnel.key(KeyEvent.KEYCODE_DPAD_UP, KeyEvent.ACTION_DOWN);
        funnel.key(KeyEvent.KEYCODE_BACK, KeyEvent.ACTION_DOWN);
        assertEquals(Word.RIGHT, screen.asked.get(0));
        assertEquals(Word.LEFT, screen.asked.get(1));
        assertEquals(Word.UP, screen.asked.get(2));
        assertEquals(Word.BACK, screen.asked.get(3));
    }

    /** A key this surface does not answer never reaches the screen, and never draws. */
    @Test
    public void a_key_this_surface_does_not_answer_never_reaches_the_screen() {
        Screen screen = new Screen(true);
        Funnel funnel = funnelOf(screen);
        assertFalse(funnel.key(KeyEvent.KEYCODE_VOLUME_UP, KeyEvent.ACTION_DOWN));
        assertFalse(funnel.key(KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE, KeyEvent.ACTION_DOWN));
        assertEquals(0, screen.asked.size());
        assertEquals(0, screen.redraws);
    }

    /**
     * A held direction keeps moving, because the repeat count is not read.
     *
     * This is how a viewer scrolls a row by holding the arrow, and it is what the Compose path does
     * today: a held key arrives as repeated key downs and each one is a move.
     */
    @Test
    public void a_held_direction_keeps_moving() {
        Screen screen = new Screen(true, true, true);
        Funnel funnel = funnelOf(screen);
        for (int repeat = 0; repeat < 3; repeat++) {
            assertTrue(funnel.key(KeyEvent.KEYCODE_DPAD_RIGHT, KeyEvent.ACTION_DOWN));
        }
        assertEquals(3, screen.asked.size());
        assertEquals(3, screen.redraws);
    }
}
