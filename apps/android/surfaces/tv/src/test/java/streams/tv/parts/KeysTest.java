package streams.tv.parts;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

import org.junit.Test;
import streams.tv.views.Word;

/**
 * The keyboard's key map, pressed. The first tests it has ever had.
 *
 * {@code parts/Keyboard.kt} is read by four screens -- sign in, add profile, edit profile and the
 * lock steps -- and every one of them is a screen a viewer cannot get past if a key does the wrong
 * thing. Nothing checked any of it: {@code KeyboardState} held five Compose snapshot delegates and
 * {@code press} was an extension function over a private list of caps, so a test in this source set
 * could not name either. {@link Keys} is that state machine with the drawing taken out, and this is
 * what the separation was for.
 *
 * Each test says what a viewer would see if it failed, because that is what decides whether the
 * behaviour is worth holding still.
 */
public class KeysTest {
    /** Collects what a finished keyboard answered, and whether it answered at all. */
    private static final class Ended implements Keys.Finish {
        String text;
        boolean ended;

        @Override
        public void finished(String text) {
            this.text = text;
            this.ended = true;
        }
    }

    private final Ended end = new Ended();
    private final Keys keys = new Keys("");

    private boolean press(Word word) {
        return keys.press(word, "Cancel", "Done", end);
    }

    private void press(Word... words) {
        for (Word word : words) {
            press(word);
        }
    }

    private String on() {
        return keys.here("Cancel", "Done").word;
    }

    /**
     * The remote starts in the middle of the home row, which is {@code Keyboard.kt:74-75}, and that
     * is the letter g. A start that moved would put a viewer's first press somewhere else on every
     * one of the four screens at once.
     */
    @Test
    public void startsOnTheMiddleOfTheHomeRow() {
        assertEquals(Keys.START_ROW, keys.row());
        assertEquals(Keys.START_COLUMN, keys.column());
        assertEquals("g", on());
    }

    /**
     * Up from the top row is the one word the keyboard does not take, which is how the page above
     * it gets the remote back. If it took it, a viewer on the top row could never leave the
     * keyboard by pressing up and the screen would feel stuck.
     */
    @Test
    public void upFromTheTopRowIsLeftForThePage() {
        press(Word.UP, Word.UP);
        assertEquals(0, keys.row());
        assertFalse("up on the top row must fall through to the page", press(Word.UP));
        assertEquals(0, keys.row());
    }

    /**
     * Every other word is used, even when it moves nothing. A keyboard that let left escape
     * sideways at the first column would lose the viewer's place in the middle of a word.
     */
    @Test
    public void theEdgesHoldTheRemoteAndStillTakeTheWord() {
        press(Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT);
        assertEquals(0, keys.column());
        assertTrue("left at the left edge is used, not passed on", press(Word.LEFT));
        assertEquals(0, keys.column());

        press(Word.DOWN, Word.DOWN, Word.DOWN, Word.DOWN);
        assertEquals(5, keys.row());
        assertTrue("down on the last row is used, not passed on", press(Word.DOWN));
        assertEquals(5, keys.row());
    }

    /** Typing puts the letter the remote is on at the end of the text, and nothing else. */
    @Test
    public void okOnALetterTypesIt() {
        press(Word.OK);
        assertEquals("g", keys.text());
        press(Word.RIGHT, Word.OK);
        assertEquals("gh", keys.text());
        assertFalse("typing does not finish the keyboard", end.ended);
    }

    /**
     * Shift gives one capital and falls back, which is what typing a name wants.
     * {@code Keyboard.kt:146-147}: the letter clears {@code upper} and the next one is small again.
     */
    @Test
    public void shiftGivesOneCapitalAndFallsBack() {
        press(Word.DOWN, Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT);
        // Left stops on the shift key's own column and not on the column before it: a key two
        // columns wide is entered at its start, which is column 1 of the two it holds.
        assertEquals(1, keys.column());
        assertEquals(Keys.Press.SHIFT, keys.here("Cancel", "Done").press);
        press(Word.OK);
        assertTrue(keys.upper());
        press(Word.UP, Word.OK);
        assertEquals("S", keys.text());
        assertFalse("one capital, then small again", keys.upper());
        press(Word.OK);
        assertEquals("Ss", keys.text());
    }

    /**
     * The marks key and the accents key each swap their own page in and back out, and neither can
     * strand the viewer on the other's page: pressing marks while the accents are showing goes back
     * to the letters, which is {@code Keyboard.kt:149-150}'s identity comparison.
     */
    @Test
    public void thePagesSwapInAndBackOut() {
        press(Word.DOWN, Word.DOWN);
        assertEquals(4, keys.row());
        press(Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT);
        assertEquals(1, keys.column());
        assertEquals(Keys.Press.MARKS, keys.here("Cancel", "Done").press);
        press(Word.OK);
        assertEquals(Keys.Page.SYMBOLS, keys.page());
        press(Word.OK);
        assertEquals(Keys.Page.LETTERS, keys.page());

        press(Word.RIGHT, Word.OK);
        assertEquals(Keys.Page.ACCENTED, keys.page());
        press(Word.LEFT, Word.OK);
        assertEquals("the marks key leaves the accents page rather than staying on it",
                Keys.Page.SYMBOLS, keys.page());
    }

    /** Space types a space and erase takes the last letter back, including the space. */
    @Test
    public void spaceAndEraseWorkOnTheText() {
        press(Word.OK);
        press(Word.DOWN, Word.DOWN);
        press(Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT);
        press(Word.RIGHT, Word.RIGHT, Word.OK);
        assertEquals("g ", keys.text());
        press(Word.RIGHT, Word.OK);
        assertEquals("g", keys.text());
        press(Word.OK, Word.OK);
        assertEquals("", keys.text());
        assertTrue("erase on an empty text is still used", press(Word.OK));
        assertEquals("", keys.text());
    }

    /** Done answers with the text; cancel and back answer with nothing, which is not the same as "". */
    @Test
    public void doneAnswersTheTextAndCancelAnswersNothing() {
        press(Word.OK);
        press(Word.DOWN, Word.DOWN, Word.DOWN);
        assertEquals(5, keys.row());
        press(Word.RIGHT, Word.OK);
        assertTrue(end.ended);
        assertEquals("g", end.text);

        Ended second = new Ended();
        Keys other = new Keys("abc");
        other.press(Word.BACK, "Cancel", "Done", second);
        assertTrue(second.ended);
        assertNull("back cancels: an answer of nothing, not an answer of empty", second.text);

        Ended third = new Ended();
        Keys cancelled = new Keys("abc");
        cancelled.press(Word.DOWN, "Cancel", "Done", third);
        cancelled.press(Word.DOWN, "Cancel", "Done", third);
        cancelled.press(Word.DOWN, "Cancel", "Done", third);
        cancelled.press(Word.LEFT, "Cancel", "Done", third);
        cancelled.press(Word.LEFT, "Cancel", "Done", third);
        cancelled.press(Word.OK, "Cancel", "Done", third);
        assertTrue(third.ended);
        assertNull(third.text);
    }

    /**
     * A key spanning more than one column is stepped over whole, not one column at a time, so the
     * modifier row takes four presses to cross and not ten.
     */
    @Test
    public void aWideKeyIsSteppedOverWhole() {
        press(Word.DOWN, Word.DOWN);
        press(Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT);
        assertEquals(1, keys.column());
        press(Word.RIGHT);
        assertEquals(2, keys.column());
        press(Word.RIGHT);
        assertEquals(4, keys.column());
        press(Word.RIGHT);
        assertEquals(7, keys.column());
        assertTrue(press(Word.RIGHT));
        assertEquals("the last key of a row holds the remote", 7, keys.column());
    }

    /**
     * Two of the three pages hold a row that stops short of the last column, and the remote has to
     * land somewhere when it comes down the right hand edge onto one.
     *
     * The Compose form did not know this. {@code Symbols}' fourth row is seven keys from column 2
     * and so is {@code Accented}'s, both ending at column 8, while the three rows above them end at
     * 9. {@code Keyboard.kt} asked {@code caps.first} for the key under the remote and threw
     * {@code NoSuchElementException} instead -- eleven presses from where the keyboard opens, on
     * all four screens that draw one, which on a device is the sign in screen closing under a
     * viewer typing a password. Found by this port on 2026-11-13, reproduced against the Kotlin in
     * {@code src/test/kotlin/streams/tv/KeyboardReachTest.kt}, and fixed in both languages.
     *
     * The last key of the row is where a viewer coming down that edge expects to be.
     */
    @Test
    public void aRowThatStopsShortHoldsTheRemoteAtItsLastKey() {
        onSymbols();
        press(Word.UP, Word.UP, Word.UP, Word.UP);
        for (int step = 0; step < 9; step++) {
            press(Word.RIGHT);
        }
        assertEquals(0, keys.row());
        assertEquals(9, keys.column());
        press(Word.DOWN, Word.DOWN, Word.DOWN);
        assertEquals(3, keys.row());
        assertEquals("the row holds no key at column 9", "~", on());
        press(Word.OK);
        assertEquals("~", keys.text());
    }

    /** Which columns each row of each page covers, stated so a changed row of letters is seen. */
    @Test
    public void eachPagesFourthRowReachesTheColumnItsLettersReach() {
        Keys.Cap[][] letters = keys.caps("Cancel", "Done");
        assertEquals("the letters page reaches the last column", Keys.COLUMNS - 1, last(letters[3]));
        onSymbols();
        assertEquals("the symbols page stops one short", Keys.COLUMNS - 2,
                last(keys.caps("Cancel", "Done")[3]));
        for (int row = 0; row < 3; row++) {
            assertEquals("row " + row + " is a full row of keys", Keys.COLUMNS - 1,
                    last(keys.caps("Cancel", "Done")[row]));
        }
    }

    /** The last column any key of a row holds. */
    private static int last(Keys.Cap[] row) {
        Keys.Cap end = row[row.length - 1];
        return end.column + end.span - 1;
    }

    /** To the marks key and through it, which is the shortest way to the symbols page. */
    private void onSymbols() {
        press(Word.DOWN, Word.DOWN, Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT, Word.OK);
        assertEquals(Keys.Page.SYMBOLS, keys.page());
    }

    /**
     * The two buttons at the foot carry the words they were given and nothing written here. A
     * keyboard that wrote its own Cancel and Done would be English in a program file.
     */
    @Test
    public void theButtonsCarryTheWordsTheyAreGiven() {
        Keys.Cap[][] rows = keys.caps("Abbrechen", "Fertig");
        assertEquals(2, rows[5].length);
        assertEquals("Abbrechen", rows[5][0].word);
        assertEquals("Fertig", rows[5][1].word);
        assertEquals(Keys.Press.CANCEL, rows[5][0].press);
        assertEquals(Keys.Press.DONE, rows[5][1].press);
    }
}
