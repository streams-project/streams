package streams.tv

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * The one place the keyboard's key map can be reached from, pressed on the two pages whose fourth
 * row is short.
 *
 * Found while porting `parts/Keyboard.kt` to Java on 2026-11-13, by a Java test asserting that every
 * row covers every column. Two of the three pages do not: `Letters`' fourth row is `zxcvbnm'`, eight
 * letters from column 2, which reaches column 9; `Symbols`' is `,.?/\|~` and `Accented`'s is
 * `œšžćčđğ`, seven each, which reach column 8. So on either of those pages a remote standing on
 * column 9 and pressed down lands on a row holding no key at that column, and the next press asks
 * `caps.first { ... }` for a key that is not there.
 *
 * It is reachable in eleven presses from where the keyboard opens, on every screen that draws one:
 * sign in, add profile, edit profile and the lock steps. This test presses exactly that.
 */
class KeyboardReachTest {
    private fun keys() = KeyboardState("")

    private fun KeyboardState.say(vararg words: Word) {
        for (word in words) press(word, "cancel", "done") {}
    }

    /** To the marks key on the modifier row, which swaps the symbols page in. */
    private fun KeyboardState.toSymbols() {
        say(Word.DOWN, Word.DOWN, Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT, Word.OK)
    }

    /**
     * The far right of a short row holds the remote instead of dropping it.
     *
     * Before the fix this did not fail an assertion: it threw `NoSuchElementException` out of
     * `press`, which on a device is the sign in screen closing under a viewer typing a password.
     * The remote lands on the last key of the row it arrived at, which is where a viewer coming
     * down the right hand edge expects to be.
     */
    @Test
    fun `the right hand edge of a short row holds the remote`() {
        val keys = keys()
        keys.toSymbols()
        keys.say(Word.UP, Word.UP, Word.UP, Word.UP)
        repeat(9) { keys.say(Word.RIGHT) }
        assertEquals(0, keys.row)
        assertEquals(9, keys.column)
        keys.say(Word.DOWN, Word.DOWN, Word.DOWN)
        assertEquals(3, keys.row)
        keys.say(Word.OK)
        assertEquals("the last key of the short row, which is the one under the remote", "~", keys.text)
    }

    /** The same on the accented page, whose fourth row is short for the same reason. */
    @Test
    fun `the accented page holds it too`() {
        val keys = keys()
        keys.say(Word.DOWN, Word.DOWN, Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT, Word.RIGHT, Word.OK)
        keys.say(Word.UP, Word.UP, Word.UP, Word.UP)
        repeat(9) { keys.say(Word.RIGHT) }
        keys.say(Word.DOWN, Word.DOWN, Word.DOWN, Word.OK)
        assertEquals("ğ", keys.text)
    }

    /** And the letters page, whose fourth row does reach column 9, is unchanged by the fix. */
    @Test
    fun `the letters page reaches its last key as it always did`() {
        val keys = keys()
        keys.say(Word.UP, Word.UP)
        repeat(9) { keys.say(Word.RIGHT) }
        keys.say(Word.DOWN, Word.DOWN, Word.DOWN, Word.OK)
        assertEquals("'", keys.text)
    }
}
