package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotEquals;
import static org.junit.Assert.assertTrue;

import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.Test;
import streams.shared.Scale;

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
     * The derivation itself: 1080 reference pixels down is the 540 dp the surface lays out in.
     *
     * <b>Why the width is not asserted here any more, which is a real loss and is replaced below.</b>
     * Until 2026-11-13 this line read {@code assertEquals(Ref.LAYOUT_W, Ref.dp(Ref.W))} and had
     * teeth, because the divisor was written independently of both numbers: a wrong 960 or a wrong
     * 1920 failed against the 2. {@code Scale.PER_DP} is now {@code CANVAS_W / (float) LAYOUT_W}, so
     * that line would be {@code x / (x / y) == y} -- true for any pair, which is the "pinned to
     * itself" form this file exists to refuse. The height keeps its teeth for the same reason the
     * width lost them: it is not in the derivation, so this says the canvas is square-on, and a
     * 1080 over 540 that disagreed with 1920 over 960 would fail here. What now holds the width pair
     * is {@code MeasureTest.kt}'s read of the Xbox surface's own statement of the same canvas, which
     * is a stronger anchor than the one it replaces: another family's source rather than this one's
     * own arithmetic.
     */
    @Test
    public void the_reference_screen_lands_exactly_on_the_layout_the_televisions_use() {
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
     * The derivation is written beside the code, and the two have to agree.
     *
     * This is the class of defect a reviewer caught on 2026-11-11: a focused logo box whose stated
     * derivation gave a different number from the one the code used. The comment in {@code Scale}
     * claims a layout of 960 x 540 against references of 1920 x 1080; this reads those four numbers
     * back out of it and puts them through the Java. A comment edited to claim a different screen,
     * or a canvas constant changed without the comment, fails.
     *
     * The height is the half with teeth, as above: the width pair is the derivation, so
     * {@code dp(1920) == 960} cannot fail. What this still catches on the width is the comment
     * drifting from {@link Scale#CANVAS_W} and {@link Scale#LAYOUT_W}, which are the numbers the
     * other televisions are held against.
     */
    @Test
    public void the_derivation_written_beside_the_scale_is_the_scale_the_code_uses() {
        int[] said = said(SCALE);
        assertEquals("the comment's reference height has to convert to its layout height",
                said[1], Ref.dp(said[3]), 0f);
        assertEquals("the canvas the code carries is the one the comment states", said[2],
                Scale.CANVAS_W);
        assertEquals("and the layout it derives the scale against", said[0], Scale.LAYOUT_W);
        assertEquals("the television's height is the one the comment states", said[3], Ref.H);
        assertEquals(said[1], Ref.LAYOUT_H);
    }

    /**
     * The scale is written in one place, and this is what stops it being written in two again.
     *
     * Until 2026-11-13 the halving was four numbers: {@code (this / 2f)} twice in {@code Measure.kt}
     * and {@code ref / 2f} twice in {@code Ref.java}, with the test that stood here reading all four
     * back out of the source and holding them equal -- a second home kept level by a check instead
     * of not existing. {@link Scale#PER_DP} is now derived from the canvas over the layout and both
     * toolkits read it, so there is no second divisor to compare and that test has nothing left to
     * say. This says the thing worth saying instead: neither file writes a divisor of its own, and
     * neither restates the derivation, so the two homes cannot come back without failing here.
     */
    @Test
    public void the_scale_is_written_once_and_neither_toolkit_keeps_a_copy() {
        String java = Repo.text(JAVA);
        String kotlin = Repo.text(KOTLIN);
        assertEquals("Ref.java must not divide by anything: it delegates", 0,
                Repo.every("(/ *\\d+(?:\\.\\d+)?f)", java).size());
        assertEquals("Measure.kt must divide by the one value and by no number", 0,
                Repo.every("(/ *\\d+(?:\\.\\d+)?f)", kotlin).size());
        assertEquals("and it divides by the shared one", 2,
                Repo.every("(this / Scale\\.PER_DP)", kotlin).size());
        for (String path : new String[] {JAVA, KOTLIN}) {
            assertEquals(path + " must not state the derivation a second time", 0,
                    Repo.every("(lays out at \\d+ by \\d+ dp)", Repo.text(path)).size());
        }
        assertEquals("and the one that does states it once", 1,
                Repo.every("(lays out at \\d+ by \\d+ dp)", Repo.text(SCALE)).size());
        assertEquals("the one divisor is derived and not typed", 1,
                Repo.every("(PER_DP = CANVAS_W / \\(float\\) LAYOUT_W)", Repo.text(SCALE)).size());
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
    private static final String SCALE =
            "apps/android/surfaces/shared/src/main/java/streams/shared/Scale.java";
}
