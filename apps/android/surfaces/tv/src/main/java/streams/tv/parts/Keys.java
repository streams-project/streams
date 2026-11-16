package streams.tv.parts;

import java.util.Locale;
import streams.tv.views.Word;

/**
 * A letter grid for a remote: where the viewer is on the keys, the text so far, and what each key
 * does. The Views twin of {@code KeyboardState} and {@code KeyboardState.press} at
 * {@code parts/Keyboard.kt:68-76} and {@code :129-154}. Four screens share it -- sign in, add
 * profile, edit profile, the lock steps.
 *
 * <h2>It draws nothing, and that is the point of separating it out</h2>
 * In Compose the grid and the key map were one file, the caps were a private list and
 * {@code press} was an extension on a {@code @Stable} class holding five snapshot delegates. None
 * of it could be reached by a test. Here the state machine answers three questions about itself --
 * where the remote is, what the text has become, and whether a word was used -- and touches no
 * toolkit, context or resource. {@code KeysTest} presses scripted sequences against it.
 *
 * That separation is note 210's named mitigation for the risk it calls the largest in the whole
 * programme: 164 snapshot state sites become plain fields plus an explicit redraw, and a missed
 * redraw is invisible. Fields that nothing can observe are the dangerous half, so this class has no
 * drawing to forget -- {@link KeysView} reads it whole after every press.
 *
 * <h2>What it answers, and the one key it does not take</h2>
 * {@code press} answers false only for UP on the top row, which is how a page above the keyboard
 * gets the remote back ({@code Keyboard.kt:136}). Every other word is used, including DOWN on the
 * last row and LEFT at the left edge, which move nothing and stay: a keyboard that let LEFT escape
 * sideways would lose the viewer's place mid word.
 */
public final class Keys {
    /** A key's size. {@code Keyboard.kt:25}. */
    static final int KEY = 63;

    /** And the distance to the next one, which is the key plus the gap. {@code Keyboard.kt:26}. */
    static final int STEP = 66;

    /** How many columns a row of keys holds. {@code Keyboard.kt:27}. */
    static final int COLUMNS = 10;

    /** How many columns each of the two buttons at the foot holds. {@code Keyboard.kt:28}. */
    static final int BUTTONS = 5;

    /** Where the remote starts: the middle of the home row. {@code Keyboard.kt:74-75}. */
    static final int START_ROW = 2;

    static final int START_COLUMN = 4;

    /** What a key does. {@code Keyboard.kt:31-40}. */
    public enum Press {
        LETTER,
        SHIFT,
        MARKS,
        ACCENTS,
        SPACE,
        ERASE,
        CANCEL,
        DONE
    }

    /** The mark a key wears instead of a word. The view decides what it is drawn with. */
    public enum Mark {
        NONE,
        SHIFT,
        SPACE,
        ERASE
    }

    /** Which set of letters is showing. {@code Keyboard.kt:42-44}. */
    public enum Page {
        LETTERS,
        SYMBOLS,
        ACCENTED
    }

    /** One key. {@code Cap} at {@code Keyboard.kt:40}. */
    public static final class Cap {
        public final Press press;
        public final int column;
        public final int span;
        public final String word;
        public final Mark mark;

        Cap(Press press, int column, int span, String word, Mark mark) {
            this.press = press;
            this.column = column;
            this.span = span;
            this.word = word;
            this.mark = mark;
        }

        /** True when the remote at this column is on this key. */
        public boolean holds(int column) {
            return column >= this.column && column < this.column + span;
        }
    }

    /** What a finished keyboard answers with: the text on Done, or null on Cancel. */
    public interface Finish {
        void finished(String text);
    }

    private static final String[] LETTERS = {"1234567890", "qwertyuiop", "asdfghjkl-", "zxcvbnm'"};
    private static final String[] SYMBOLS = {"1234567890", "!@#$%^&*()", "_=+[]{};:\"", ",.?/\\|~"};
    private static final String[] ACCENTED = {"àáâäãåæçèé", "êëìíîïñòóô", "õöøùúûüýßÿ", "œšžćčđğ"};

    private String text;
    private Page page = Page.LETTERS;
    private boolean upper;
    private int row = START_ROW;
    private int column = START_COLUMN;

    public Keys(String start) {
        this.text = start == null ? "" : start;
    }

    public String text() {
        return text;
    }

    public Page page() {
        return page;
    }

    public boolean upper() {
        return upper;
    }

    public int row() {
        return row;
    }

    public int column() {
        return column;
    }

    /** The letters of a page, which the view needs for nothing and a test reads to say which page. */
    static String[] lettersOf(Page page) {
        switch (page) {
            case SYMBOLS:
                return SYMBOLS;
            case ACCENTED:
                return ACCENTED;
            default:
                return LETTERS;
        }
    }

    /**
     * Every key as it stands now. Six rows: four of letters, the modifier row, and the two buttons.
     * {@code caps} at {@code Keyboard.kt:53-66}.
     *
     * @param cancel the word on the left button, already in the viewer's language
     * @param done   and on the right
     */
    public Cap[][] caps(String cancel, String done) {
        String[] letters = lettersOf(page);
        Cap[][] rows = new Cap[6][];
        rows[0] = line(letters[0], 0);
        rows[1] = line(letters[1], 0);
        rows[2] = line(letters[2], 0);
        Cap[] fourth = line(letters[3], 2);
        rows[3] = new Cap[fourth.length + 1];
        rows[3][0] = new Cap(Press.SHIFT, 0, 2, null, Mark.SHIFT);
        System.arraycopy(fourth, 0, rows[3], 1, fourth.length);
        rows[4] = new Cap[] {
            new Cap(Press.MARKS, 0, 2, "!#$", Mark.NONE),
            new Cap(Press.ACCENTS, 2, 2, "àáâ", Mark.NONE),
            new Cap(Press.SPACE, 4, 3, null, Mark.SPACE),
            new Cap(Press.ERASE, 7, 3, null, Mark.ERASE),
        };
        rows[5] = new Cap[] {
            new Cap(Press.CANCEL, 0, BUTTONS, cancel == null ? "" : cancel, Mark.NONE),
            new Cap(Press.DONE, BUTTONS, BUTTONS, done == null ? "" : done, Mark.NONE),
        };
        return rows;
    }

    /** One row of letters, each a key of its own. {@code Keyboard.kt:46-50}. */
    private Cap[] line(String letters, int from) {
        // Counted in code points and not in chars: the accented page is Latin-1 today and every
        // letter is one char, but a page of letters is the one place a surface would meet a pair,
        // and splitting one would draw half a letter and type half a letter.
        int[] points = letters.codePoints().toArray();
        Cap[] caps = new Cap[points.length];
        for (int at = 0; at < points.length; at++) {
            String letter = new String(points, at, 1);
            // `Locale.ROOT`, not the device's: a Turkish television would otherwise turn i into a
            // dotless capital, and the viewer would type a letter that is not on the key.
            String said = upper ? letter.toUpperCase(Locale.ROOT) : letter;
            caps[at] = new Cap(Press.LETTER, from + at, 1, said, Mark.NONE);
        }
        return caps;
    }

    /**
     * Which key of a row the remote is on, as a place in that row's own list, or -1 for a row it is
     * not on. {@link KeysView} and {@link #press} ask this one question, so the key that lights is
     * the key that answers.
     */
    public int lit(int atRow) {
        return lit(atRow, caps("", ""));
    }

    private int lit(int atRow, Cap[][] rows) {
        if (atRow != row || atRow < 0 || atRow >= rows.length) {
            return -1;
        }
        Cap[] line = rows[atRow];
        for (int at = 0; at < line.length; at++) {
            if (line[at].holds(column)) {
                return at;
            }
        }
        // Not every row covers every column, and the Compose form did not know it: `Symbols` is
        // `,.?/\\|~` and `Accented` is a seven letter row, both from column 2, so both end at
        // column 8 while the three rows above them end at 9. A remote that came down the right hand
        // edge stands on a column with no key. The last key of the row is where a viewer coming
        // down that edge expects to be, so it holds there. `parts/Keyboard.kt:132-138` carries the
        // same fix and the same reason: until 2026-11-13 it asked `first` and threw
        // NoSuchElementException on the eleventh press from where the keyboard opens.
        return line.length - 1;
    }

    /** The key the remote is on now. */
    public Cap here(String cancel, String done) {
        return here(caps(cancel, done));
    }

    private Cap here(Cap[][] rows) {
        return rows[row][lit(row, rows)];
    }

    /**
     * Answers a word from the remote. {@code Keyboard.kt:129-154}.
     *
     * @return false only when the word was not used, which is UP on the top row
     */
    public boolean press(Word word, String cancel, String done, Finish finish) {
        Cap[][] rows = caps(cancel, done);
        Cap here = here(rows);
        switch (word) {
            case UP:
                if (row > 0) {
                    row -= 1;
                } else {
                    return false;
                }
                break;
            case DOWN:
                if (row < rows.length - 1) {
                    row += 1;
                }
                break;
            case LEFT:
                if (here.column > 0) {
                    column = here.column - 1;
                }
                break;
            case RIGHT:
                if (here.column + here.span < COLUMNS) {
                    column = here.column + here.span;
                }
                break;
            case BACK:
                finish.finished(null);
                break;
            case OK:
                chose(here, finish);
                break;
            default:
                break;
        }
        return true;
    }

    private void chose(Cap here, Finish finish) {
        switch (here.press) {
            case LETTER:
                text += here.word;
                // One letter and the shift falls back, which is what a viewer typing a name
                // expects and what `Keyboard.kt:146` does.
                upper = false;
                break;
            case SHIFT:
                upper = !upper;
                break;
            case MARKS:
                page = page == Page.SYMBOLS ? Page.LETTERS : Page.SYMBOLS;
                break;
            case ACCENTS:
                page = page == Page.ACCENTED ? Page.LETTERS : Page.ACCENTED;
                break;
            case SPACE:
                text += " ";
                break;
            case ERASE:
                if (!text.isEmpty()) {
                    // By code point, for the same reason the letters are counted that way: on the
                    // accented page a pair would be half erased and leave an unpaired surrogate.
                    text = text.substring(0, text.offsetByCodePoints(text.length(), -1));
                }
                break;
            case CANCEL:
                finish.finished(null);
                break;
            case DONE:
                finish.finished(text);
                break;
            default:
                break;
        }
    }
}
