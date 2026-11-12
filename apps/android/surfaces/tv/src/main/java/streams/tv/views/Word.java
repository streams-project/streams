package streams.tv.views;

import android.view.KeyEvent;

/**
 * What a remote can say. The Java twin of {@code enum class Word} at
 * {@code tv/app/Remote.kt:26}, with {@code wordOf} ({@code :28-36}) and {@code turn}
 * ({@code :39-43}) on it.
 *
 * Six words and no more. Every television screen in this surface answers these and nothing else,
 * which is what lets the whole surface have one focusable node and paint its own focus: there is
 * nothing for the toolkit's focus search to steer, because a screen is handed a word and decides
 * for itself what moved.
 */
public enum Word {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    OK,
    BACK;

    /**
     * The word a key says, or null for a key this surface does not answer.
     *
     * Null and not a seventh word: an unanswered key has to fall through to the system, so that
     * volume, home and the assistant still work. {@code Remote.kt:94} returns false for it and this
     * returns null for the same reason.
     *
     * Two keys are here that the Compose path did not have: {@code KEYCODE_BUTTON_A} and
     * {@code KEYCODE_BUTTON_B}. A game controller is a direction and select device on every
     * television box, and note 179's correction makes it relevant -- {@code apps/xbox} takes this
     * surface, where A and B are the only select and back the pad has. They cost two lines here and
     * are unreachable from a remote, so they change nothing for a viewer who has none.
     */
    public static Word of(int keyCode) {
        switch (keyCode) {
            case KeyEvent.KEYCODE_DPAD_UP:
                return UP;
            case KeyEvent.KEYCODE_DPAD_DOWN:
                return DOWN;
            case KeyEvent.KEYCODE_DPAD_LEFT:
                return LEFT;
            case KeyEvent.KEYCODE_DPAD_RIGHT:
                return RIGHT;
            case KeyEvent.KEYCODE_DPAD_CENTER:
            case KeyEvent.KEYCODE_ENTER:
            case KeyEvent.KEYCODE_NUMPAD_ENTER:
            case KeyEvent.KEYCODE_BUTTON_A:
                return OK;
            case KeyEvent.KEYCODE_BACK:
            case KeyEvent.KEYCODE_BUTTON_B:
                return BACK;
            default:
                return null;
        }
    }

    /**
     * The same word on a page that runs right to left.
     *
     * Pages say Left for towards the start of a line. In a right to left language the start is on
     * the right, so the arrows swap here, once, for every page -- {@code Remote.kt:77-80} and
     * {@code :93}. The player keeps its own left to right way, so its screen does not turn its
     * words.
     */
    public Word turned() {
        switch (this) {
            case LEFT:
                return RIGHT;
            case RIGHT:
                return LEFT;
            default:
                return this;
        }
    }
}
