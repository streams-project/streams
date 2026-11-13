package streams.tv

import androidx.compose.ui.unit.dp
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import streams.shared.ref

/**
 * The television surface's measured numbers, each held against the derivation or the capture it
 * came from. The Android twin of `apps/apple/Tests/TelevisionTests/Numbers.swift`.
 *
 * Note 215 records "no test" against every number below. `./check` runs no gradle
 * (`build.gradle.kts:20`), so until a person or CI runs this module any of them could move by a
 * hundred reference pixels and every check in the repository would stay green.
 *
 * The rule these follow, from that note and from a defect a reviewer caught on 2026-11-11 (a
 * focused logo box whose stated derivation gave a different number from the one the code used):
 * **a number is never asserted against itself.** `assertEquals(328, STEP)` passes forever. Each
 * test here either
 *  * derives the number from another number in another place -- the pitch from the tile and the
 *    gutter, the card's text width from the card and its inset, the hero from the page and its
 *    margins -- or
 *  * holds a raw captured measurement against the capture that recorded it, which for this surface
 *    is `reference/netflix-on-the-tv.md`'s reference table and the citation comments in `browse/Rows.kt`.
 *
 * Each test says which of the two it is and why. `Sources.kt` says why they read the source as
 * text and what that costs a rewrite.
 */
class NumbersTest {
    private val root = repoRoot()
    private val rowsSaid = File(root, "$MAIN/browse/Rows.kt").readText()
    private val rows = withoutComments(rowsSaid)
    private val hero = withoutComments(File(root, "$MAIN/browse/HeroCard.kt").readText())
    private val reference = File(root, REFERENCE).readText()

    /**
     * The row screen's table in `reference/netflix-on-the-tv.md:49`, read a row at a time rather
     * than as one sentence about it. Order: closed width, closed height, focused width, focused
     * height, gutter, pitch -- so an index here moves only when that table does.
     *
     * Until 2026-11-12 this read a sentence in the television page's README, which also stated
     * what *that page* drew, and these numbers were held equal to it. Two surfaces were pinned to
     * each other through a file belonging to neither, and when the page was deleted the pin went
     * with it. The capture is the only thing both ever answered to.
     */
    private val captured =
        tileRow("closed tile") + tileRow("focused tile") + listOf(pxRow("gutter"), pxRow("pitch"))

    /** How far a surface may sit from the capture, as the reference states it itself. */
    private val tolerance =
        number("""within \*\*(\d+) px\*\*""", reference, "the tolerance $REFERENCE allows")

    /**
     * The closed tile is the size the capture recorded, to the 2 px the reference table claims.
     *
     * A raw measurement with no derivation, so this is the citation half of the rule: the numbers in
     * `Rows.kt`'s `Poster` have to equal the numbers `reference/netflix-on-the-tv.md:49` says this project
     * draws, and those have to stay within the 2 px of the capture that the same line claims. A
     * tile edited in the code and not in the table fails. A table edited to excuse a tile fails the
     * 2 px claim it makes in its own sentence.
     */
    @Test
    fun `the closed tile is the captured tile within the two pixels the reference claims`() {
        val (wide, high) = size("Poster")
        assertNear("closed tile width", captured[0], wide)
        assertNear("closed tile height", captured[1], high)
    }

    /** The same for the card the focused tile opens into. */
    @Test
    fun `the focused card is the captured card within the two pixels the reference claims`() {
        val (wide, high) = size("OpenCard")
        assertNear("focused card width", captured[2], wide)
        assertNear("focused card height", captured[3], high)
    }

    /**
     * The pitch is the tile plus the gutter between tiles. A derivation, and the one worth having
     * on `STEP`: 328 is not a measurement, it is 311 and 17 added up, and a tile resized without
     * the pitch following leaves a row with the wrong gap in it at every tile.
     */
    @Test
    fun `the tile pitch is the tile plus the gutter beside it`() {
        val step = number("""private const val STEP = (\d+)""", rows, "STEP")
        val (tile, _) = size("Poster")
        val gutter = captured[4]
        assertEquals("the pitch is the tile plus the gutter", tile + gutter, step)
        assertEquals("and the reference records the same pitch", captured[5], step)
    }

    /**
     * The closed row starts on the page margin and steps by the pitch, so the tile the viewer has
     * come from peeks in by exactly the gutter.
     *
     * A derivation with a consequence worth stating: the previous tile is drawn at
     * `PAGE_EDGE - STEP` (`Rows.kt:173`), so its right edge lands at `PAGE_EDGE - gutter`. That is
     * the sliver the reference shows at the left of a closed row, and it is a result of the pitch
     * rather than a number anybody chose. Writing the peek as its own constant would be the drift
     * this catches.
     */
    @Test
    fun `the tile the viewer came from peeks in by exactly the gutter`() {
        val step = number("""private const val STEP = (\d+)""", rows, "STEP")
        val (tile, _) = size("Poster")
        assertTrue("the row has to start on the margin", rows.contains("(PAGE_EDGE + index * STEP).ref"))
        assertTrue("and step back by one pitch for the tile before", rows.contains("(PAGE_EDGE - STEP).ref"))
        assertEquals(
            "the peeking tile's right edge sits one gutter short of the margin",
            PAGE_EDGE - captured[4],
            PAGE_EDGE - step + tile,
        )
    }

    /**
     * The first poster beside the open card clears it by the gutter: the literal at `Rows.kt:137`
     * is the page margin, the card's own width and the gutter, added up as reference pixels.
     *
     * A derivation, and the one the rule was written for. The card's width appears twice in that
     * file -- once as the card's size and once inside this sum -- and nothing but this test ties
     * them together. A card widened to 920 with the sum left at 904 would overlap the poster by 16
     * reference pixels, which looks like a drawing bug and is a transcription one.
     *
     * The gutter in this sum is not the gutter the closed row uses: the row steps by the capture's
     * 17 and this sum clears the card by 19. Both sit inside the tolerance the reference allows, so
     * this holds the sum to the capture rather than to the row, and the two px between them are
     * left alone. Narrowing them is a drawing change on a surface nobody ran today, and it belongs
     * to whoever owns the television's geometry, not to its test.
     */
    @Test
    fun `the poster beside the open card clears it by the gutter`() {
        val sum = theOne(Regex("""\(PAGE_EDGE \+ (\d+) \+ (\d+)\)\.ref"""), rows, "the next poster's x in Rows.kt")
        val (card, _) = size("OpenCard")
        assertEquals("the sum has to use the card's own width", card, sum.groupValues[1].toInt())
        assertNear("the gutter that clears the open card", captured[4], sum.groupValues[2].toInt())
    }

    /**
     * The hero is the page less its two margins. The direct twin of
     * `apps/apple/Tests/TelevisionTests/Numbers.swift:46`, and the reason the margin is a constant.
     *
     * The number-bearing half is the written form: `HeroCard.kt:54` has to still say
     * `(1920 - 2 * PAGE_EDGE).ref` and not the 888 dp it comes to. A rewrite that writes the answer
     * loses the reason, and the next person to change the margin finds the hero does not follow.
     */
    @Test
    fun `the hero is the page less its two margins`() {
        assertTrue(
            "the hero's width has to stay written as the page less its two margins",
            hero.contains("(1920 - 2 * PAGE_EDGE).ref"),
        )
        assertEquals(888.dp, (1920 - 2 * PAGE_EDGE).ref)
        assertEquals(1920.ref - PAGE_EDGE.ref - PAGE_EDGE.ref, (1920 - 2 * PAGE_EDGE).ref)
    }

    /**
     * The card's name is as wide as the card less its inset on each side, and the comment that says
     * so names the same card.
     *
     * This is the exact defect a reviewer caught on 2026-11-11, and the only place in this surface
     * where a derivation is written out in prose next to the number it produced. Three things have
     * to agree: the comment's stated card width, the card's actual size, and the text width against
     * the inset the same line applies. `Rows.kt:100-102` and `:103`.
     */
    @Test
    fun `the open card's name is the card less its inset on each side`() {
        val said = theOne(Regex("""The card is (\d+) wide"""), rowsSaid, "the card's width in Rows.kt's comment")
        val inset = theOne(Regex("""less the\s*//\s*(\d+) inset on each side"""), rowsSaid, "the stated inset")
        val text = theOne(Regex("""\.padding\(start = (\d+)\.ref, bottom = \d+\.ref\)\.width\((\d+)\.ref\)"""), rows, "the name's box")
        val (card, _) = size("OpenCard")
        assertEquals("the comment has to name the card it is beside", card, said.groupValues[1].toInt())
        assertEquals("the stated inset has to be the one applied", inset.groupValues[1].toInt(), text.groupValues[1].toInt())
        assertEquals(
            "the name is the card less twice its inset",
            card - 2 * text.groupValues[1].toInt(),
            text.groupValues[2].toInt(),
        )
    }

    /**
     * The citation beside the card still names the capture it came from, with the numbers that
     * capture holds.
     *
     * The card's 904 x 508 is a raw measurement, so this is the citation half of the rule. The
     * comment cites `row_screen.focused_tile` at 906 x 510 and says it was matched to 904 x 508 in
     * gap 9; `reference/netflix-on-the-tv.md:49` has to record the same 906 x 510. A comment rewritten to point
     * at a different capture, or a capture quoted with different numbers, fails here.
     */
    @Test
    fun `the citation beside the card names the capture the reference records`() {
        assertTrue(
            "the citation has to keep naming its capture and its gap",
            rowsSaid.contains("row_screen.focused_tile") &&
                rowsSaid.contains("tv-geometry-1920x1080.json") &&
                rowsSaid.contains("00-gaps.md gap 9"),
        )
        val quoted = theOne(Regex("""focused_tile` (\d+) x (\d+)"""), rowsSaid, "the capture's own numbers")
        assertEquals(captured[2], quoted.groupValues[1].toInt())
        assertEquals(captured[3], quoted.groupValues[2].toInt())
        val matched = theOne(Regex("""matched to (\d+) x (\d+) in 00-gaps\.md"""), rowsSaid, "what it was matched to")
        val (card, high) = size("OpenCard")
        assertEquals("the citation has to name the card this surface draws", card, matched.groupValues[1].toInt())
        assertEquals(high, matched.groupValues[2].toInt())
    }

    /**
     * The open card sits in the band the reference pins the focused row to.
     *
     * A raw measurement held against its capture: `reference/netflix-on-the-tv.md:70` records the
     * band as y 199 to 708, found in 24 of 24 sampled frames. The card's own y and height are in
     * `Rows.kt:87` and `:88`, so the claim is checkable rather than remembered.
     */
    @Test
    fun `the open card sits in the band the reference pins the focused row to`() {
        val band = theOne(
            Regex("""focus outline sits at \*\*y (\d+)-(\d+)\*\*"""),
            reference,
            "the focused band in $REFERENCE",
        )
        val top = band.groupValues[1].toInt()
        val bottom = band.groupValues[2].toInt()
        val y = number("""fun OpenCard\([\s\S]*?\.offset\(PAGE_EDGE\.ref, (\d+)\.ref\)""", rows, "the open card's y")
        val (_, high) = size("OpenCard")
        assertNear("the card's top against the band's", top, y)
        assertNear("the card's bottom against the band's", bottom, y + high)
    }

    /**
     * The page margin is the number the reference table records this project deciding on, and every
     * page reaches it through the constant.
     *
     * 72 is a decision and not a measurement: `reference/netflix-on-the-tv.md:23` records the reference's own
     * safe margin as 68 px at 1920 and this project choosing 72 in note 188. So this is the
     * citation half of the rule for the number, and for the constant it is the relationship the
     * number participates in -- no page writes the margin as a literal, which is what makes the
     * number changeable at all. On 2026-11-12 the scan found zero bare margin literals across the
     * surface; `.height(72.ref)` in `watch/Moments.kt` and `.size(72.ref)` in `watch/Watch.kt` are
     * a height and a square, not a margin, so only the first argument of `offset` is read.
     */
    @Test
    fun `the page margin is the recorded decision and no page writes it again`() {
        val measuredAt = number("""reference pixel\*\* at (\d+)x""", reference, "the width $REFERENCE measures at")
        val chosen = number("""This project uses \*\*(\d+)\*\*""", reference, "the margin decision in $REFERENCE")
        assertEquals(1920, measuredAt)
        assertEquals("the margin has to be the one the reference records", chosen, PAGE_EDGE)

        val written = mutableListOf<String>()
        File(root, MAIN).walkTopDown()
            .filter { it.isFile && it.extension == "kt" }
            .sortedBy { it.path }
            .forEach { file ->
                val text = withoutComments(file.readText())
                for (found in Regex("""\boffset\s*\(""").findAll(text)) {
                    val x = firstArgument(text, found.range.last).trim()
                    if (Regex("""\(?\s*($PAGE_EDGE|$SUB_EDGE)\s*\)?\s*\.ref""").matches(x)) {
                        written += "${file.toRelativeString(File(root, MAIN))}:${lineAt(text, found.range.first)} $x"
                    }
                }
            }
        assertEquals("every page margin has to go through the constant", emptyList<String>(), written)
    }

    /** The two reference pixels the table's own sentence allows between a capture and this page. */
    private fun assertNear(what: String, captured: Int, drawn: Int) = assertTrue(
        "$what: $drawn against the captured $captured is more than the $tolerance px $REFERENCE allows",
        kotlin.math.abs(captured - drawn) <= tolerance,
    )

    /** The size a named part of `Rows.kt` draws itself at, as `(wide, high)` reference pixels. */
    /** One `| <name> | **W x H**, ... |` row of the reference's row-screen table. */
    private fun tileRow(name: String): List<Int> =
        theOne(Regex("""\| $name \| \*\*(\d+) x (\d+)\*\*"""), reference, "the $name row in $REFERENCE")
            .groupValues.drop(1).map { it.toInt() }

    /** One `| <name> | **N px** |` row of the same table. */
    private fun pxRow(name: String): Int =
        theOne(Regex("""\| $name \| \*\*(\d+) px\*\*"""), reference, "the $name row in $REFERENCE")
            .groupValues[1].toInt()

    private fun size(part: String): Pair<Int, Int> {
        val found = theOne(
            Regex("""fun $part\([\s\S]*?\.size\((\d+)\.ref,\s*(\d+)\.ref\)"""),
            rows,
            "$part's own size in Rows.kt",
        )
        return found.groupValues[1].toInt() to found.groupValues[2].toInt()
    }

    private fun number(pattern: String, text: String, what: String) =
        theOne(Regex(pattern), text, what).groupValues[1].toInt()

    private companion object {
        const val MAIN = "apps/android/surfaces/tv/src/main/kotlin/streams/tv"

        /**
         * The television page's README, which is where this project's row-screen capture and the
         * numbers taken from it are written down. Read rather than copied: a test that copied them
         * would be two transcriptions of one measurement, which is the thing being guarded against.
         */
        const val REFERENCE = "reference/netflix-on-the-tv.md"
    }
}
