package streams.tv.reminder;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertTrue;

import org.junit.Test;
import streams.tv.notifications.Reading;

/**
 * One title's facts, read by Java straight out of the contract.
 *
 * <p>The body is written as wire text and not built from {@code Contract.TitleDetail}'s
 * constructor, for the reason {@code NoticesTest} gives: what is being checked is the reading, and
 * a test that built the object would never notice a field renamed on the server.
 */
public class ComingTest {
    private static final String DUNE = "{"
            + "\"title\":\"movie:7\",\"name\":\"Dune\","
            + "\"available_from\":\"2026-12-25\",\"reminder_set\":true,"
            + "\"images\":["
            + "{\"role\":\"backdrop\",\"url\":\"http://box/b7.jpg\"},"
            + "{\"role\":\"poster\",\"url\":\"http://box/p7.jpg\"}]}";

    @Test
    public void a_body_from_the_server_becomes_the_title_the_screen_draws() {
        Coming coming = Coming.read(DUNE);
        assertEquals("movie:7", coming.key);
        assertEquals("Dune", coming.name);
        assertEquals("2026-12-25", coming.day);
        assertTrue(coming.set);
        assertTrue(coming.known());
    }

    /**
     * A title already available carries no {@code available_from} and nothing to be reminded of, so
     * the screen says it knows nothing rather than offering a reminder for a thing to watch now.
     * The same condition the phone's button hangs on at {@code Title.kt:300}.
     */
    @Test
    public void a_title_that_has_already_arrived_is_not_something_to_be_reminded_of() {
        Coming here = Coming.read("{\"title\":\"movie:7\",\"name\":\"Dune\"}");
        assertEquals("", here.day);
        assertFalse(here.set);
        assertFalse(here.known());
    }

    @Test
    public void a_body_this_build_cannot_read_is_a_screen_that_knows_nothing() {
        assertFalse(Coming.read("{\"title\":").known());
        assertFalse(Coming.read("{\"name\":\"Dune\"}").known());
        assertFalse(Coming.read("").known());
        assertFalse(Coming.read(null).known());
    }

    @Test
    public void the_poster_wins_over_the_wide_pictures() {
        assertEquals("http://box/p7.jpg", Coming.read(DUNE).poster);
        assertEquals("the backdrop is the fallback",
                "http://box/b7.jpg",
                Coming.read(DUNE.replace("\"role\":\"poster\"", "\"role\":\"logo\"")).poster);
        assertEquals("", Coming.read("{\"title\":\"movie:7\",\"name\":\"Dune\"}").poster);
    }

    /**
     * And the order is the Kotlin's own, held against the file that states it.
     *
     * {@code Coming.poster} is written twice with {@code shared/Art.kt}'s {@code tall}, because that
     * one takes the Kotlin arm's {@code List<contract.Image>} and this takes the Java arm's. Two
     * statements of one rule drift, so this reads the Kotlin and asserts the poster comes first
     * there as it does here.
     */
    @Test
    public void the_order_is_the_one_shared_art_states() {
        String kotlin = Reading.text(
                "apps/android/surfaces/shared/src/main/kotlin/streams/shared/Art.kt");
        int from = kotlin.indexOf("fun tall(");
        assertTrue("shared/Art.kt no longer has tall()", from > 0);
        String line = kotlin.substring(from, kotlin.indexOf("\n", kotlin.indexOf("=", from)));
        assertTrue("the poster is asked for first",
                line.contains("find(images, ImageRole.POSTER)"));
        assertTrue("and the wide pictures are behind it", line.contains("?: wide(images)"));
    }

    @Test
    public void turning_a_reminder_the_other_way_leaves_everything_else_alone() {
        Coming coming = Coming.read(DUNE);
        Coming off = coming.turned(false);
        assertFalse(off.set);
        assertEquals("Dune", off.name);
        assertEquals("2026-12-25", off.day);
        assertEquals("http://box/p7.jpg", off.poster);
        assertSame("and turning it the way it already is changes nothing",
                coming, coming.turned(true));
    }
}
