package streams.phone

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.dp
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import streams.shared.decoration
import streams.shared.says
import streams.shared.saysItsWords

/**
 * What a screen reader is handed by a composed screen, read off the merged semantics tree.
 *
 * `shared/.../Heard.kt` reads what a `Modifier` chain declares and states plainly that it proves
 * nothing about the tree. `SpokenTest.kt` covers the "right helper on the right box" half by
 * reading sources. Neither covers what a reader actually reaches, in what order, or what
 * `mergeDescendants` gathers. This does, and it is the Compose half of note 228.
 *
 * As in the Views twin, the first two tests keep the rest honest: a walker stuck returning an
 * empty list would make every assertion below pass while proving nothing, so one test asserts
 * silence where there should be silence and another asserts words where there should be words.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class HeardTreeTest {
    @get:Rule val compose = createComposeRule()

    private fun heard(content: @androidx.compose.runtime.Composable () -> Unit): List<HeardTree> {
        compose.setContent(content)
        return HeardTree.under(compose.onRoot().fetchSemanticsNode())
    }

    @Test
    fun `a reader hears nothing from a box nobody labelled`() {
        assertEquals(emptyList<HeardTree>(), heard { Box(Modifier.size(48.dp)) })
    }

    @Test
    fun `a reader hears the words a control was given and what it says it is`() {
        val only = heard { Box(Modifier.size(48.dp).says("Play")) }.single()
        assertEquals("Play", only.label)
        assertEquals(Role.Button, only.role)
    }

    @Test
    fun `a control that stands two ways says which after its words`() {
        val only = heard { Box(Modifier.size(48.dp).says("Gladiator", "In My List")) }.single()
        assertEquals("Gladiator", only.label)
        assertEquals("the state is read after the label, not folded into it", "In My List", only.state)
    }

    /**
     * The pair below is one assertion in two tests, because `setContent` may be called once per
     * test. Separating them is not tidiness: the first version of this put `decoration()` on an
     * empty box, which is silent whatever the helper does, so it passed with the whole helper
     * replaced by `this`. It asserted a helper worked while measuring nothing it did. Caught on
     * 2026-11-13 by mutating `decoration()`; the words drawn inside are what makes it real, and
     * the first test exists to prove those words are heard when nothing silences them.
     */
    @Test
    fun `words drawn inside a plain box are heard`() {
        assertEquals(
            listOf("Poster of Gladiator"),
            heard { Box(Modifier.size(48.dp)) { Text("Poster of Gladiator") } }.map { it.label },
        )
    }

    @Test
    fun `a decoration silences the words drawn inside it`() {
        assertEquals(
            emptyList<HeardTree>(),
            heard { Box(Modifier.size(48.dp).decoration()) { Text("Poster of Gladiator") } },
        )
    }

    @Test
    fun `a control that gathers its children is one stop and not three`() {
        // The thing no other test here can see: `mergeDescendants` collapses the words drawn
        // inside into one reading. Three Texts, one node, and the words joined in drawn order.
        val heard = heard {
            Box(Modifier.size(120.dp).saysItsWords()) {
                Column {
                    Text("Continue")
                    Text("Watching")
                }
            }
        }
        assertEquals("a gathering node is one stop, not one per word drawn inside", 1, heard.size)
        assertTrue(
            "and it is heard by the words drawn inside it, in order: ${heard.single()}",
            heard.single().label.contains("Continue") && heard.single().label.contains("Watching"),
        )
        assertTrue("Continue is read first", heard.single().label.indexOf("Continue") < heard.single().label.indexOf("Watching"))
    }

    @Test
    fun `a reader reaches things in the order they are drawn and knows how deep they sit`() {
        val heard = heard {
            Column {
                Box(Modifier.size(48.dp).says("First"))
                Column { Box(Modifier.size(48.dp).says("Second")) }
            }
        }
        assertEquals("the order a reader reaches them is the order they are drawn",
            listOf("First", "Second"), heard.map { it.label })
        // And the shape, which is not the composable shape: a `Column` declares no semantics, so
        // the nested box is not nested in the tree a reader walks. Asserted so that the day
        // Compose changes it, this says so instead of a reader quietly getting a different screen.
        assertEquals("the merged tree flattens what a Column nests", listOf(1, 1), heard.map { it.depth })
    }
}
