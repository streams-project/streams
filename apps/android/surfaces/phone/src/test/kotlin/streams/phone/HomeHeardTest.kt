package streams.phone

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import streams.ports.Seen

/**
 * A real screen, composed, read as a screen reader would read it.
 *
 * `HeardTreeTest` builds trees by hand and proves the helpers reach a reader. This proves a screen
 * uses them. It is the half that was actually broken on 2026-11-11, when Home drew 61 nodes, 19 of
 * them reachable, and all 19 carried neither text nor a description -- a state every test in this
 * repository passed through.
 *
 * The brand here is a made-up word, and not only because `launcher/tests/one_name.rs` refuses the
 * real one in a program file. A header that drew a hardcoded name would pass a test written with
 * the real name and fail this one, which is the stronger assertion.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class HomeHeardTest {
    @get:Rule val compose = createComposeRule()

    private fun heardOn(seen: Seen): List<HeardTree> {
        val shell = Showing(seen)
        compose.setContent { Home(shell, seen) }
        return HeardTree.under(compose.onRoot().fetchSemanticsNode())
    }

    @Test
    fun `a page the menu has no label for is still heard by name`() {
        // The defect commit edfaf28 fixed: with no nav entry for this route and no `app.title`,
        // the header drew nothing at all. Nothing drawn is nothing heard, so a reader landed on a
        // page with no name. The brand is the answer, and this is what holds it.
        val heard = heardOn(Seen(brand = "Bramble"))
        assertTrue(
            "a reader has to be able to hear what page this is, and heard: $heard",
            heard.any { it.label.contains("Bramble") },
        )
    }

    @Test
    fun `a screen reader is handed something on an empty home`() {
        // The 2026-11-11 state in one line: a screen that draws and says nothing passes every
        // source-reading test in this repository and fails this one.
        val heard = heardOn(Seen(brand = "Bramble"))
        assertTrue("a drawn screen that says nothing is the defect: $heard", heard.isNotEmpty())
    }
}
