package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotEquals;
import static org.junit.Assert.assertTrue;

import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.Test;

/**
 * The half-dp reference scale in Java. The twin of
 * {@code shared/src/test/kotlin/streams/shared/MeasureTest.kt}, and the same rule: <b>the scale is
 * pinned to its derivation, never to itself.</b> {@code assertEquals(0.5f, scale)} would pass
 * forever and prove nothing.
 *
 * Note 209 names this scale as the part of the migration that historically overruns, because XML
 * has no half-dp form and a rewrite that reaches for {@code dimens.xml} has to halve every measured
 * number by hand on the way in, where the first rounding error is invisible on screen and
 * permanent. So these tests say what the scale is for: a measurement taken off a 1920 x 1080
 * capture, written into the code as the number that was measured, lands where it was measured on a
 * 960 x 540 dp screen.
 */
public class RefTest {
    /**
     * The derivation itself. 1920 reference pixels across is the 960 dp the surface lays out in,
     * and 1080 down is 540.
     */
    @Test
    public void the_reference_screen_lands_exactly_on_the_layout_the_televisions_use() {
        assertEquals(Ref.LAYOUT_W, Ref.dp(Ref.W), 0f);
        assertEquals(Ref.LAYOUT_H, Ref.dp(Ref.H), 0f);
    }

    /**
     * The scale is linear, which is what lets a position be written as a sum of measurements.
     *
     * {@code tv/browse/Rows.kt:137} places the first poster beside the open card at
     * {@code (PAGE_EDGE + 904 + 19).ref} -- a margin, a card and a gutter added up as reference
     * pixels and converted once. That only means "the three of them end to end" if converting the
     * sum equals summing the conversions.
     */
    @Test
    public void a_sum_of_reference_measurements_converts_to_the_sum_of_their_dp() {
        assertEquals(Ref.dp(72) + Ref.dp(904) + Ref.dp(19), Ref.dp(72 + 904 + 19), 0f);
        assertEquals(Ref.dp(311) + Ref.dp(17), Ref.dp(311 + 17), 0f);
    }

    /**
     * An odd reference measurement lands on a half dp and keeps it. This is the property with no
     * XML form, stated on the odd numbers the television surface actually uses: the closed tile's
     * 311 wide ({@code tv/browse/Rows.kt:61}), the gutter's 19 ({@code :137}), the hero's 767 high
     * ({@code tv/browse/HeroCard.kt:54}) and the focus border's 3 ({@code Rows.kt:110}).
     *
     * A rewrite that rounds any of these to a whole dp moves the thing by half a dp, which is one
     * physical pixel on a 1080p television and two on a 4K one.
     */
    @Test
    public void an_odd_reference_measurement_keeps_its_half_dp() {
        for (int odd : new int[] {311, 19, 767, 3, 505}) {
            assertEquals(odd + " reference pixels is " + odd + " halves of a dp", odd / 2f, Ref.dp(odd), 0f);
            assertNotEquals(odd + ".ref is " + Ref.dp(odd) + " dp: it must keep its half, or it has been rounded",
                    0f, Ref.dp(odd) % 1f, 0f);
        }
        assertEquals(155.5f, Ref.dp(311), 0f);
    }

    /** An even measurement is a whole dp, so the scale is not rounding everything to halves. */
    @Test
    public void an_even_reference_measurement_is_a_whole_dp() {
        assertEquals(36f, Ref.dp(72), 0f);
        assertEquals(452f, Ref.dp(904), 0f);
        assertEquals(164f, Ref.dp(328), 0f);
    }

    /** Type is measured off the same captures, so it is on the same scale. */
    @Test
    public void type_sizes_use_the_same_half_scale_as_lengths() {
        assertEquals(22f, Ref.sp(44), 0f);
        assertEquals(540f, Ref.sp(1080), 0f);
        assertEquals(16.5f, Ref.sp(33), 0f);
    }

    /**
     * Pixels round the way the toolkit rounds, which is a half up.
     *
     * Compose converts a dp to pixels as the value times the density, rounded to the nearest
     * integer, and resolves a tie upwards. {@code Math.round(float)} is {@code floor(x + 0.5)}, so
     * the two agree for every positive value, and a screen rewritten from Kotlin to Java does not
     * move by a pixel on the way. 1 reference pixel is half a dp and lands on 1 physical pixel at
     * density 1, not on 0.
     */
    @Test
    public void pixels_round_a_half_upwards_the_way_the_toolkit_does() {
        assertEquals(1, Ref.px(1, 1f));
        assertEquals(2, Ref.px(3, 1f));
        assertEquals(156, Ref.px(311, 1f));
        assertEquals(311, Ref.px(311, 2f));
        assertEquals(960, Ref.px(Ref.W, 1f));
        assertEquals(1920, Ref.px(Ref.W, 2f));
    }

    /**
     * And the consequence, which is why {@code Rows.kt:137} writes a sum instead of adding three
     * converted numbers: converting a sum is not summing the conversions, in pixels.
     *
     * Three reference pixels is 1.5 dp and rounds to 2 physical pixels at density 1. Two of them
     * placed end to end measure 4 that way and 3 the right way, so an inset written as two
     * conversions is a third of its width out. This is the arithmetic note 210 refused to move into
     * fractional dp resources, where every value rounds on its own.
     */
    @Test
    public void a_sum_is_converted_once_because_summing_conversions_is_a_different_number() {
        assertEquals(2, Ref.px(3, 1f));
        assertEquals(3, Ref.px(3 + 3, 1f));
        assertNotEquals(Ref.px(3, 1f) + Ref.px(3, 1f), Ref.px(3 + 3, 1f));
    }

    /**
     * The derivation is written beside the code, in both languages, and all three have to agree.
     *
     * This is the class of defect a reviewer caught on 2026-11-11: a focused logo box whose stated
     * derivation gave a different number from the one the code used. The comment in each file claims
     * a layout of 960 x 540 against references of 1920 x 1080; this reads those four numbers back
     * out of both files and puts them through the Java. A comment edited to claim a different
     * screen, a scale changed without the comment, or the two languages drifting apart, all fail.
     */
    @Test
    public void the_derivation_written_beside_the_scale_is_the_scale_the_code_uses() {
        int[] java = said(JAVA);
        int[] kotlin = said(KOTLIN);
        assertEquals("the two languages state the same layout width", kotlin[0], java[0]);
        assertEquals("and the same layout height", kotlin[1], java[1]);
        assertEquals("and the same reference width", kotlin[2], java[2]);
        assertEquals("and the same reference height", kotlin[3], java[3]);
        assertEquals("the comment's reference width has to convert to its layout width",
                java[0], Ref.dp(java[2]), 0f);
        assertEquals("and its reference height to its layout height", java[1], Ref.dp(java[3]), 0f);
        assertEquals("the canvas the code carries is the one the comment states", java[2], Ref.W);
        assertEquals(java[3], Ref.H);
        assertEquals(java[0], Ref.LAYOUT_W);
        assertEquals(java[1], Ref.LAYOUT_H);
    }

    /**
     * And the two scales divide by the same number, read out of both files.
     *
     * The derivation test above would still pass if one language halved and the other quartered, as
     * long as both comments were edited to match. This reads the arithmetic itself: two divisors in
     * the Kotlin (a length and a type size) and two in the Java, and all four are the same number.
     */
    @Test
    public void the_java_scale_divides_by_what_the_kotlin_scale_divides_by() {
        List<String> kotlin = Repo.every("\\(this / (\\d+)f\\)", Repo.text(KOTLIN));
        List<String> java = Repo.every("ref / (\\d+)f", Repo.text(JAVA));
        assertEquals("the Kotlin states the scale twice, for a length and for a type size",
                2, kotlin.size());
        assertEquals("and so does the Java", 2, java.size());
        for (String divisor : java) {
            assertEquals("every divisor in both files is the same number", kotlin.get(0), divisor);
        }
        assertEquals("and the scale is a half, which is what both comments derive", "2", kotlin.get(0));
    }

    /** The four numbers a file's own derivation comment states. */
    private static int[] said(String path) {
        Matcher found = Repo.theOne(
                "lays out at (\\d+) by (\\d+) dp.*?measure (\\d+) by (\\d+)",
                Pattern.DOTALL,
                Repo.text(path),
                "the derivation comment in " + path);
        int[] numbers = new int[4];
        for (int at = 0; at < 4; at++) {
            numbers[at] = Integer.parseInt(found.group(at + 1));
        }
        assertTrue("the comment has to state four numbers", numbers[3] > 0);
        return numbers;
    }

    private static final String JAVA =
            "apps/android/surfaces/tv/src/main/java/streams/tv/views/Ref.java";
    private static final String KOTLIN =
            "apps/android/surfaces/shared/src/main/kotlin/streams/shared/Measure.kt";
}
