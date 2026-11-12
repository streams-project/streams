package streams.tv

import androidx.compose.ui.Modifier
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * What each helper in `app/Spoken.kt` hands a screen reader, and whether the screens still call
 * them. The first tests any of the five have had.
 *
 * Note 215 names the television surface's accessibility as the largest risk in the Java and Views
 * rewrite, and records that nothing in the repository checks it: `./check` runs no gradle
 * (`build.gradle.kts:20` says so in its own comment), so the 22 call sites measured below could all
 * be deleted and every check would stay green.
 *
 * A correction to note 215 while this is being written: it says the 99 call sites use "the six
 * helpers in `surfaces/shared/.../Says.kt`". This surface does not use them. It has its own five in
 * `app/Spoken.kt`, and they are not the same five -- every one of them sets `selected`, because
 * this surface paints its own focus and the toolkit has one real focusable for the whole screen
 * (`Spoken.kt:26-32`), so `selected` is the only thing that tells a person where the remote is.
 * `Says.kt` sets `selected` on nothing at all. The two sets had to be tested separately.
 */
class SpokenTest {
    /**
     * A part the remote can stop on says what it is, that it is pressable, and whether the remote
     * is on it now.
     *
     * `selected` is set either way and not only when true. A reader that is told nothing when the
     * remote leaves a part has no way to know it left, and on a surface whose focus is painted
     * rather than held by the toolkit there is nothing else to ask.
     */
    @Test
    fun `a part the remote can stop on says whether the remote is on it`() {
        val on = one(Modifier.says("Play", on = true))
        assertEquals("Play", on.label)
        assertTrue(on.isButton)
        assertEquals(true, on.selected)

        val off = one(Modifier.says("Play", on = false))
        assertEquals(
            "a part the remote has left must say so, not fall silent",
            false,
            off.selected,
        )
        assertTrue(off.keys.contains("Selected"))
    }

    /** With no words for it, a part still says where the remote is. Half a label beats none. */
    @Test
    fun `a part with no words still says whether the remote is on it`() {
        val node = one(Modifier.says(null, on = true))
        assertEquals(true, node.selected)
        assertTrue(node.isButton)
        assertFalse("no words means no description, not an empty one", node.keys.contains("ContentDescription"))
    }

    /**
     * A part that is only described is not a button and is not focusable, and that separation is
     * the whole reason there are two overloads. A season heading says what it is; nothing presses
     * it. A rewrite that collapses the two makes a reader offer to press every label on the screen.
     */
    @Test
    fun `a part that is only described does not claim to be pressable`() {
        val node = one(Modifier.says("Season 2"))
        assertEquals("Season 2", node.label)
        assertNull("a described part is not a button", node.role)
        assertNull("and the remote does not stop on it", node.selected)
        assertEquals(setOf("ContentDescription"), node.keys)
    }

    /**
     * No words means no node. `Spoken.kt:36-37` is explicit: a key this surface asks for that the
     * strings do not hold leaves the part unlabelled rather than putting English in the code, and
     * `words[...]` answers empty until a run has read its language.
     *
     * The alternative -- `contentDescription = ""` -- is worse than nothing, because it makes a
     * reader stop on the part and read out nothing.
     */
    @Test
    fun `a part with no words to say adds no node at all`() {
        assertTrue(heard(Modifier.says(null)).isEmpty())
        assertTrue(heard(Modifier.says("")).isEmpty())
    }

    /** A pill, a tab, a key, a line of a menu: its own drawn words are its label. */
    @Test
    fun `a part whose drawn words are its label gathers them`() {
        val node = one(Modifier.saysItsWords(on = true))
        assertTrue("the words drawn inside have to become the label", node.gathers)
        assertTrue(node.isButton)
        assertEquals(true, node.selected)
        assertFalse(
            "a gathering part must not also carry words of its own, or they would drift",
            node.keys.contains("ContentDescription"),
        )
    }

    /** Artwork behind a name, a mark, an arrow: it goes away rather than becoming a silent stop. */
    @Test
    fun `a decoration goes away instead of becoming a silent stop`() {
        val node = one(Modifier.decoration())
        assertTrue(node.cleared)
        assertEquals(emptySet<String>(), node.keys)
    }

    /**
     * Two things heard one after the other are joined by a line end, so no language's punctuation
     * or word order is decided in the code (`Spoken.kt:74-78`).
     *
     * A comma joiner reads wrong in Arabic and Japanese and is the obvious thing a rewrite would
     * reach for, so the absence of one is stated rather than assumed.
     */
    @Test
    fun `two things heard together are joined by a line end and no punctuation`() {
        assertEquals("Stranger Things\nTrending Now", sayBoth("Stranger Things", "Trending Now"))
        assertFalse(sayBoth("a", "b").contains(","))
        assertFalse(sayBoth("a", "b").contains(" "))
    }

    /** Either side missing leaves the other alone, with no stray break around it. */
    @Test
    fun `a missing half leaves no stray break`() {
        assertEquals("Trending Now", sayBoth(null, "Trending Now"))
        assertEquals("Stranger Things", sayBoth("Stranger Things", ""))
        assertEquals("", sayBoth(null, null))
        assertEquals("", sayBoth("", ""))
    }

    /**
     * Where the words a reader hears come from. **This is a census of call sites and not a reading
     * of a tree**: it proves every spoken part on this surface still asks the strings file for its
     * words, and proves nothing about what a reader would hear on a screen.
     *
     * The rule it holds is `Spoken.kt:34-37`: the words are never written here, they arrive from
     * `strings/{lang}.json` through `seen.words`. An English literal in a label is invisible on an
     * English build and is the whole of what a person hears in the other seventeen languages.
     */
    @Test
    fun `no part is given English words from the code`() {
        val literals = mutableListOf<String>()
        var sites = 0
        walk { path, text ->
            for (found in CALL.findAll(text)) {
                sites++
                val argument = firstArgument(text, found.range.last)
                if (argument.trimStart().startsWith("\"")) {
                    literals += "$path:${text.take(found.range.first).count { it == '\n' } + 1} $argument"
                }
            }
        }
        assertTrue("the scan found no call sites at all, so it has stopped reading the surface", sites >= 14)
        assertEquals("every spoken label has to come from the strings file", emptyList<String>(), literals)
    }

    /**
     * The spoken call sites, per file, exactly.
     *
     * A total alone would let a rewrite move a screen's labels onto another screen and stay green,
     * so the shape is held per file. Which means this map has to be edited whenever a label is
     * added, moved or removed -- and that is the point: note 215's open item is that "S3 can drop
     * any of the semantics call sites and every check in the repository still passes". Editing this
     * map is a rewriter stating, in the diff, which parts stopped speaking.
     *
     * Counted 2026-11-12 at the tree these tests were written against, by this test's own scan:
     * 22 calls across 12 files. Note 215's figure of 99 across 29 files for all three surfaces is
     * not reproducible by this method -- the same files give 95 in 27 counting calls with comments
     * blanked, 93 or 97 or 101 counting matching lines with `rg` depending on whether the two
     * helper definition files and `sayBoth` are included. The order is right and the exact number
     * is method-dependent, so the method is stated here rather than the number quoted.
     */
    @Test
    fun `every screen that spoke still speaks`() {
        val census = sortedMapOf<String, Int>()
        walk { path, text ->
            val calls = SAYS.findAll(text).count()
            if (calls > 0) census[path] = calls
        }
        assertEquals(SPOKEN, census.toMap())
        assertEquals(22, census.values.sum())
    }

    /** Every `.kt` file of this surface's main source, with its comments blanked out. */
    private fun walk(read: (String, String) -> Unit) {
        val main = File(repoRoot(), MAIN)
        assertTrue("$MAIN has moved", main.isDirectory)
        main.walkTopDown()
            .filter { it.isFile && it.extension == "kt" && it.name != "Spoken.kt" }
            .sortedBy { it.path }
            .forEach { read(it.toRelativeString(main), withoutComments(it.readText())) }
    }

    private companion object {
        const val MAIN = "apps/android/surfaces/tv/src/main/kotlin/streams/tv"

        /** Any of this surface's five helpers, applied to something. */
        val SAYS = Regex("""\.(saysItsWords|says|decoration)\s*[({]""")

        /** The two that take words, which are the ones that could be given English. */
        val CALL = Regex("""\.(saysItsWords|says)\s*\(""")

        val SPOKEN = mapOf(
            "browse/HeroCard.kt" to 3,
            "browse/Rows.kt" to 2,
            "browse/TopBar.kt" to 3,
            "parts/Face.kt" to 1,
            "parts/Menu.kt" to 1,
            "parts/Pill.kt" to 1,
            "parts/TickGrid.kt" to 1,
            "profiles/ProfileMenu.kt" to 1,
            "title/Episodes.kt" to 4,
            "title/PosterGrid.kt" to 1,
            "title/Reactions.kt" to 1,
            "title/TitlePage.kt" to 3,
        )
    }
}
