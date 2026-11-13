package streams.phone;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

import android.app.Activity;
import android.view.View;
import android.view.accessibility.AccessibilityNodeInfo;
import android.widget.FrameLayout;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.robolectric.Robolectric;
import org.robolectric.RobolectricTestRunner;
import org.robolectric.annotation.Config;
import streams.shared.Says;

/**
 * That a phone screen can hand a reader what {@link Says} decides, and what the node it builds
 * really carries.
 *
 * <h2>Why this file is here</h2>
 * {@link Says} counts 97 production call sites, 71 of them on this family, and until 2026-11-13 the
 * Java form of it sat in {@code surfaces/tv/src/main/java/streams/tv/views/}, where nothing the
 * phone compiles could name it (notes 235 and 242). It moved to {@code surfaces/shared} with
 * {@code Words}, {@code Ink}, {@code Shade} and {@code RowMemory}, and this file is the proof the
 * move did what it was for: it is Java, it is in the phone's own test source set, and it names a
 * class the phone's module does not hold. Point the import below at {@code streams.tv.views.Says}
 * and this module stops compiling, which is the claim stated as a failure.
 *
 * <h2>What is asserted, which is not what the television's tests assert</h2>
 * {@code tv/.../views/SaysTest.java} reads the decisions off the value without a device, and
 * {@code tv/.../views/HeardTest.java} walks a whole tree. The phone's question is the single node:
 * a tile it draws is one view, and what a reader is handed for it is read back off the real
 * {@link AccessibilityNodeInfo} the toolkit builds, not off the value that was asked for. The two
 * halves can disagree -- a reading can be decided correctly and applied to nothing -- and that is
 * the failure note 215 recorded: on 2026-11-11 Home drew 61 nodes, 19 of them reachable, and all 19
 * carried neither text nor a description.
 *
 * <h2>Two conditions, both of which make a reading empty rather than wrong</h2>
 * A detached view answers null to all of this, because {@code createAccessibilityNodeInfo}
 * populates against a window, so the tree hangs off a real one. And Robolectric cannot stand up
 * this project's {@code compileSdk 37} on JDK 25, so the sdk is pinned at 34, for the reason
 * {@code tv/.../views/Heard.java} gives in full.
 */
@RunWith(RobolectricTestRunner.class)
@Config(sdk = 34)
public class SaysTest {
    private Activity activity;
    private FrameLayout root;

    @Before
    public void attachAWindow() {
        activity = Robolectric.buildActivity(Activity.class).setup().get();
        root = new FrameLayout(activity);
        activity.setContentView(root);
    }

    /** A view in a real window, which is the only kind that answers. */
    private View tile() {
        View tile = new View(activity);
        root.addView(tile);
        return tile;
    }

    /** A tile draws artwork and no words, so its name is the one thing a reader has. */
    @Test
    public void a_tile_says_its_title_to_a_reader() {
        View tile = tile();
        Says.onto(tile, Says.says("Gladiator"));
        AccessibilityNodeInfo node = tile.createAccessibilityNodeInfo();
        assertEquals("Gladiator", String.valueOf(node.getContentDescription()));
        assertEquals("a tile is something to press", "android.widget.Button",
                String.valueOf(node.getClassName()));
        assertTrue("and it is read as one thing, not as its parts",
                node.isScreenReaderFocusable());
    }

    /**
     * The rule at {@code shared/Says.kt:46-52}: a title whose words have not arrived is left
     * unlabelled, not labelled with nothing. A description set to empty is a reader stopping on a
     * control and hearing silence, which is worse than falling through to what is drawn.
     */
    @Test
    public void a_tile_with_no_words_yet_is_left_unlabelled_rather_than_labelled_with_nothing() {
        View tile = tile();
        Says.onto(tile, Says.says(""));
        AccessibilityNodeInfo node = tile.createAccessibilityNodeInfo();
        assertNull("no description at all", node.getContentDescription());
        assertTrue("and what it draws is still read as one thing", node.isScreenReaderFocusable());
    }

    /** What the phone's list state is heard as: the name first, then what it is, as two readings. */
    @Test
    public void a_title_already_in_my_list_says_so_after_its_name() {
        View tile = tile();
        Says.onto(tile, Says.says("Gladiator", "In My List"));
        AccessibilityNodeInfo node = tile.createAccessibilityNodeInfo();
        assertEquals("Gladiator", String.valueOf(node.getContentDescription()));
        assertEquals("the state is read after the label and not folded into it", "In My List",
                String.valueOf(node.getStateDescription()));
    }

    /**
     * The art inside a tile that already says the title's name is one more thing to stop on and
     * hear nothing from, so it and everything under it go away.
     */
    @Test
    public void the_art_inside_a_tile_is_never_stopped_on() {
        View art = tile();
        Says.onto(art, Says.decoration());
        assertEquals("the subtree goes with it, not just the view",
                View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS,
                art.getImportantForAccessibility());
        assertNull("and it is described as nothing", art.getContentDescription());
    }

    /**
     * A tile in a row is heard as its title and then the row it sits in. The break is a line end
     * and not a comma or a word, so no language's punctuation or word order is decided here.
     */
    @Test
    public void a_title_and_the_row_it_sits_in_are_read_as_one_label() {
        assertEquals("Gladiator\nTrending Now", Says.sayBoth("Gladiator", "Trending Now"));
        assertEquals("a row with no name yet leaves the title alone", "Gladiator",
                Says.sayBoth("Gladiator", ""));
        assertEquals("and a title with no name yet leaves the row alone", "Trending Now",
                Says.sayBoth("", "Trending Now"));
    }
}
