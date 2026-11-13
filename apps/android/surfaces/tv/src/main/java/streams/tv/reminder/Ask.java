package streams.tv.reminder;

import streams.shared.Says;
import streams.shared.ThemeKt;
import streams.shared.Words;
import streams.tv.views.Word;

/**
 * Where the viewer is on the reminder screen, and what the remote does from there. It knows nothing
 * about drawing.
 *
 * <p>Two stops and a question with two answers, which is the smallest state machine on this
 * surface and still gets one: {@code AskTest} presses every word at every stop on this machine's
 * JVM, because nothing here touches a toolkit, a shell or a context.
 *
 * <h2>The key map, designed for this screen and not inherited</h2>
 * <table>
 * <tr><th>word</th><th>what it does</th><th>at the edge</th></tr>
 * <tr><td>Up</td><td>the line above</td><td>on the first line, nothing moves</td></tr>
 * <tr><td>Down</td><td>the line below</td><td>on the second line, nothing moves</td></tr>
 * <tr><td>Left</td><td>leaves the screen</td><td>the lines are the left edge of this page</td></tr>
 * <tr><td>Right</td><td>nothing. There is no second column</td><td>--</td></tr>
 * <tr><td>OK</td><td>on the first line turns the reminder the other way; on the second leaves</td>
 *     <td>--</td></tr>
 * <tr><td>Back</td><td>leaves the screen</td><td>--</td></tr>
 * </table>
 *
 * <p>That is {@code settings/Settings.kt:244-250} exactly -- the confirm panel's
 * {@code Word.UP -> if (confirmAt > 0)}, {@code Word.DOWN -> if (confirmAt < 1)},
 * {@code Word.OK -> if (confirmAt == 0) ... else dismiss},
 * {@code Word.BACK, Word.LEFT -> dismiss} -- and it is the same key map because it is the same
 * question. A viewer who has dismissed one two line panel on this television knows how to dismiss
 * this one.
 *
 * <h2>Which line the remote opens on</h2>
 * The second: the one that leaves. {@code Settings.kt:234} opens its confirm on
 * {@code confirmAt = 1} for a reason that holds here -- a panel that opens on the line that
 * <em>does</em> the thing turns one stray OK into an action the viewer never asked for. Setting a
 * reminder is small and undoable, so this is not a safety rule; it is the surface being consistent
 * about where a two line panel opens.
 */
public final class Ask {
    /** The line that turns the reminder on or off. */
    public static final int TURN = 0;

    /** The line that leaves. */
    public static final int CLOSE = 1;

    /** The two lines. */
    public static final int LINES = 2;

    /** What OK chooses from here. */
    public enum Chose {
        /** Nothing: a title this screen knows nothing about. */
        NOTHING,
        /** Turn the reminder the other way. */
        TURN,
        /** Leave. */
        CLOSE
    }

    private Coming title;
    private int at = CLOSE;

    public Ask(Coming title) {
        this.title = title == null ? Coming.NOTHING : title;
    }

    /**
     * A fresh answer for the same title. Where the viewer is, is not touched: the answer that comes
     * back from setting a reminder is this same screen with {@code reminder_set} the other way, and
     * a focus reset by it would move under the viewer's hand the moment they pressed OK.
     */
    public void arrived(Coming title) {
        this.title = title == null ? Coming.NOTHING : title;
    }

    public Coming title() {
        return title;
    }

    /** Which line the remote is on. */
    public int at() {
        return at;
    }

    /**
     * One word from the remote. True when something moved, which is the one thing that redraws.
     *
     * False for Left, Back and OK, as {@code Spot.press} and {@code Place.press} answer false for
     * the words that leave or choose: neither moves anything on this page.
     */
    public boolean press(Word word) {
        switch (word) {
            case UP:
                if (at > TURN) {
                    at -= 1;
                    return true;
                }
                return false;
            case DOWN:
                if (at < CLOSE) {
                    at += 1;
                    return true;
                }
                return false;
            case LEFT:
            case RIGHT:
            case OK:
            case BACK:
            default:
                return false;
        }
    }

    /** Whether this word leaves the screen. Left and Back, and nothing else. */
    public boolean leaves(Word word) {
        return word == Word.LEFT || word == Word.BACK;
    }

    /** What OK chooses from here. */
    public Chose choose() {
        if (at == CLOSE) {
            return Chose.CLOSE;
        }
        return title.known() ? Chose.TURN : Chose.NOTHING;
    }

    /** Which way the reminder would go if OK were pressed on the first line now. */
    public boolean turningOn() {
        return !title.set;
    }

    /**
     * The words on the first line: what pressing OK there would leave behind.
     *
     * {@code remind.me} when there is no reminder and {@code remind.set} when there is, which is
     * the phone's own pair at {@code Title.kt:342-343}. Both are read through {@link Words#of}, so
     * a run that has not read its language yet draws nothing rather than English.
     */
    public String turnKey() {
        return title.set ? "remind.set" : "remind.me";
    }

    /** The words on the second line. */
    public String closeKey() {
        return "common.cancel";
    }

    /**
     * When the title arrives, in the viewer's language: {@code gate.coming} with its {@code {date}}
     * filled, which is the one line the phone draws above its button ({@code Title.kt:302-303}).
     *
     * Filled through {@code shared/Theme.kt:66}'s {@code fill}, which wraps the day in the
     * bidirectional isolates, and not by {@code String.replace} here: a date dropped into an Arabic
     * sentence without them comes out in the wrong order on screen. {@code Words.counted} calls the
     * same function for the same reason, and that is this surface's one statement of the rule.
     *
     * Empty words stay empty: an unfilled {@code gate.coming} is a line this run has no language
     * for, and filling a template nobody will read would put a bare date on the screen.
     */
    public String comingWords(Words words) {
        String said = words.of("gate.coming");
        if (said.isEmpty() || title.day.isEmpty()) {
            return "";
        }
        return ThemeKt.fill(said, "date", title.day);
    }

    /**
     * What a person hears as the remote moves.
     *
     * Three things in the order a person wants them: what this screen is about (the title's name
     * and when it arrives), then the line the remote is standing on. They are joined by {@link
     * Says#sayBoth}, so the break is a line end and no language's word order is decided here.
     *
     * <p>The title is repeated at both stops on purpose. This screen is two lines on an otherwise
     * empty page, and a reader that moved from "Remind Me" to "Cancel" and heard only "Cancel"
     * would have lost what was being cancelled.
     */
    public String says(Words words) {
        if (!title.known()) {
            return "";
        }
        String what = Says.sayBoth(title.name, comingWords(words));
        return Says.sayBoth(what, words.of(at == CLOSE ? closeKey() : turnKey()));
    }
}
