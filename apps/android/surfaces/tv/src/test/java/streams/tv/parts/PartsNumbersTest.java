package streams.tv.parts;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import org.junit.Test;
import streams.tv.views.Round;

/**
 * Every measured number this package had to carry, read out of the Kotlin it came from and out of
 * the Java that replaced it, and held equal.
 *
 * <h2>Why both halves, which is the rule and not a habit</h2>
 * {@code src/test/kotlin/streams/tv/Sources.kt:12-22} states it, and it is the owner's: a number is
 * never asserted against itself, because {@code assertEquals(328, STEP)} passes forever and catches
 * nothing. Each number here is read from the Kotlin line it was measured onto and from the Java
 * constant that now carries it, and neither side is written into this file. Three failures follow
 * and all three are wanted: a number changed in the Kotlin alone, a number changed in the Java
 * alone, and a number deleted from either, which fails naming the pattern that stopped matching.
 *
 * <h2>And why the Kotlin half can be read at all</h2>
 * Because the Kotlin is still there, and in this package it has to be. {@code parts} is what the
 * other eight folders draw with -- {@code PageHead} in 6 files, {@code Face} in 8, {@code Pill} in
 * 5, {@code Facts} in 8, {@code Keyboard} in 4 -- and every one of those callers is still Compose.
 * Note 210's rule is that a package deletes its Kotlin in the same change that adds its Java; this
 * package cannot, and the report says so rather than this file quietly carrying the exception.
 * When the callers convert and the lead deletes the Kotlin, every assertion below loses its Kotlin
 * half deliberately, in that diff. That is what {@code Sources.kt} is for.
 *
 * <h2>What the reference can and cannot settle</h2>
 * {@code reference/netflix-on-the-tv.md} measures the browse screens: the tiles, the gutter, the
 * pitch, the hero, the safe margin. It holds no entry for a settings menu's pill, a keyboard's keys
 * or a tick grid's columns, so for those the Kotlin is the only record of the measurement and the
 * equality above is the whole of what can honestly be checked. What the reference does settle is
 * the <em>safe margin</em>, and {@link #everyPartStaysInsideTheSafeMargin()} holds all three of
 * this package's full width parts to it.
 */
public class PartsNumbersTest {
    private static final String MAIN = "apps/android/surfaces/tv/src/main/kotlin/streams/tv";

    private static final String REFERENCE = "reference/netflix-on-the-tv.md";

    private final String badge = Text.code(MAIN + "/parts/Badge.kt");
    private final String facts = Text.code(MAIN + "/parts/Facts.kt");
    private final String pill = Text.code(MAIN + "/parts/Pill.kt");
    private final String face = Text.code(MAIN + "/parts/Face.kt");
    private final String menu = Text.code(MAIN + "/parts/Menu.kt");
    private final String head = Text.code(MAIN + "/parts/PageHead.kt");
    private final String grid = Text.code(MAIN + "/parts/TickGrid.kt");
    private final String keys = Text.code(MAIN + "/parts/Keyboard.kt");

    /** A badge: the gap between two, the room inside one, its word and how black its plate is. */
    @Test
    public void aBadgeKeepsItsNumbers() {
        assertEquals(Text.number("spacedBy\\((\\d+)\\.ref\\)", badge, "the gap between badges"),
                Badges.GAP);
        assertEquals(Text.number("padding\\(horizontal = (\\d+)\\.ref", badge, "a badge across"),
                Badges.PAD_ACROSS);
        assertEquals(Text.number("vertical = (\\d+)\\.ref", badge, "a badge down"), Badges.PAD_DOWN);
        assertEquals(Text.number("fontSize = (\\d+)\\.refType", badge, "a badge's word"), Badges.TYPE);
        assertEquals(Text.hundredths("alpha = (0\\.\\d+)f", badge, "a badge's plate"), Badges.PLATE);
    }

    /** The line of facts: the dot between two, the room each side of it, and the size of a fact. */
    @Test
    public void theFactsLineKeepsItsNumbers() {
        assertEquals(Text.number("\\.size\\((\\d+)\\.ref\\)", facts, "the dot"), Facts.DOT);
        assertEquals(Text.number("padding\\(horizontal = (\\d+)\\.ref\\)", facts, "beside the dot"),
                Facts.DOT_SIDE);
        assertEquals(Text.number("fontSize = (\\d+)\\.refType", facts, "a fact"), Facts.TYPE);
    }

    /** The large button, including the 120 ms its colours take, which a Views port can lose silently. */
    @Test
    public void aPillKeepsItsNumbers() {
        assertEquals(Text.number("CHANGE_MS = (\\d+)", pill, "how long the colour takes"),
                Pill.CHANGE_MS);
        assertEquals(Text.number("\\.height\\((\\d+)\\.ref\\)", pill, "a pill's height"), Pill.HIGH);
        assertEquals(Text.number("padding\\(horizontal = (\\d+)\\.ref\\)", pill, "beside its word"),
                Pill.PAD);
        assertEquals(Text.number("\\.size\\((\\d+)\\.ref\\)", pill, "its icon"), Pill.ICON);
        assertEquals(Text.number("\\.width\\((\\d+)\\.ref\\)", pill, "after its icon"), Pill.GAP);
        assertEquals(Text.number("fontSize = (\\d+)\\.refType", pill, "its word"), Pill.TYPE);
        assertEquals(Text.hundredths("alpha = (0\\.\\d+)f", pill, "its plate at rest"), Pill.QUIET);
    }

    /** A face's letter is a share of its size, which is the one number that makes every face. */
    @Test
    public void aFaceKeepsItsOneNumber() {
        assertEquals(Text.number("size \\* (\\d+) /", face, "the letter's share"), Face.LETTER_SHARE);
        assertEquals(Text.number("size \\* \\d+ / (\\d+)", face, "what it is a share of"),
                Face.LETTER_SHARE_OF);
        assertTrue("a letter is smaller than the face it sits in",
                Face.LETTER_SHARE < Face.LETTER_SHARE_OF);
    }

    /** A menu: its two word columns, its width, its lines and the two marks they can wear. */
    @Test
    public void aMenuKeepsItsNumbers() {
        assertEquals(Text.number("MENU_WORDS = (\\d+)", menu, "where words start with no icons"),
                Menu.WORDS);
        assertEquals(Text.number("\\}\\) (\\d+) else MENU_WORDS", menu, "and with icons"),
                Menu.WORDS_PAST_ICONS);
        assertEquals(Text.number("width: Int = (\\d+)", menu, "a menu's width"), Menu.WIDE);
        assertEquals(Text.number("spacedBy\\((\\d+)\\.ref\\)", menu, "between two lines"), Menu.GAP);
        assertEquals(Text.number("\\.height\\((\\d+)\\.ref\\)", menu, "a line's height"), Menu.HIGH);
        assertEquals(Text.number("\\.offset\\((\\d+)\\.ref\\)\\.size\\(\\d+\\.ref\\)", menu,
                "where a line's icon sits"), Menu.ICON_AT);
        assertEquals(Text.number("\\.offset\\(\\d+\\.ref\\)\\.size\\((\\d+)\\.ref\\)", menu,
                "how large it is"), Menu.ICON);
        assertEquals(Text.number("fontSize = (\\d+)\\.refType", menu, "a line's word"), Menu.TYPE);
        assertEquals(Text.hundredths("alpha = (0\\.\\d+)f", menu, "a kept line's plate"), Menu.KEPT);
        assertTrue("words with icons start past the icon they follow",
                Menu.WORDS_PAST_ICONS >= Menu.ICON_AT + Menu.ICON);
    }

    /** The head of a settings style page: two lines on the page's own column. */
    @Test
    public void aPageHeadKeepsItsNumbers() {
        assertEquals(Text.number("Text\\(title, Modifier\\.offset\\(SUB_EDGE\\.ref, (\\d+)\\.ref\\)",
                head, "the title's line"), PageHead.TITLE_AT);
        assertEquals(Text.number("Text\\(title,.*fontSize = (\\d+)\\.refType", head, "its size"),
                PageHead.TITLE_TYPE);
        assertEquals(Text.number("Text\\(help, Modifier\\.offset\\(SUB_EDGE\\.ref, (\\d+)\\.ref\\)",
                head, "the help line"), PageHead.HELP_AT);
        assertEquals(Text.number("Text\\(help,.*fontSize = (\\d+)\\.refType", head, "its size"),
                PageHead.HELP_TYPE);
        assertTrue("the help sits under the title and not on it", PageHead.HELP_AT > PageHead.TITLE_AT);
    }

    /** The tick grid: four columns, eight lines each, and a plate that reaches around its word. */
    @Test
    public void theTickGridKeepsItsNumbers() {
        assertEquals(Text.number("GRID_LINES = (\\d+)", grid, "lines in a column"), Ticks.LINES);
        assertEquals(Text.number("ColumnX = listOf\\((\\d+),", grid, "the first column"), Ticks.COLUMNS[0]);
        assertEquals(Text.number("ColumnX = listOf\\(\\d+, (\\d+),", grid, "the second"), Ticks.COLUMNS[1]);
        assertEquals(Text.number("ColumnX = listOf\\(\\d+, \\d+, (\\d+),", grid, "the third"),
                Ticks.COLUMNS[2]);
        assertEquals(Text.number("ColumnX = listOf\\(\\d+, \\d+, \\d+, (\\d+)\\)", grid, "the fourth"),
                Ticks.COLUMNS[3]);
        assertEquals(Text.number("FIRST_MIDDLE = (\\d+)", grid, "the first line's middle"),
                Ticks.FIRST_MIDDLE);
        assertEquals(Text.number("\\bLINE = (\\d+)", grid, "the drop to the next line"), Ticks.LINE);
        assertEquals(Text.number("\\.width\\((\\d+)\\.ref\\)", grid, "a line's plate"), Ticks.WIDE);
        assertEquals(Text.number("\\.height\\((\\d+)\\.ref\\)", grid, "its height"), Ticks.HIGH);
        assertEquals(Text.number("\\.offset\\((\\d+)\\.ref\\)\\.size\\(\\d+\\.ref\\)", grid,
                "where the tick sits"), Ticks.TICK_AT);
        assertEquals(Text.number("\\.offset\\(\\d+\\.ref\\)\\.size\\((\\d+)\\.ref\\)", grid,
                "how large it is"), Ticks.TICK);
        assertEquals(Text.number("fontSize = (\\d+)\\.refType", grid, "a word"), Ticks.TYPE);
    }

    /**
     * The plate reaches back from the word by the same number the word is drawn forward by, so the
     * word lands exactly on its column. {@code TickGrid.kt:40} and {@code :54} write it twice and
     * this holds the two together; {@link Ticks#BACK} writes it once.
     */
    @Test
    public void aTickedWordLandsOnItsColumn() {
        assertEquals(Text.number("\\.offset\\(\\(x - (\\d+)\\)\\.ref", grid, "how far back the plate reaches"),
                Ticks.BACK);
        assertEquals(Text.number("\\.offset\\((\\d+)\\.ref\\), color = ink", grid, "and the word forward"),
                Ticks.BACK);
        for (int at = 0; at < Ticks.LINES * Ticks.COLUMNS.length; at++) {
            assertEquals("line " + at + " does not land on its column",
                    Ticks.columnOf(at), Ticks.columnOf(at) - Ticks.BACK + Ticks.BACK);
            assertEquals("line " + at + " is not centred on its line",
                    Ticks.FIRST_MIDDLE + (at % Ticks.LINES) * Ticks.LINE,
                    Ticks.topOf(at) + Ticks.HIGH / 2);
        }
        assertEquals(Text.number("\\(middle - (\\d+)\\)\\.ref", grid, "half a plate"), Ticks.HIGH / 2);
    }

    /** The keyboard's grid, its panel and the two buttons at its foot. */
    @Test
    public void theKeyboardKeepsItsNumbers() {
        assertEquals(Text.number("KEY = (\\d+)", keys, "a key"), Keys.KEY);
        assertEquals(Text.number("STEP = (\\d+)", keys, "the step to the next"), Keys.STEP);
        assertEquals(Text.number("COLUMNS = (\\d+)", keys, "columns"), Keys.COLUMNS);
        assertEquals(Text.number("BUTTONS = (\\d+)", keys, "the buttons' span"), Keys.BUTTONS);
        assertEquals(Text.number("var row by mutableIntStateOf\\((\\d+)\\)", keys, "the starting row"),
                Keys.START_ROW);
        assertEquals(Text.number("var column by mutableIntStateOf\\((\\d+)\\)", keys, "and column"),
                Keys.START_COLUMN);
        assertEquals(Text.number("\\.size\\((\\d+)\\.ref, \\d+\\.ref\\)", keys, "the panel's width"),
                KeysView.WIDE);
        assertEquals(Text.number("\\.size\\(\\d+\\.ref, (\\d+)\\.ref\\)", keys, "its height"),
                KeysView.HIGH);
        assertEquals(Text.number("Box\\(Modifier\\.offset\\((\\d+)\\.ref", keys, "where the keys start across"),
                KeysView.INSET_X);
        assertEquals(Text.number("Box\\(Modifier\\.offset\\(\\d+\\.ref, (\\d+)\\.ref\\)", keys, "and down"),
                KeysView.INSET_Y);
        assertEquals(Text.number("if \\(pill\\) (\\d+) else row \\* STEP", keys, "the buttons' line"),
                KeysView.BUTTON_TOP);
        assertEquals(Text.number("if \\(pill\\) (\\d+) else KEY", keys, "their height"),
                KeysView.BUTTON_HIGH);
        assertEquals(Text.number("Cancel\\) 0 else (\\d+)\\)", keys, "where the right button starts"),
                KeysView.BUTTON_RIGHT_AT);
        assertEquals(Text.number("if \\(pill\\) (\\d+) else cap\\.span", keys, "their width"),
                KeysView.BUTTON_WIDE);
        assertEquals(Text.number("fontSize = (\\d+)\\.refType", keys, "a key's word"), KeysView.TYPE);
        assertEquals(Text.number("Modifier\\.size\\((\\d+)\\.ref\\), tint = ink", keys, "a key's mark"),
                KeysView.MARK);
        assertEquals(Text.hundredths(
                "background\\(Color\\.White\\.copy\\(alpha = (0\\.\\d+)f\\), shapeOf\\(Round\\.panel\\)\\)",
                keys, "the panel"), KeysView.PANEL);
        assertEquals(Text.number("\\.border\\((\\d+)\\.ref", keys, "its edge"), KeysView.EDGE_WIDE);
        assertEquals(Text.hundredths("border\\(\\d+\\.ref, Color\\.White\\.copy\\(alpha = (0\\.\\d+)f\\)",
                keys, "how white its edge is"), KeysView.EDGE);
        assertEquals(Text.hundredths(
                "else Color\\.White\\.copy\\(alpha = (0\\.\\d+)f\\), shape\\)", keys, "a key at rest"),
                KeysView.QUIET);
    }

    /**
     * The keyboard's four derivations, which are what make its numbers one measurement rather than
     * sixteen. A gap changed on its own, or a key resized, fails here before it draws off the panel.
     */
    @Test
    public void theKeyboardsNumbersDeriveFromEachOther() {
        Keys.Cap one = new Keys("").caps("", "")[0][0];
        assertEquals("a key one column wide is a key", Keys.KEY, KeysView.wideOf(one));
        Keys.Cap wide = new Keys("").caps("", "")[4][2];
        assertEquals("a key three columns wide is three steps less the gap it swallows",
                3 * Keys.STEP - (Keys.STEP - Keys.KEY), KeysView.wideOf(wide));
        assertEquals("the buttons sit one inner margin below the last row of keys",
                KeysView.INSET_Y, KeysView.BUTTON_TOP - (4 * Keys.STEP + Keys.KEY));
        assertTrue("the panel holds its widest row and its lowest", KeysView.fits());
        assertTrue("the panel holds a full row of keys",
                KeysView.INSET_X + (Keys.COLUMNS * Keys.STEP - (Keys.STEP - Keys.KEY))
                        + KeysView.INSET_X <= KeysView.WIDE);
    }

    /** The three pages of letters, character for character, in both languages. */
    @Test
    public void theThreePagesOfLettersCameAcross() {
        assertEquals(Text.theOne("Letters = listOf\\((.*)\\)", keys, "the letters").group(1),
                written(Keys.lettersOf(Keys.Page.LETTERS)));
        assertEquals(Text.theOne("Symbols = listOf\\((.*)\\)", keys, "the symbols").group(1),
                written(Keys.lettersOf(Keys.Page.SYMBOLS)));
        assertEquals(Text.theOne("Accented = listOf\\((.*)\\)", keys, "the accents").group(1),
                written(Keys.lettersOf(Keys.Page.ACCENTED)));
    }

    /**
     * The three parts that span the page stay inside the safe margin
     * {@code reference/netflix-on-the-tv.md:23} measured, which is the one thing in this package the
     * reference can settle. A television overscans: a word outside this is a word a viewer may
     * never see, and nothing in a unit test on a desk would otherwise notice.
     */
    @Test
    public void everyPartStaysInsideTheSafeMargin() {
        String reference = Text.text(REFERENCE);
        int left = Text.number("\\*\\*(\\d+) px left", reference, "the safe margin on the left");
        int right = Text.number("px left, (\\d+) px right", reference, "and on the right");
        int edge = streams.tv.views.Ref.W - right;

        assertTrue("a page head's column is inside the safe margin", Round.SUB_EDGE >= left);

        int last = Ticks.COLUMNS[Ticks.COLUMNS.length - 1];
        assertTrue("the tick grid's first column is inside the safe margin",
                Ticks.columnOf(0) - Ticks.BACK >= left);
        assertTrue("and its last plate ends inside it", last - Ticks.BACK + Ticks.WIDE <= edge);

        assertTrue("the keyboard, placed on the page's column, ends inside the safe margin",
                Round.SUB_EDGE + KeysView.WIDE <= edge);
    }

    /** Three strings as Kotlin writes them in a {@code listOf}, for comparing the two sources. */
    private static String written(String[] rows) {
        StringBuilder said = new StringBuilder();
        for (String row : rows) {
            if (said.length() > 0) {
                said.append(", ");
            }
            said.append('"').append(row.replace("\\", "\\\\").replace("\"", "\\\"")).append('"');
        }
        return said.toString();
    }
}
