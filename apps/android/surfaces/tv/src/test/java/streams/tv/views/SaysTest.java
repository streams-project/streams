package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

import java.util.Arrays;
import java.util.TreeSet;
import org.junit.Test;

/**
 * What a part says, and what it leaves unsaid.
 *
 * What these prove and what they do not, stated the way {@code src/test/kotlin/streams/tv/Heard.kt}
 * states it for the Compose side:
 * <ul>
 * <li>proved -- the decisions. An empty label is left unset rather than set to nothing; an empty
 * state is not a state; a switch with no words says nothing at all; a decoration takes its subtree
 * with it; a part the remote is on says so either way. Every one of those is a line of
 * {@code Says.kt} or {@code Spoken.kt} that a reader hears the consequence of.</li>
 * <li>not proved -- the tree. What a real screen reader is handed, what order it reaches things in,
 * and what a gathered node really gathers all need a laid-out hierarchy, and there is no
 * Robolectric and no {@code ui-test} on this machine. The instrument for that half is the one that
 * found the original defect: {@code uiautomator dump} on the Streamer, every reachable node
 * carrying a text or a description, which is note 210's wave 2 check and is not claimed here.</li>
 * </ul>
 */
public class SaysTest {
    private static final String LABEL = "مشاهدة";
    private static final String STATE = "مفتوح";

    /** A control with words is a button that says them and gathers what it draws. */
    @Test
    public void a_control_with_words_is_a_button_that_says_them() {
        Says.Said said = Says.says(LABEL);
        assertEquals(LABEL, said.label);
        assertEquals(Says.Role.BUTTON, said.role);
        assertTrue(said.gathers);
        assertFalse(said.cleared);
        assertFalse(said.silent());
        assertNull("a control that is not on or off says neither", said.selected);
    }

    /**
     * An empty label is not set, and the control still says it is a button whose own words are its
     * label. {@code Says.kt:54-62}, exactly: a control described as nothing is worse than one a
     * reader falls through to its contents for, and the words are empty for as long as a run has not
     * read its language.
     */
    @Test
    public void an_empty_label_falls_through_to_the_control_s_own_words() {
        Says.Said said = Says.says("");
        assertEquals("", said.label);
        assertTrue(said.silent());
        assertTrue("it still gathers, so a reader hears what is drawn inside it", said.gathers);
        assertEquals(Says.Role.BUTTON, said.role);
        assertEquals(Says.saysItsWords().label, said.label);
        assertEquals(Says.saysItsWords().role, said.role);
        assertEquals(Says.says(null).gathers, said.gathers);
    }

    /** A state is read after the label, and an empty state is not a state. */
    @Test
    public void an_empty_state_is_not_a_state() {
        assertEquals(STATE, Says.says(LABEL, STATE).state);
        assertEquals("", Says.says(LABEL, "").state);
        assertEquals("", Says.says(LABEL, null).state);
        assertEquals(STATE, Says.saysItsWords(STATE).state);
        assertEquals("", Says.saysItsWords("").state);
    }

    /**
     * A switch with no words says nothing at all -- not an empty label, nothing -- so the view is
     * left exactly as it was. {@code Says.kt:77-85} returns the chain unchanged for this case, and
     * {@code Heard.kt:58-64} names it as the case worth having: a helper that decided to do nothing
     * is indistinguishable from a helper nobody called, and that is on purpose.
     */
    @Test
    public void a_switch_with_no_words_says_nothing_at_all() {
        assertTrue(Says.saysSwitch("").nothing);
        assertTrue(Says.saysSwitch(null).nothing);
        assertFalse(Says.saysSwitch(LABEL).nothing);
        assertEquals(Says.Role.SWITCH, Says.saysSwitch(LABEL).role);
        assertFalse("a switch does not gather: its words are drawn beside it, not in it",
                Says.saysSwitch(LABEL).gathers);
    }

    /** And nothing stays nothing, whatever is added to it. */
    @Test
    public void nothing_stays_nothing() {
        assertTrue(Says.saysSwitch("").and(STATE).nothing);
        assertTrue(Says.saysSwitch("").on(true).nothing);
        assertEquals("", Says.saysSwitch("").and(STATE).state);
        assertNull(Says.saysSwitch("").on(true).selected);
    }

    /** A decoration goes away, and takes what is drawn inside it with it. */
    @Test
    public void a_decoration_goes_away_and_takes_its_subtree_with_it() {
        Says.Said said = Says.decoration();
        assertTrue(said.cleared);
        assertEquals("", said.label);
        assertEquals(Says.Role.NONE, said.role);
        assertFalse(said.gathers);
    }

    /**
     * What a drawn part means is said with no role claimed, and nothing is said when there are no
     * words for it. {@code Spoken.kt:41-42}.
     */
    @Test
    public void what_a_part_means_claims_no_role_and_says_nothing_when_there_are_no_words() {
        assertEquals(LABEL, Says.means(LABEL).label);
        assertEquals(Says.Role.NONE, Says.means(LABEL).role);
        assertTrue(Says.means("").nothing);
        assertTrue(Says.means(null).nothing);
    }

    /**
     * A part the remote can stop on says whether the remote is on it -- either way, and even with no
     * words.
     *
     * This is the only thing that tells a person where they are on a screen whose focus is painted
     * rather than held by the toolkit ({@code Spoken.kt:44-48}), so "not selected" has to be said
     * and not merely not said: a reader that is never told false cannot tell a part it has left from
     * a part it has arrived at.
     */
    @Test
    public void a_part_the_remote_is_on_says_so_either_way() {
        assertEquals(Boolean.TRUE, Says.stop(LABEL, true).selected);
        assertEquals(Boolean.FALSE, Says.stop(LABEL, false).selected);
        assertEquals(Boolean.FALSE, Says.stop("", false).selected);
        assertEquals(Says.Role.BUTTON, Says.stop(LABEL, true).role);
        assertEquals(Boolean.TRUE, Says.saysItsWords(true).selected);
        assertTrue(Says.saysItsWords(true).gathers);
    }

    /**
     * Two things are read as one label with a line end between them, so no language's punctuation or
     * word order is decided in the code. {@code Spoken.kt:79-80}, including what it does with an
     * empty side.
     */
    @Test
    public void two_things_are_read_as_one_label_with_a_line_end_between_them() {
        assertEquals(LABEL + "\n" + STATE, Says.sayBoth(LABEL, STATE));
        assertEquals(LABEL, Says.sayBoth(LABEL, ""));
        assertEquals(LABEL, Says.sayBoth(LABEL, null));
        assertEquals(STATE, Says.sayBoth("", STATE));
        assertEquals("", Says.sayBoth("", ""));
        assertEquals("", Says.sayBoth(null, null));
    }

    /**
     * Every helper the two Kotlin files have, this file has.
     *
     * The point of the whole class: the two toolkits must not diverge while both exist. A helper
     * added to {@code Says.kt} or {@code Spoken.kt} with no Java twin is a phone screen or a Compose
     * screen that says something the Java screens cannot, which is how one surface of this app ends
     * up readable and the other one does not.
     */
    @Test
    public void every_helper_the_kotlin_has_is_named_here() {
        TreeSet<String> kotlin = new TreeSet<>();
        kotlin.addAll(Repo.every("fun (?:Modifier\\.)?(\\w+)\\(", Repo.text(SAYS)));
        kotlin.addAll(Repo.every("fun (?:Modifier\\.)?(\\w+)\\(", Repo.text(SPOKEN)));
        assertEquals("the two files have five helper names between them",
                new TreeSet<>(Arrays.asList("decoration", "sayBoth", "says", "saysItsWords", "saysSwitch")),
                kotlin);
        String java = Repo.text(JAVA);
        for (String name : kotlin) {
            assertTrue(name + " has no Java twin in " + JAVA, java.contains(name + "("));
        }
    }

    private static final String SAYS =
            "apps/android/surfaces/shared/src/main/kotlin/streams/shared/Says.kt";
    private static final String SPOKEN =
            "apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Spoken.kt";
    private static final String JAVA =
            "apps/android/surfaces/tv/src/main/java/streams/tv/views/Says.java";
}
