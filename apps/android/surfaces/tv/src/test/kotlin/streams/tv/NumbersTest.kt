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
 * Note 215 recorded "no test" against every number below, because `./check` ran no gradle and
 * any of them could move by a hundred reference pixels with every check still green. Since
 * 2026-11-13 `./check` runs these through `platform/tools/check_android.py`.
 *
 * The rule these follow, from that note and from a defect a reviewer caught on 2026-11-11 (a
 * focused logo box whose stated derivation gave a different number from the one the code used):
 * **a number is never asserted against itself.** `assertEquals(328, STEP)` passes forever. Each
 * test here either
 *  * derives the number from another number in another place -- the pitch from the tile and the
 *    gutter, the card's text width from the card and its inset, the hero from the page and its
 *    margins -- or
 *  * holds a raw captured measurement against the capture that recorded it, which for this surface
 *    is `reference/netflix-on-the-tv.md`'s reference table and the citation comments in
 *    `browse/Cards.java`.
 *
 * Each test says which of the two it is and why. `Sources.kt` says why they read the source as
 * text and what that costs a rewrite.
 *
 * ## 2026-11-13: these read the Java that draws the screen, not the Compose that no longer does
 *
 * Until today every number below was read out of `browse/Rows.kt` and `browse/HeroCard.kt`. Those
 * are the two Compose files `browse/RowsView.java`, `browse/Cards.java` and `browse/HeroView.java`
 * replaced, and neither has had a code caller since the port; what kept them in the tree was this
 * file reading them. `Sources.kt:17-21` wrote down in advance what to do about that: "Re-pointing
 * each of these at the Java form is how a rewriter states, in the diff, that the number came
 * across." This is that diff. Every reference pin moved onto the live surface; none was spent.
 *
 * The reading is still the written form and not the compiled constant, for a reason worth stating:
 * these constants are package-private to `streams.tv.browse` and this test is `streams.tv`, so it
 * cannot name them. It does not need to. `src/test/java/streams/tv/browse/NumbersTest.java` holds
 * each compiled constant against the same written number this file reads, so the chain runs
 * reference -> written number -> constant the screen draws with, and no link in it asserts itself.
 */
class NumbersTest {
    private val root = repoRoot()
    private val cardsSaid = File(root, "$JAVA/browse/Cards.java").readText()
    private val cards = withoutComments(cardsSaid)
    private val rows = withoutComments(File(root, "$JAVA/browse/RowsView.java").readText())
    private val hero = withoutComments(File(root, "$JAVA/browse/HeroView.java").readText())
    private val page = withoutComments(File(root, "$JAVA/browse/BrowseView.java").readText())
    // The canvas moved to `shared/Scale.java` on 2026-11-13, with the halving it derives (note 251).
    private val canvas = withoutComments(File(root, "$SHARED_JAVA/Scale.java").readText())
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
     * A raw measurement with no derivation, so this is the citation half of the rule: the numbers
     * `Cards.Poster` draws itself at have to equal the numbers `reference/netflix-on-the-tv.md:49`
     * says this project draws, and those have to stay within the 2 px of the capture that the same
     * line claims. A tile edited in the code and not in the table fails. A table edited to excuse a
     * tile fails the 2 px claim it makes in its own sentence.
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
        val (wide, high) = size("Open")
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
        val step = number("""\bSTEP = (\d+);""", rows, "STEP")
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
     * `Round.PAGE_EDGE - STEP` (`RowsView.java:283`), so its right edge lands at
     * `PAGE_EDGE - gutter`. That is the sliver the reference shows at the left of a closed row, and
     * it is a result of the pitch rather than a number anybody chose. Writing the peek as its own
     * constant would be the drift this catches.
     */
    @Test
    fun `the tile the viewer came from peeks in by exactly the gutter`() {
        val step = number("""\bSTEP = (\d+);""", rows, "STEP")
        val (tile, _) = size("Poster")
        assertTrue("the row has to start on the margin", rows.contains("Round.PAGE_EDGE + slot * STEP"))
        assertTrue("and step back by one pitch for the tile before", rows.contains("Round.PAGE_EDGE - STEP"))
        assertEquals(
            "the peeking tile's right edge sits one gutter short of the margin",
            PAGE_EDGE - captured[4],
            PAGE_EDGE - step + tile,
        )
    }

    /**
     * The first poster beside the open card clears it by the gutter: `RowsView.Open.AHEAD_X` is the
     * page margin, the card's own width and the gutter, added up as reference pixels.
     *
     * A derivation, and the one the rule was written for. The Compose this replaced wrote the
     * card's width into that sum as a second literal, and nothing but a test tied the two together;
     * the Java names `Cards.Open.WIDE` inside the sum, so this holds the form that makes the
     * transcription impossible rather than the transcription.
     *
     * The gutter in this sum is not the gutter the closed row uses: the row steps by the capture's
     * 17 and this sum clears the card by 19. Both sit inside the tolerance the reference allows, so
     * this holds the sum to the capture rather than to the row, and the two px between them are
     * left alone. Narrowing them is a drawing change on a surface nobody ran today, and it belongs
     * to whoever owns the television's geometry, not to its test.
     */
    @Test
    fun `the poster beside the open card clears it by the gutter`() {
        val sum = theOne(
            Regex("""AHEAD_X = Round\.PAGE_EDGE \+ Cards\.Open\.WIDE \+ (\d+);"""),
            rows,
            "the next poster's x in RowsView.java",
        )
        assertNear("the gutter that clears the open card", captured[4], sum.groupValues[1].toInt())
    }

    /**
     * The hero is the page less its two margins. The direct twin of
     * `apps/apple/Tests/TelevisionTests/Numbers.swift:46`, and the reason the margin is a constant.
     *
     * The number-bearing half is the written form: `HeroView.java` has to still say
     * `Ref.W - 2 * Round.PAGE_EDGE` and not the 888 dp it comes to. A rewrite that writes the
     * answer loses the reason, and the next person to change the margin finds the hero does not
     * follow. The 1920 it rests on is read out of `shared/Scale.java` and held against the width
     * `reference/netflix-on-the-tv.md` says it measured at, so the canvas is the capture's canvas.
     */
    @Test
    fun `the hero is the page less its two margins`() {
        assertTrue(
            "the hero's width has to stay written as the page less its two margins",
            hero.contains("Ref.W - 2 * Round.PAGE_EDGE") && page.contains("Ref.W - 2 * Round.PAGE_EDGE"),
        )
        val wide = number("""CANVAS_W = (\d+);""", canvas, "the reference canvas in Scale.java")
        assertEquals(
            "the canvas has to be the width the reference measured at",
            number("""reference pixel\*\* at (\d+)x""", reference, "the width $REFERENCE measures at"),
            wide,
        )
        assertEquals(888.dp, (wide - 2 * PAGE_EDGE).ref)
        assertEquals(wide.ref - PAGE_EDGE.ref - PAGE_EDGE.ref, (wide - 2 * PAGE_EDGE).ref)
    }

    /**
     * The card's name is as wide as the card less its inset on each side, and the comment that says
     * so names the same numbers.
     *
     * This is the exact defect a reviewer caught on 2026-11-11, and the only place in this surface
     * where a derivation is written out in prose next to the number it produced. Three things have
     * to agree: the width the prose states, the card's actual size against the inset the same card
     * applies, and the form the width is written in -- `WIDE - 2 * INSET` and not 856.
     * `Cards.java:122-130`.
     */
    @Test
    fun `the open card's name is the card less its inset on each side`() {
        val said = theOne(
            Regex("""(\d+) is the number at"""),
            cardsSaid,
            "the name's width in Cards.java's comment",
        )
        val inset = number("""\bINSET = (\d+);""", cards, "the open card's inset")
        val (card, _) = size("Open")
        assertTrue(
            "the width has to stay written as the subtraction and not as the answer",
            cards.contains("NAME_WIDE = WIDE - 2 * INSET"),
        )
        assertTrue(
            "and the same inset has to be the one that places the name",
            cards.contains("RefLayout.at(INSET, 0, NAME_WIDE,"),
        )
        assertEquals(
            "the comment has to name the width the card and its inset come to",
            card - 2 * inset,
            said.groupValues[1].toInt(),
        )
    }

    /**
     * The citation beside the card still names the capture it came from, with the numbers that
     * capture holds.
     *
     * The card's 904 x 508 is a raw measurement, so this is the citation half of the rule. The
     * comment cites `row_screen.focused_tile` at 906 x 510 and says it was matched to 904 x 508 in
     * gap 9; `reference/netflix-on-the-tv.md:49` has to record the same 906 x 510. A comment
     * rewritten to point at a different capture, or a capture quoted with different numbers, fails
     * here. It was carried from the Compose into `Cards.java`'s own header on the port rather than
     * summarised, which is why there is still something to read.
     */
    @Test
    fun `the citation beside the card names the capture the reference records`() {
        assertTrue(
            "the citation has to keep naming its capture and its gap",
            cardsSaid.contains("row_screen.focused_tile") &&
                cardsSaid.contains("tv-geometry-1920x1080.json") &&
                cardsSaid.contains("00-gaps.md} gap 9"),
        )
        val quoted = theOne(Regex("""focused_tile\} is[\s*]*(\d+) x (\d+)"""), cardsSaid, "the capture's own numbers")
        assertEquals(captured[2], quoted.groupValues[1].toInt())
        assertEquals(captured[3], quoted.groupValues[2].toInt())
        val matched = theOne(
            Regex("""matched to (\d+) x (\d+) in \{@code 00-gaps\.md\}"""),
            cardsSaid,
            "what it was matched to",
        )
        val (card, high) = size("Open")
        assertEquals("the citation has to name the card this surface draws", card, matched.groupValues[1].toInt())
        assertEquals(high, matched.groupValues[2].toInt())
    }

    /**
     * The open card sits in the band the reference pins the focused row to.
     *
     * A raw measurement held against its capture: `reference/netflix-on-the-tv.md:70` records the
     * band as y 199 to 708, found in 24 of 24 sampled frames. The card's own y is
     * `RowsView.Open.CARD_Y` and its height is `Cards.Open.HIGH`, so the claim is checkable rather
     * than remembered.
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
        val y = number("""\bCARD_Y = (\d+);""", rows, "the open card's y")
        val (_, high) = size("Open")
        assertNear("the card's top against the band's", top, y)
        assertNear("the card's bottom against the band's", bottom, y + high)
    }

    /**
     * The dim row below the open one sits where the reference measured it, on both screens it
     * appears on.
     *
     * Four numbers with no check at all until 2026-11-13: `RowsView.Next`'s two tile tops and two
     * name tops were held against `Rows.kt`'s `if (high) ... else ...` and against nothing else, so
     * deleting that file would have left them asserted against the test's own memory of them. The
     * reference holds all four, in two different sections, and neither says the number straight:
     *
     *  * the tiles, by what is left of them. The vertical rhythm line records the next row's tiles
     *    starting at 998 with 82 px visible, and the home screen records 79 px of the first row's
     *    tiles visible under the hero. On a canvas this tall, 1080 - 82 and 1080 - 79 are the two
     *    tops the code draws, and the second one is a derivation the reference never writes out.
     *  * the names, by their ink. The reference measured the band the glyphs cover (948-978 below a
     *    row, 951-975 below the hero); the code positions the text's box, which starts above the
     *    ink by the line's leading. So the check is that the box sits above the measured ink and
     *    within one quarter of the 32 reference pixel line it is set in -- which catches a name
     *    moved, swapped between the two screens or drifted, and does not pretend to a precision the
     *    two measurements do not have.
     */
    @Test
    fun `the dim row below sits where the reference measured it`() {
        val high = number("""reference pixel\*\* at \d+x(\d+)""", reference, "the height $REFERENCE measures at")
        val below = theOne(
            Regex("""next row title (\d+)-\d+, next row tiles from (\d+) with \*\*(\d+) px visible\*\*"""),
            reference,
            "the next row in $REFERENCE's vertical rhythm",
        )
        val first = theOne(
            Regex("""one row title at y (\d+)-\d+ and \*\*(\d+) px of the\s+first row's tiles"""),
            reference,
            "the first row under the hero in $REFERENCE",
        )
        val lowY = number("""\bLOW_Y = (\d+);""", rows, "the dim row's y under an open row")
        val highY = number("""\bHIGH_Y = (\d+);""", rows, "the dim row's y under the hero")
        assertNear("the dim row under an open row", below.groupValues[2].toInt(), lowY)
        assertNear("and what is left of its tiles", high - below.groupValues[3].toInt(), lowY)
        assertNear("the dim row under the hero", high - first.groupValues[2].toInt(), highY)

        val lead = number("""Type\.line\(context, (\d+), Type\.BOLD, Color\.WHITE\);\s*addView\(title, RefLayout\.at\(Round\.PAGE_EDGE, LOW_NAME_Y\)\)""", rows, "the dim row's name size") / 4
        val lowName = number("""\bLOW_NAME_Y = (\d+);""", rows, "the dim row's name under an open row")
        val highName = number("""\bHIGH_NAME_Y = (\d+);""", rows, "the dim row's name under the hero")
        assertAbove("the name under an open row", below.groupValues[1].toInt(), lowName, lead)
        assertAbove("the name under the hero", first.groupValues[1].toInt(), highName, lead)
        assertNear(
            "the two screens' names sit as far apart as the reference measured them",
            first.groupValues[1].toInt() - below.groupValues[1].toInt(),
            highName - lowName,
        )
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
     * a height and a square, not a margin, so only the first argument of `offset` is read. The Java
     * half of the same scan is `NumbersTest.java`'s `no_java_part_writes_the_page_margin_again`.
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

    /**
     * A text's box top against the ink the reference measured: above it, and by no more than the
     * leading a line of that size carries.
     */
    private fun assertAbove(what: String, ink: Int, box: Int, lead: Int) = assertTrue(
        "$what: a box at $box against ink measured at $ink is not within the $lead px a line leaves above it",
        ink - box in 0..lead,
    )

    /** One `| <name> | **W x H**, ... |` row of the reference's row-screen table. */
    private fun tileRow(name: String): List<Int> =
        theOne(Regex("""\| $name \| \*\*(\d+) x (\d+)\*\*"""), reference, "the $name row in $REFERENCE")
            .groupValues.drop(1).map { it.toInt() }

    /** One `| <name> | **N px** |` row of the same table. */
    private fun pxRow(name: String): Int =
        theOne(Regex("""\| $name \| \*\*(\d+) px\*\*"""), reference, "the $name row in $REFERENCE")
            .groupValues[1].toInt()

    /** The size a named card of `Cards.java` draws itself at, as `(wide, high)` reference pixels. */
    private fun size(part: String): Pair<Int, Int> {
        val found = theOne(
            Regex("""class $part[\s\S]*?WIDE = (\d+);[\s\S]*?HIGH = (\d+);"""),
            cards,
            "$part's own size in Cards.java",
        )
        return found.groupValues[1].toInt() to found.groupValues[2].toInt()
    }

    private fun number(pattern: String, text: String, what: String) =
        theOne(Regex(pattern), text, what).groupValues[1].toInt()

    private companion object {
        const val MAIN = "apps/android/surfaces/tv/src/main/kotlin/streams/tv"

        /** The Java and Views surface, which is what draws a television screen since the port. */
        const val JAVA = "apps/android/surfaces/tv/src/main/java/streams/tv"

        /** What every surface shares in Java, where the reference scale and its canvas live. */
        const val SHARED_JAVA = "apps/android/surfaces/shared/src/main/java/streams/shared"

        /**
         * The television page's README, which is where this project's row-screen capture and the
         * numbers taken from it are written down. Read rather than copied: a test that copied them
         * would be two transcriptions of one measurement, which is the thing being guarded against.
         */
        const val REFERENCE = "reference/netflix-on-the-tv.md"
    }
}
