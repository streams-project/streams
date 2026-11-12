package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;

import android.view.KeyEvent;
import java.util.List;
import java.util.TreeSet;
import org.junit.Test;

/**
 * What a key says, and what a right to left page does to it.
 *
 * These run without a device although they name {@link KeyEvent}, because a key code is a
 * {@code static final int} and javac writes the number into the class file. Nothing here touches an
 * instance of anything the platform owns.
 */
public class WordTest {
    /** The map at {@code tv/app/Remote.kt:28-36}, key for key. */
    @Test
    public void every_key_a_remote_sends_says_the_word_the_compose_path_said() {
        assertEquals(Word.UP, Word.of(KeyEvent.KEYCODE_DPAD_UP));
        assertEquals(Word.DOWN, Word.of(KeyEvent.KEYCODE_DPAD_DOWN));
        assertEquals(Word.LEFT, Word.of(KeyEvent.KEYCODE_DPAD_LEFT));
        assertEquals(Word.RIGHT, Word.of(KeyEvent.KEYCODE_DPAD_RIGHT));
        assertEquals(Word.OK, Word.of(KeyEvent.KEYCODE_DPAD_CENTER));
        assertEquals(Word.OK, Word.of(KeyEvent.KEYCODE_ENTER));
        assertEquals(Word.OK, Word.of(KeyEvent.KEYCODE_NUMPAD_ENTER));
        assertEquals(Word.BACK, Word.of(KeyEvent.KEYCODE_BACK));
    }

    /**
     * A key this surface does not answer says nothing, so it falls through to the system.
     *
     * The case that matters is the media keys: the player screen has its own map and the television
     * remote's play and pause must reach it, not be swallowed by a page that does not use them.
     * Volume, home and the assistant are the same question with a worse failure.
     */
    @Test
    public void a_key_this_surface_does_not_answer_says_nothing() {
        for (int code : new int[] {
                KeyEvent.KEYCODE_VOLUME_UP,
                KeyEvent.KEYCODE_VOLUME_DOWN,
                KeyEvent.KEYCODE_HOME,
                KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE,
                KeyEvent.KEYCODE_MEDIA_FAST_FORWARD,
                KeyEvent.KEYCODE_A,
                KeyEvent.KEYCODE_SEARCH,
                KeyEvent.KEYCODE_UNKNOWN}) {
            assertNull("key code " + code + " is not this surface's", Word.of(code));
        }
    }

    /**
     * A game pad's A and B are select and back, which the Compose path did not have.
     *
     * Note 179's correction makes this reachable: {@code apps/xbox} takes this surface, and a pad is
     * the only controller there. Two lines, and unreachable from a remote.
     */
    @Test
    public void a_game_pad_selects_with_a_and_goes_back_with_b() {
        assertEquals(Word.OK, Word.of(KeyEvent.KEYCODE_BUTTON_A));
        assertEquals(Word.BACK, Word.of(KeyEvent.KEYCODE_BUTTON_B));
    }

    /** A right to left page swaps the arrows along a line and leaves the others alone. */
    @Test
    public void a_right_to_left_page_swaps_left_and_right_and_nothing_else() {
        assertEquals(Word.RIGHT, Word.LEFT.turned());
        assertEquals(Word.LEFT, Word.RIGHT.turned());
        assertEquals(Word.UP, Word.UP.turned());
        assertEquals(Word.DOWN, Word.DOWN.turned());
        assertEquals(Word.OK, Word.OK.turned());
        assertEquals(Word.BACK, Word.BACK.turned());
    }

    /** And turning twice is where it started, for every word, which is what makes the swap safe to apply once per page. */
    @Test
    public void turning_twice_is_where_it_started() {
        for (Word word : Word.values()) {
            assertEquals(word, word.turned().turned());
        }
    }

    /**
     * The two languages name the same words, read out of the Kotlin.
     *
     * The sixth word is the one that matters: a screen's {@code press} is a switch over these, so a
     * word added on one side and not the other is a screen that cannot answer a key it is sent, or a
     * key nothing sends. This fails when either side gains or loses a word.
     */
    @Test
    public void the_kotlin_remote_and_this_enum_name_the_same_six_words() {
        List<String> named = Repo.every("Word\\.([A-Z_]+)", Repo.text(REMOTE));
        TreeSet<String> kotlin = new TreeSet<>(named);
        TreeSet<String> java = new TreeSet<>();
        for (Word word : Word.values()) {
            java.add(word.name());
        }
        assertEquals(java, kotlin);
        assertEquals(6, java.size());
    }

    private static final String REMOTE =
            "apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Remote.kt";
}
