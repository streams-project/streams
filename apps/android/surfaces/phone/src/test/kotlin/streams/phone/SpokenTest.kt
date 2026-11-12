package streams.phone

import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Whether the parts of this surface a person touches still say what they are.
 *
 * **These read the source and not a tree.** What each helper puts on a node is tested where the
 * helpers live, in `surfaces/shared/src/test/kotlin/streams/shared/SaysTest.kt`, off the real
 * `SemanticsConfiguration`. What nothing offline can test is the composed tree: the two libraries
 * that would compose one in a unit test, `androidx.compose.ui:ui-test` and `org.robolectric`, are
 * both absent from `~/.gradle/caches/modules-2/files-2.1` and gradle runs `--offline` on this
 * machine (verified 2026-11-12). So the question asked here is the one that can be answered from
 * the source and still matters: is a helper applied to every control that needs one, and is it
 * still applied where it was.
 *
 * Note 215's open item is exactly that the answer is unguarded -- "S3 can drop any of the
 * semantics call sites and every check in the repository still passes" -- because `./check` runs
 * no gradle at all (`apps/android/surfaces/tv/build.gradle.kts:20`).
 */
class SpokenTest {
    /**
     * Every control that answers a tap says something, except two that are named here with their
     * reasons.
     *
     * This is the measurement `Says.kt:16-21` was written from, turned into a check.
     * `Modifier.clickable` makes a node that answers a tap and does not gather what is drawn inside
     * it, so a control with no helper on it is reached by a reader and read out as nothing. On
     * 2026-11-12 this surface had 62 chains that answer a tap: 50 carry a helper, 10 are tap
     * regions rather than controls, and 2 are below.
     *
     * A chain with `indication = null` is not a control. It draws no press feedback, which a scrim
     * that dismisses a sheet and a box that swallows taps so they do not reach the scrim both do
     * and a button never does. Ten of the 62 are those, in the profile sheets, and they are
     * separated by that mark rather than by being listed.
     *
     * The two exceptions are exact, so a third fails this test and fixing either of these fails it
     * too -- the list cannot outlive the gap:
     *  * `watch/Watch.kt` -- the whole player surface, which takes a tap to show its controls. It
     *    is the screen, not a control on it, and it has no words of its own.
     *  * `profiles/EditProfile.kt` -- the confirm button of the password sheet. **This one is a
     *    real gap**, found while writing this test: the sheet's Cancel beside it speaks and this
     *    one does not, so a reader hears one of the two buttons. It is in `src/main`, which this
     *    package does not own, so it is recorded here and reported rather than fixed.
     */
    @Test
    fun `every control that answers a tap says something`() {
        var taps = 0
        var spoken = 0
        var regions = 0
        val silent = sortedSetOf<String>()
        walk { path, text ->
            for (chain in chains(text)) {
                if (chain.calls.none { it in TAPS }) continue
                taps++
                when {
                    SAYS.containsMatchIn(chain.text) -> spoken++
                    INDICATION.containsMatchIn(chain.text) -> regions++
                    else -> silent += "$path:${lineAt(text, chain.at)}"
                }
            }
        }
        assertTrue("the scan found $taps tap chains: it has stopped reading the surface", taps >= 60)
        assertEquals(KNOWN_SILENT, silent.map { it.substringBefore(':') }.toSortedSet())
        assertEquals("two controls are known silent and no more", 2, silent.size)
        assertEquals("the controls that speak, the tap regions and the two known silent are all of them", taps, spoken + regions + silent.size)
    }

    /**
     * The spoken call sites, per file, exactly.
     *
     * A total alone would let a rewrite move one screen's labels onto another and stay green, so
     * the shape is held per file. Which means this map has to be edited whenever a label is added,
     * moved or removed, and that is the point: editing it is a rewriter stating, in the diff, which
     * parts stopped speaking.
     *
     * Counted 2026-11-12 by this test's own scan: 71 calls across 14 files on the phone. Note 215's
     * figure of 99 across 29 files for all three surfaces is not reproducible by this method -- the
     * same files give 95 in 27 counting calls with comments blanked (71 phone, 22 television, 2
     * shared), and 93, 97 or 101 counting matching lines with `rg` depending on whether the two
     * helper definition files and `sayBoth` are included. The order is right and the exact number
     * is method-dependent, so the method is recorded here rather than the number quoted.
     */
    @Test
    fun `every screen that spoke still speaks`() {
        val census = sortedMapOf<String, Int>()
        walk { path, text ->
            val calls = SAYS.findAll(text).count()
            if (calls > 0) census[path] = calls
        }
        assertEquals(SPOKEN, census.toMap())
        assertEquals(71, census.values.sum())
    }

    /**
     * No part is given English words from the code.
     *
     * `Says.kt:47-48` is explicit that a label comes from `strings/` or from the title's own name
     * and is never an English literal. An English label is invisible on an English build and is the
     * whole of what a person hears in the other seventeen languages. `WordsTest` checks that the
     * keys this surface asks for exist; this checks that it asks by key at all.
     *
     * 33 of the surface's 71 call sites take words. The other 38 are `saysItsWords` and
     * `decoration`, which take none, so there is nothing for them to be given in English.
     */
    @Test
    fun `no part is given English words from the code`() {
        val literals = sortedSetOf<String>()
        var sites = 0
        walk { path, text ->
            for (found in WORDED.findAll(text)) {
                sites++
                if (firstArgument(text, found.range.last).trimStart().startsWith("\"")) {
                    literals += "$path:${lineAt(text, found.range.first)}"
                }
            }
        }
        assertTrue("the scan found $sites call sites: it has stopped reading the surface", sites >= 30)
        assertEquals(emptySet<String>(), literals.toSet())
    }

    /** Every `.kt` file of this surface's main source, with its comments blanked out. */
    private fun walk(read: (String, String) -> Unit) {
        val main = File(repoRoot(), MAIN)
        assertTrue("$MAIN has moved", main.isDirectory)
        main.walkTopDown()
            .filter { it.isFile && it.extension == "kt" }
            .sortedBy { it.path }
            .forEach { read(it.toRelativeString(main), withoutComments(it.readText())) }
    }

    private companion object {
        const val MAIN = "apps/android/surfaces/phone/src/main/kotlin/streams/phone"

        /** Anything that makes a node answer a tap. */
        val TAPS = setOf("clickable", "combinedClickable", "selectable", "toggleable")

        /**
         * Any of the shared helpers, anywhere in the chain -- including inside a `.then(...)`,
         * which `profiles/GateGrid.kt:166` uses to pick a label by whether the profile has a PIN.
         */
        val SAYS = Regex("""\.(saysItsWords|saysSwitch|says|decoration)\s*[({]""")

        /** The two that take words, which are the ones that could be given English. */
        val WORDED = Regex("""\.(saysSwitch|says)\s*\(""")

        /** A chain that draws no press feedback is a tap region and not a control. */
        val INDICATION = Regex("""indication\s*=\s*null""")

        val KNOWN_SILENT = sortedSetOf("profiles/EditProfile.kt", "watch/Watch.kt")

        val SPOKEN = mapOf(
            "browse/Billboard.kt" to 2,
            "browse/Browse.kt" to 2,
            "browse/Header.kt" to 4,
            "browse/Home.kt" to 2,
            "browse/Rows.kt" to 8,
            "notifications/Notifications.kt" to 4,
            "parts/BottomBar.kt" to 1,
            "profiles/AddProfile.kt" to 5,
            "profiles/EditProfile.kt" to 20,
            "profiles/GateGrid.kt" to 5,
            "profiles/PinSheet.kt" to 2,
            "title/Episodes.kt" to 5,
            "title/MoreLike.kt" to 1,
            "title/Title.kt" to 10,
        )
    }
}
