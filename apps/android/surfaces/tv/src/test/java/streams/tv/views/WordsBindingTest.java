package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import java.util.HashMap;
import java.util.Map;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import streams.shared.Speaking;
import streams.shared.ThemeKt;

/**
 * The words of the interface, bound at run time.
 *
 * All of them arrive from the server per language ({@code Seen.words}), so there is no build-time
 * copy of any of them and an Android string resource could only hold a stale one. What is tested
 * here is the two answers a surface depends on: a key the words hold reads what they hold, and a key
 * they do not hold reads nothing at all.
 *
 * The values are form names rather than real sentences on purpose: this file holds no word a viewer
 * would see, in any language. The one real fact it reads is read out of {@code strings/en.json}.
 */
public class WordsBindingTest {
    /** The isolates {@code Theme.kt:66-67} wraps a filled value in, so a number keeps its own order. */
    private static final String OPEN = "⁨";
    private static final String CLOSE = "⁩";

    private static final String SEASON = "title.season";
    private static final String SEARCH = "nav.search";

    private String code;
    private String digits;

    @Before
    public void readWhatTheLanguageWas() {
        code = Speaking.INSTANCE.getCode();
        digits = Speaking.INSTANCE.getDigits();
    }

    @After
    public void putTheLanguageBack() {
        Speaking.INSTANCE.setCode(code);
        Speaking.INSTANCE.setDigits(digits);
    }

    /** English, with the forms English really has: one and many, and no other. */
    private static Map<String, String> english() {
        Map<String, String> book = new HashMap<>();
        book.put(SEARCH, "{n}:search");
        book.put(SEASON + ".one", "{n}:one");
        book.put(SEASON + ".many", "{n}:many");
        return book;
    }

    /** And a language with every form, as Arabic has. */
    private static Map<String, String> everyForm() {
        Map<String, String> book = english();
        book.put(SEASON + ".two", "{n}:two");
        book.put(SEASON + ".few", "{n}:few");
        book.put(SEASON + ".other", "{n}:other");
        return book;
    }

    private static Words wordsOf(Map<String, String> book) {
        Words words = new Words();
        words.read(book);
        return words;
    }

    /** A key the words hold reads what they hold. */
    @Test
    public void a_key_the_words_hold_reads_what_they_hold() {
        assertEquals("{n}:search", wordsOf(english()).of(SEARCH));
    }

    /**
     * A key the words do not hold reads nothing: not the key, not English, not a dash.
     *
     * The reason is at {@code shared/Says.kt:46-52} and {@code tv/app/Spoken.kt:41-42}, and it is
     * about what a screen reader says: a part labelled with a placeholder is read out as the
     * placeholder, while a part that draws nothing is a visible gap and gets fixed. The key is built
     * here out of pieces so that this file does not contain a key-shaped literal that no language
     * holds -- a scan of the surfaces for the keys they ask for would otherwise read it as a gap.
     */
    @Test
    public void a_key_the_words_do_not_hold_reads_nothing() {
        String absent = SEASON + "." + "forty";
        assertEquals("", wordsOf(english()).of(absent));
        assertEquals("", wordsOf(english()).counted(absent, 2));
    }

    /** And a run that has not read its language yet says nothing for every key. */
    @Test
    public void a_run_that_has_not_read_its_language_says_nothing() {
        Words words = new Words();
        assertEquals(0, words.size());
        assertEquals("", words.of(SEARCH));
        assertEquals("", words.counted(SEASON, 1));
    }

    /** Fresh words are read; the same words again are not a change, so a screen is not rebound for nothing. */
    @Test
    public void fresh_words_are_a_change_and_the_same_words_are_not() {
        Words words = new Words();
        Map<String, String> book = english();
        assertTrue(words.read(book));
        assertFalse(words.read(book));
        assertTrue(words.read(everyForm()));
        assertEquals(6, words.size());
    }

    /**
     * A counted word takes the plural form of the language, and {@code {n}} is filled in isolated.
     *
     * Checked on the forms English has rather than on the forms CLDR names, because that is where
     * the fallback does the work.
     */
    @Test
    public void a_counted_word_takes_the_plural_form_of_the_language() {
        Speaking.INSTANCE.setCode("en");
        Speaking.INSTANCE.setDigits("latn");
        Words words = wordsOf(english());
        assertEquals(OPEN + "1" + CLOSE + ":one", words.counted(SEASON, 1));
    }

    /**
     * And the fallback order is the one that makes two seasons read correctly in English.
     *
     * English has {@code title.season.one} and {@code title.season.many} and no {@code other}
     * (read below out of the strings themselves), while CLDR's form for two in English is
     * {@code other}. So the chain has to be form, then other, then many, then one: two seasons land
     * on {@code many}. An order that tried {@code one} before {@code many} would read "1 Season" for
     * every count above one, which is the kind of defect that ships because the singular looks like
     * a plural at a glance.
     */
    @Test
    public void two_of_something_falls_through_other_to_many_and_not_to_one() {
        Speaking.INSTANCE.setCode("en");
        Speaking.INSTANCE.setDigits("latn");
        Words words = wordsOf(english());
        assertEquals("other", ThemeKt.pluralOf(2, "en"));
        assertEquals(OPEN + "2" + CLOSE + ":many", words.counted(SEASON, 2));
        assertEquals(OPEN + "7" + CLOSE + ":many", words.counted(SEASON, 7));
    }

    /** The fact the test above rests on, read out of the strings and not remembered. */
    @Test
    public void english_really_has_no_other_form_for_the_family_above() {
        String book = Repo.text("strings/en.json");
        assertTrue("English has to hold the form the fallback lands on",
                book.contains("\"" + SEASON + ".many\""));
        assertFalse("and not the form CLDR asks for, or the fallback would not be exercised",
                book.contains("\"" + SEASON + ".other\""));
    }

    /**
     * A language with every form uses its own, and its digits.
     *
     * Two seasons is the form named for two, three is the form named for a few, and the number is
     * written in the digits the language writes numbers in -- all three through
     * {@code Theme.kt}'s own rules, which this calls rather than copies.
     */
    @Test
    public void a_language_with_every_form_uses_its_own_form_and_its_own_digits() {
        Speaking.INSTANCE.setCode("ar");
        Speaking.INSTANCE.setDigits("arab");
        Words words = wordsOf(everyForm());
        assertEquals(OPEN + "٢" + CLOSE + ":two", words.counted(SEASON, 2));
        assertEquals(OPEN + "٣" + CLOSE + ":few", words.counted(SEASON, 3));
        assertEquals(OPEN + "١٥" + CLOSE + ":many", words.counted(SEASON, 15));
        assertEquals(OPEN + "٠" + CLOSE + ":other", words.counted(SEASON, 0));
    }

    /**
     * And the Java answer is the Kotlin answer, for every form of two languages.
     *
     * The drift check. {@code Theme.kt:131}'s {@code counted} is the statement of this rule that the
     * Compose surfaces read; this one is what the Java surfaces read. While both exist they have to
     * agree, key for key and count for count, or one surface of one language reads a different word
     * from the other.
     */
    @Test
    public void the_java_counted_word_is_the_kotlin_counted_word() {
        Map<String, String> book = everyForm();
        Words words = wordsOf(book);
        for (String language : new String[] {"en", "ar", "ru", "pl", "ja", "fr"}) {
            Speaking.INSTANCE.setCode(language);
            for (int n = 0; n <= 25; n++) {
                assertEquals(language + " " + n, ThemeKt.counted(book, SEASON, n), words.counted(SEASON, n));
            }
        }
    }
}
