package streams.shared

import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * What each helper in `Says.kt` hands a screen reader. The first tests any of the six have had.
 *
 * Why these exist: `Says.kt`'s own comment records the measurement that made it necessary -- Home
 * drew 61 nodes on emulator-5554 on 2026-11-11, 19 reachable, all 19 silent -- and nothing in this
 * repository checks that the fix is still there. `./check` runs no gradle at all
 * (`apps/android/surfaces/tv/build.gradle.kts:20` says so), so until a person or CI runs this
 * module's tests every one of the helpers could be emptied out and every check would stay green.
 *
 * Each test is named after what a person would lose. What they read is the real
 * `SemanticsConfiguration` the toolkit builds from the helper's own properties lambda; [Heard]'s
 * comment says what that does and does not prove, and the limit is the offline toolkit's, not a
 * choice.
 *
 * None of these pins a number or a string to itself. Each one states a rule about what a reader
 * gets -- a label is never the empty string, a labelled control always says what kind of thing it
 * is, a decoration goes away rather than going quiet -- and the rules are what a rewrite into a
 * toolkit with no `Modifier.semantics` has to reproduce with `contentDescription`,
 * `setStateDescription` and `importantForAccessibility`.
 */
class SaysTest {
    /**
     * `saysItsWords()` gathers and names, and deliberately sets no description of its own: the
     * words the control already draws are its label, so a label here could only drift from them.
     *
     * This is the defect `Says.kt:17-21` measured. `Modifier.clickable` makes a node that answers a
     * tap and does not gather what is drawn inside it, so the word "Play" sat one node away from the
     * button with nothing able to focus it. A rewrite that sets a description instead of gathering
     * passes a naive "does it have a label" check and brings the drift straight back.
     */
    @Test
    fun `a control that draws its own words gathers them instead of being given a label`() {
        val node = one(Modifier.saysItsWords())
        assertTrue("the words inside the control have to become its label", node.gathers)
        assertEquals(Role.Button, node.role)
        assertEquals("a gathering control must not carry words of its own", setOf("Role"), node.keys)
    }

    /** A control that also stands one of two ways says which, after its own words and not instead. */
    @Test
    fun `a state is read after the control's own words`() {
        val node = one(Modifier.saysItsWords("in my list"))
        assertTrue(node.gathers)
        assertEquals(Role.Button, node.role)
        assertEquals("in my list", node.state)
        assertEquals(setOf("Role", "StateDescription"), node.keys)
    }

    /**
     * An empty state is left unset. A run that has not read its language yet has no words for the
     * state, and `stateDescription = ""` makes a reader say the control's name and then stop on
     * nothing, which is worse than saying only the name.
     */
    @Test
    fun `an empty state is left unset rather than read as nothing`() {
        val node = one(Modifier.saysItsWords(""))
        assertEquals(setOf("Role"), node.keys)
        assertEquals("", node.state)
    }

    /** A control with nothing written on it -- an icon, a tile, a face -- is given words. */
    @Test
    fun `a control with no words of its own is given words`() {
        val node = one(Modifier.says("Play"))
        assertEquals("Play", node.label)
        assertEquals(Role.Button, node.role)
        assertTrue("a given label still gathers, so nothing inside is read loose", node.gathers)
    }

    /**
     * The same rule as the state, for the label, and it is the one `Says.kt:50-53` calls out:
     * `words[...]` is a map lookup that answers empty until a language has been read, so an empty
     * label must fall back to gathering rather than be set.
     *
     * A reader on a node described as "" stops and says nothing. A reader on a gathering node reads
     * whatever the control draws. The second is always at least as good as the first, which is why
     * this is a rule and not a preference.
     */
    @Test
    fun `an empty label falls back to gathering rather than describing the node as nothing`() {
        val node = one(Modifier.says(""))
        assertTrue(node.gathers)
        assertEquals(setOf("Role"), node.keys)
        assertEquals("", node.label)
    }

    /** Label then state, in that order, both from the strings file. */
    @Test
    fun `an on-off control says what it is and which way it stands`() {
        val node = one(Modifier.says("My List", "in my list"))
        assertEquals("My List", node.label)
        assertEquals("in my list", node.state)
        assertEquals(Role.Button, node.role)
        assertEquals(setOf("Role", "ContentDescription", "StateDescription"), node.keys)
    }

    /** And an on-off control with no state words says only its name. */
    @Test
    fun `an on-off control with no state words says only its name`() {
        val node = one(Modifier.says("My List", ""))
        assertEquals("My List", node.label)
        assertEquals("", node.state)
        assertEquals(setOf("Role", "ContentDescription"), node.keys)
    }

    /**
     * A switch is the one helper that does not gather, and the reason is in `Says.kt:72-76`: the
     * toolkit's `Switch` already says it is a switch and which way it stands, and the words for what
     * it switches are drawn beside it in a row the switch is no part of. Gathering here would pull
     * that row's text into the switch and say it twice.
     *
     * A rewrite that makes every helper gather, for tidiness, loses this.
     */
    @Test
    fun `a switch is named without gathering the row it sits in`() {
        val node = one(Modifier.saysSwitch("Autoplay next episode"))
        assertFalse("a switch must not gather the words drawn beside it", node.gathers)
        assertEquals(Role.Switch, node.role)
        assertEquals("Autoplay next episode", node.label)
    }

    /** With no words for it, the switch is left exactly as it was: the toolkit's own node stands. */
    @Test
    fun `a switch with no words is left alone`() {
        assertTrue(
            "an unworded switch must add no node at all, not an empty one",
            heard(Modifier.saysSwitch("")).isEmpty(),
        )
    }

    /**
     * A decoration goes away. `Says.kt:93-95` is explicit that this clears the subtree rather than
     * setting an empty description, because an empty description leaves one more thing a reader
     * stops on and reads out as nothing.
     *
     * The two are a line apart in any toolkit and they are not the same for a person:
     * `contentDescription = ""` on a View is a stop, `importantForAccessibility = NO` is not.
     */
    @Test
    fun `a decoration goes away instead of becoming a silent stop`() {
        val node = one(Modifier.decoration())
        assertTrue("a decoration has to clear, not describe", node.cleared)
        assertEquals("a cleared node sets nothing", emptySet<String>(), node.keys)
        assertNull(node.role)
    }

    /**
     * Every helper that gives a reader words also says what kind of thing is being read, and the
     * helper that takes words away says nothing at all. Stated over all six at once so a seventh
     * helper, or a rewrite of any of them, cannot quietly leave the role off: a reader that is told
     * "Play" and not told it is a button offers no way to press it.
     */
    @Test
    fun `every helper that speaks says what kind of thing it is`() {
        val speaking = listOf(
            "saysItsWords()" to Modifier.saysItsWords(),
            "saysItsWords(state)" to Modifier.saysItsWords("on"),
            "says(label)" to Modifier.says("Play"),
            "says(label, state)" to Modifier.says("My List", "in my list"),
            "saysSwitch(label)" to Modifier.saysSwitch("Autoplay"),
        )
        for ((name, modifier) in speaking) {
            val node = one(modifier)
            assertTrue("$name must say what kind of thing it is", node.role != null)
            assertFalse("$name must not describe a node as the empty string", "" == node.label && node.keys.contains("ContentDescription"))
        }
        assertTrue("decoration() must not speak", one(Modifier.decoration()).keys.isEmpty())
    }
}
