package streams.phone

import androidx.compose.ui.semantics.Role
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import contract.PlayState
import contract.PlayView
import contract.TitleKey
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import streams.ports.Seen
import streams.shared.Speaking
import streams.shared.fill

/**
 * The phone player, read as a screen reader is handed it.
 *
 * The phone twin of `surfaces/tv/.../PlayerHeardTest.kt`. On the phone the whole player surface
 * answers a tap (`Watch.kt`) to pause or unpause the film, and after three seconds the clock and
 * progress bar at the foot hide themselves. Before `Watch.kt` carried `says`, a playing player
 * never told a reader whether the film was playing or paused, and once the three seconds passed the
 * tap surface had no words in its subtree at all -- a clickable stop that said nothing.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class PlayerHeardTest {
    @get:Rule val compose = createComposeRule()

    @Before fun latinDigits() {
        Speaking.arabicDigits = false
        Speaking.digits = "latn"
    }

    private fun playing(state: PlayState, lengthMs: Long = 634_000) = Seen(
        words = words,
        language = "en",
        brand = BRAND,
        server = SERVER,
        play = PlayView(state = state, title = TitleKey("movie:12")),
        positionMs = 41_000,
        lengthMs = lengthMs,
    )

    private fun heardOn(seen: Seen, advanceMs: Long = 0): List<HeardTree> {
        compose.mainClock.autoAdvance = false
        compose.setContent { Watch(Showing(seen), seen, tv = false) }
        if (advanceMs > 0) compose.mainClock.advanceTimeBy(advanceMs)
        return HeardTree.under(compose.onRoot().fetchSemanticsNode())
    }

    @Test
    fun `a playing film says a tap would pause it`() {
        val heard = heardOn(playing(PlayState.PLAYING))
        assertTrue(
            "nothing in a playing player says a tap would pause it, and heard: $heard",
            heard.any { it.label == words.getValue("say.pause") && it.role == Role.Button },
        )
        assertTrue(
            "a playing player must not offer to play: $heard",
            heard.none { it.label == words.getValue("common.play") },
        )
    }

    @Test
    fun `a paused film says a tap would play it`() {
        val heard = heardOn(playing(PlayState.PAUSED))
        assertTrue(
            "nothing in a paused player says a tap would play it, and heard: $heard",
            heard.any { it.label == words.getValue("common.play") && it.role == Role.Button },
        )
        assertTrue(
            "a paused player must not still be offering to pause: $heard",
            heard.none { it.label == words.getValue("say.pause") },
        )
    }

    @Test
    fun `the position is said in the words the strings give and not only drawn as two clocks`() {
        val heard = heardOn(playing(PlayState.PLAYING))
        val said = words.getValue("say.position").fill("at", "0:41").fill("length", "10:34")
        assertTrue(
            "the bar does not say how far through the film it is, and heard: $heard",
            heard.any { it.label == said },
        )
        val transport = heard.first { it.label == words.getValue("say.pause") }
        assertEquals(
            "the transport control carries the position as its state",
            said,
            transport.state,
        )
    }

    @Test
    fun `a player with no length yet says where it is without an end`() {
        val heard = heardOn(playing(PlayState.PLAYING, lengthMs = 0))
        val transport = heard.first { it.label == words.getValue("say.pause") }
        assertEquals("0:41", transport.state)
    }

    @Test
    fun `the player still speaks after the controls hide`() {
        val seen = playing(PlayState.PLAYING)
        val heard = heardOn(seen, advanceMs = 3_500)
        val said = words.getValue("say.position").fill("at", "0:41").fill("length", "10:34")
        assertEquals(
            "a tap surface with hidden controls must not become a silent control",
            emptyList<String>(),
            silentControls(compose.onRoot().fetchSemanticsNode()),
        )
        val transport = heard.single()
        assertEquals(words.getValue("say.pause"), transport.label)
        assertEquals(said, transport.state)
        assertEquals(Role.Button, transport.role)
    }
}
