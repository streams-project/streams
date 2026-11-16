package streams.tv

import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.NativeKeyEvent
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performKeyPress
import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import streams.ports.Seen

/**
 * The first dead end of note 239, and the oldest one in this repository.
 *
 * Measured on a Google TV Streamer on 2026-11-13: with the server unreachable the app was
 * force-stopped and cold-started, and drew a flat `#0b0b10` rectangle. `uiautomator dump` read **9
 * nodes, 0 clickable, 0 focusable, 0 words**, held across 8 probes. No message, no retry, no
 * address screen, no way forward. Note 206 recorded the same screen on 2026-11-11 for a first run;
 * what changed by 2026-11-13 is that a signed-in viewer reaches it too, which is the commoner case
 * for somebody running their own server.
 *
 * **The hard half is that this screen may have no words to say anything with.** Every word the app
 * draws arrives from `GET /v1/strings/{lang}` (`core/Host.kt`); nothing ships on the device. So the
 * case worth testing is not a tidy one with English loaded, it is `words = emptyMap()` -- and the
 * test below runs both, because a fix that only works once the server has answered fixes nothing
 * about a server that has not.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class NoServerHeardTest {
    @get:Rule val compose = createComposeRule()

    private val book: Map<String, String> by lazy {
        Json.parseToJsonElement(File(repoRoot(), "strings/en.json").readText()).jsonObject
            .mapValues { (_, word) -> word.jsonPrimitive.content }
    }

    private fun kept(words: Map<String, String>) = Seen(server = ADDRESS, words = words)

    private fun heard(): List<HeardScreen> = HeardScreen.under(compose.onRoot().fetchSemanticsNode())

    /** What a reader is handed after the wait is over, with whatever words the app has. */
    private fun afterTheWait(words: Map<String, String>): List<HeardScreen> {
        compose.mainClock.autoAdvance = false
        val seen = kept(words)
        compose.setContent { Reaching(Watching(seen), seen) }
        compose.mainClock.advanceTimeBy(LOST_AFTER_MS + 1_000)
        return heard()
    }

    @Test
    fun `a television that cannot reach its server hands a reader something`() {
        // The measured state, in the state it was measured in: nothing has arrived, so there are
        // no words, and the screen has to say something anyway.
        val heard = afterTheWait(emptyMap())
        assertTrue("a drawn screen that says nothing is the defect: $heard", heard.any { it.label.isNotEmpty() })
    }

    @Test
    fun `and something on it can be reached with a remote`() {
        // The other measured half: 0 focusable. A remote has nothing to explore with, so words on a
        // screen with nothing focusable on it are still a dead end.
        val heard = afterTheWait(emptyMap())
        assertTrue("nothing on this screen can be focused, so a remote cannot reach it: $heard", heard.any { it.focusable })
    }

    @Test
    fun `the thing a remote reaches has a name, even with no words loaded`() {
        val heard = afterTheWait(emptyMap())
        val stops = heard.filter { it.focusable }
        assertEquals(
            "a focusable node with no name is what the player was measured to be: $heard",
            emptyList<HeardScreen>(),
            stops.filter { it.label.isEmpty() },
        )
    }

    @Test
    fun `the address it cannot reach is said, because it is the one thing that needs no strings`() {
        val heard = afterTheWait(emptyMap())
        assertTrue("the viewer is not told which address failed: $heard", heard.any { it.label.contains(ADDRESS) })
    }

    @Test
    fun `with words loaded a reader is told what went wrong and what to press`() {
        val heard = afterTheWait(book)
        val said = heard.flatMap { it.label.split("\n") }
        assertTrue("the trouble is not said: $heard", book.getValue("error.no_server") in said)
        assertTrue("the way out is not named: $heard", book.getValue("say.try_again") in said)
        // And the address drawn in its own right, not only carried on the two controls. Asserted
        // here, with words loaded, because that is the only state where an exact match picks the
        // drawn line out: the screen's box and the button both carry the address as the tail of a
        // longer label. Taking the drawn line off changed nothing a reader hears -- measured, by
        // taking it off -- so without this it is held by no test at all.
        assertTrue("the address is on the controls and nowhere a viewer can read it: $heard", heard.any { it.label == ADDRESS })
    }

    @Test
    fun `the wait is a wait and not a verdict`() {
        // A screen that said "cannot reach your server" the instant a slow server was slow would be
        // lying, and a test that only ever looked after the wait would not notice.
        compose.mainClock.autoAdvance = false
        val seen = kept(book)
        compose.setContent { Reaching(Watching(seen), seen) }
        compose.mainClock.advanceTimeBy(LOST_AFTER_MS - 1_000)
        val heard = heard()
        assertTrue(
            "the server is given ${LOST_AFTER_MS}ms before it is called unreachable, and heard: $heard",
            heard.none { it.label.contains(book.getValue("error.no_server")) },
        )
    }

    @Test
    fun `the way out asks the same address again`() {
        val shell = Watching(kept(book))
        compose.mainClock.autoAdvance = false
        compose.setContent { Lost(shell, shell.seen.value) }
        compose.mainClock.advanceTimeBy(100)
        compose.onRoot().performKeyPress(ok())
        assertEquals(
            "OK on this screen has to do the one thing that can change the situation",
            1,
            shell.retries,
        )
    }

    @Test
    fun `the no-route case is the one that goes through the wait`() {
        // Read from the source, because the screen that used to be drawn here (`Waiting`) is
        // correct for a page that is about to arrive and wrong for one that never will: it draws
        // the top bar from `seen.nav`, and with no server there is no nav, so it draws nothing. If
        // this branch is pointed back at it the composed tests above still pass -- they compose
        // `Reaching` directly -- and the television goes blank again.
        val text = withoutComments(File(repoRoot(), TELEVISION).readText())
        val branch = theOne(Regex("""target == null \|\| target == Target\.WAIT -> (\w+)\("""), text, "the waiting branch")
        assertEquals("Reaching", branch.groupValues[1])
    }

    /** The centre of a remote's ring, pressed down: this surface acts on key down. */
    private fun ok() = KeyEvent(
        NativeKeyEvent(android.view.KeyEvent.ACTION_DOWN, android.view.KeyEvent.KEYCODE_DPAD_CENTER),
    )

    private companion object {
        const val ADDRESS = "https://bramble.example"
        const val TELEVISION = "apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Television.kt"
    }
}
