package streams.phone

import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Every word this surface asks for has to be in the strings file.
 *
 * This is the hole the says() pass fell into: `platform/tools/check_strings.py` checks the other
 * seventeen languages against `en.json` and nothing checked `en.json` against the code, so sixteen
 * labels were wired to keys no language had. `words[...]` is a map lookup, `says()` leaves an empty
 * label unset, and the result was a control that answered a tap and still said nothing -- exactly
 * the defect the pass was for, passing every check.
 */
class WordsTest {
    /**
     * Keys this surface asks for that no language has yet, each one a gap with its own owner. The
     * set is exact on purpose: a new one fails this test, and filling one of these fails it too,
     * so the list cannot quietly outlive the gap.
     *
     * Every key this surface asks for is of this shape and no other literal in it is, so the
     * reading needs no list of families to tell a key from a path: on 2026-11-12 the whole main
     * source set gave these three and nothing else.
     *
     * `app.title` is the phone header's last fallback when the menu has no label for the page
     * (`browse/Home.kt:100`); the Apple surface falls through it to `seen.brand`
     * (`apps/apple/Sources/SurfacesPhone/browse/Home.swift:55`) and the phone does not, so the
     * header can be empty. `language.digits` and `language.text_scale` left this list on
     * 2026-11-12, when both were filled in all eighteen language files with the value the code
     * already fell back to; they were per-language data rather than labels. `app.title` stays,
     * because filling it is not the obvious fix: the Apple surface falls through to `seen.brand`
     * and this one does not, so the open question is whether the phone should do the same and the
     * key is wanted at all. That is the owner's, and until he takes it the gap is real.
     */
    private val gaps = setOf("app.title")

    @Test
    fun `every word the phone asks for is in the strings file`() {
        val root = repoRoot()
        val book = Json.parseToJsonElement(File(root, "strings/en.json").readText()).jsonObject
        val missing = sortedSetOf<String>()
        File(root, "apps/android/surfaces/phone/src/main/kotlin").walkTopDown()
            .filter { it.isFile && it.extension == "kt" }
            .forEach { source ->
                for (found in KEY.findAll(source.readText())) {
                    val key = found.groupValues[1]
                    // `counted` asks for a family by its base and the book holds the forms.
                    if (key in book || "$key.one" in book) continue
                    missing += key
                }
            }
        assertEquals(gaps, missing.toSet())
    }

    /** Where the strings live, found by walking up: a test's working folder is not promised. */
    private fun repoRoot(): File {
        var here: File? = File("").absoluteFile
        while (here != null && !File(here, "strings/en.json").isFile) here = here.parentFile
        return requireNotNull(here) { "no strings/en.json above ${File("").absolutePath}" }
    }

    private companion object {
        val KEY = Regex("\"([a-z][a-z0-9_]*(?:\\.[a-z0-9_]+)+)\"")
    }
}
