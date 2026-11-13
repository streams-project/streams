package streams.tv.reminder;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import android.app.Activity;
import android.content.Context;
import android.view.KeyEvent;
import android.view.View;
import android.widget.FrameLayout;
import java.util.ArrayList;
import java.util.List;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.robolectric.Robolectric;
import org.robolectric.RobolectricTestRunner;
import org.robolectric.annotation.Config;
import streams.tv.browse.Art;
import streams.tv.notifications.Reading;
import streams.tv.parts.PageHead;

/**
 * The reminder screen built for real, keys pressed at it, and what a screen reader is handed read
 * off the {@link android.view.accessibility.AccessibilityNodeInfo} tree the toolkit builds.
 *
 * <p>Follows {@code views/HeardTest.java}. {@code @Config(sdk = 34)} for {@code Heard.java}'s own
 * reason: Robolectric cannot stand up {@code compileSdk 37} on JDK 25.
 */
@RunWith(RobolectricTestRunner.class)
@Config(sdk = 34)
public class ReminderHeardTest {
    private static final Coming DUNE =
            new Coming("movie:7", "Dune", "", "2026-12-25", false);

    /** A screen that counts what the funnel asked it to redraw. */
    private static final class Counted extends ReminderView {
        int redraws;

        Counted(Context context) {
            super(context, NO_PICTURES);
        }

        @Override
        public void changed() {
            redraws += 1;
            super.changed();
        }
    }

    private static final Art.Source NO_PICTURES = new Art.Source() {
        @Override
        public Art.Stop fetch(String url, int wide, int high, Art.Done done) {
            return new Art.Stop() {
                @Override
                public void stop() {
                }
            };
        }
    };

    /** What the screen asked of the shell, in the order it asked. */
    private static final class Asks implements ReminderView.Asked {
        final List<String> said = new ArrayList<String>();

        @Override
        public void turn(Coming title, boolean on) {
            said.add("turn " + title.key + " " + on);
        }

        @Override
        public void leave() {
            said.add("leave");
        }
    }

    private Counted screen;
    private Asks asks;

    @Before
    public void attachAWindow() {
        // A detached view answers null to all of this: `createAccessibilityNodeInfo` populates
        // against a window.
        Activity activity = Robolectric.buildActivity(Activity.class).setup().get();
        FrameLayout root = new FrameLayout(activity);
        activity.setContentView(root);
        screen = new Counted(activity);
        asks = new Asks();
        screen.asks(asks);
        screen.speak(Reading.said());
        root.addView(screen);
    }

    private void show(Coming title) {
        screen.show(new Ask(title), title);
        screen.measure(View.MeasureSpec.makeMeasureSpec(1920, View.MeasureSpec.EXACTLY),
                View.MeasureSpec.makeMeasureSpec(1080, View.MeasureSpec.EXACTLY));
        screen.layout(0, 0, 1920, 1080);
    }

    private void press(int keyCode) {
        screen.dispatchKeyEvent(new KeyEvent(KeyEvent.ACTION_DOWN, keyCode));
    }

    // --- the numbers this screen shares with the rest of the television --------------------------

    /**
     * The page begins under a page head, and {@link ReminderView#HEAD} is that head's own number.
     *
     * This screen draws its own head because a {@code PageHead} takes two <em>keys</em> and the
     * words at the top of this one are a title's name, which is data. This is where the number and
     * the part it came from meet, so they cannot drift.
     */
    @Test
    public void the_page_begins_under_a_page_head() {
        assertEquals(PageHead.under(), ReminderView.HEAD);
        assertEquals(ReminderView.HEAD + ReminderView.GAP, ReminderView.FIRST);
    }

    /** And the poster is the reference's own closed tile, read out of the file the code cites. */
    @Test
    public void the_poster_is_the_measured_closed_tile() {
        String measured = Reading.text("reference/netflix-on-the-tv.md");
        assertTrue("the closed tile is " + ReminderView.POSTER_WIDE + " x 505",
                measured.contains("**" + ReminderView.POSTER_WIDE + " x 505** portrait poster"));
        assertEquals("laid out at 504 so the half-dp scale lands on a whole pixel",
                504, ReminderView.POSTER_HIGH);
        assertTrue("and the measured gap beside it",
                measured.contains("gap " + ReminderView.GAP + ", facts line"));
    }

    // --- what a reader is handed ------------------------------------------------------------------

    @Test
    public void a_reader_arriving_hears_the_title_when_it_arrives_and_the_line_it_is_on() {
        show(DUNE);
        assertEquals("Dune\nComing on ⁨2026-12-25⁩\nCancel", screen.says());
        assertEquals("and it is a live region, so it is said again when it changes",
                View.ACCESSIBILITY_LIVE_REGION_POLITE, screen.getAccessibilityLiveRegion());
    }

    @Test
    public void a_reader_hears_the_other_line_when_the_remote_moves_to_it() {
        show(DUNE);
        press(KeyEvent.KEYCODE_DPAD_UP);
        assertEquals("Dune\nComing on ⁨2026-12-25⁩\nRemind Me", screen.says());
    }

    /**
     * The two lines, and nothing else.
     *
     * The title's name, the line under it and the poster are what the screen's own reading already
     * carries, so none of the three is a stop of its own: a reader walking this screen finds the
     * two things it can press and is not made to step over the three it cannot.
     */
    @Test
    public void a_reader_stops_on_the_two_lines_and_nothing_else() {
        show(DUNE);
        assertEquals(List.of(
                // The screen's own live region first: on a television that paints its own focus,
                // this is the thing that says where the remote has arrived.
                "Dune\nComing on ⁨2026-12-25⁩\nCancel",
                "Remind Me",
                "Cancel"), Reading.labels(screen));
    }

    /**
     * The one measured on hardware: a node that says it is a control and hands a reader no words.
     *
     * Four television screens do this today -- the player, search, the profile menu and Get Help --
     * and this screen is asserted not to be a fifth, at both of its stops and in both states of the
     * reminder.
     */
    @Test
    public void nothing_on_this_screen_claims_to_be_a_control_without_saying_what_it_is() {
        show(DUNE);
        assertEquals(List.of(), Reading.unnamed(screen));
        press(KeyEvent.KEYCODE_DPAD_UP);
        assertEquals(List.of(), Reading.unnamed(screen));
        show(DUNE.turned(true));
        assertEquals(List.of(), Reading.unnamed(screen));
        show(Coming.NOTHING);
        assertEquals(List.of(), Reading.unnamed(screen));
    }

    // --- the funnel --------------------------------------------------------------------------------

    /**
     * Note 210's acceptance check on this screen: {@code changed()} fires exactly as often as
     * {@code press} answered true.
     *
     * Two lines, so Up moves once and the second Up is the top; Down moves once and the second Down
     * is the bottom. Two moves, two redraws, and the two dead presses plus Right redraw nothing.
     */
    @Test
    public void the_screen_redraws_exactly_when_a_key_moved_something() {
        show(DUNE);
        screen.redraws = 0;
        press(KeyEvent.KEYCODE_DPAD_UP);
        press(KeyEvent.KEYCODE_DPAD_UP);
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        press(KeyEvent.KEYCODE_DPAD_RIGHT);
        assertEquals(2, screen.redraws);
        assertEquals("and none of that asked anything of the shell", List.of(), asks.said);
    }

    // --- the key map ---------------------------------------------------------------------------------

    @Test
    public void left_leaves_the_screen_as_it_does_on_the_settings_page() {
        show(DUNE);
        press(KeyEvent.KEYCODE_DPAD_LEFT);
        assertEquals(List.of("leave"), asks.said);
    }

    @Test
    public void back_leaves_the_screen() {
        show(DUNE);
        press(KeyEvent.KEYCODE_BACK);
        assertEquals(List.of("leave"), asks.said);
    }

    @Test
    public void ok_on_the_second_line_leaves() {
        show(DUNE);
        press(KeyEvent.KEYCODE_DPAD_CENTER);
        assertEquals(List.of("leave"), asks.said);
    }

    /**
     * OK on the first line sends the command and draws the answer at once, without waiting for the
     * server, which is what the phone does at {@code Title.kt:314-316}.
     */
    @Test
    public void ok_on_the_first_line_sets_the_reminder_and_says_so_before_the_server_answers() {
        show(DUNE);
        press(KeyEvent.KEYCODE_DPAD_UP);
        press(KeyEvent.KEYCODE_DPAD_CENTER);
        assertEquals(List.of("turn movie:7 true"), asks.said);
        assertEquals("Dune\nComing on ⁨2026-12-25⁩\nReminder Set", screen.says());
        assertEquals("and the line the remote is on says what it would do next",
                List.of("Dune\nComing on ⁨2026-12-25⁩\nReminder Set",
                        "Reminder Set", "Cancel"), Reading.labels(screen));
    }

    @Test
    public void ok_on_the_first_line_of_a_title_already_reminded_clears_it() {
        show(DUNE.turned(true));
        press(KeyEvent.KEYCODE_DPAD_UP);
        press(KeyEvent.KEYCODE_DPAD_CENTER);
        assertEquals(List.of("turn movie:7 false"), asks.said);
        assertEquals("Dune\nComing on ⁨2026-12-25⁩\nRemind Me", screen.says());
    }

    @Test
    public void ok_on_a_title_this_screen_knows_nothing_about_asks_for_nothing() {
        show(Coming.NOTHING);
        press(KeyEvent.KEYCODE_DPAD_UP);
        press(KeyEvent.KEYCODE_DPAD_CENTER);
        assertEquals(List.of(), asks.said);
    }

    @Test
    public void right_does_nothing_at_all() {
        show(DUNE);
        press(KeyEvent.KEYCODE_DPAD_RIGHT);
        assertEquals(List.of(), asks.said);
        assertEquals("Dune\nComing on ⁨2026-12-25⁩\nCancel", screen.says());
    }

    @Test
    public void a_key_this_screen_does_not_answer_falls_through_to_the_system() {
        show(DUNE);
        assertFalse("volume still works",
                screen.dispatchKeyEvent(new KeyEvent(KeyEvent.ACTION_DOWN,
                        KeyEvent.KEYCODE_VOLUME_UP)));
    }
}
