package streams.tv

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import contract.PlayState
import contract.PlayView
import contract.TitleKey
import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import streams.ports.Seen
import androidx.compose.ui.semantics.Role
import streams.shared.Speaking
import streams.shared.fill

/**
 * The player, read as a screen reader is handed it.
 *
 * Dead end 2 of note 239, measured on a Google TV Streamer on 2026-11-13: a film played, the
 * player drew its full control set, focus moved visibly across the reaction pills and the
 * transport, and the one focusable node's `content-desc` was `''` at all five stops. Browse and the
 * title page name their focus; the player did not. A person using a screen reader can operate none
 * of it.
 *
 * Dead end 3 in the half this surface owns: nothing told a reader whether the film was playing or
 * paused. The television does draw a glyph, so an eye can see it -- but the glyph is an `Icon` with
 * no description, and the control around it had no name, so a reader got neither the control nor
 * the state. The control is now named by its action, which is how a reader learns the state:
 * "Pause" means it is playing.
 *
 * The words are read out of `strings/en.json` rather than written here, for the reason
 * `HomeHeardTest` gives on the phone: a screen that drew a hardcoded word would pass a test written
 * with that word and fail this one.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class PlayerHeardTest {
    @get:Rule val compose = createComposeRule()

    /**
     * The clock is drawn through `number()`, which reads a process wide setting the real app writes
     * from the language file. Another test in this source set leaving it on Arabic digits would
     * turn "0:41" into "٠:٤١" here, so it is set rather than assumed.
     */
    @Before fun latinDigits() { Speaking.digits = "latn" }

    private val book: Map<String, String> by lazy {
        Json.parseToJsonElement(File(repoRoot(), "strings/en.json").readText()).jsonObject
            .mapValues { (_, word) -> word.jsonPrimitive.content }
    }

    private fun word(key: String) = requireNotNull(book[key]) { "strings/en.json has no $key" }

    private fun playing(state: PlayState) = Seen(
        words = book,
        language = "en",
        brand = "Bramble",
        server = "https://bramble.example",
        play = PlayView(state = state, title = TitleKey("movie:12")),
        positionMs = 41_000,
        lengthMs = 634_000,
    )

    /**
     * The controls hide themselves three seconds after the remote was last touched, through a
     * `delay` the test clock would run straight past. Reading a playing player therefore means
     * holding the clock still; a paused player keeps its controls up on its own.
     */
    private fun heardOn(seen: Seen): List<HeardScreen> {
        compose.mainClock.autoAdvance = false
        compose.setContent { Watch(Watching(seen), seen) }
        return HeardScreen.under(compose.onRoot().fetchSemanticsNode())
    }

    @Test
    fun `the node the remote is on has a name`() {
        // The measured defect in one line: focus lands, and what it lands on is nameless.
        val heard = heardOn(playing(PlayState.PLAYING))
        val stops = heard.filter { it.focusable }
        assertTrue("the player has to be reachable by a remote at all, and heard: $heard", stops.isNotEmpty())
        assertEquals(
            "every stop a remote can land on has to have a name, and heard: $heard",
            emptyList<HeardScreen>(),
            stops.filter { it.label.isEmpty() },
        )
    }

    @Test
    fun `a playing film says the key would pause it`() {
        val heard = heardOn(playing(PlayState.PLAYING))
        // The control, by role. The screen's own box says the same word as a live region, so a
        // test that asked only whether the word is anywhere on the screen would pass with the
        // control's own label taken off -- measured, by taking it off.
        assertTrue(
            "nothing in a playing player says it is playing, and heard: $heard",
            heard.any { it.label == word("say.pause") && it.role == Role.Button },
        )
    }

    @Test
    fun `a paused film says the key would play it`() {
        val heard = heardOn(playing(PlayState.PAUSED))
        assertTrue(
            "nothing in a paused player says it is paused, and heard: $heard",
            heard.any { it.label == word("common.play") && it.role == Role.Button },
        )
        assertTrue(
            "a paused player must not still be offering to pause: $heard",
            heard.none { it.label == word("say.pause") },
        )
    }

    @Test
    fun `the transport is the one the remote is on when the player opens`() {
        // The transport is where the remote starts (`zone = Reach.BAR`), so it is the control that
        // must say it is the chosen one -- on this surface `selected` is the only thing that tells
        // a reader where it is, because the focus is painted rather than held by the toolkit.
        val heard = heardOn(playing(PlayState.PLAYING))
        // By role, because the screen's own box says the same word: it is a live region carrying
        // what the remote is on, and the control it names says it too. Two nodes with one word is
        // the shape this surface has on purpose (`app/Spoken.kt`), not a duplicate to remove.
        val transport = heard.single { it.label == word("say.pause") && it.role == Role.Button }
        assertEquals("the remote starts on the transport and has to say so: $heard", true, transport.selected)
    }

    @Test
    fun `the glyphs over the picture are named, not just drawn`() {
        // Back and Start Over are glyphs with no word beside them unless the remote is on them, so
        // before this they were shapes a reader could not find at all.
        val heard = heardOn(playing(PlayState.PLAYING))
        val said = heard.map { it.label }
        for (key in listOf("player.back", "player.restart")) {
            assertTrue("$key is drawn and not said, and heard: $heard", word(key) in said)
        }
    }

    @Test
    fun `the reaction pills a paused film offers are named`() {
        // They are drawn only while the film is paused, which is also the only time the remote can
        // reach them (`Reach.RATE` is reachable from the transport only when paused).
        val heard = heardOn(playing(PlayState.PAUSED))
        for (key in listOf("player.up", "player.down", "player.love")) {
            // By role and not by the word alone. The word is drawn inside the pill, so a reader
            // sweeping the screen finds it either way; what it could not find before is a *stop*
            // that says whether the remote is on it, which on this surface is the only thing that
            // tells a person where they are (`app/Spoken.kt`). Measured by taking the label off:
            // asking for the word alone passed with the pill unnamed.
            val pill = heard.firstOrNull { it.label == word(key) && it.role == Role.Button }
            assertNotNull("$key is drawn but is not a stop a reader can be put on: $heard", pill)
            assertNotNull("$key must say whether the remote is on it: $heard", pill!!.selected)
        }
    }

    @Test
    fun `the position is said and not only drawn as two clocks`() {
        // `say.position` is "{at} of {length}". The bar between the clocks is a shape with no words
        // in it; the clocks either side are drawn text that reads as two loose numbers.
        val heard = heardOn(playing(PlayState.PLAYING))
        // Built with `fill`, not spelled out: it wraps each value in bidi isolates so a clock keeps
        // its order inside a right to left sentence, and a test that wrote the plain string would
        // be asserting that it does not.
        val said = word("say.position").fill("at", "0:41").fill("length", "10:34")
        assertTrue("a reader gets two loose numbers and no sentence, and heard: $heard", heard.any { it.label == said })
    }
}
