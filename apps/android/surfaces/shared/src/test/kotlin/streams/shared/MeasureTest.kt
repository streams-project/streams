package streams.shared

import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The half dp reference scale, `Measure.kt:12` and `:15`. The first tests it has had.
 *
 * Note 215 puts this first among the eleven measured design numbers for one reason: it is the only
 * one with no XML form. Every television screen in this project lays out at 960 x 540 dp while the
 * captures it was built from measure 1920 x 1080, so `311.ref` is 155.5 dp -- a size an Android
 * dimension resource cannot hold, since `dimen` takes a number and a unit and 155.5dp is legal text
 * but `311.ref` is not expressible as one resource at all. A rewrite into Views reaches for
 * `dimens.xml`, finds every reference number has to be halved by hand on the way in, and the first
 * rounding error is invisible on screen and permanent.
 *
 * So these tests pin the scale to the derivation stated in `Scale.java` rather than to itself: the
 * reference screen has to land exactly on the layout the televisions use. Since 2026-11-13 that
 * derivation is also the divisor both languages read, so what each test can still prove is called
 * out where it changed.
 */
class MeasureTest {
    /**
     * The derivation itself: 1080 reference pixels down is the 540 dp the surface lays out in.
     *
     * `assertEquals(0.5f, scale)` would pass forever and prove nothing. This says what the scale is
     * for: a measurement taken off a 1920 x 1080 capture, written into the code as the number that
     * was measured, lands where it was measured on a 960 x 540 dp screen.
     *
     * **The width line that stood here is gone, deliberately.** `assertEquals(960.dp, 1920.ref)`
     * had teeth while `Measure.kt` divided by its own 2: a wrong number in either place failed
     * against the other. Since 2026-11-13 the divisor is `Scale.PER_DP`, which is `CANVAS_W` over
     * `LAYOUT_W`, so that line asserts `x / (x / y) == y` and passes for any pair -- the form this
     * file's own rule refuses. The height still has teeth because it is not in the derivation: it
     * says the canvas is square-on, and a 1080 over 540 disagreeing with 1920 over 960 fails here.
     * The width pair is held by the test below instead, against another family's source.
     */
    @Test
    fun `the reference screen lands exactly on the layout the televisions use`() {
        assertEquals(540.dp, 1080.ref)
    }

    /**
     * The two numbers the scale is derived from, held against another television of this project.
     *
     * This is where the width pair is pinned now that it derives the divisor. `apps/xbox` states
     * the same canvas and the same layout in C++, written from the same captures by a different
     * hand, and the Apple television states the canvas again where
     * `tv/.../browse/NumbersTest.java` reads it. A derivation cannot catch a wrong 1920; a second
     * family saying 1920 can.
     */
    @Test
    fun `the canvas the scale derives from is the one the other televisions state`() {
        val xbox = File(repoRoot(), XBOX).readText()
        assertEquals(
            "the canvas the Xbox surface measures off has to be ours",
            Scale.CANVAS_W,
            one("""constexpr int W = (\d+);""", xbox),
        )
        assertEquals(
            "and the dp it lays that canvas out in",
            Scale.LAYOUT_W,
            one("""constexpr int LAYOUT_W = (\d+);""", xbox),
        )
    }

    /**
     * The scale is linear, which is what lets a position be written as a sum of measurements.
     *
     * `apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Rows.kt:137` places the first
     * poster beside the open card at `(PAGE_EDGE + 904 + 19).ref` -- a margin, a card and a gutter
     * added up as reference pixels and converted once. That only means "the three of them end to
     * end" if converting the sum equals summing the conversions. If `ref` ever rounded, it would
     * not, and the three-part derivation at that line would quietly stop being one.
     */
    @Test
    fun `a sum of reference measurements converts to the sum of their dp`() {
        assertEquals(72.ref + 904.ref + 19.ref, (72 + 904 + 19).ref)
        assertEquals((311 + 17).ref, 311.ref + 17.ref)
    }

    /**
     * An odd reference measurement lands on a half dp and keeps it. This is the property with no
     * XML form, so it is stated on the odd numbers the television surface actually uses: the closed
     * tile's 311 wide (`tv/browse/Rows.kt:61`), the gutter's 19 (`:137`), the hero's 767 high
     * (`tv/browse/HeroCard.kt:54`) and the focus border's 3 (`Rows.kt:110`).
     *
     * A rewrite that rounds any of these to a whole dp moves the thing by half a dp, which is one
     * physical pixel on a 1080p television and two on a 4K one.
     */
    @Test
    fun `an odd reference measurement keeps its half dp`() {
        for (odd in listOf(311, 19, 767, 3, 505)) {
            assertEquals("$odd reference pixels is $odd halves of a dp", odd / 2f, odd.ref.value, 0f)
            assertTrue(
                "$odd.ref is ${odd.ref.value} dp: it must keep its half, or it has been rounded",
                odd.ref.value % 1f != 0f,
            )
        }
        assertEquals(155.5.dp, 311.ref)
    }

    /** An even measurement is a whole dp, so the scale is not rounding everything to halves. */
    @Test
    fun `an even reference measurement is a whole dp`() {
        assertEquals(36.dp, 72.ref)
        assertEquals(452.dp, 904.ref)
        assertEquals(164.dp, 328.ref)
    }

    /** Type is measured off the same captures, so it is on the same scale. */
    @Test
    fun `type sizes use the same half scale as lengths`() {
        assertEquals(22.sp, 44.refType)
        assertEquals(540.sp, 1080.refType)
        assertEquals(16.5.sp, 33.refType)
    }

    /**
     * The derivation is written beside the code, and the two have to agree.
     *
     * This is the class of defect a reviewer caught on 2026-11-11: a focused logo box whose stated
     * derivation gave a different number from the one the code used. The comment in `Scale.java`
     * claims a layout of 960 x 540 against references of 1920 x 1080; the test reads those four
     * numbers back out of the comment and puts them through the code. A comment edited to claim a
     * different screen, or a canvas constant changed without the comment, fails.
     *
     * The height conversion is the half with teeth, for the reason the first test gives. On the
     * width, what this catches is the comment drifting from the two constants the scale is derived
     * from -- which is why both are read, and not only the arithmetic.
     */
    @Test
    fun `the derivation written beside the scale is the scale the code uses`() {
        val source = File(repoRoot(), SOURCE).readText()
        val said = Regex("""lays out at (\d+) by (\d+) dp.*?measure (\d+) by (\d+)""", RegexOption.DOT_MATCHES_ALL)
            .find(source)
        assertTrue("the derivation comment has gone from $SOURCE", said != null)
        val (wide, high, refWide, refHigh) = said!!.destructured.toList().map { it.toInt() }
        assertEquals("and its reference height to its layout height", high.dp, refHigh.ref)
        assertEquals("the comment's canvas is the canvas the scale derives from", refWide, Scale.CANVAS_W)
        assertEquals("and its layout is the layout", wide, Scale.LAYOUT_W)
    }

    /** The one match of a pattern, as a number: two matches would mean the number was copied. */
    private fun one(pattern: String, text: String): Int {
        val all = Regex(pattern).findAll(text).map { it.groupValues[1] }.toList()
        assertEquals("expected one `$pattern`, found ${all.size} $all", 1, all.size)
        return all.single().toInt()
    }

    /** Where the repository is, found by walking up: a test's working folder is not promised. */
    private fun repoRoot(): File {
        var here: File? = File("").absoluteFile
        while (here != null && !File(here, "strings/en.json").isFile) here = here.parentFile
        return requireNotNull(here) { "no strings/en.json above ${File("").absolutePath}" }
    }

    private companion object {
        /** Where the derivation is stated, which is where the divisor is derived. */
        const val SOURCE = "apps/android/surfaces/shared/src/main/java/streams/shared/Scale.java"

        /** The other television, which states the same canvas in another language. */
        const val XBOX = "apps/xbox/surfaces/remote/paint.h"
    }
}
