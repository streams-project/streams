package streams.tv

import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.NativeKeyEvent
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performKeyPress
import contract.Episode
import contract.EpisodeRef
import contract.Resume
import contract.RouteTarget
import contract.Season
import contract.SeasonSummary
import contract.Target
import contract.TitleDetail
import contract.TitleKey
import contract.contractJson
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
 * The episodes page while a season is being read, pressed on the real screen with a remote.
 *
 * Nothing here was found by a test. It was found by reading, while this screen was ported to Xbox
 * (`apps/xbox/surfaces/remote/title.cpp`), and all of it lives in the gap between what the page
 * holds and what has arrived -- which is why every test below answers the server by hand, or does
 * not answer it at all, and presses keys in between. A fixture that answers before the first press
 * cannot see any of it.
 *
 * The screen is composed as it ships and driven through `Modifier.remote`, so what is proved is
 * what a viewer sees and what the screen asks for, not a helper's arithmetic beside it. What is
 * drawn is read back through [HeardScreen], which is also what says where the remote is: an
 * episode card is one stop that gathers its words and carries `selected` when the white border is
 * on it (`app/Spoken.kt`'s `saysItsWords`).
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class TitleKeysTest {
    @get:Rule val compose = createComposeRule()

    private val key = TitleKey("tv:9")

    /** The real words, so a card is read as a viewer hears it and not as a literal written here. */
    private val book: Map<String, String> by lazy {
        Json.parseToJsonElement(File(repoRoot(), "strings/en.json").readText()).jsonObject
            .mapValues { (_, word) -> word.jsonPrimitive.content }
    }

    private fun series(seasons: List<Long>, resume: Resume? = null) = TitleDetail(
        name = "A series",
        title = key,
        seasons = seasons.map { SeasonSummary(episodeCount = 10, number = it) },
        resume = resume,
    )

    /** A season's answer, as the server sends it: `/v1/titles/tv/9/seasons/<n>`. */
    private fun answer(number: Long, count: Int) = contractJson.encodeToString(
        Season.serializer(),
        Season(number = number, episodes = (1..count).map { Episode(name = "Name $it", number = it.toLong()) }),
    )

    private fun press(code: Int) {
        compose.onRoot().performKeyPress(KeyEvent(NativeKeyEvent(android.view.KeyEvent.ACTION_DOWN, code)))
        compose.waitForIdle()
    }

    private fun down() = press(android.view.KeyEvent.KEYCODE_DPAD_DOWN)
    private fun up() = press(android.view.KeyEvent.KEYCODE_DPAD_UP)
    private fun left() = press(android.view.KeyEvent.KEYCODE_DPAD_LEFT)
    private fun ok() = press(android.view.KeyEvent.KEYCODE_DPAD_CENTER)

    // The number in a card's words is wrapped in the bidirectional isolates `fill` puts around
    // every filled value, so what sits between "Episode" and the digits is not a space.
    private val anEpisode = Regex("""Episode[^0-9]*(\d+)""")

    /**
     * The episode cards on screen, by number, in the order a reader reaches them.
     *
     * A page of episodes draws a window and not the whole season: the card above the one the remote
     * is on, and the four below it (`NEAR`). So a season of seven read from its first card is five
     * cards on screen, and that is what a viewer sees.
     */
    private fun cards(): List<Int> = HeardScreen.words(compose.onRoot().fetchSemanticsNode())
        .mapNotNull { anEpisode.find(it.label)?.groupValues?.get(1)?.toInt() }

    /** The card the white border is on, which is the one OK would play. Empty when there is none. */
    private fun lit(): List<Int> = HeardScreen.under(compose.onRoot().fetchSemanticsNode())
        .filter { it.selected == true }
        .mapNotNull { anEpisode.find(it.label)?.groupValues?.get(1)?.toInt() }

    /** Puts the episodes page up with nothing answered yet, and the shell that is recording it. */
    private fun episodesPage(data: TitleDetail): Watching {
        val shell = Watching()
        compose.setContent { Episodes(shell, Seen(words = book), data) { } }
        compose.waitForIdle()
        return shell
    }

    /**
     * What every viewer saw on every season change: the season before it.
     *
     * The read of a season takes as long as the link is slow, and for the whole of it the cards were
     * the previous season's names, stills and descriptions, under a column already marking the new
     * season as chosen, with nothing on the page saying so. The page now draws the cards of the
     * season it is on and none at all while that season is being read, which is what this surface
     * already does for a page with nothing to show yet (`browse/Waiting.kt`).
     */
    @Test
    fun `no card of the season before it is drawn while a season is being read`() {
        val shell = episodesPage(series(listOf(1L, 2L)))
        assertEquals(listOf("/v1/titles/tv/9/seasons/1"), shell.asks)
        shell.answer(200, answer(1, 7))
        compose.waitForIdle()
        assertEquals("season 1 answered, so season 1 is drawn", (1..5).toList(), cards())

        // Season 2 is chosen and its answer is left in the air, which is where the viewer is.
        left()
        down()
        ok()
        assertEquals("the page asks the server for the season just chosen", 2, shell.asks.size)
        assertEquals("the cards of the season before it are still on screen", emptyList<Int>(), cards())

        shell.answer(200, answer(2, 4))
        compose.waitForIdle()
        assertEquals("season 2 answered, so season 2 is drawn", (1..4).toList(), cards())
        assertEquals("and the remote is on its first card", listOf(1), lit())
    }

    /**
     * The tail of the same thing, and the reason it is worth a test of its own: with the season
     * before it on screen, a held Down walks that list's bound -- every repeat of a held key reaches
     * `press` -- and OK then asks for an episode the chosen season may not have.
     *
     * `/watch/tv/9/2/7` out of season 1's seven, where season 2 has four. The server bounds the
     * magnitudes and checks the title, finds no source for an episode that does not exist, and the
     * viewer reads "this title cannot play right now" on a title that plays fine.
     */
    @Test
    fun `a season that has not answered plays nothing, however long Down is held`() {
        val shell = episodesPage(series(listOf(1L, 2L)))
        shell.answer(200, answer(1, 7))
        compose.waitForIdle()
        left()
        down()
        ok()

        // Down held through the read. A held key arrives as a repeat of `KeyDown` and every repeat
        // reaches `press`, so this is one key held, not six presses -- which is what it takes to
        // walk the stale list far enough to ask for an episode the chosen season has not got.
        repeat(6) { down() }
        ok()
        assertEquals("nothing is played out of a season that has not answered", emptyList<String>(), shell.went)

        shell.answer(200, answer(2, 4))
        compose.waitForIdle()
        ok()
        assertEquals("and what plays is the first card of the season that answered", listOf("/watch/tv/9/2/1"), shell.went)
    }

    /**
     * Down on a plain first open, before the first season has answered.
     *
     * The last index of a list with nothing in it is -1, so one press put the card at -1 and nothing
     * put it back: `placed` starts true for a series nobody has started, so the arrival's own guard
     * never ran. The column was then drawn a full card low, with an empty slot where episode 1
     * belongs and no card carrying the border that says where the remote is -- reachable by one
     * press on a slow link, with nothing on screen saying anything was loading.
     */
    @Test
    fun `down before the first season answers leaves the remote on its first card`() {
        val shell = episodesPage(series(listOf(1L)))
        down()
        shell.answer(200, answer(1, 3))
        compose.waitForIdle()

        assertEquals((1..3).toList(), cards())
        assertEquals("one card carries the border, and it is the first", listOf(1), lit())
        ok()
        assertEquals(listOf("/watch/tv/9/1/1"), shell.went)
    }

    /** Down still walks the cards and stops at the last, so the floor is a bound and not a brake. */
    @Test
    fun `down walks the cards and stops at the last`() {
        val shell = episodesPage(series(listOf(1L)))
        shell.answer(200, answer(1, 3))
        compose.waitForIdle()
        repeat(5) { down() }
        assertEquals(listOf(3), lit())
        ok()
        assertEquals(listOf("/watch/tv/9/1/3"), shell.went)
    }

    /** The page still opens on the episode the viewer is on, which is what the pair is read for. */
    @Test
    fun `the page opens on the episode the viewer left off at`() {
        val resume = Resume(episode = EpisodeRef(episode = 5, season = 2), positionS = 120)
        val shell = episodesPage(series(listOf(1L, 2L), resume))
        assertEquals(listOf("/v1/titles/tv/9/seasons/2"), shell.asks)
        shell.answer(200, answer(2, 7))
        compose.waitForIdle()

        assertEquals("the remote is on the episode the viewer is on", listOf(5), lit())
        ok()
        assertEquals(listOf("/watch/tv/9/2/5"), shell.went)
    }

    /**
     * A card an arrival leaves out of range is brought back into it.
     *
     * Held directly, because with the two above in place nothing reaches it: the cards empty on a
     * season change, the floor keeps the card off -1, and a season is read once per choosing. It is
     * a bound and not a fix, and it is the one line that catches a card out of range however it got
     * there -- a shorter list landing under a card that is already past its end draws a page with
     * nothing on it lit, which is the same screen the floor is for.
     */
    @Test
    fun `a card is read inside the list that arrived`() {
        assertEquals("nothing arrived: the first card, not -1", 0, cardIn(0 + 1, 0))
        assertEquals("a shorter list landed under the card", 3, cardIn(6, 4))
        assertEquals("a card the list has is left where it is", 2, cardIn(2, 4))
    }
}
