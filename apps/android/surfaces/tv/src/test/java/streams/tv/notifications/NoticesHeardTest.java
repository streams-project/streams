package streams.tv.notifications;

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
import streams.tv.parts.PageHead;
import streams.tv.views.Word;

/**
 * The notifications screen built for real, keys pressed at it, and what a screen reader is handed
 * read off the {@link android.view.accessibility.AccessibilityNodeInfo} tree the toolkit builds.
 *
 * <p>Follows {@code views/HeardTest.java} rather than asserting on source text. The defect that
 * pattern exists for was measured on 2026-11-11 -- Home drew 61 nodes, 19 reachable, all 19
 * carrying neither text nor a description -- and on 2026-11-13 four television screens still left
 * the focused node unnamed. This is a fifth screen and it does not join them, which is asserted
 * here and not claimed.
 *
 * <p>{@code @Config(sdk = 34)} for {@code Heard.java}'s own reason: Robolectric cannot stand up
 * this project's {@code compileSdk 37}, because {@code ApplicationSharedMemory.create} reflects on
 * a {@code FileDescriptor} that JDK 25 refuses. What that costs is in note 228.
 */
@RunWith(RobolectricTestRunner.class)
@Config(sdk = 34)
public class NoticesHeardTest {
    /** A screen that counts what the funnel asked it to redraw. */
    private static final class Counted extends NoticesView {
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

    /** No picture adapter: these tests are about words, and a still with no url asks for nothing. */
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
    private static final class Asks implements NoticesView.Asked {
        final List<String> said = new ArrayList<String>();

        @Override
        public void chose(Notices.Notice notice) {
            said.add("chose " + notice.id);
        }

        @Override
        public void read(Notices.Notice notice) {
            said.add("read " + notice.id);
        }

        @Override
        public void leave() {
            said.add("leave");
        }

        @Override
        public void moved() {
            said.add("moved");
        }
    }

    /**
     * How many stops a reader reaches before the first notice: the screen's own live region, then
     * the page head's name. Named so an index into the walk reads as what it is.
     */
    private static final int LINES = 2;

    private Activity activity;
    private FrameLayout root;
    private Counted screen;
    private Asks asks;

    @Before
    public void attachAWindow() {
        // A detached view answers null to all of this: `createAccessibilityNodeInfo` populates
        // against a window, so a tree with no `setContentView` reads nothing however it is labelled.
        activity = Robolectric.buildActivity(Activity.class).setup().get();
        root = new FrameLayout(activity);
        activity.setContentView(root);
        screen = new Counted(activity);
        asks = new Asks();
        screen.asks(asks);
        screen.speak(Reading.said());
        root.addView(screen);
    }

    private static Notices.Notice notice(String id, String key, String name, String kind,
            String day, boolean unread) {
        return new Notices.Notice(id, key, name, "", kind, day, unread);
    }

    private static Notices feed() {
        List<Notices.Notice> items = new ArrayList<Notices.Notice>();
        items.add(notice("n1", "movie:7", "Dune", "notification.coming", "2026-12-01", true));
        items.add(notice("n2", "tv:9", "Stranger Things", "notification.new_season", "2026-11-30",
                false));
        items.add(notice("n3", "movie:3", "Heat", "notification.reminder_live", "2026-11-29",
                false));
        return new Notices(items, "", false);
    }

    private void show(Notices what) {
        screen.show(new Spot(what), what);
        // The screen is measured and laid out, because a node's bounds and a view's visibility to a
        // reader both come from a layout pass.
        screen.measure(View.MeasureSpec.makeMeasureSpec(1920, View.MeasureSpec.EXACTLY),
                View.MeasureSpec.makeMeasureSpec(1080, View.MeasureSpec.EXACTLY));
        screen.layout(0, 0, 1920, 1080);
    }

    private void press(int keyCode) {
        screen.dispatchKeyEvent(new KeyEvent(KeyEvent.ACTION_DOWN, keyCode));
    }

    // --- the numbers this screen shares with the rest of the television --------------------------

    /**
     * The list starts under the page head, and {@link Spot#HEAD} is that head's own number.
     *
     * Written as a number in {@code Spot} because that class holds no toolkit and {@code PageHead}
     * is a {@code View}. This is where the two meet, so they cannot drift.
     */
    @Test
    public void the_list_starts_under_the_page_head() {
        assertEquals(PageHead.under(), Spot.HEAD);
    }

    // --- what a reader is handed ------------------------------------------------------------------

    @Test
    public void a_reader_arriving_on_an_empty_screen_hears_its_name_and_that_it_is_empty() {
        show(Notices.NOTHING);
        assertEquals("Notifications\nNo notifications yet.", screen.says());
        assertEquals("and it is a live region, so it is said again when it changes",
                View.ACCESSIBILITY_LIVE_REGION_POLITE, screen.getAccessibilityLiveRegion());
    }

    @Test
    public void a_reader_is_handed_the_notice_the_remote_is_on() {
        show(feed());
        assertEquals("Coming Soon\nDune\n2026-12-01\nNot read yet", screen.says());
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        assertEquals("New Season\nStranger Things\n2026-11-30", screen.says());
    }

    /**
     * One stop per notice, and the four things each line draws are not four more.
     *
     * A reader that stopped on the still, then the kind, then the name, then the day would hear a
     * notice in four fragments and have to assemble it. The line carries the whole reading and its
     * parts are marked as decoration, so the tree holds exactly what a person needs to walk.
     */
    @Test
    public void a_reader_stops_once_per_notice_and_not_once_per_part() {
        show(feed());
        assertEquals(List.of(
                // The screen's own live region first: on a television that paints its own focus,
                // this is the thing that says where the remote has arrived.
                "Coming Soon\nDune\n2026-12-01\nNot read yet",
                "Notifications",
                "Coming Soon\nDune\n2026-12-01",
                "New Season\nStranger Things\n2026-11-30",
                "Now Available\nHeat\n2026-11-29"), Reading.labels(screen));
    }

    @Test
    public void the_line_the_remote_is_on_says_so_and_the_others_do_not() {
        show(feed());
        List<Reading.Stop> stops = Reading.stops(screen);
        assertTrue("the first notice is the one the remote is on", stops.get(LINES).on);
        assertFalse(stops.get(LINES + 1).on);
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        stops = Reading.stops(screen);
        assertFalse(stops.get(LINES).on);
        assertTrue(stops.get(LINES + 1).on);
    }

    @Test
    public void whether_a_notice_has_been_read_is_said_after_its_words_and_not_folded_into_them() {
        show(feed());
        List<Reading.Stop> stops = Reading.stops(screen);
        assertEquals("Not read yet", stops.get(LINES).state);
        assertEquals("", stops.get(LINES + 1).state);
        assertEquals("and the line says it is something to press",
                "android.widget.Button", stops.get(LINES).kind);
    }

    /**
     * The one measured on hardware: a node that says it is a control and hands a reader no words.
     *
     * Four television screens do this today -- the player, search, the profile menu and Get Help --
     * and this screen is asserted not to be a fifth, in both of its states.
     */
    @Test
    public void nothing_on_this_screen_claims_to_be_a_control_without_saying_what_it_is() {
        show(feed());
        assertEquals(List.of(), Reading.unnamed(screen));
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        assertEquals(List.of(), Reading.unnamed(screen));
        show(Notices.NOTHING);
        assertEquals(List.of(), Reading.unnamed(screen));
    }

    /** The words of the empty screen are not still being said under a screen that has notices. */
    @Test
    public void a_hidden_part_is_not_read_out() {
        show(Notices.NOTHING);
        assertTrue(Reading.labels(screen).contains("No notifications yet."));
        show(feed());
        assertFalse(Reading.labels(screen).contains("No notifications yet."));
    }

    // --- the funnel --------------------------------------------------------------------------------

    /**
     * Note 210's acceptance check, on this screen: {@code changed()} fires exactly as often as
     * {@code press} answered true, and nothing else calls it.
     *
     * Three downs from the top of three notices: two move and the third is the end of the list.
     * Then two ups move and the third is the top. Five moves, five redraws, and the three dead
     * presses redraw nothing.
     */
    @Test
    public void the_screen_redraws_exactly_when_a_key_moved_something() {
        show(feed());
        screen.redraws = 0;
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        press(KeyEvent.KEYCODE_DPAD_UP);
        press(KeyEvent.KEYCODE_DPAD_UP);
        press(KeyEvent.KEYCODE_DPAD_UP);
        press(KeyEvent.KEYCODE_DPAD_RIGHT);
        assertEquals(4, screen.redraws);
        assertEquals("and each move told the host to think about reading ahead",
                List.of("moved", "moved", "moved", "moved"), asks.said);
    }

    // --- the key map ---------------------------------------------------------------------------------

    @Test
    public void left_leaves_the_screen_as_it_does_on_the_settings_page() {
        show(feed());
        press(KeyEvent.KEYCODE_DPAD_LEFT);
        assertEquals(List.of("leave"), asks.said);
    }

    @Test
    public void back_leaves_the_screen() {
        show(feed());
        press(KeyEvent.KEYCODE_BACK);
        assertEquals(List.of("leave"), asks.said);
    }

    @Test
    public void right_does_nothing_at_all() {
        show(feed());
        press(KeyEvent.KEYCODE_DPAD_RIGHT);
        assertEquals(List.of(), asks.said);
        assertEquals("Coming Soon\nDune\n2026-12-01\nNot read yet", screen.says());
    }

    /**
     * OK opens the title, and the notice is read at once rather than when the server answers.
     *
     * The phone does the same at {@code Notifications.kt:189-200}: the dot must not wait on the
     * network, so the line is drawn read, the server is told, and the count is refreshed.
     */
    @Test
    public void ok_opens_the_title_and_marks_the_notice_read_without_waiting() {
        show(feed());
        press(KeyEvent.KEYCODE_DPAD_CENTER);
        assertEquals(List.of("read n1", "chose n1"), asks.said);
        assertEquals("the line stopped saying it is unread", "",
                Reading.stops(screen).get(LINES).state);
        assertEquals("Coming Soon\nDune\n2026-12-01", screen.says());
    }

    @Test
    public void ok_on_a_notice_that_was_already_read_tells_the_server_nothing() {
        show(feed());
        press(KeyEvent.KEYCODE_DPAD_DOWN);
        asks.said.clear();
        press(KeyEvent.KEYCODE_DPAD_CENTER);
        assertEquals(List.of("chose n2"), asks.said);
    }

    @Test
    public void ok_on_an_empty_screen_chooses_nothing() {
        show(Notices.NOTHING);
        press(KeyEvent.KEYCODE_DPAD_CENTER);
        assertEquals(List.of(), asks.said);
    }

    /** Every word this surface answers is answered here, and nothing else is taken from the system. */
    @Test
    public void a_key_this_screen_does_not_answer_falls_through_to_the_system() {
        show(feed());
        assertFalse("volume still works",
                screen.dispatchKeyEvent(new KeyEvent(KeyEvent.ACTION_DOWN,
                        KeyEvent.KEYCODE_VOLUME_UP)));
        assertEquals(null, Word.of(KeyEvent.KEYCODE_VOLUME_UP));
    }
}
