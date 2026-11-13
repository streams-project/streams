package streams.tv.reminder;

import android.content.Context;
import android.graphics.Color;
import android.widget.TextView;
import java.util.Map;
import streams.shared.Ink;
import streams.shared.Says;
import streams.shared.Type;
import streams.shared.Words;
import streams.tv.browse.Art;
import streams.tv.parts.Menu;
import streams.tv.views.Ref;
import streams.tv.views.RefLayout;
import streams.tv.views.RemoteHost;
import streams.tv.views.Round;
import streams.tv.views.Word;

/**
 * The reminder screen: a title that has not arrived, and one question about it.
 *
 * <p>The television's answer to the Remind Me button on the phone's title page
 * ({@code phone/title/Title.kt:312-349}). The phone had it and the television had none, so a
 * viewer could ask to be told about a coming title from their pocket and not from the room it
 * would be watched in.
 *
 * <h2>What the television does not do, and why</h2>
 * The phone also sets a local alarm ({@code phone/reminder/ReminderScheduler.kt}) so the device
 * raises its own notification on the day. This screen does not, and that is a decision rather than
 * a gap:
 *
 * <ul>
 * <li>The server already makes the notice. {@code server-api/src/routes/notifications.rs:56} reads
 * this profile's reminders into the feed and {@code NoticeKind.REMINDER_LIVE} is the kind it gives
 * them, so a reminder that comes true arrives on {@code tv/notifications} with no alarm anywhere.
 * That screen is the other half of this pass and did not exist either.</li>
 * <li>An {@code AlarmManager} alarm needs a {@code BroadcastReceiver} declared in a manifest. This
 * module has no {@code AndroidManifest.xml} at all, so writing one is a change to the television
 * module's shape and to the app's merged manifest -- neither of which belongs inside a screen.</li>
 * <li>And a television is unplugged, asleep or on another input most of the time, so the device
 * that would raise the alarm is the least likely of a viewer's devices to be awake for it.</li>
 * </ul>
 * What this costs is real: a viewer with only a television is told when they next open the app,
 * not on the day. The way to close it is a system notification from the app module, which is for
 * the lead to route.
 */
public class ReminderView extends RemoteHost implements Words.Bound {
    /**
     * The poster: <b>311 x 505</b>, measured.
     *
     * {@code reference/netflix-on-the-tv.md}, "A row screen, exact": a closed tile is 311 x 505,
     * aspect 0.616. The height is taken as 504 because {@code browse/Cards.Poster.HIGH} already
     * does -- the surface lays out at half the reference, and 505 halves to 252.5 while 504 lands
     * on a whole pixel. One pixel of the reference's own +/-1 px of video compression.
     */
    public static final int POSTER_WIDE = 311;

    /** And its height. Same measurement, rounded as {@code browse/Cards.Poster} rounds it. */
    public static final int POSTER_HIGH = 504;

    /**
     * The gap beside and below: <b>25 px</b>, measured.
     *
     * {@code reference/netflix-on-the-tv.md}, "Vertical rhythm of a row screen": "tiles 199-708,
     * gap 25, facts line 733-755".
     */
    public static final int GAP = 25;

    /**
     * Where the head ends and the page begins: {@code parts/PageHead.under()}, 178.
     *
     * This screen draws its own head rather than using {@code parts/PageHead}, because a page head
     * takes two <em>keys</em> and the words at the top of this one are a title's name, which is
     * data. The two places and the two sizes are that part's own -- {@code TITLE_AT 74},
     * {@code TITLE_TYPE 44}, {@code HELP_AT 148}, {@code HELP_TYPE 30} -- so the screen reads like
     * every other page on this television. {@code ReminderHeardTest} holds {@link #HEAD} against
     * {@code PageHead.under()}, where Android is real, so the two cannot drift.
     */
    public static final int HEAD = 178;

    /** The title's name, where a page head draws its own. */
    public static final int TITLE_AT = 74;

    /** And at the size a page head draws it. */
    public static final int TITLE_TYPE = 44;

    /** The line under it, where a page head draws its help line. */
    public static final int UNDER_AT = 148;

    /** And at that size. */
    public static final int UNDER_TYPE = 30;

    /** The top of the page proper. */
    public static final int FIRST = HEAD + GAP;

    /** What this screen asks of whoever owns the shell. */
    public interface Asked {
        /**
         * The viewer turned the reminder the other way. The host sends {@code CommandRemind} --
         * {@code shell.act(ActionCommand(CommandRemind(on, title)))}, which is what the phone sends
         * at {@code Title.kt:316}.
         *
         * It is a seam and not a call from here because {@code Shell.act} takes
         * {@code contract.Action}, the Kotlin arm of the generated types. Note 240 put the Java arm
         * in the same jar under different binary names; neither can be passed where the other is
         * wanted.
         */
        void turn(Coming title, boolean on);

        /** Left, Back, or OK on the second line. */
        void leave();
    }

    private final TextView name;
    private final TextView under;
    private final Art poster;
    private final Menu lines;
    private final Words words = new Words();

    private Ask ask = new Ask(Coming.NOTHING);
    private Asked asked;

    public ReminderView(Context context, Art.Source pictures) {
        super(context);
        setBackgroundColor(Ink.BACK);

        name = Type.line(context, TITLE_TYPE, Type.BOLD, Color.WHITE);
        addView(name, RefLayout.at(Round.SUB_EDGE, TITLE_AT,
                Ref.W - 2 * Round.SUB_EDGE, RefLayout.WRAP));

        under = Type.line(context, UNDER_TYPE, Type.BOOK, Ink.MUTED);
        addView(under, RefLayout.at(Round.SUB_EDGE, UNDER_AT,
                Ref.W - 2 * Round.SUB_EDGE, RefLayout.WRAP));

        poster = new Art(context, pictures, null);
        addView(poster, RefLayout.at(Round.SUB_EDGE, FIRST, POSTER_WIDE, POSTER_HIGH));

        lines = new Menu(context);
        addView(lines, RefLayout.at(Round.SUB_EDGE + POSTER_WIDE + GAP, FIRST,
                RefLayout.WRAP, RefLayout.WRAP));

        // The name and the line under it are the words the screen's own reading already carries,
        // and the poster is the title the name names. None of the three is a stop.
        Says.onto(name, Says.decoration());
        Says.onto(under, Says.decoration());
    }

    public void asks(Asked asked) {
        this.asked = asked;
    }

    /** The words this screen draws, re-read when the language changes. */
    public void speak(Map<String, String> said) {
        if (words.read(said)) {
            words.into(this);
        }
    }

    /**
     * True when this screen runs right to left: the arrows turn and every child mirrors.
     *
     * Not an override, for the reason {@code BrowseView.mirror} gives.
     */
    public void mirror(boolean rtl) {
        setRtl(rtl);
    }

    /**
     * A fresh answer. Where the viewer is, is not touched, which is {@link Ask#arrived(Coming)}:
     * the answer that comes back from setting a reminder is this screen with the reminder the other
     * way, and a focus reset by it would move under the viewer's hand.
     */
    public void show(Ask ask, Coming title) {
        this.ask = ask;
        ask.arrived(title);
        paint();
    }

    /** The screen's own focus, for the host to read. */
    public Ask ask() {
        return ask;
    }

    /**
     * One word from the remote. The order {@code BrowseView.press} settled: OK first, then what
     * moved, then what leaves.
     */
    @Override
    protected boolean press(Word word) {
        if (word == Word.OK) {
            Ask.Chose what = ask.choose();
            if (what == Ask.Chose.TURN && asked != null) {
                boolean on = ask.turningOn();
                // Drawn turned at once and told to the server after, which is what the phone does
                // at `Title.kt:314-316`: the words on the line must not wait on the network.
                asked.turn(ask.title(), on);
                ask.arrived(ask.title().turned(on));
                paint();
            } else if (what == Ask.Chose.CLOSE && asked != null) {
                asked.leave();
            }
            return true;
        }
        if (ask.press(word)) {
            paint();
            return true;
        }
        if (ask.leaves(word)) {
            if (asked != null) {
                asked.leave();
            }
            return true;
        }
        return false;
    }

    // `changed()` is not overridden here, which is the one difference from the other two Views
    // screens and is deliberate. `RemoteHost.changed()` invalidates, and nothing on this screen
    // moves or resizes when a key is pressed: the two lines swap their plate and their ink in
    // place. `BrowseView` and `NoticesView` override it because their parts really do move.

    /** Puts the state on the children. The one place that reads {@link Ask} and writes a view. */
    private void paint() {
        Coming title = ask.title();
        name.setText(title.name);
        under.setText(ask.comingWords(words));
        poster.show(title.poster, POSTER_WIDE, POSTER_HIGH);
        lines.show(new Menu.Line[] {
                new Menu.Line(words.of(ask.turnKey())),
                new Menu.Line(words.of(ask.closeKey())),
        });
        lines.mark(ask.at());
        // What a person hears as the remote moves. `RemoteHost.setSays` makes it a polite live
        // region, so it is said again each time it changes -- which is each time this runs.
        setSays(ask.says(words));
    }

    @Override
    public void words(Words said) {
        paint();
    }
}
