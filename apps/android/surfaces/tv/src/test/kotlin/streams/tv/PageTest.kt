package streams.tv

import contract.Screen
import contract.ScreenName
import contract.Section
import contract.SectionKind
import contract.Tile
import contract.TitleKey
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The first tests this module has had. Each one is named after the defect it keeps shut.
 *
 * They run on the plain functions in `browse/Page.kt`, with no screen, no device and no toolkit, so
 * `./gradlew :surfaces:tv:testDebugUnitTest` proves them in seconds on any machine.
 */
class PageTest {
    private fun tile(name: String) = Tile(name = name, title = TitleKey("movie:1"))

    private fun row(id: String, title: String, kind: SectionKind = SectionKind.ROW) =
        Section(id = id, kind = kind, title = title, tiles = listOf(tile("A title")))

    private fun screen(name: String, sections: List<Section>) =
        Screen(name = ScreenName(name), sections = sections, ttlS = 60, cursor = "c1")

    /**
     * The regression a reviewer caught on 2026-11-11: the core never asks for the games screen and
     * never clears the answer it holds, so standing on the Games tab the surface had the previous
     * tab's answer in hand and drew its rows under the Games name.
     */
    @Test
    fun `games draws no rows from the page before it`() {
        val held = screen("films", listOf(row("r1", "Films row")))
        val rows = rowsOn("games", "games", held.name.value, held.sections, emptyList())
        assertTrue("Games must be empty whatever answer is held", rows.isEmpty())
    }

    /** The same leak through the hero: a page with no rows of its own has no hero of its own. */
    @Test
    fun `games draws no hero from the page before it`() {
        val held = screen("films", listOf(row("h1", "Hero", SectionKind.HERO)))
        assertNull(heroOn("games", held.sections))
    }

    /** And the rows read past the first twelve: Games never grows from another page's cursor. */
    @Test
    fun `games takes no rows read past the first answer`() {
        val held = screen("films", listOf(row("r1", "Films row")))
        val extra = listOf(row("r2", "More films"))
        assertTrue(rowsOn("games", "games", held.name.value, held.sections, extra).isEmpty())
        assertTrue(!answerIsThisPage("games", "games", "films"))
    }

    /** A page whose answer is still the tab before it draws the answer's rows but never grows them. */
    @Test
    fun `a page does not grow from another page's answer`() {
        val held = screen("films", listOf(row("r1", "Films row")))
        val extra = listOf(row("r2", "Series row"))
        val rows = rowsOn("series", "series", held.name.value, held.sections, extra)
        assertEquals(listOf("r1"), rows.map { it.id })
        assertTrue(!answerIsThisPage("series", "series", held.name.value))
    }

    /** Home and the kids home name no screen on the route, so the answer's own name stands for it. */
    @Test
    fun `home grows from the answer it was given`() {
        val held = screen("home", listOf(row("r1", "Home row")))
        val extra = listOf(row("r2", "Read past the first twelve"))
        val rows = rowsOn("home", null, held.name.value, held.sections, extra)
        assertEquals(listOf("r1", "r2"), rows.map { it.id })
        assertTrue(answerIsThisPage("home", null, held.name.value))
    }

    /** A row with no titles in it is an empty line under a name, so it is not drawn. */
    @Test
    fun `a row with no tiles is not drawn`() {
        val empty = Section(id = "r0", kind = SectionKind.ROW, title = "Nothing", tiles = emptyList())
        val held = screen("home", listOf(empty, row("r1", "Home row")))
        assertEquals(listOf("r1"), rowsOn("home", null, held.name.value, held.sections, emptyList()).map { it.id })
    }

    /** Only the kinds this surface draws as a row of tiles are rows. A hero is not one of them. */
    @Test
    fun `only row kinds are drawn as rows`() {
        val held = screen(
            "home",
            listOf(
                row("h1", "Hero", SectionKind.HERO),
                row("r1", "Row"),
                row("r2", "Ranked", SectionKind.RANKED),
                row("r3", "Continue", SectionKind.CONTINUE),
                row("r4", "Grid", SectionKind.GRID),
            ),
        )
        val rows = rowsOn("home", null, held.name.value, held.sections, emptyList())
        assertEquals(listOf("r1", "r2", "r3", "r4"), rows.map { it.id })
        assertEquals("A title", heroOn("home", held.sections)?.name)
    }
}
