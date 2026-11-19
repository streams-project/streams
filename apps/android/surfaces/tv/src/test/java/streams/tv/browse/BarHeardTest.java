package streams.tv.browse;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

import android.app.Activity;
import android.view.View;
import android.view.ViewGroup;
import android.view.accessibility.AccessibilityNodeInfo;
import android.widget.FrameLayout;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.robolectric.Robolectric;
import org.robolectric.RobolectricTestRunner;
import org.robolectric.annotation.Config;
import streams.shared.Speaking;
import streams.tv.views.Word;

/**
 * The top bar built for real, and what a screen reader is handed read off the
 * {@link AccessibilityNodeInfo} tree the toolkit builds.
 *
 * <h2>Why this file exists</h2>
 * {@code SaidTest} is a census of call sites: it proves the bar still asks for its words, and
 * proves nothing about whether a reader reaches them. {@code views/HeardTest} walks a real tree but
 * only over views it builds itself. Neither would have caught a control added to this bar with no
 * words on it, which is the defect this project has found four times this week -- Home drawing 61
 * nodes, 19 reachable and all 19 silent on 2026-11-11, and four television screens leaving the
 * focused node unnamed on 2026-11-13.
 *
 * <p>So this is the walk, over the bar as a page of rows really draws it, and the thing it asserts
 * is the general one: <b>nothing on the bar is reached and silent</b>. A control added to this bar
 * later is in its scope without anybody remembering to add it.
 *
 * <p>{@code @Config(sdk = 34)} for {@code views/Heard.java}'s own reason: Robolectric cannot stand
 * up this project's {@code compileSdk 37}. What that costs is in note 228.
 */
@RunWith(RobolectricTestRunner.class)
@Config(sdk = 34)
public class BarHeardTest {
    /** No picture adapter: this is about words, and a face with no url asks for nothing. */
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

    /** The words a run has read, as far as this bar is concerned. */
    private static Map<String, String> said() {
        Map<String, String> words = new HashMap<String, String>();
        words.put("nav.search", "Search");
        words.put("notifications.title", "Notifications");
        words.put("say.notifications_unread", "Notifications, {n} new");
        return words;
    }

    private static Feed feed(int unread) {
        List<Feed.Tab> tabs = Arrays.asList(
                new Feed.Tab("Home", "home"),
                new Feed.Tab("Series", "series"));
        return new Feed(tabs, 0, new Feed.Who("Ibrahim", "", false), "", new ArrayList<Feed.Row>(),
                null, "", unread);
    }

    private Activity activity;
    private FrameLayout root;
    private BrowseView screen;

    @Before
    public void attachAWindow() {
        // A detached view answers null to all of this: `createAccessibilityNodeInfo` populates
        // against a window, so a tree with no `setContentView` reads nothing however it is marked.
        activity = Robolectric.buildActivity(Activity.class).setup().get();
        root = new FrameLayout(activity);
        activity.setContentView(root);
        // The digits a count is written in are a global the words set as a run reads its language.
        // Pinned here, because another test in this JVM may have left it on another language and
        // the strings this file holds are Latin.
        Speaking.INSTANCE.setDigits("latn");
        screen = new BrowseView(activity, NO_PICTURES);
        screen.asks(new BrowseView.Asked() {
            @Override
            public void chose(Place.Chose what, Feed.Tab tab, Feed.Card card) {
            }

            @Override
            public void leave() {
            }

            @Override
            public void moved() {
            }
        });
        screen.speak(said());
        root.addView(screen);
    }

    /**
     * The places on the bar, in the order a viewer walks them: the profile, search, the two tabs,
     * the bell. Written out so the walk below presses real keys in a real direction.
     */
    private static final int[] ALONG = {Feed.WHO_TAB, Feed.SEARCH_TAB, 0, 1, Feed.BELL_TAB};

    /**
     * The bar drawn with the remote standing somewhere on it.
     *
     * The remote is walked there with the keys a viewer would press, and never put there by
     * reaching into {@link Place}: a stop only a test can reach is a stop a viewer cannot.
     */
    private List<String> barAt(int focus, int unread) {
        Feed feed = feed(unread);
        Place place = new Place(feed, 0);
        int from = at(place.tab());
        int to = at(focus);
        for (int step = from; step != to; step += to > from ? 1 : -1) {
            assertTrue("the remote cannot walk the bar as far as place " + focus,
                    place.press(to > from ? Word.RIGHT : Word.LEFT));
        }
        assertEquals("the walk did not land where it was aimed", focus, place.tab());
        screen.show(place, feed, "");
        return labels(screen);
    }

    private static int at(int place) {
        for (int index = 0; index < ALONG.length; index++) {
            if (ALONG[index] == place) {
                return index;
            }
        }
        throw new IllegalArgumentException("not a place on this bar: " + place);
    }

    /**
     * Nothing on the bar is reached and silent.
     *
     * The one assertion this file is for, made at every place the remote can stand on the bar, and
     * made over whatever the bar draws rather than over a list of its parts.
     */
    @Test
    public void nothing_on_the_bar_is_reached_and_says_nothing() {
        int[] places = ALONG;
        for (int at = 0; at < places.length; at++) {
            for (int unread : new int[] {0, 4}) {
                List<String> heard = barAt(places[at], unread);
                assertFalse("the bar was reached by nothing at all, so this proves nothing",
                        heard.isEmpty());
                assertEquals("something on the bar is reached and says nothing, at place "
                        + places[at] + " with " + unread + " waiting",
                        new ArrayList<String>(), silent(screen));
            }
        }
    }

    /**
     * The bell says how many notices are waiting, in the words the strings give, and says the
     * screen's own name when none are.
     *
     * {@code say.notifications_unread} is in all eighteen languages and had no television caller on
     * either family until this. The count is drawn on the bell, so this reading is the only way a
     * person who cannot see it can have it at all.
     */
    @Test
    public void the_bell_says_how_many_notices_are_waiting() {
        assertTrue("the bell does not say the count the server sent",
                barAt(Feed.BELL_TAB, 4).contains("Notifications, ⁨4⁩ new"));
        assertTrue("with nothing waiting the bell says the screen's own name",
                barAt(Feed.BELL_TAB, 0).contains("Notifications"));
        assertFalse("and does not say a count of none",
                barAt(Feed.BELL_TAB, 0).contains("Notifications, ⁨0⁩ new"));
    }

    /**
     * The bell says whether the remote is on it, either way.
     *
     * A surface whose focus is painted has nothing else to tell a reader with: a part that falls
     * silent when the remote leaves gives a person listening no way to know it left.
     * {@code SpokenTest.kt:29-34}.
     */
    @Test
    public void the_bell_says_whether_the_remote_is_on_it() {
        barAt(Feed.BELL_TAB, 4);
        assertEquals(Boolean.TRUE, selectedOf("Notifications, ⁨4⁩ new"));
        barAt(0, 4);
        assertEquals("a part the remote has left must say so, not fall silent",
                Boolean.FALSE, selectedOf("Notifications, ⁨4⁩ new"));
    }

    /** The count drawn on the bell is not a stop of its own, or a reader would hear it twice. */
    @Test
    public void the_count_drawn_on_the_bell_is_not_a_second_stop() {
        List<String> heard = barAt(Feed.BELL_TAB, 4);
        assertFalse("the number alone is a stop with no noun on it", heard.contains("4"));
        assertFalse(heard.contains("⁨4⁩"));
    }

    /**
     * The count is drawn on the bell when there is one, in the digits the language writes, and is
     * not drawn at all when there is none.
     *
     * The count is decoration to a reader, so everything above this is blind to it: a nought
     * drawn on the bell, or a Western 4 drawn in an Arabic bar, says nothing wrong to a screen
     * reader and is plainly wrong to anyone looking. Read off the drawn text itself.
     */
    @Test
    public void the_count_is_drawn_when_there_is_one_and_not_when_there_is_none() {
        barAt(0, 4);
        assertTrue("the count the server sent is not drawn: " + drawn(screen),
                drawn(screen).contains("4"));
        barAt(0, 0);
        assertFalse("a count of none is drawn on the bell: " + drawn(screen),
                drawn(screen).contains("0"));
    }

    @Test
    public void the_count_is_drawn_in_the_digits_of_the_language() {
        Speaking.INSTANCE.setDigits("arab");
        try {
            barAt(0, 4);
            assertTrue("the count is not drawn in the language's own digits: " + drawn(screen),
                    drawn(screen).contains("\u0664"));
            assertFalse("a Western digit is drawn on an Arabic bar: " + drawn(screen),
                    drawn(screen).contains("4"));
        } finally {
            Speaking.INSTANCE.setDigits("latn");
        }
    }

    /**
     * The bell goes on saying the remote is on it when the run reads its language after the bar was
     * drawn.
     *
     * The bell's reading is built from the words, the count and where the remote is, and they never
     * arrive together: {@code words} rebuilds it from what {@code show} last kept, and a kept
     * "the remote is not here" turns the stop off for a person who is standing on it.
     */
    @Test
    public void the_bell_keeps_the_remote_when_the_words_arrive_after_it() {
        barAt(Feed.BELL_TAB, 4);
        Map<String, String> later = said();
        later.put("say.notifications_unread", "Alerts, {n} waiting");
        screen.speak(later);
        assertEquals("the words arriving after the bar took the remote off the bell",
                Boolean.TRUE, selectedOf("Alerts, \u20684\u2069 waiting"));
    }

    /** Every word drawn on a part of the bar that is showing. */
    private static List<String> drawn(View root) {
        List<String> found = new ArrayList<String>();
        drawnUnder(root, found);
        return found;
    }

    private static void drawnUnder(View view, List<String> found) {
        if (view.getVisibility() != View.VISIBLE) {
            return;
        }
        if (view instanceof android.widget.TextView) {
            found.add(((android.widget.TextView) view).getText().toString());
        }
        if (view instanceof ViewGroup) {
            ViewGroup group = (ViewGroup) view;
            for (int at = 0; at < group.getChildCount(); at++) {
                drawnUnder(group.getChildAt(at), found);
            }
        }
    }

    // --- the walk -------------------------------------------------------------------------------

    /** Everything a reader would stop on, in the order it would reach them. */
    private static List<String> labels(View root) {
        List<String> found = new ArrayList<String>();
        walk(root, found, false);
        return found;
    }

    /** And everything it would stop on that has nothing to read. */
    private static List<String> silent(View root) {
        List<String> found = new ArrayList<String>();
        walk(root, found, true);
        return found;
    }

    private static void walk(View view, List<String> found, boolean quietOnly) {
        if (view.getImportantForAccessibility() == View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS) {
            return;
        }
        if (view.getVisibility() != View.VISIBLE) {
            return;
        }
        AccessibilityNodeInfo node = view.createAccessibilityNodeInfo();
        if (node != null && view.getImportantForAccessibility() != View.IMPORTANT_FOR_ACCESSIBILITY_NO) {
            String label = say(node.getContentDescription());
            if (label.isEmpty()) {
                label = say(node.getText());
            }
            // A node a reader stops on is one it can focus. A plain group with no words is passed
            // over by the reader and by this walk, which is the same rule `views/Heard.java` uses.
            boolean stop = node.isFocusable() || node.isClickable() || !label.isEmpty();
            if (stop && (quietOnly == label.isEmpty())) {
                // A silent stop is reported by what it is, because its words are the thing it has
                // not got: a list of empty strings names nothing a reader of the failure can fix.
                found.add(quietOnly ? view.getClass().getName() + " " + node.getClassName() : label);
            }
        }
        if (view instanceof ViewGroup) {
            ViewGroup group = (ViewGroup) view;
            for (int at = 0; at < group.getChildCount(); at++) {
                walk(group.getChildAt(at), found, quietOnly);
            }
        }
    }

    /**
     * Whether the part with these words says the remote is on it.
     *
     * Asked of the screen's children and not of the screen, which is a trap this test fell into
     * while it was being written: {@code RemoteHost.setSays} puts the same words on the screen
     * itself as a live region, so a walk that started at the root answered with the root's own
     * {@code selected} -- which is never set -- and the test reported the bell silent about the
     * focus while the bell was saying it correctly.
     */
    private Boolean selectedOf(String label) {
        for (int at = 0; at < screen.getChildCount(); at++) {
            Boolean found = selectedUnder(screen.getChildAt(at), label);
            if (found != null) {
                return found;
            }
        }
        return null;
    }

    private static Boolean selectedUnder(View view, String label) {
        if (view.getImportantForAccessibility() == View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS) {
            return null;
        }
        AccessibilityNodeInfo node = view.createAccessibilityNodeInfo();
        if (node != null && label.equals(say(node.getContentDescription()))) {
            return Boolean.valueOf(node.isSelected());
        }
        if (view instanceof ViewGroup) {
            ViewGroup group = (ViewGroup) view;
            for (int at = 0; at < group.getChildCount(); at++) {
                Boolean found = selectedUnder(group.getChildAt(at), label);
                if (found != null) {
                    return found;
                }
            }
        }
        return null;
    }

    private static String say(CharSequence words) {
        return words == null ? "" : words.toString();
    }

    /** The bar is really there, so a walk that found nothing is a broken walk and not a clean bar. */
    @Test
    public void the_walk_reaches_the_bar_at_all() {
        List<String> heard = barAt(0, 2);
        assertNotNull(heard);
        assertTrue("the profile is a stop on this bar", heard.contains("Ibrahim"));
        assertTrue("and so is search", heard.contains("Search"));
        assertTrue("and so is a tab", heard.contains("Home"));
        assertTrue("and so is the bell", heard.contains("Notifications, ⁨2⁩ new"));
    }
}
