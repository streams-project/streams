package streams.tv

import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.dp
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

/**
 * The walker, kept honest.
 *
 * Every screen reading test below this one asserts that something was heard. A walker stuck
 * returning an empty list, or one that called every node a stop, would make those assertions pass
 * or fail for reasons that have nothing to do with the screens. So: one test asserts silence where
 * there should be silence, one asserts words where there should be words, and one asserts that an
 * unnamed focusable node is reported as a stop with no name -- which is the shape the television
 * player was measured to have and the shape a walker is most tempted to drop.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class HeardScreenTest {
    @get:Rule val compose = createComposeRule()

    private fun heard(content: @Composable () -> Unit): List<HeardScreen> {
        compose.setContent(content)
        return HeardScreen.under(compose.onRoot().fetchSemanticsNode())
    }

    @Test
    fun `a box nobody labelled and nobody can focus is not a stop`() {
        assertEquals(emptyList<HeardScreen>(), heard { Box(Modifier.size(48.dp)) })
    }

    @Test
    fun `a part that was given words is heard saying them`() {
        val only = heard { Box(Modifier.size(48.dp).says("Pause", on = true)) }.single()
        assertEquals("Pause", only.label)
        assertEquals(Role.Button, only.role)
        assertEquals(true, only.selected)
        assertFalse("a drawn part is not what a remote focuses on this surface", only.focusable)
    }

    @Test
    fun `words drawn inside a plain box are heard`() {
        assertEquals(
            listOf("Start Over"),
            heard { Box(Modifier.size(48.dp)) { Text("Start Over") } }.map { it.label },
        )
    }

    @Test
    fun `a focusable node with no words is a stop with no name`() {
        // The measured shape of the television player on 2026-11-13: focus lands, and the node it
        // lands on has `content-desc=''`. A walker that dropped it would report a screen with
        // nothing wrong with it.
        val only = heard { Box(Modifier.size(48.dp).focusable()) }.single()
        assertTrue(only.focusable)
        assertEquals("", only.label)
    }

    @Test
    fun `the words half and the focus half are reported apart`() {
        val all = heard {
            Box(Modifier.size(48.dp).focusable())
            Box(Modifier.size(48.dp).says("Play on"))
        }
        assertEquals("two stops, one of each kind, and heard: $all", 2, all.size)
        assertEquals(1, all.count { it.focusable })
        assertEquals(1, HeardScreen.words(compose.onRoot().fetchSemanticsNode()).size)
    }
}
