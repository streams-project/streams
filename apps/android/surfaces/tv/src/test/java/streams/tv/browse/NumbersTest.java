package streams.tv.browse;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import java.util.regex.Matcher;
import org.junit.Test;
import streams.tv.views.Ink;
import streams.tv.views.Ref;
import streams.tv.views.Round;

/**
 * Every measured number this port had to carry, read out of the Kotlin and out of the Java and held
 * equal.
 *
 * <h2>Why this shape and not {@code assertEquals(328, STEP)}</h2>
 * {@code src/test/kotlin/streams/tv/NumbersTest.kt}'s own rule, which is the owner's: "a number is
 * never asserted against itself. {@code assertEquals(328, STEP)} passes forever." Its ten tests hold
 * the Kotlin against {@code pstream/notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md}'s reference table and against the derivations written
 * beside each number, and they still pass, because the Kotlin they read is still there.
 *
 * What they cannot see is whether the number came across. {@code Sources.kt:18-22} predicted exactly
 * this: "porting this surface to Java and Views will break them. That is intended... Re-pointing
 * each of these at the Java form is how a rewriter states, in the diff, that the number came
 * across." This is that statement, in the form that is honest while both languages are in the tree:
 * each number is read from the Kotlin line it was measured onto and from the Java constant that
 * replaced it, and the two have to agree. Neither side is trusted and neither is copied into this
 * file.
 *
 * So there are three failure modes and all three are wanted: a number edited in the Kotlin and not
 * the Java fails; a number edited in the Java and not the Kotlin fails; and a number deleted from
 * either fails, with a message naming the pattern that stopped matching.
 *
 * When the lead deletes the dead Kotlin, every assertion in this file has to lose its Kotlin half
 * deliberately, in that diff. That is the point.
 */
public class NumbersTest {
    private final String rowsSaid = Source.text(Source.KOTLIN + "/browse/Rows.kt");
    private final String rows = Source.bare(rowsSaid);
    private final String hero = Source.code(Source.KOTLIN + "/browse/HeroCard.kt");
    private final String bar = Source.code(Source.KOTLIN + "/browse/TopBar.kt");
    private final String browse = Source.code(Source.KOTLIN + "/browse/Browse.kt");
    private final String round = Source.code(Source.KOTLIN + "/app/Round.kt");
    private final String theme = Source.code(Source.SHARED + "/Theme.kt");
    private final String addresses = Source.code(Source.SHARED + "/Addresses.kt");

    private final String rowsJava = Source.code(Source.JAVA + "/browse/RowsView.java");
    private final String cardsJava = Source.code(Source.JAVA + "/browse/Cards.java");
    private final String heroJava = Source.code(Source.JAVA + "/browse/HeroView.java");
    private final String barJava = Source.code(Source.JAVA + "/browse/BarView.java");
    private final String viewJava = Source.code(Source.JAVA + "/browse/BrowseView.java");
    private final String partsJava = Source.code(Source.JAVA + "/browse/Parts.java");
    private final String roundJava = Source.code(Source.JAVA + "/views/Round.java");

    /** The size a named composable of {@code Rows.kt} draws itself at, as {@code .size(w, h)}. */
    private int[] kotlinSize(String part) {
        Matcher found = Source.theOne(
                "fun " + part + "\\([\\s\\S]*?\\.size\\((\\d+)\\.ref,\\s*(\\d+)\\.ref\\)",
                rows, part + "'s own size in Rows.kt");
        return new int[] {Integer.parseInt(found.group(1)), Integer.parseInt(found.group(2))};
    }

    // --- the three sizes the whole surface rests on ---------------------------------------------

    /**
     * The closed tile is 311 by 504 in both languages. {@code Rows.kt:61} and
     * {@code Cards.java}'s {@code Poster}.
     *
     * A raw Netflix measurement with no derivation, so the Kotlin side of it is pinned to the
     * reference table by {@code NumbersTest.kt}'s first test and this only has to prove the port.
     */
    @Test
    public void the_closed_tile_came_across() {
        int[] said = kotlinSize("Poster");
        assertEquals("the closed tile's width", said[0], Cards.Poster.WIDE);
        assertEquals("the closed tile's height", said[1], Cards.Poster.HIGH);
        assertTrue("and both stay integer literals in the Java, not a computed pair",
                cardsJava.contains("WIDE = " + said[0] + ";")
                        && cardsJava.contains("HIGH = " + said[1] + ";"));
    }

    /** And the card the focused tile opens into is 904 by 508. {@code Rows.kt:88}. */
    @Test
    public void the_open_card_came_across() {
        int[] said = kotlinSize("OpenCard");
        assertEquals("the focused card's width", said[0], Cards.Open.WIDE);
        assertEquals("the focused card's height", said[1], Cards.Open.HIGH);
    }

    /**
     * The page margin is 72 in both, and the Java reaches it through a constant as every Kotlin page
     * does.
     *
     * 72 is a decision and not a measurement -- {@code pstream/notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md:191} records the reference's
     * own safe margin as 68 px at 1920 and this project choosing 72 in note 188 -- so what is worth
     * checking is the relationship: no page writes it again, which is what makes it changeable at
     * all. {@code NumbersTest.kt}'s last test scans the Kotlin for a bare margin literal;
     * {@link #no_java_part_writes_the_page_margin_again} does the same for the Java.
     */
    @Test
    public void the_page_margin_came_across() {
        assertEquals(Source.number("const val PAGE_EDGE = (\\d+)", round, "PAGE_EDGE in Round.kt"),
                Round.PAGE_EDGE);
        assertEquals(Source.number("const val SUB_EDGE = (\\d+)", round, "SUB_EDGE in Round.kt"),
                Round.SUB_EDGE);
        assertEquals(72, Round.PAGE_EDGE);
    }

    /**
     * No Java part of this page writes the page margin as a literal either.
     *
     * The Java twin of {@code NumbersTest.kt:246-270}, and the reason that test exists: a margin
     * written out at one call site is a margin that stops following when it changes. The scan looks
     * for the number in a coordinate position, which is what {@code RefLayout.at(} takes, so a 72
     * that is a height or a square would not be caught -- the same exemption the Kotlin scan makes
     * for {@code watch/Moments.kt}'s {@code .height(72.ref)}.
     */
    @Test
    public void no_java_part_writes_the_page_margin_again() {
        StringBuilder written = new StringBuilder();
        for (String name : Source.filesUnder(Source.JAVA + "/browse", ".java")) {
            String text = Source.code(Source.JAVA + "/browse/" + name);
            int at = Source.count("RefLayout\\.at\\(\\s*" + Round.PAGE_EDGE + "\\b", text);
            if (at > 0) {
                written.append(name).append(" x").append(at).append(" ");
            }
        }
        assertEquals("every page margin has to go through the constant", "", written.toString());
    }

    // --- the derivations, which are the ones worth having ---------------------------------------

    /**
     * The pitch is the tile plus the gutter beside it, in both languages.
     *
     * {@code NumbersTest.kt}'s third test: 328 is not a measurement, it is 311 and 17 added up, and
     * a tile resized without the pitch following leaves a row with the wrong gap at every tile.
     */
    @Test
    public void the_tile_pitch_came_across() {
        int said = Source.number("private const val STEP = (\\d+)", rows, "STEP in Rows.kt");
        assertEquals("the pitch", said, RowsView.STEP);
        assertEquals(328, RowsView.STEP);
        assertEquals("and it is still the tile plus the 17 gutter beside it",
                Cards.Poster.WIDE + 17, RowsView.STEP);
    }

    /**
     * The first poster beside the open card clears it by the gutter, and both languages still write
     * the three-part derivation rather than the answer.
     *
     * This is the number {@code NumbersTest.kt}'s rule was written for, in its own words: "The card's
     * width appears twice in that file -- once as the card's size and once inside this sum -- and
     * nothing but this test ties them together. A card widened to 920 with the sum left at 904 would
     * overlap the poster by 16 reference pixels, which looks like a drawing bug and is a
     * transcription one."
     *
     * The gutter here is 19 and not the 17 the closed row uses. {@code pstream/notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md:206} records
     * 19 as what the television page used before 2026-11-11 and 17 as what it uses now, so this
     * surface is the one that did not move. Carried as the recorded fact rather than quietly
     * corrected: it is a drawing change on a surface nobody ran, and it belongs to whoever owns the
     * television's geometry, not to its port.
     */
    @Test
    public void the_nineteen_gutter_came_across_as_a_derivation() {
        Matcher said = Source.theOne("\\(PAGE_EDGE \\+ (\\d+) \\+ (\\d+)\\)\\.ref", rows,
                "the next poster's x in Rows.kt");
        int card = Integer.parseInt(said.group(1));
        int gutter = Integer.parseInt(said.group(2));
        assertEquals("the Kotlin sum has to use the card's own width", Cards.Open.WIDE, card);
        assertEquals(19, gutter);

        Matcher mine = Source.theOne(
                "AHEAD_X = Round\\.PAGE_EDGE \\+ Cards\\.Open\\.WIDE \\+ (\\d+);", rowsJava,
                "the next poster's x in RowsView.java");
        assertEquals("the Java has to keep the same three-part sum, not the 995 it comes to",
                gutter, Integer.parseInt(mine.group(1)));
        assertEquals(Round.PAGE_EDGE + card + gutter, RowsView.Open.AHEAD_X);
        assertEquals(995, RowsView.Open.AHEAD_X);
    }

    /**
     * The open card's name is the card less its inset on each side, in both languages.
     *
     * {@code NumbersTest.kt}'s seventh test is the exact defect a reviewer caught on 2026-11-11: a
     * stated derivation giving a different number from the one the code used. Three things have to
     * agree on the Kotlin side -- the comment's stated card width, the card's actual size, and the
     * text width against the inset -- and on the Java side the width is written as the subtraction,
     * so there is nothing to disagree with.
     */
    @Test
    public void the_cards_name_width_came_across_as_a_derivation() {
        Matcher text = Source.theOne(
                "\\.padding\\(start = (\\d+)\\.ref, bottom = \\d+\\.ref\\)\\.width\\((\\d+)\\.ref\\)",
                rows, "the name's box in Rows.kt");
        int inset = Integer.parseInt(text.group(1));
        int wide = Integer.parseInt(text.group(2));
        assertEquals("the inset", inset, Cards.Open.INSET);
        assertEquals("the name's width", wide, Cards.Open.NAME_WIDE);
        assertTrue("the Java has to write it as the subtraction and not as 856",
                cardsJava.contains("NAME_WIDE = WIDE - 2 * INSET"));
        assertEquals(Cards.Open.WIDE - 2 * Cards.Open.INSET, Cards.Open.NAME_WIDE);
        assertEquals(856, Cards.Open.NAME_WIDE);
    }

    /**
     * The hero is the page less its two margins, and both languages still write that and not 888.
     *
     * {@code NumbersTest.kt}'s sixth test says why in one sentence: "A rewrite that writes the
     * answer loses the reason, and the next person to change the margin finds the hero does not
     * follow." This is the rewrite, so the sentence is now a check on two files.
     */
    @Test
    public void the_hero_width_came_across_as_a_derivation() {
        assertTrue("the Kotlin has to keep the derivation",
                hero.contains("(1920 - 2 * PAGE_EDGE).ref"));
        assertTrue("and so does the Java",
                viewJava.contains("Ref.W - 2 * Round.PAGE_EDGE")
                        && heroJava.contains("Ref.W - 2 * Round.PAGE_EDGE"));
        assertEquals("the reference canvas is the 1920 the Kotlin writes out", 1920, Ref.W);
        assertEquals(1776, Ref.W - 2 * Round.PAGE_EDGE);
        assertEquals("which is the 888 dp it comes to", 888f, Ref.dp(Ref.W - 2 * Round.PAGE_EDGE), 0f);
    }

    // --- every remaining number of note 210 section 6 that belongs to this page ------------------

    /** The four row constants beside the pitch. {@code Rows.kt:45-48}. */
    @Test
    public void the_row_constants_came_across() {
        assertEquals(Source.number("private const val AHEAD = (\\d+)", rows, "AHEAD"),
                RowsView.AHEAD);
        assertEquals(Source.number("private const val NEXT_POSTERS = (\\d+)", rows, "NEXT_POSTERS"),
                RowsView.NEXT_POSTERS);
        assertEquals(Source.number("private const val SLIDE_MS = (\\d+)", rows, "SLIDE_MS"),
                RowsView.SLIDE_MS);
        assertEquals(Source.number("private const val FADE_MS = (\\d+)", rows, "FADE_MS"),
                RowsView.FADE_MS);
        assertEquals(5, RowsView.AHEAD);
        assertEquals(6, RowsView.NEXT_POSTERS);
    }

    /**
     * The row's own coordinates: where the posters sit, where the card sits, where the tile the
     * viewer came from peeks in, and where the dim row below goes.
     * {@code Rows.kt:87}, {@code :132}, {@code :137}, {@code :164}, {@code :169}, {@code :173}.
     */
    @Test
    public void the_row_coordinates_came_across() {
        assertEquals(Source.number(
                "fun OpenCard\\([\\s\\S]*?\\.offset\\(PAGE_EDGE\\.ref, (\\d+)\\.ref\\)", rows,
                "the open card's y"), RowsView.Open.CARD_Y);
        Matcher peek = Source.theOne("\\.offset\\(\\((-\\d+)\\)\\.ref, (\\d+)\\.ref\\)", rows,
                "the peeking poster in Rows.kt");
        assertEquals("the tile the viewer came from", Integer.parseInt(peek.group(1)),
                RowsView.Open.BEHIND_X);
        assertEquals("and the band the posters sit in", Integer.parseInt(peek.group(2)),
                RowsView.Open.POSTER_Y);
        assertEquals(-264, RowsView.Open.BEHIND_X);
        assertEquals(202, RowsView.Open.POSTER_Y);
        assertEquals(200, RowsView.Open.CARD_Y);

        Matcher tops = Source.theOne("val top = if \\(high\\) (\\d+) else (\\d+)", rows,
                "the dim row's two tops");
        assertEquals(Integer.parseInt(tops.group(1)), RowsView.Next.HIGH_Y);
        assertEquals(Integer.parseInt(tops.group(2)), RowsView.Next.LOW_Y);
        Matcher names = Source.theOne("\\(if \\(high\\) (\\d+) else (\\d+)\\)\\.ref", rows,
                "the dim row's two name tops");
        assertEquals(Integer.parseInt(names.group(1)), RowsView.Next.HIGH_NAME_Y);
        assertEquals(Integer.parseInt(names.group(2)), RowsView.Next.LOW_NAME_Y);

        assertTrue("the closed row still steps back by one pitch for the tile before",
                rows.contains("(PAGE_EDGE - STEP).ref"));
        assertTrue("and the Java does the same",
                rowsJava.contains("Round.PAGE_EDGE - STEP"));
        assertTrue("and still starts on the margin and steps by the pitch",
                rows.contains("(PAGE_EDGE + index * STEP).ref"));
        assertTrue(rowsJava.contains("Round.PAGE_EDGE + slot * STEP"));
    }

    /**
     * The curve every move on this page uses, with its four control points unchanged.
     *
     * Note 210 section 6's own row: {@code CubicBezierEasing(0.22f, 0.61f, 0.2f, 1f)} becomes
     * {@code new PathInterpolator(0.22f, 0.61f, 0.2f, 1f)}, "the same four control points, exact".
     * Both are a cubic Bezier through (0,0) and (1,1) with those two handles, so this is the one
     * animation number in the port where nothing is approximated -- which is worth a check, because
     * a reader cannot tell a wrong curve from a right one by looking at a screen.
     */
    @Test
    public void the_glide_curve_came_across_exactly() {
        Matcher said = Source.theOne(
                "CubicBezierEasing\\((0\\.\\d+)f, (0\\.\\d+)f, (0\\.\\d+)f, (\\d+)f\\)", rows,
                "the Glide easing in Rows.kt");
        Matcher mine = Source.theOne(
                "new PathInterpolator\\((0\\.\\d+)f, (0\\.\\d+)f, (0\\.\\d+)f, (\\d+)f\\)", rowsJava,
                "the Glide interpolator in RowsView.java");
        for (int point = 1; point <= 4; point++) {
            assertEquals("control point " + point, said.group(point), mine.group(point));
        }
        assertEquals("0.22", said.group(1));
    }

    /** The hero's own coordinates and the three gaps inside it. {@code HeroCard.kt:53-93}. */
    @Test
    public void the_hero_coordinates_came_across() {
        Matcher spot = Source.theOne("\\.offset\\(PAGE_EDGE\\.ref, (\\d+)\\.ref\\)\\s*\\.size", hero,
                "the hero's y in HeroCard.kt");
        assertEquals(Integer.parseInt(spot.group(1)), HeroView.Y);
        assertEquals(Source.number("PAGE_EDGE\\)\\.ref, (\\d+)\\.ref\\)", hero, "the hero's height"),
                HeroView.HIGH);
        assertEquals(143, HeroView.Y);
        assertEquals(767, HeroView.HIGH);

        Matcher column = Source.theOne(
                "padding\\(start = (\\d+)\\.ref, bottom = (\\d+)\\.ref\\)\\.width\\((\\d+)\\.ref\\)",
                hero, "the hero's column in HeroCard.kt");
        assertEquals(Integer.parseInt(column.group(1)), HeroView.INSET);
        assertEquals(Integer.parseInt(column.group(2)), HeroView.FOOT);
        assertEquals(Integer.parseInt(column.group(3)), HeroView.COLUMN);
        assertEquals(30, HeroView.INSET);
        assertEquals(34, HeroView.FOOT);
        assertEquals(760, HeroView.COLUMN);
    }

    /** The bar's centre line, the height of its round parts, the face, and the two columns. */
    @Test
    public void the_bar_numbers_came_across() {
        assertEquals(Source.number("private const val BAR_MID = (\\d+)", bar, "BAR_MID"),
                BarView.BAR_MID);
        assertEquals(Source.number("private const val BAR_HIGH = (\\d+)", bar, "BAR_HIGH"),
                BarView.BAR_HIGH);
        assertEquals(Source.number("private const val FACE = (\\d+)", bar, "FACE"), BarView.FACE);
        assertEquals(68, BarView.BAR_MID);
        assertEquals(64, BarView.BAR_HIGH);
        assertEquals(48, BarView.FACE);

        assertEquals(Source.number("\\.offset\\((\\d+)\\.ref, \\(BAR_MID - BAR_HIGH / 2\\)\\.ref\\)\\s*\\.size",
                bar, "search's x"), BarView.SEARCH_X);
        assertEquals(Source.number("Row\\(Modifier\\.offset\\((\\d+)\\.ref", bar, "the tabs' x"),
                BarView.TABS_X);
        assertEquals(Source.number("Spacer\\(Modifier\\.width\\((\\d+)\\.ref\\)\\)", bar,
                "the gap between tabs"), BarView.TAB_GAP);
        assertEquals(539, BarView.SEARCH_X);
        assertEquals(609, BarView.TABS_X);
        assertEquals(8, BarView.TAB_GAP);

        assertEquals("six tabs fit the bar",
                Source.number("\\.take\\((\\d+)\\)", bar, "how many tabs fit"), BarView.TABS);
        assertTrue("and a genre is not one of them",
                bar.contains("it.screen.value != \"genre\""));
        assertEquals("genre", BarView.NOT_A_TAB);

        assertTrue("the mark still sits a margin and a bar height in from the far edge",
                bar.contains("(1920 - PAGE_EDGE - BAR_HIGH).ref"));
        assertTrue(barJava.contains("Ref.W - Round.PAGE_EDGE - BAR_HIGH"));
    }

    /** The bar's two negative places. {@code TopBar.kt:31} and {@code :34}. */
    @Test
    public void the_two_negative_bar_places_came_across() {
        assertEquals(Source.number("const val SEARCH_TAB = (-\\d+)", bar, "SEARCH_TAB"),
                Feed.SEARCH_TAB);
        assertEquals(Source.number("const val WHO_TAB = (-\\d+)", bar, "WHO_TAB"), Feed.WHO_TAB);
        assertEquals(-2, Feed.SEARCH_TAB);
        assertEquals(-3, Feed.WHO_TAB);
    }

    /**
     * The row rise and the read-ahead reach, which are the three numbers this port left with one
     * home instead of two.
     *
     * {@code RISE = 810}, {@code RISE_MS = 320} and {@code REACH = 2} were {@code Browse.kt:42-43}
     * and {@code :53}. Converting that file removed the Compose {@code AnimatedContent} they drove,
     * so unlike every other number in this test there is no Kotlin left to hold them against. Their
     * Kotlin home is gone, not their value: {@code BrowseView.RISE}, {@code BrowseView.RISE_MS} and
     * {@code Place.REACH} carry them with the citation and the derivation.
     *
     * So this asserts the end state note 210 wants for every one of these numbers -- exactly one
     * home -- in both directions. If somebody re-adds a Kotlin copy, there are two homes and this
     * fails; if somebody deletes the Java constant, the literal is gone and this fails. It is the
     * weakest test in this file and it is the honest one, and the report names these three as the
     * numbers whose cross-check was spent.
     */
    @Test
    public void the_rise_and_the_reach_have_exactly_one_home() {
        assertTrue("the Java holds the rise as a literal",
                viewJava.contains("RISE = 810;") && viewJava.contains("RISE_MS = 320;"));
        assertTrue("and the reach",
                Source.code(Source.JAVA + "/browse/Place.java").contains("REACH = 2;"));
        assertEquals(810, BrowseView.RISE);
        assertEquals(320, BrowseView.RISE_MS);
        assertEquals(2, Place.REACH);
        assertEquals("and the Kotlin must not hold a second copy of any of the three", "",
                (browse.contains("RISE") ? "RISE " : "")
                        + (browse.contains("RISE_MS") ? "RISE_MS " : "")
                        + (browse.contains("const val REACH") ? "REACH " : ""));
    }

    /** The brand's corner scale on a television. {@code Round.kt:18}. */
    @Test
    public void the_corner_scale_came_across() {
        Matcher said = Source.theOne("private const val TV_SCALE = (\\d\\.\\d)f", round, "TV_SCALE");
        assertEquals(Float.parseFloat(said.group(1)), Round.TV_SCALE, 0f);
        assertEquals(1.5f, Round.TV_SCALE, 0f);
        assertTrue("the Java rounds it the same way the Kotlin does",
                round.contains("(brand * TV_SCALE).roundToInt()")
                        && roundJava.contains("Math.round(brand * TV_SCALE)"));
        assertEquals("and the roles are still derived from the field and not restated",
                Round.FIELD / 2, Round.CHIP);
        assertEquals(Round.FIELD * 2 / 3, Round.KEY);
    }

    /**
     * The six brand colours are the same six on both sides.
     *
     * This is the assertion that makes {@link Ink} safe. {@code Theme.kt:16-21} holds them as
     * Compose {@code Color} values, which are a {@code @JvmInline value class} over a {@code ULong}
     * and so unreadable from Java, so the Views surface has to restate them -- and a brand colour
     * stated twice is drift waiting to happen. Held equal here rather than by care.
     */
    @Test
    public void the_brand_colours_came_across() {
        assertEquals("Back", colour("Back"), Ink.BACK);
        assertEquals("Plate", colour("Plate"), Ink.PLATE);
        assertEquals("Raised", colour("Raised"), Ink.RAISED);
        assertEquals("Muted", colour("Muted"), Ink.MUTED);
        assertEquals("Accent", colour("Accent"), Ink.ACCENT);
        assertEquals("Join", colour("Join"), Ink.JOIN);
    }

    private int colour(String name) {
        Matcher said = Source.theOne("val " + name + " = Color\\(0x([0-9A-Fa-f]{8})\\)", theme,
                name + " in Theme.kt");
        return (int) Long.parseLong(said.group(1), 16);
    }

    /** White and black at a share of their strength, as hundredths, read the same by both sides. */
    @Test
    public void a_share_of_a_colour_is_the_share_the_kotlin_asks_for() {
        Matcher edge = Source.theOne(
                "border\\((\\d+)\\.ref, Color\\.White\\.copy\\(alpha = 0\\.(\\d+)f\\)",
                hero, "the hero's border in HeroCard.kt");
        assertTrue("the Java asks for the same stroke and the same share",
                heroJava.contains("Round.PANEL, Ink.PLATE, " + edge.group(1)
                        + ", Ink.white(" + edge.group(2) + "))"));
        assertEquals("and a share in hundredths is that share of full strength",
                Math.round(255 * 0.28f), Ink.white(28) >>> 24);
        assertEquals(255, Ink.white(100) >>> 24);
        assertEquals(0, Ink.white(0) >>> 24);
        assertEquals("white is white at every share", 0xFFFFFF, Ink.white(40) & 0xFFFFFF);
        assertEquals("and black is black", 0, Ink.black(40) & 0xFFFFFF);
        assertEquals("a dimmed colour keeps its own colour and loses its strength",
                Ink.MUTED & 0xFFFFFF, Ink.dim(Ink.MUTED, 50) & 0xFFFFFF);
        assertEquals(128, Ink.dim(Ink.MUTED, 50) >>> 24);
    }

    /**
     * A series is a title whose key starts with the four characters {@code shared/Addresses.kt:17}
     * tests for, which is what the hero's Play asks before it plays anything.
     */
    @Test
    public void the_series_test_came_across() {
        Matcher said = Source.theOne("startsWith\\(\"([a-z]+:)\"\\)", addresses,
                "the series prefix in Addresses.kt");
        Feed.Card series = new Feed.Card(said.group(1) + "9", "", "", "", null);
        Feed.Card film = new Feed.Card("movie:9", "", "", "", null);
        assertTrue(series.series());
        assertTrue("the Java has to test for the same prefix",
                Source.code(Source.JAVA + "/browse/Feed.java")
                        .contains("key.startsWith(\"" + said.group(1) + "\")"));
        assertTrue("and for nothing else", !film.series());
    }

    // --- the parts duplicated from tv/parts, held to their originals -----------------------------

    /**
     * The three numbers of a badge, the two of a facts dot, and the four of a pill, held against
     * {@code tv/parts/}.
     *
     * {@link Parts} duplicates four composables that belong to wave 2's package T4, because a
     * composable cannot be a child of a {@code ViewGroup} and the hero needs its buttons before T4
     * lands. This is what makes the duplication safe until then: a badge or a pill resized in one
     * place and not the other fails here, and when T4 lands this test is what tells it which numbers
     * it has to carry.
     */
    @Test
    public void the_borrowed_parts_kept_their_numbers() {
        String badge = Source.code(Source.KOTLIN + "/parts/Badge.kt");
        String facts = Source.code(Source.KOTLIN + "/parts/Facts.kt");
        String pill = Source.code(Source.KOTLIN + "/parts/Pill.kt");

        int gap = Source.number("spacedBy\\((\\d+)\\.ref\\)", badge, "a badge's gap");
        assertTrue("a badge row still spaces its badges by " + gap,
                partsJava.contains("spot.leftMargin = Ref.px(context, " + gap + ");"));
        Matcher chip = Source.theOne(
                "padding\\(horizontal = (\\d+)\\.ref, vertical = (\\d+)\\.ref\\)", badge,
                "a badge's padding");
        assertTrue("a badge still pads " + chip.group(1) + " and " + chip.group(2),
                partsJava.contains("Ref.px(context, " + chip.group(1) + "), Ref.px(context, "
                        + chip.group(2) + "), Ref.px(context, " + chip.group(1) + "),"));
        int badgeType = Source.number("fontSize = (\\d+)\\.refType", badge, "a badge's size");
        assertTrue("a badge's words are still " + badgeType + " reference pixels",
                partsJava.contains("chip = Type.line(context, " + badgeType + ", Type.BOLD"));

        int factType = Source.number("fontSize = (\\d+)\\.refType", facts, "a fact's size");
        assertTrue("and a fact's are still " + factType,
                partsJava.contains("fact = Type.line(context, " + factType + ", Type.BOLD"));

        Matcher dot = Source.theOne(
                "padding\\(horizontal = (\\d+)\\.ref\\)\\.size\\((\\d+)\\.ref\\)", facts,
                "the dot between two facts");
        assertTrue("the dot is still " + dot.group(2) + " across",
                partsJava.contains("Ref.px(context, " + dot.group(2) + "), Ref.px(context, "
                        + dot.group(2) + ")"));
        assertTrue("and still " + dot.group(1) + " from each fact",
                partsJava.contains("leftMargin = Ref.px(context, " + dot.group(1) + ")")
                        && partsJava.contains("rightMargin = Ref.px(context, " + dot.group(1) + ")"));

        Matcher shape = Source.theOne(
                "height\\((\\d+)\\.ref\\)\\.background\\(back, CircleShape\\)\\.padding\\(horizontal = (\\d+)\\.ref\\)",
                pill, "a pill's height and padding");
        assertTrue("a pill is still " + shape.group(1) + " high",
                partsJava.contains("Round.pill(getContext(), " + shape.group(1) + ","));
        assertTrue("and still pads " + shape.group(2),
                partsJava.contains("setPadding(Ref.px(context, " + shape.group(2) + "), 0,"));
    }
}
