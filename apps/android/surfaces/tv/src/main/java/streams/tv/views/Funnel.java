package streams.tv.views;

import android.view.KeyEvent;

/**
 * One key in, one redraw out. The whole of this surface's navigation, with no toolkit in it.
 *
 * <h2>What this replaces</h2>
 * {@code Modifier.remote} at {@code tv/app/Remote.kt:71-95} is a focus requester, a focusable, a
 * semantics block and an {@code onPreviewKeyEvent} whose body is five decisions: is the screen
 * live, is this a key down, is it a word, does the page mirror, and did the screen use it. Those
 * five decisions are this class. {@link RemoteHost} is the view they are attached to and holds no
 * decision of its own.
 *
 * <h2>Why they are here and not in the view</h2>
 * Compose redrew from snapshot state. In Views every one of the 164 snapshot state sites across the
 * tracked Android Kotlin becomes a plain field plus an explicit redraw, and note 210 names a missed
 * redraw as the largest source of new defects in the whole programme: the build is green, the tests
 * pass, and one screen silently does not move. The mitigation it states is a single funnel -- the
 * screen's state is written in {@code press} and nowhere else, and {@code changed()} is called from
 * one line. This is that line.
 *
 * It is also why these five decisions are not inside {@code dispatchKeyEvent}: a {@link KeyEvent}
 * cannot be built in a unit test on this machine (no Robolectric and no {@code ui-test} in
 * {@code ~/.gradle}, and gradle runs offline), so a funnel that took an event would be a funnel
 * nobody could press. This one takes two integers.
 */
public final class Funnel {
    /** What a screen does with a word. True when it used it, exactly as {@code Remote.kt:47} has it. */
    public interface Press {
        boolean press(Word word);
    }

    /** What happens when something moved. One implementation per screen, usually {@code invalidate()}. */
    public interface Redraw {
        void changed();
    }

    private final Press press;
    private final Redraw redraw;

    /**
     * Under the opening, or under a panel, a screen keeps its place and takes no keys, and takes
     * them back after. {@code live = on && !LocalCovered.current} at {@code Remote.kt:76}: two
     * reasons in Compose, one field here, because a covered screen and a screen that gave the
     * remote to its own overlay are the same thing to the keys.
     */
    private boolean live = true;

    /** True when the page runs right to left, so Left and Right swap on the way in. */
    private boolean mirrored;

    public Funnel(Press press, Redraw redraw) {
        this.press = press;
        this.redraw = redraw;
    }

    public boolean isLive() {
        return live;
    }

    public void setLive(boolean live) {
        this.live = live;
    }

    public boolean isMirrored() {
        return mirrored;
    }

    public void setMirrored(boolean mirrored) {
        this.mirrored = mirrored;
    }

    /**
     * A key, as the view received it. True when the screen used it, and then the screen has been
     * told to redraw; false when it did not, and then the key has to go on to the system.
     *
     * The action is filtered to a key down to match {@code Remote.kt:94}. The repeat count is not
     * read and not filtered, which is also what the Compose path does today: a held direction
     * arrives as repeated key downs and moves the focus again, which is how a viewer scrolls a row
     * by holding the arrow.
     *
     * @param keyCode {@code KeyEvent.getKeyCode()}
     * @param action  {@code KeyEvent.getAction()}
     */
    public boolean key(int keyCode, int action) {
        if (!live || action != KeyEvent.ACTION_DOWN) {
            return false;
        }
        Word word = Word.of(keyCode);
        if (word == null) {
            return false;
        }
        if (mirrored) {
            word = word.turned();
        }
        if (!press.press(word)) {
            return false;
        }
        redraw.changed();
        return true;
    }
}
