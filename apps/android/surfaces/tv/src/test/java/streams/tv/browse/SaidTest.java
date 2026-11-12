package streams.tv.browse;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

import java.util.List;
import java.util.SortedMap;
import java.util.TreeMap;
import org.junit.Test;
import streams.tv.views.Says;

/**
 * What this page hands a screen reader, and whether its parts still speak at all.
 *
 * <h2>Why a census and not only a set of readings</h2>
 * {@code src/test/kotlin/streams/tv/SpokenTest.kt}'s last test holds the spoken call sites per file,
 * exactly, and says why in its own words: "A total alone would let a rewrite move a screen's labels
 * onto another screen and stay green, so the shape is held per file. Which means this map has to be
 * edited whenever a label is added, moved or removed -- and that is the point: note 215's open item
 * is that S3 can drop any of the semantics call sites and every check in the repository still
 * passes. Editing this map is a rewriter stating, in the diff, which parts stopped speaking."
 *
 * This is <b>the rewrite that test was written against</b>, and its census still passes untouched,
 * because the Kotlin it counts is still in the tree. So that test now proves nothing about the
 * screen a viewer sees, and this file is what replaces it for this package: the same census over the
 * Java, with the Kotlin's own counts for the three files that were converted written beside it.
 *
 * <h2>What this does not prove, said plainly</h2>
 * It is a census of call sites and a reading of values, not a reading of a tree. What a real screen
 * reader is handed on a real screen needs {@code uiautomator dump} on the Streamer, which is note
 * 210's wave 2 check 4 and is not claimed here.
 */
public class SaidTest {
    /**
     * The spoken parts of this page, per file, exactly. Counted against the Kotlin they replaced.
     *
     * The Kotlin's own census, from {@code SpokenTest.kt:205-218}: {@code browse/HeroCard.kt} 3,
     * {@code browse/Rows.kt} 2, {@code browse/TopBar.kt} 3, and {@code browse/Browse.kt} 0 -- 8 in
     * all for this package. The Java carries more, and every extra one is named below, because a
     * number that only went up is not evidence of anything by itself.
     */
    private static final SortedMap<String, Integer> SPOKEN = new TreeMap<String, Integer>();

    static {
        // Counted by this test's own scan at the tree it was written against, 2026-11-12. The
        // Kotlin's own census for this package, from `SpokenTest.kt:205-218`: `browse/HeroCard.kt`
        // 3, `browse/Rows.kt` 2, `browse/TopBar.kt` 3 -- 8 in all. The Java carries 20, and the
        // difference is not more speech, it is speech Compose performed without a call site:
        //  * every drawn box that is not a control -- a shade, a border, a plate, an icon beside
        //    its own word -- has to be marked, because in Views an unmarked drawn box is a node a
        //    reader stops on and reads out as nothing. That is the defect `Says.kt:16-21` measured
        //    on this very screen: Home drew 61 nodes, 19 reachable, all 19 silent;
        //  * Compose's `Icon(painter, null)` and `AsyncImage(url, null)` carried the same meaning
        //    in their null argument, which is not a call site and so was never counted.
        // So the right comparison is per part and not per number, and the comment on each line
        // below says which parts.

        // The closed tile's name (`Rows.kt:61`), the open card's own words (`:91`), and the three
        // drawn boxes Views has to mark: the foot shade (`:96`), the white focus border (`:110`)
        // and -- inside `Art` -- the artwork itself (`:94`, Compose's null argument).
        SPOKEN.put("Cards.java", 4);
        // The artwork and the two shades over it (`HeroCard.kt:61-63`) and the logo, which says the
        // title's name because the logo is the name drawn as a picture (`:70-76`).
        SPOKEN.put("HeroView.java", 3);
        // The profile (`TopBar.kt:83`), search (`:101`) and a tab (`:56`), plus the caret and the
        // search glass, which are `Icon(painter, null)` today. Search is written twice because its
        // reading needs the word and the focus, and they arrive from different places.
        SPOKEN.put("BarView.java", 6);
        // A row's name, read as what it means and not as a button, once for each of the two rows
        // this file draws.
        SPOKEN.put("RowsView.java", 2);
        // The dot between two facts, the pill's icon, the pill's own drawn words, and the face.
        SPOKEN.put("Parts.java", 4);
        // Every picture declares what it is where it is built: its words, or decoration.
        SPOKEN.put("Art.java", 1);
    }

    /** Which of this package's Java files speak, and how often. */
    @Test
    public void every_part_that_spoke_still_speaks() {
        SortedMap<String, Integer> census = new TreeMap<String, Integer>();
        List<String> files = Source.filesUnder(Source.JAVA + "/browse", ".java");
        assertFalse("the scan found no files, so it has stopped reading the package", files.isEmpty());
        for (String name : files) {
            String text = Source.code(Source.JAVA + "/browse/" + name);
            int calls = Source.count("Says\\.onto\\(", text);
            if (calls > 0) {
                census.put(name, Integer.valueOf(calls));
            }
        }
        assertEquals(SPOKEN, census);
        assertEquals("twenty readings across six files", 20, total(census));
    }

    private static int total(SortedMap<String, Integer> census) {
        int sum = 0;
        for (Integer one : census.values()) {
            sum += one.intValue();
        }
        return sum;
    }

    /**
     * No part is given English words from the code.
     *
     * The Java twin of {@code SpokenTest.kt:142-157}, and the rule is {@code Spoken.kt:34-37}'s: the
     * words are never written here, they arrive from {@code strings/{lang}.json} through
     * {@code seen.words}. An English literal in a label is invisible on an English build and is the
     * whole of what a person hears in the other seventeen languages.
     *
     * The scan looks at what is handed to the two helpers that take words. A reading built from a
     * key is allowed, because a key is resolved through {@code Words} and answers empty when the
     * strings do not hold it; a reading built from a literal string is not.
     */
    @Test
    public void no_part_is_given_english_words_from_the_code() {
        StringBuilder found = new StringBuilder();
        int sites = 0;
        for (String name : Source.filesUnder(Source.JAVA + "/browse", ".java")) {
            String text = Source.code(Source.JAVA + "/browse/" + name);
            java.util.regex.Matcher said = java.util.regex.Pattern
                    .compile("Says\\.(says|stop|means|saysItsWords)\\(\\s*\"([^\"]*)\"")
                    .matcher(text);
            while (said.find()) {
                if (!said.group(2).isEmpty()) {
                    found.append(name).append(" ").append(said.group()).append("; ");
                }
            }
            sites += Source.count("Says\\.(says|stop|means|saysItsWords)\\(", text);
        }
        // Ten at the tree this was written against: two in `Cards`, one in `HeroView`, three in
        // `BarView`, two in `RowsView`, one in `Parts`, one in `Art`. A floor and not an equality,
        // because the census above is what holds the shape; this only has to notice a scan that
        // has stopped reading the package altogether.
        assertTrue("the scan found no call sites at all, so it has stopped reading", sites >= 10);
        assertEquals("every spoken label has to come from the words or the catalogue", "",
                found.toString());
    }

    // --- the readings themselves ----------------------------------------------------------------

    /**
     * A tile says the title's name, because the artwork is the title.
     *
     * {@code Rows.kt:56-57}: "The artwork is the title, so the title's name is what a person hears
     * in place of seeing it; the badges drawn on top say the rest themselves."
     */
    @Test
    public void a_tile_says_the_titles_name() {
        Says.Said said = Says.says("Stranger Things");
        assertEquals("Stranger Things", said.label);
        assertEquals(Says.Role.BUTTON, said.role);
        assertFalse(said.silent());
    }

    /**
     * A tile with no name falls through to what is inside it rather than being described as nothing.
     *
     * {@code Says.kt:54-62}, exactly: "An empty label is left alone rather than set: a control
     * described as nothing is worse than one a reader falls through to its contents for, and the
     * words are empty for as long as a run has not read its language yet." This is the state a
     * browse page is in for the first seconds of every run.
     */
    @Test
    public void a_tile_with_no_name_is_not_described_as_nothing() {
        Says.Said said = Says.says("");
        assertTrue(said.silent());
        assertTrue("it gathers what is drawn inside it instead", said.gathers);
    }

    /**
     * The open card's own drawn words are its label, and it always says the remote is on it, because
     * it is the one card of the row that is open. {@code Rows.kt:83-91}.
     */
    @Test
    public void the_open_card_says_its_own_words_and_that_the_remote_is_on_it() {
        Says.Said said = Says.saysItsWords(true);
        assertTrue(said.gathers);
        assertEquals(Boolean.TRUE, said.selected);
        assertFalse("and carries no words of its own, or they would drift from what is drawn",
                said.label.length() > 0);
    }

    /**
     * The profile at the left of the bar says whose it is and whether the remote is on it, either
     * way. {@code TopBar.kt:83}.
     *
     * {@code selected} is set when it is false as well, which {@code SpokenTest.kt:29-34} explains:
     * "A reader that is told nothing when the remote leaves a part has no way to know it left, and
     * on a surface whose focus is painted rather than held by the toolkit there is nothing else to
     * ask."
     */
    @Test
    public void the_profile_says_whose_it_is_and_whether_the_remote_is_on_it() {
        Says.Said on = Says.stop("Ibrahim", true);
        assertEquals("Ibrahim", on.label);
        assertEquals(Boolean.TRUE, on.selected);
        Says.Said off = Says.stop("Ibrahim", false);
        assertEquals("a part the remote has left must say so, not fall silent",
                Boolean.FALSE, off.selected);
    }

    /** And with no words for it, it still says where the remote is. {@code Spoken.kt:49-53}. */
    @Test
    public void a_stop_with_no_words_still_says_where_the_remote_is() {
        Says.Said said = Says.stop("", true);
        assertEquals(Boolean.TRUE, said.selected);
        assertTrue(said.silent());
    }

    /**
     * A row's name is read as what it means and claims no role: nothing presses a row heading.
     *
     * {@code SpokenTest.kt:61-65} is the test that separates the two, and its reason is the one that
     * matters here: "A rewrite that collapses the two makes a reader offer to press every label on
     * the screen."
     */
    @Test
    public void a_row_name_is_described_and_does_not_claim_to_be_pressable() {
        Says.Said said = Says.means("Trending Now");
        assertEquals("Trending Now", said.label);
        assertEquals(Says.Role.NONE, said.role);
        assertNull("and the remote does not stop on it", said.selected);
    }

    /** A row with no name says nothing at all rather than adding a silent stop. */
    @Test
    public void a_row_with_no_name_adds_nothing() {
        assertTrue(Says.means("").nothing);
        assertTrue(Says.means(null).nothing);
    }

    /**
     * The shades, the borders and the plates go away with their subtrees instead of becoming silent
     * stops. {@code HeroCard.kt:61-63} and the five drawn boxes this port added.
     */
    @Test
    public void a_drawn_shade_goes_away_instead_of_becoming_a_silent_stop() {
        Says.Said said = Says.decoration();
        assertTrue(said.cleared);
        assertTrue(said.silent());
        assertEquals(Says.Role.NONE, said.role);
    }

    /**
     * A title and the row it sits in are heard as one label, joined by a line end.
     *
     * {@code Spoken.kt:74-78} and {@code SpokenTest.kt:110-122}: the break is a line end rather than
     * a word or a comma, so no language's punctuation or word order is decided in the code. A comma
     * joiner reads wrong in Arabic and Japanese and is the obvious thing a rewrite would reach for,
     * so the absence of one is stated rather than assumed. {@link Place#says} is the one caller.
     */
    @Test
    public void a_title_and_its_row_are_heard_as_one_label() {
        assertEquals("Stranger Things\nTrending Now",
                Says.sayBoth("Stranger Things", "Trending Now"));
        assertFalse(Says.sayBoth("a", "b").contains(","));
        assertEquals("Trending Now", Says.sayBoth(null, "Trending Now"));
        assertEquals("Stranger Things", Says.sayBoth("Stranger Things", ""));
        assertEquals("", Says.sayBoth("", ""));
    }

    /**
     * Every key this page asks the words for is a key and not a sentence.
     *
     * The television surface has no words test of its own -- note 210's open items record it as a
     * defect today, with 169 distinct key shaped literals and nothing checking them -- and
     * {@code check_strings.py} reading the surfaces is wave 0's package S3. This is not that check:
     * it only holds this package's own keys to the shape {@code WordsTest.kt:65} uses, so a sentence
     * typed where a key belongs fails here even before S3 lands.
     */
    @Test
    public void every_word_this_page_asks_for_is_a_key() {
        StringBuilder wrong = new StringBuilder();
        int keys = 0;
        for (String name : Source.filesUnder(Source.JAVA + "/browse", ".java")) {
            String text = Source.code(Source.JAVA + "/browse/" + name);
            java.util.regex.Matcher found = java.util.regex.Pattern
                    .compile("(?:words|said)\\.of\\(\\s*\"([^\"]+)\"|"
                            + "Pill\\(context, \"([^\"]+)\"")
                    .matcher(text);
            while (found.find()) {
                String key = found.group(1) != null ? found.group(1) : found.group(2);
                keys++;
                if (!key.matches("[a-z][a-z0-9_]*(\\.[a-z0-9_]+)+")) {
                    wrong.append(name).append(" ").append(key).append("; ");
                }
            }
        }
        assertTrue("the scan found no keys, so it has stopped reading", keys >= 3);
        assertEquals("", wrong.toString());
    }
}
