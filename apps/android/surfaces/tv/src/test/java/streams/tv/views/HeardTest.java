package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import android.app.Activity;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.TextView;
import java.util.List;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.robolectric.Robolectric;
import org.robolectric.RobolectricTestRunner;
import org.robolectric.annotation.Config;

/**
 * What a screen reader is actually handed by a Views tree, read off the real
 * {@link android.view.accessibility.AccessibilityNodeInfo} the toolkit builds.
 *
 * <p>Every other accessibility test in this repository reads source as text or reads a
 * {@code Modifier} chain without composing it. Those catch a helper applied to the wrong box. None
 * of them catches a helper that applies cleanly and reaches no reader, which is the failure that
 * matters: on 2026-11-11 Home drew 61 nodes, 19 reachable, and all 19 carried neither text nor a
 * description. This is the first test here that would have failed that day.
 *
 * <p>The first test below is the one the rest depend on. {@link Heard#under} returning an empty
 * list for everything would make every assertion here pass while proving nothing, so one test
 * asserts the walker stays silent where it should and another asserts it speaks where it should.
 * Neither can hold if the walker is stuck.
 */
@RunWith(RobolectricTestRunner.class)
@Config(sdk = 34)
public class HeardTest {
    private Activity activity;
    private FrameLayout root;

    @Before
    public void attachAWindow() {
        // A detached view answers null to all of this, so the tree has to hang off a real window.
        activity = Robolectric.buildActivity(Activity.class).setup().get();
        root = new FrameLayout(activity);
        activity.setContentView(root);
    }

    private View child() {
        View v = new View(activity);
        root.addView(v);
        return v;
    }

    @Test
    public void a_reader_hears_nothing_from_a_view_nobody_labelled() {
        child();
        assertEquals("an unlabelled view is not something to stop on", List.of(), Heard.under(root));
    }

    @Test
    public void a_reader_hears_the_words_a_control_was_given() {
        Says.onto(child(), Says.says("Play"));
        List<Heard> heard = Heard.under(root);
        assertEquals("exactly one thing to stop on", 1, heard.size());
        assertEquals("Play", heard.get(0).label);
        assertEquals("and it says it is a button", "android.widget.Button", heard.get(0).kind);
    }

    @Test
    public void a_control_that_stands_two_ways_says_which_after_its_words() {
        Says.onto(child(), Says.says("Gladiator", "In My List"));
        Heard only = Heard.under(root).get(0);
        assertEquals("Gladiator", only.label);
        assertEquals("the state is read after the label, not folded into it", "In My List", only.state);
    }

    @Test
    public void a_decoration_is_never_stopped_on() {
        Says.onto(child(), Says.decoration());
        assertEquals("a picture that means nothing says nothing", List.of(), Heard.under(root));
    }

    @Test
    public void a_part_the_remote_is_on_says_so() {
        Says.onto(child(), Says.stop("Films", true));
        Says.onto(child(), Says.stop("Series", false));
        List<Heard> heard = Heard.under(root);
        assertEquals(2, heard.size());
        assertTrue("the one the remote is on says it is", heard.get(0).selected);
        assertTrue("and the one it is not on does not", !heard.get(1).selected);
    }

    @Test
    public void the_words_a_control_draws_are_the_words_it_is_heard_by() {
        TextView t = new TextView(activity);
        t.setText("Continue Watching");
        root.addView(t);
        Says.onto(t, Says.saysItsWords());
        assertEquals("Continue Watching", Heard.under(root).get(0).label);
    }

    @Test
    public void nothing_under_a_hidden_part_is_reached() {
        FrameLayout hidden = new FrameLayout(activity);
        root.addView(hidden);
        View inside = new View(activity);
        hidden.addView(inside);
        Says.onto(inside, Says.says("Buried"));
        hidden.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS);
        assertEquals("a labelled view under a hidden parent reaches no reader",
                List.of(), Heard.under(root));
    }

    @Test
    public void a_reader_reaches_things_in_the_order_they_are_drawn_and_knows_how_deep_they_sit() {
        Says.onto(child(), Says.says("First"));
        FrameLayout inner = new FrameLayout(activity);
        root.addView(inner);
        View deep = new View(activity);
        inner.addView(deep);
        Says.onto(deep, Says.says("Second"));
        List<Heard> heard = Heard.under(root);
        assertEquals(List.of("First", "Second"), List.of(heard.get(0).label, heard.get(1).label));
        assertEquals("the first sits one below the root", 1, heard.get(0).depth);
        assertEquals("and the second one below that", 2, heard.get(1).depth);
    }
}
