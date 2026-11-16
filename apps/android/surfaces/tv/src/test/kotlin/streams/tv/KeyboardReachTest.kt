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
     * What the painter would light, reached the way `Keyboard` reaches it: `lit` names a place in
     * the row and the row is read at it.
     */
    private fun KeyboardState.drawn(): String {
        val line = caps(page, upper, "cancel", "done")[row]
        val at = lit(row)
        if (at !in line.indices) return "<no key lit>"
        return line[at].word.orEmpty()
    }

    /**
     * The far right of a short row holds the remote instead of dropping it, and lights the key it
     * holds.
     *
     * Before the fix this did not fail an assertion: it threw `NoSuchElementException` out of
     * `press`, which on a device is the sign in screen closing under a viewer typing a password.
     * The remote lands on the last key of the row it arrived at, which is where a viewer coming
     * down the right hand edge expects to be, and `lit` gives `Keyboard` and `press` one answer so
     * the key that lights is the key that types.
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
        assertEquals("the painter lights the same key that OK types", "~", keys.drawn())
        keys.say(Word.OK)
        assertEquals("the last key of the short row, which is the one under the remote", "~", keys.text)
        keys.say(Word.LEFT)
        assertEquals("Left off the ragged stop steps from the key that held the remote", 7, keys.column)
        assertEquals("|", keys.drawn())
    }

    /** The same on the accented page, whose fourth row is short for the same reason. */
    @Test
    fun `the accented page holds it too`() {
        val keys = keys()
        keys.say(Word.DOWN, Word.DOWN, Word.LEFT, Word.LEFT, Word.LEFT, Word.LEFT, Word.RIGHT, Word.OK)
        keys.say(Word.UP, Word.UP, Word.UP, Word.UP)
        repeat(9) { keys.say(Word.RIGHT) }
        keys.say(Word.DOWN, Word.DOWN, Word.DOWN)
        assertEquals("ğ", keys.drawn())
        keys.say(Word.OK)
        assertEquals("ğ", keys.text)
    }

    /** And the letters page, whose fourth row does reach column 9, is unchanged by the fix. */
    @Test
    fun `the letters page reaches its last key as it always did`() {
        val keys = keys()
        keys.say(Word.UP, Word.UP)
        repeat(9) { keys.say(Word.RIGHT) }
        keys.say(Word.DOWN, Word.DOWN, Word.DOWN)
        assertEquals("'", keys.drawn())
        keys.say(Word.OK)
        assertEquals("'", keys.text)
    }

    /**
     * Every stop the keyboard can reach lights a key, and at each of the four ragged stops the lit
     * key is the last key of that row (`apps/xbox/surfaces/remote/keys_test.cpp:344-430`).
     */
    @Test
    fun `every reachable stop lights a key and the four ragged stops light the row's last`() {
        val seen = mutableSetOf<String>()
        val queue = ArrayDeque<List<Word>>()
        queue.add(emptyList())
        seen.add("${keys().page[3]}:false:2:4")
        val all = listOf(Word.UP, Word.DOWN, Word.LEFT, Word.RIGHT, Word.OK, Word.BACK)
        var stops = 0
        var unlit = 0
        var ragged = 0
        var wrongLit = 0
        while (queue.isNotEmpty()) {
            val path = queue.removeFirst()
            val from = keys()
            for (step in path) from.press(step, "cancel", "done") {}
            stops += 1
            val line = caps(from.page, from.upper, "cancel", "done")[from.row]
            val at = from.lit(from.row)
            if (at !in line.indices) unlit += 1
            if (line.none { it.holds(from.column) }) {
                ragged += 1
                if (at != line.lastIndex) wrongLit += 1
            }
            for (word in all) {
                val next = keys()
                for (step in path) next.press(step, "cancel", "done") {}
                next.press(word, "cancel", "done") {}
                if (next.lit(next.row) < 0) unlit += 1
                val key = "${next.page[3]}:${next.upper}:${next.row}:${next.column}"
                if (seen.add(key)) queue.add(path + word)
            }
        }
        assertEquals(360, stops)
        assertEquals(0, unlit)
        assertEquals(4, ragged)
        assertEquals(0, wrongLit)
    }
}
