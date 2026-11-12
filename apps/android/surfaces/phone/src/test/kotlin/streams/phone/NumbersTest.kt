package streams.phone

import java.io.File
import kotlin.math.floor
import kotlin.math.roundToInt
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The phone surface's measured numbers, each held against the derivation or the capture it came
 * from. The phone side of `apps/apple/Tests/TelevisionTests/Numbers.swift`.
 *
 * Note 215 records "no test" against every number below, and `./check` runs no gradle
 * (`apps/android/surfaces/tv/build.gradle.kts:20`), so nothing in the repository would notice any
 * of them moving.
 *
 * Every number here came off a capture of the reference phone, and all of them came off the *same*
 * capture: 1080 x 2400 px at 2.5625 px/dp, which is a 936.6 dp screen. That is what makes real
 * derivation tests possible instead of self-pins. The dp in the code are the pixels that were
 * measured divided by that density, the ratios are the measured box's own shape, and three separate
 * files cite the same screen height -- so each one can be checked against the others rather than
 * against itself.
 *
 * `Sources.kt` says why these read the source as text and what that costs a rewrite.
 */
class NumbersTest {
    private val root = repoRoot()
    private val rows = File(root, "$MAIN/browse/Rows.kt").readText()
    private val empty = File(root, "$MAIN/browse/Empty.kt").readText()
    private val billboard = File(root, "$MAIN/browse/Billboard.kt").readText()
    private val sizes = File(root, "$MAIN/app/Sizes.kt").readText()

    /** The capture every number on this surface was read off, as `browse/Empty.kt:40` records it. */
    private val capture = theOne(
        Regex("""(\d+) x (\d+) at ([\d.]+) px/dp"""),
        empty,
        "the capture's size and density in Empty.kt",
    ).groupValues.drop(1)

    private val captureWide = capture[0].toInt()
    private val captureHigh = capture[1].toInt()
    private val density = capture[2].toFloat()

    /**
     * The capture and the screen the other files cite are the same phone.
     *
     * This runs first because the rest of the file depends on it. `browse/Empty.kt:40` gives the
     * capture in pixels and its density; `browse/Empty.kt:35` and `browse/Billboard.kt:60` both
     * cite a 936.6 dp screen without saying where it came from. 2400 px over 2.5625 px/dp is
     * 936.59, so they do come from this capture and a number measured in one file may be checked
     * against a number measured in another. If this fails, every derivation below is joining two
     * different phones and none of them means what it says.
     */
    @Test
    fun `the capture and the screen the other files cite are the same phone`() {
        val said = theOne(
            Regex("""(\d+\.\d+) dp of screen over a (\d+\.\d+) dp row pitch is (\d+\.\d+)"""),
            empty,
            "the screen height in Empty.kt",
        ).groupValues[1].toFloat()
        assertEquals("the capture's height in dp is the screen the code cites", said, captureHigh / density, 0.05f)
        val billboardScreen = theOne(SHARE_SAID, billboard, "the screen height in Billboard.kt").groupValues[3].toFloat()
        assertEquals("and Billboard.kt cites the same screen", said, billboardScreen, 0f)
    }

    /**
     * The tile's dp are the pixels that were measured divided by the capture's density.
     *
     * A derivation. `browse/Rows.kt:80` records the `uiautomator` reading of 2026-11-11 -- row 1
     * reads 305 x 427 px, like every row after it -- and the constants beside it are that reading
     * in dp. The halves matter: 427 px is 166.63 dp, so the 166.6 in the code is the measurement and
     * a tidy 167 would be a sixth of a pixel out at every tile and a dp out over six rows.
     */
    @Test
    fun `the tile is the measured pixels at the capture's density`() {
        val pixels = theOne(Regex("""row 1 reads (\d+) x (\d+) px"""), rows, "the tile's measured pixels")
        assertEquals(
            "the tile's width is the measured ${pixels.groupValues[1]} px at $density px/dp",
            pixels.groupValues[1].toFloat() / density,
            TileWide.value,
            0.05f,
        )
        assertEquals(
            "and its height the measured ${pixels.groupValues[2]} px",
            pixels.groupValues[2].toFloat() / density,
            TileHigh.value,
            0.05f,
        )
    }

    /**
     * The tile keeps the shape its own comment states. A derivation, and the one that catches a
     * tile resized on one axis: `browse/Rows.kt:79` says 5 : 7, and 119 x 7 is 833 as is 166.6 x 5.
     */
    @Test
    fun `the tile keeps the shape its comment states`() {
        val shape = theOne(
            Regex("""Tiles on the reference: ([\d.]+) x ([\d.]+) dp \((\d+) : (\d+)\)"""),
            rows,
            "the tile's stated shape",
        ).groupValues
        assertEquals("the comment has to name the tile beside it", TileWide.value, shape[1].toFloat(), 0f)
        assertEquals(TileHigh.value, shape[2].toFloat(), 0f)
        val short = shape[3].toFloat()
        val long = shape[4].toFloat()
        assertEquals(
            "the tile is $short : $long, so its width times $long is its height times $short",
            TileWide.value * long,
            TileHigh.value * short,
            0.01f,
        )
    }

    /**
     * The tile is the same measurement the Apple phone draws.
     *
     * One capture, two surfaces, so the numbers are one measurement written twice and nothing in
     * either repository ties them together. `apps/apple/Sources/SurfacesPhone/browse/Rows.swift:11`
     * states the same 119 x 166.6 and the same 6 and 12. A tile changed on one platform and not the
     * other is the parity defect this whole programme exists to avoid, and it would otherwise only
     * be found by holding two phones side by side.
     */
    @Test
    fun `the tile and its gaps are the same on both platforms`() {
        val apple = File(root, "$APPLE/browse/Rows.swift").readText()
        val tile = theOne(
            Regex("""public let tileWide: CGFloat = ([\d.]+)\s*\n\s*public let tileHigh: CGFloat = ([\d.]+)"""),
            apple,
            "the Apple phone's tile",
        ).groupValues
        assertEquals(TileWide.value, tile[1].toFloat(), 0f)
        assertEquals(TileHigh.value, tile[2].toFloat(), 0f)
        val gaps = theOne(
            Regex("""private let tileGap: CGFloat = ([\d.]+)\s*\n\s*private let rowStart: CGFloat = ([\d.]+)"""),
            apple,
            "the Apple phone's gaps",
        ).groupValues
        assertEquals(TileGap.value, gaps[1].toFloat(), 0f)
        assertEquals(RowStart.value, gaps[2].toFloat(), 0f)
    }

    /**
     * The gap between tiles and the row's start are the measured ones, rounded to the dp.
     *
     * A raw measurement held against its citation: `browse/Rows.kt:87` says the reference's are 5.9
     * and 12.1 dp, and the code rounds both. Rounding is the right call for a gap that no capture
     * could resolve more finely, but it has to stay a rounding of those two numbers -- a gap
     * "simplified" to 8 and 16 is a different row.
     */
    @Test
    fun `the gap and the row's start are the measured ones rounded`() {
        val said = theOne(Regex("""reference: ([\d.]+) and ([\d.]+) dp"""), rows, "the stated gaps").groupValues
        assertEquals(said[1].toFloat().roundToInt().toFloat(), TileGap.value, 0f)
        assertEquals(said[2].toFloat().roundToInt().toFloat(), RowStart.value, 0f)
    }

    /**
     * The billboard's share of the screen is the measured card over the measured screen.
     *
     * A derivation, and `browse/Billboard.kt:58-62` writes the whole of it out: the card is
     * 381.7 x 536.2 dp on a 936.6 dp screen, which is 536.2 / 936.6 = 0.572. So the division is
     * done here rather than trusted. This is the defect class a reviewer caught on 2026-11-11 -- a
     * stated derivation giving a different number from the one the code used -- and this is the
     * number on this surface most exposed to it, because 0.572 is three digits nobody would
     * recompute by eye.
     */
    @Test
    fun `the billboard's share is the measured card over the measured screen`() {
        val said = theOne(SHARE_SAID, billboard, "the billboard's stated derivation").groupValues
        val cardHigh = said[2].toFloat()
        val screen = said[3].toFloat()
        assertEquals("the comment has to divide the two numbers it just gave", cardHigh, said[4].toFloat(), 0f)
        assertEquals(screen, said[5].toFloat(), 0f)
        val share = number("""private const val CARD_SHARE = ([\d.]+)f""", billboard, "CARD_SHARE")
        assertEquals(
            "$cardHigh / $screen to the three digits the constant carries",
            round3(cardHigh / screen),
            share,
            0f,
        )
        assertEquals("and the comment's own answer is the same", said[6].toFloat(), share, 0f)
    }

    /**
     * The billboard's shape is the measured card's shape, and its inset the measured one rounded.
     *
     * A derivation from the same measured card: 381.7 / 536.2 is 0.7119, which is the 0.712 in the
     * code. Stated because the ratio and the share are two numbers derived from one pair of
     * measurements, and a card re-measured has to move both.
     */
    @Test
    fun `the billboard's shape and inset are the measured card's`() {
        val card = theOne(SHARE_SAID, billboard, "the billboard's measured card").groupValues
        val shape = theOne(
            Regex("""([\d.]+) wide for every 1 high, ([\d.]+) dp in from each side"""),
            billboard,
            "the billboard's stated shape",
        ).groupValues
        val ratio = number("""private const val CARD_RATIO = ([\d.]+)f""", billboard, "CARD_RATIO")
        assertEquals(
            "${card[1]} / ${card[2]} to three digits",
            round3(card[1].toFloat() / card[2].toFloat()),
            ratio,
            0f,
        )
        assertEquals("and the comment's own ratio is the same", shape[1].toFloat(), ratio, 0f)
        assertEquals(
            "the inset is the measured ${shape[2]} dp rounded",
            shape[2].toFloat().roundToInt(),
            CARD_SIDE,
        )
    }

    /**
     * The placeholder row name bar is the captured pixels at the capture's density, and it starts
     * on the row's own gutter.
     *
     * A derivation, and the longest one on this surface: `browse/Empty.kt:38-44` gives the bar's
     * pixel box off the capture itself -- y 1543 to 1621, x 31 to 1048 -- states that this is 79 px
     * and 30.8 dp, and states that the x runs on the row's 12.1 dp gutter. Four things are checked
     * against each other: the stated 79 px against the y pair, the 30.8 dp against the 79 px at the
     * capture's density, the x pair against the capture's own width less the gutter twice, and the
     * gutter in pixels against `RowStart`.
     */
    @Test
    fun `the placeholder bar is the captured pixels and starts on the row's gutter`() {
        val y = theOne(Regex("""bar runs y (\d+)-(\d+), which is (\d+) px"""), empty, "the bar's y").groupValues
        val x = theOne(
            Regex("""x (\d+)-(\d+), which starts and ends on the row's own[\s*]+([\d.]+) dp gutter"""),
            empty,
            "the bar's x",
        ).groupValues
        val high = y[3].toInt()
        assertEquals(
            "the stated ${y[3]} px is the span y ${y[1]} to ${y[2]} inclusive",
            y[2].toInt() - y[1].toInt() + 1,
            high,
        )
        val bar = number("""private val BarHigh = ([\d.]+)\.dp""", empty, "BarHigh")
        assertEquals("$high px at $density px/dp", high / density, bar, 0.05f)
        val said = number("""So it is ([\d.]+) dp high""", empty, "the stated bar height")
        assertEquals("and the comment's own answer is the same", said, bar, 0f)

        val left = x[1].toInt()
        assertEquals(
            "the bar is as wide as the row: the capture less the gutter on each side",
            captureWide - 2 * left,
            x[2].toInt() - left + 1,
        )
        assertEquals("the stated gutter is $left px at $density px/dp", left / density, x[3].toFloat(), 0.05f)
        assertEquals("and the row starts on it, rounded", x[3].toFloat().roundToInt().toFloat(), RowStart.value, 0f)
    }

    /**
     * A waiting page draws as many rows as the screen holds, whole. A derivation:
     * `browse/Empty.kt:35` is 936.6 dp of screen over a 217.0 dp row pitch, which is 4.3, and a
     * third of a row is not drawn.
     */
    @Test
    fun `a waiting page draws the whole rows the screen holds`() {
        val said = theOne(
            Regex("""(\d+\.\d+) dp of screen over a (\d+\.\d+) dp row pitch is (\d+\.\d+)"""),
            empty,
            "the waiting rows' derivation",
        ).groupValues
        val screen = said[1].toFloat()
        val pitch = said[2].toFloat()
        assertEquals("the comment's own division", round1(screen / pitch), said[3].toFloat(), 0f)
        assertEquals(
            "the rows drawn are the whole ones",
            floor(screen / pitch).toInt(),
            number("""private const val WAITING_ROWS = (\d+)""", empty, "WAITING_ROWS").toInt(),
        )
    }

    /**
     * The page margin by width class is the same on both platforms, and the phone's is the row's
     * own gutter.
     *
     * `app/Sizes.kt:9` claims "These equal Sizes.swift" and nothing checked it, so the claim is the
     * derivation and this test is it: the two breakpoints and the three margins have to match the
     * Swift file, number for number.
     *
     * Beyond that agreement the three margins are held as far as they can honestly be held. 12 is
     * the measured row gutter, so it is checked against `RowStart`; 24 is twice it. **40 has no
     * derivation and no citation anywhere in this repository** -- it is a choice for a tablet no
     * Netflix tablet measurement exists for (`browse/Billboard.kt:70-71` says the same of the card
     * share). So it is held by the cross-platform agreement and by its place in an increasing
     * sequence, and this comment is the record that it is the one number here with nothing behind
     * it. Asserting `40 == 40` would hide that; saying so does not.
     */
    @Test
    fun `the page margin by width class is the same on both platforms`() {
        assertTrue("the claim this test checks has gone from Sizes.kt", sizes.contains("These equal Sizes.swift"))
        val swift = File(root, "$APPLE/app/Sizes.swift").readText()
        val here = theOne(
            Regex("""WidthClass\.COMPACT -> (\d+)\s*\n\s*WidthClass\.MEDIUM -> (\d+)\s*\n\s*WidthClass\.EXPANDED -> (\d+)"""),
            sizes,
            "the margins in Sizes.kt",
        ).groupValues.drop(1).map { it.toInt() }
        val there = theOne(
            Regex("""case \.compact: (\d+)\s*\n\s*case \.medium: (\d+)\s*\n\s*case \.expanded: (\d+)"""),
            swift,
            "the margins in Sizes.swift",
        ).groupValues.drop(1).map { it.toInt() }
        assertEquals("the three margins have to agree across the two platforms", there, here)

        val breaks = theOne(
            Regex("""wide < (\d+) -> WidthClass\.COMPACT\s*\n\s*wide < (\d+) -> WidthClass\.MEDIUM"""),
            sizes,
            "the breakpoints in Sizes.kt",
        ).groupValues.drop(1).map { it.toInt() }
        val theirBreaks = theOne(
            Regex("""width < (\d+) \? \.compact : width < (\d+) \? \.medium"""),
            swift,
            "the breakpoints in Sizes.swift",
        ).groupValues.drop(1).map { it.toInt() }
        assertEquals("and so do the two breakpoints", theirBreaks, breaks)

        assertEquals("a phone's margin is the row's own gutter", RowStart.value.roundToInt(), here[0])
        assertEquals("a medium window's is twice a phone's", 2 * here[0], here[1])
        assertTrue("the margins have to grow with the window", here[0] < here[1] && here[1] < here[2])
        assertTrue("and so do the breakpoints", breaks[0] < breaks[1])
    }

    private fun round3(value: Float) = (value * 1000f).roundToInt() / 1000f

    private fun round1(value: Float) = (value * 10f).roundToInt() / 10f

    private fun number(pattern: String, text: String, what: String) =
        theOne(Regex(pattern), text, what).groupValues[1].toFloat()

    private companion object {
        const val MAIN = "apps/android/surfaces/phone/src/main/kotlin/streams/phone"
        const val APPLE = "apps/apple/Sources/SurfacesPhone"

        /** The billboard's measured card, its screen and the division it was put through. */
        val SHARE_SAID = Regex(
            """([\d.]+) x ([\d.]+) dp on a ([\d.]+) dp screen, which is ([\d.]+) / ([\d.]+) = ([\d.]+)""",
        )
    }
}
