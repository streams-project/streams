package streams.phone

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import streams.shared.Brand

/**
 * What a screen reader can reach on a loaded Home, and in what order.
 *
 * This is the half note 228 could not see. Its tests prove every control carries words; on
 * 2026-11-13 two phones with two vendors' screen readers showed that the chip row, the row headers
 * and the tiles are never reached (note 239, section 3). [Reached] takes the order from the
 * toolkit's own accessibility delegate, which is the order a screen reader walks.
 *
 * Two things about the window decide whether this measures a phone or measures the toolkit's
 * defaults, and both were got wrong once:
 *
 * * **the screen**, because the reading depends on what is laid out. In the default 320 x 470 dp
 *   window the billboard alone fills the page and no row is composed at all, so a test at that
 *   size reports "no row is reachable" while measuring the window.
 * * **the status bar**, because the reading depends on where things are. Robolectric reports no
 *   inset, every box starts at y = 0, and an order that is decided by a tie on `top` comes out
 *   right here and wrong on every phone. That is exactly what happened: see [STATUS_BAR]. Every
 *   reading below is taken with a status bar, and one test holds the two against each other.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34], qualifiers = "w411dp-h914dp-xxhdpi")
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class HomeReachTest {
    @get:Rule val compose = createComposeRule()

    /** A loaded Home on a phone with a status bar, read as a screen reader is walked through it. */
    private fun readingOnHome(inset: Int = STATUS_BAR): List<Stop> {
        val seen = homeSeen()
        compose.setContent { Brand { Home(Showing(seen), seen) } }
        statusBar(compose, inset)
        return Reached.of(compose.onRoot().fetchSemanticsNode())
    }

    @Test
    fun `the reading does not depend on the status bar`() {
        // The guard for the defect that made the first fix hollow. An order held up by where a box
        // happens to start reads correctly under a window with no inset and wrongly under a window
        // with one, and every phone has one. The two readings have to be the same reading.
        val seen = homeSeen()
        compose.setContent { Brand { Home(Showing(seen), seen) } }
        val bare = Reached.of(compose.onRoot().fetchSemanticsNode()).words()
        statusBar(compose)
        val withBar = Reached.of(compose.onRoot().fetchSemanticsNode()).words()
        assertEquals("a status bar of ${STATUS_BAR} px changes what a reader is handed", bare, withBar)
    }

    @Test
    fun `a reader reaches the top of the page before its middle`() {
        val reading = readingOnHome()
        val name = reading.firstWith(words.getValue("nav.home"))
        val bell = reading.firstWith(words.getValue("notifications.title"))
        val chip = reading.firstWith(words.getValue("nav.series"))
        val billboard = reading.firstWith(lead.name)
        assertTrue("the page's own name is never read: ${reading.words()}", name >= 0)
        assertTrue("the bell is never reached: ${reading.words()}", bell >= 0)
        assertTrue("the chip row is never reached: ${reading.words()}", chip >= 0)
        assertTrue("the billboard is never reached: ${reading.words()}", billboard >= 0)
        // The header and the chips are drawn above the billboard and pinned there, so a reader
        // walked down the page reaches them first.
        assertTrue(
            "the chip row is drawn above the billboard and read after it: chips at stop $chip of " +
                "${reading.size}, billboard at $billboard: ${reading.words()}",
            chip < billboard,
        )
        assertTrue(
            "the page's name is drawn above the billboard and read after it, at stop $name: " +
                reading.words(),
            name < billboard,
        )
    }

    @Test
    fun `a reader is walked down the page, in the order it is drawn`() {
        // The first stops, named, because "before the billboard" is satisfied by any order that
        // happens to put them first and this is the order a viewer sees: the page's name and the
        // bell on the top bar, the five chips under it, then the billboard with its two buttons.
        val reading = readingOnHome().take(10).map { it.label }
        assertEquals(
            listOf(
                words.getValue("nav.home"),
                words.getValue("notifications.title"),
                words.getValue("nav.series"),
                words.getValue("nav.films"),
                words.getValue("nav.games"),
                words.getValue("nav.new"),
                words.getValue("nav.categories"),
                lead.name,
                words.getValue("common.play"),
                words.getValue("list.add"),
            ),
            reading.map { it.substringBefore(",") },
        )
    }

    @Test
    fun `a reader reaches a row's name and its tiles`() {
        val reading = readingOnHome()
        val row = homeScreen.sections[1]
        val rowName = reading.firstWith(row.title.orEmpty())
        val tile = reading.firstWith(row.tiles.orEmpty().first().name)
        assertTrue("the row header is never reached: ${reading.words()}", rowName >= 0)
        assertTrue("the first tile of the first row is never reached: ${reading.words()}", tile >= 0)
    }

    @Test
    fun `the bar at the foot of the page is read after the chips at its head`() {
        val reading = readingOnHome()
        val search = reading.firstWith(words.getValue("nav.search"))
        val chip = reading.firstWith(words.getValue("nav.series"))
        assertTrue("the bar is never reached: ${reading.words()}", search >= 0)
        assertTrue(
            "the bar is drawn at the foot of the page and the chips at its head, and the bar is " +
                "read first: bar at $search, chips at $chip: ${reading.words()}",
            search > chip,
        )
    }

    @Test
    fun `every control of a loaded home is reachable`() {
        val reading = readingOnHome()
        // What a viewer sees on the first screen of a loaded Home.
        val wanted = listOf(
            words.getValue("nav.home"),
            words.getValue("notifications.title"),
            words.getValue("nav.series"),
            words.getValue("nav.films"),
            words.getValue("nav.games"),
            words.getValue("nav.new"),
            words.getValue("nav.categories"),
            lead.name,
            words.getValue("common.play"),
            words.getValue("list.add"),
        )
        val missing = wanted.filter { reading.firstWith(it) < 0 }
        assertEquals("a reader cannot reach these: ${reading.words()}", emptyList<String>(), missing)
    }
}
