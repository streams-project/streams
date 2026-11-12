package streams.phone

import contract.More
import contract.Screen
import contract.ScreenName
import contract.Section
import contract.SectionKind
import contract.Tile
import contract.TitleKey
import contract.contractJson
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The paging in `browse/Paging.kt`, tested on the plain parts: [Depth] against a shell that holds
 * its answer ([Asked]), and [pageStamp] against two answers.
 *
 * Each test is named after the defect it keeps shut. None of them touches a screen, a device or the
 * toolkit, so `:surfaces:phone:testDebugUnitTest` proves them in seconds.
 *
 * The shell answering only when a test says to is what gives these tests their teeth: every defect
 * here lives in the gap between an ask going out and its answer coming back, and a shell that
 * answered inside `ask` would close that gap and every test would pass against the broken code.
 */
class PagingTest {
    private fun tile(name: String, key: String) = Tile(name = name, title = TitleKey(key))

    private fun row(id: String, cursor: String?, tiles: List<Tile> = listOf(tile("A", "movie:1"))) =
        Section(id = id, kind = SectionKind.ROW, title = id, tiles = tiles, cursor = cursor)

    /** What the server would send back: a part of a row, and the cursor for the part after it. */
    private fun part(cursor: String?, tiles: List<Tile>): String =
        contractJson.encodeToString(More.serializer(), More(cursor = cursor, tiles = tiles))

    /**
     * The defect watched on a device: two requests for `page-...-36` and two for `-48` in one scroll
     * to the bottom. A fling delivers more than one scroll position before an answer lands, and a
     * reading that does not record that it is already asking spends the same cursor twice.
     */
    @Test
    fun `a row flung twice before the answer lands asks its cursor once`() {
        val asked = Asked()
        val depth = Depth(asked)
        val row = row("r1", "c1")

        depth.reach(row, 15, 14)
        depth.reach(row, 15, 14)

        assertEquals(listOf("/v1/more/c1"), asked.asks)
        assertEquals("the first ask is still in flight", 1, asked.unanswered)
    }

    /** The part that arrives carries the cursor for the next one, and that is what is asked for. */
    @Test
    fun `the next part is asked for with the cursor the last part carried`() {
        val asked = Asked()
        val depth = Depth(asked)
        val row = row("r1", "c1")

        depth.reach(row, 15, 14)
        asked.answer(200, part("c2", listOf(tile("B", "movie:2"))))
        depth.reach(row, 16, 15)

        assertEquals(listOf("/v1/more/c1", "/v1/more/c2"), asked.asks)
    }

    /**
     * A server that refused once is not asked the same thing again. Asking on every frame against a
     * server that has already said no is a loop, and it was reachable: the end of a read wrote the
     * cursor back whatever the answer was.
     */
    @Test
    fun `a row that was refused stops asking`() {
        val asked = Asked()
        val depth = Depth(asked)
        val row = row("r1", "c1")

        depth.reach(row, 15, 14)
        asked.answer(500, "")
        depth.reach(row, 15, 14)
        depth.reach(row, 15, 14)

        assertEquals(listOf("/v1/more/c1"), asked.asks)
        assertEquals("the refused part is not held as tiles", listOf("A"), depth.tiles(row).map { it.name })
    }

    /** A row the server handed no cursor for is a row that is whole: it asks for nothing, ever. */
    @Test
    fun `a row with no cursor asks for nothing`() {
        val asked = Asked()
        val depth = Depth(asked)

        depth.reach(row("r1", null), 15, 14)

        assertTrue("a whole row must not reach the network", asked.asks.isEmpty())
    }

    /**
     * The read-ahead window: a row reads the next part once the viewer is within seven titles of its
     * end, and not before. Seven is two rows' worth of sideways travel, measured off the reference's
     * 3.29 tiles a row (`Paging.kt:44`). A row with nothing in it reads nothing at all, which is the
     * state a row is in for the frame before its first tiles arrive.
     */
    @Test
    fun `a row reads ahead only inside the last seven titles`() {
        val asked = Asked()
        val depth = Depth(asked)
        val row = row("r1", "c1")

        depth.reach(row, 0, 0)
        assertTrue("a row holding nothing reads nothing", asked.asks.isEmpty())

        depth.reach(row, 15, 7)
        assertTrue("eight titles from the end is not near enough", asked.asks.isEmpty())

        depth.reach(row, 15, 8)
        assertEquals(listOf("/v1/more/c1"), asked.asks)
    }

    /**
     * Parts are merged on the title's key, not on the tile: the same title can arrive twice when a
     * part overlaps the one before it, and a lazy row with the same key twice throws.
     */
    @Test
    fun `a title that arrives twice is held once`() {
        val asked = Asked()
        val depth = Depth(asked)
        val row = row("r1", "c1", listOf(tile("A", "movie:1"), tile("B", "movie:2")))

        depth.reach(row, 15, 14)
        asked.answer(200, part(null, listOf(tile("B again", "movie:2"), tile("C", "movie:3"))))

        assertEquals(listOf("A", "B", "C"), depth.tiles(row).map { it.name })
    }

    /**
     * One reading a row, not one for the page. Held as a single value for every row, the row the
     * viewer flung second would have asked with the first row's cursor, which is the wrong part of
     * the wrong row.
     */
    @Test
    fun `two rows read with their own cursors`() {
        val asked = Asked()
        val depth = Depth(asked)
        val first = row("r1", "r1c1")
        val second = row("r2", "r2c1")

        depth.reach(first, 15, 14)
        depth.reach(second, 15, 14)
        assertEquals(listOf("/v1/more/r1c1", "/v1/more/r2c1"), asked.asks)

        asked.answer(200, part("r1c2", listOf(tile("C", "movie:3"))))
        asked.answer(200, part("r2c2", listOf(tile("D", "movie:4"))))

        depth.reach(first, 16, 15)
        depth.reach(second, 16, 15)
        assertEquals(
            listOf("/v1/more/r1c1", "/v1/more/r2c1", "/v1/more/r1c2", "/v1/more/r2c2"),
            asked.asks,
        )
        assertEquals(listOf("A", "C"), depth.tiles(first).map { it.name })
        assertEquals(listOf("A", "D"), depth.tiles(second).map { it.name })
    }

    private fun screen(cursor: String?, sections: List<Section>) =
        Screen(name = ScreenName("home"), sections = sections, ttlS = 60, cursor = cursor)

    /**
     * The stamp names the answer a page's read parts belong to. Two answers that are the same answer
     * must stamp the same, or every draw would throw away the parts read on the one before and start
     * the page's paging again.
     */
    @Test
    fun `the same answer stamps the same`() {
        val held = screen("c1", listOf(row("r1", "c1")))

        assertEquals(pageStamp("home", held), pageStamp("home", held.copy()))
    }

    /**
     * And a page built again stamps differently, on either half of what it carries. Keyed on the
     * route alone, a refreshed Home kept every row's old cursor while the rows went back to the
     * fifteen tiles the answer carries, so a row either stopped growing or asked again for tiles it
     * was already showing (`Paging.kt:62`).
     */
    @Test
    fun `an answer built again stamps differently`() {
        val held = screen("c1", listOf(row("r1", "c1")))

        assertNotEquals(pageStamp("home", held), pageStamp("home", held.copy(cursor = "c9")))
        assertNotEquals(
            pageStamp("home", held),
            pageStamp("home", held.copy(sections = held.sections + row("r2", null))),
        )
        assertNotEquals(pageStamp("home", held), pageStamp("home", null))
    }
}
