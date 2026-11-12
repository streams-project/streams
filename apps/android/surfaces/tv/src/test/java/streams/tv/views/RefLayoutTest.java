package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.Test;

/**
 * Where {@link RefLayout} puts a child, and the right to left mirror.
 *
 * The placement is tested and not the view: there is no Robolectric and no {@code ui-test} in
 * {@code ~/.gradle} on this machine and gradle runs offline (the reason is written out at
 * {@code shared/build.gradle.kts:52-58}), so a laid-out hierarchy is a device test. What can be
 * held here is the arithmetic, which is where note 210 puts the risk: the mirror is one line for the
 * whole surface, and a wrong sign in it mirrors every screen at once.
 *
 * The numbers are read out of {@code tv/browse/Rows.kt} and {@code tv/app/Round.kt} rather than
 * copied, for the reason {@code src/test/kotlin/streams/tv/NumbersTest.kt} gives: a test that
 * copied them would be two transcriptions of one measurement, which is the drift being guarded
 * against.
 */
public class RefLayoutTest {
    private final String rows = Repo.text(ROWS);
    private final int margin = number("const val PAGE_EDGE = (\\d+)", Repo.text(ROUND), "PAGE_EDGE");
    private final int step = number("private const val STEP = (\\d+)", rows, "STEP");

    /** A child sits at the coordinate it was measured at, converted once. */
    @Test
    public void a_child_sits_where_it_was_measured() {
        assertEquals(Ref.px(margin, 1f), RefLayout.leftOf(margin, 904, 452, 960, false, 1f));
        assertEquals(Ref.px(200, 1f), RefLayout.topOf(200, 1f));
        assertEquals(Ref.px(margin, 2f), RefLayout.leftOf(margin, 904, 904, 1920, false, 2f));
    }

    /**
     * The closed row starts on the page margin and steps by the tile pitch, which is the derivation
     * {@code Rows.kt:121} is written as and the one a layout has to keep.
     */
    @Test
    public void the_closed_row_steps_by_the_pitch_from_the_margin() {
        assertTrue("the row has to start on the margin and step by the pitch",
                rows.contains("(PAGE_EDGE + index * STEP).ref"));
        for (int index = 0; index < 6; index++) {
            assertEquals(Ref.px(margin + index * step, 1f),
                    RefLayout.leftOf(margin + index * step, 311, 156, 960, false, 1f));
        }
        // And the tile the viewer came from peeks in by exactly the gutter, which is a result of
        // the pitch rather than a number anybody chose: `Rows.kt:173` draws it at PAGE_EDGE - STEP.
        assertEquals(Ref.px(margin - step, 1f), RefLayout.leftOf(margin - step, 311, 156, 960, false, 1f));
    }

    /**
     * Mirroring a child whose width was measured is the reference subtraction, exactly: note 210's
     * {@code x' = 1920 - x - w}, taken in reference space and converted once.
     *
     * The open card is at the page margin and is 904 wide ({@code Rows.kt:87-88}), so in Arabic its
     * left edge is at 1920 - 72 - 904 = 944 reference pixels, which is the margin on the other side
     * of the screen -- a card the width of the page less its two margins, mirrored, lands on the
     * other margin. If the subtraction were wrong by the card's width or by the margin, that
     * identity would not hold.
     */
    @Test
    public void mirroring_a_measured_child_is_the_reference_subtraction() {
        int card = 904;
        for (float density : new float[] {1f, 1.33125f, 1.5f, 2f}) {
            int parent = Ref.px(Ref.W, density);
            int wide = Ref.px(card, density);
            assertEquals("mirrored at density " + density,
                    Ref.px(Ref.W - margin - card, density),
                    RefLayout.leftOf(margin, card, wide, parent, true, density));
        }
        assertEquals(Ref.px(944, 1f), RefLayout.leftOf(margin, 904, 452, 960, true, 1f));
    }

    /**
     * And mirroring a mirrored coordinate lands where it started, which is the property that says
     * the sign is right rather than that one number is right.
     */
    @Test
    public void mirroring_twice_lands_where_it_started() {
        int[][] spots = {{margin, 904}, {margin, 311}, {margin + step, 311}, {0, Ref.W}, {995, 311}};
        for (float density : new float[] {1f, 1.33125f, 2f}) {
            for (int[] spot : spots) {
                int parent = Ref.px(Ref.W, density);
                int wide = Ref.px(spot[1], density);
                int mirrored = Ref.W - spot[0] - spot[1];
                assertEquals("mirroring " + spot[0] + " by " + spot[1] + " twice at " + density,
                        RefLayout.leftOf(spot[0], spot[1], wide, parent, false, density),
                        RefLayout.leftOf(mirrored, spot[1], wide, parent, true, density));
            }
        }
    }

    /**
     * A child that measures itself mirrors in pixels, because there is no reference width to
     * subtract, and that lands within one pixel of the reference arithmetic rather than on it.
     *
     * Found by this test and not assumed: the first version of it claimed the two agree exactly at a
     * whole density, and they do not. A reference measurement is half a dp, so the mirrored
     * coordinate of an odd remainder lands on a half dp -- at density 1 that is half a physical
     * pixel, which the pixel subtraction has nowhere to put, and the two answers differ by one. At
     * density 2 a half dp is a whole pixel and they agree for every remainder.
     *
     * The consequence is worth more than the test: <b>a mirrored child should declare its reference
     * width</b>, and then {@link RefLayout#leftOf} takes the subtraction in reference space and
     * there is nothing to round twice. A line of text is the case where that is not possible.
     */
    @Test
    public void a_child_that_measures_itself_mirrors_in_pixels_and_may_land_a_pixel_off() {
        int odd = 311;
        int even = 312;
        // Where a half dp is a whole pixel, the two mirrors agree for every remainder.
        for (int label : new int[] {odd, even}) {
            assertEquals("at density 2 a half dp is a whole pixel",
                    Ref.px(Ref.W - margin - label, 2f),
                    RefLayout.leftOf(margin, RefLayout.WRAP, Ref.px(label, 2f), Ref.px(Ref.W, 2f), true, 2f));
        }
        // And an even remainder is a whole dp, so it agrees at density 1 as well.
        assertEquals(0, (Ref.W - margin - even) % 2);
        assertEquals(Ref.px(Ref.W - margin - even, 1f),
                RefLayout.leftOf(margin, RefLayout.WRAP, Ref.px(even, 1f), Ref.px(Ref.W, 1f), true, 1f));
        // An odd one cannot be, and this says so by one pixel rather than allowing up to one.
        assertEquals(1, (Ref.W - margin - odd) % 2);
        assertEquals("the reference mirror of an odd remainder is a half dp the pixel mirror has "
                        + "nowhere to put",
                Ref.px(Ref.W - margin - odd, 1f) - 1,
                RefLayout.leftOf(margin, RefLayout.WRAP, Ref.px(odd, 1f), Ref.px(Ref.W, 1f), true, 1f));
        // Never more than one, at any density a television reports.
        for (float density : new float[] {1f, 1.33125f, 1.5f, 2f, 2.625f, 3f}) {
            for (int label : new int[] {odd, even, 904, 19, 1}) {
                int pixels = RefLayout.leftOf(margin, RefLayout.WRAP, Ref.px(label, density),
                        Ref.px(Ref.W, density), true, density);
                int reference = Ref.px(Ref.W - margin - label, density);
                assertTrue("at density " + density + " a child " + label + " wide mirrors to "
                                + pixels + " in pixels and " + reference + " in reference space",
                        Math.abs(pixels - reference) <= 1);
            }
        }
    }

    /**
     * The first poster beside the open card clears it by the gutter, read out of the sum
     * {@code Rows.kt:137} writes.
     *
     * The derivation, in reference space: the card starts on the margin and is 904 wide, so the
     * poster's x has to be the margin plus the card plus the gutter. A card widened without the sum
     * following would overlap the poster, which looks like a drawing bug and is a transcription one.
     * The same three numbers go through {@link RefLayout#leftOf} to say that the layout converts the
     * sum rather than the parts.
     */
    @Test
    public void the_poster_beside_the_open_card_clears_it_by_the_gutter() {
        Matcher sum = Repo.theOne("\\(PAGE_EDGE \\+ (\\d+) \\+ (\\d+)\\)\\.ref", 0, rows,
                "the next poster's x in " + ROWS);
        int card = Integer.parseInt(sum.group(1));
        int gutter = Integer.parseInt(sum.group(2));
        Matcher size = Repo.theOne("fun OpenCard\\([\\s\\S]*?\\.size\\((\\d+)\\.ref,\\s*(\\d+)\\.ref\\)", 0,
                rows, "the open card's own size in " + ROWS);
        assertEquals("the sum has to use the card's own width", Integer.parseInt(size.group(1)), card);
        int poster = margin + card + gutter;
        assertEquals("the poster starts one gutter past the card's right edge",
                gutter, poster - (margin + card));
        assertEquals("and the layout converts the sum, not the three parts",
                Ref.px(poster, 1f), RefLayout.leftOf(poster, 311, 156, 960, false, 1f));
    }

    /** A measured child is given its measured size, and a wrapping one is not. */
    @Test
    public void a_measured_child_is_the_size_it_was_measured_at() {
        RefLayout.Params spot = RefLayout.at(margin, 200, 904, 508);
        assertEquals(margin, spot.refX);
        assertEquals(200, spot.refY);
        assertEquals(904, spot.refW);
        assertEquals(508, spot.refH);
        RefLayout.Params own = RefLayout.at(margin, 146);
        assertEquals(RefLayout.WRAP, own.refW);
        assertEquals(RefLayout.WRAP, own.refH);
        assertTrue("a wrapping size must not be a coordinate anybody could mean", RefLayout.WRAP < 0);
    }

    private static int number(String pattern, String text, String what) {
        return Integer.parseInt(Repo.theOne(pattern, Pattern.MULTILINE, text, what).group(1));
    }

    private static final String ROWS =
            "apps/android/surfaces/tv/src/main/kotlin/streams/tv/browse/Rows.kt";
    private static final String ROUND =
            "apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Round.kt";
}
