package streams.tv

import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Every word this surface asks for has to be in the strings file. The television twin of
 * `apps/android/surfaces/phone/src/test/kotlin/streams/phone/WordsTest.kt`.
 *
 * The hole is the same one and it is wider here. `platform/tools/check_strings.py`, which `./check`
 * does run, compares the seventeen other languages against `en.json` and nothing compares `en.json`
 * against the code. So a label wired to a key no language holds passes every check in the
 * repository: `words[...]` is a map lookup that answers empty, `says()` leaves an empty label unset
 * (`app/Spoken.kt:41-42`), and the result is a part that draws nothing and says nothing.
 *
 * Wider here because this surface asks for 169 distinct key-shaped literals against the phone's 87,
 * and because a television part that draws nothing cannot be found by touch -- a remote has nothing
 * to explore with.
 */
class WordsTest {
    /**
     * Keys this surface asks for that `en.json` does not hold. The set is exact on purpose: a new
     * one fails this test, and filling one of these fails it too, so the list cannot outlive the
     * gap.
     *
     * None of the three is a label. All three are per-language settings that the surface reads and
     * falls back on in the code, and all three are in `ar.json` (302 keys against English's 290),
     * which is where the twelve extra Arabic keys note 215 records live. `language.digits` and
     * `language.text_scale` are read at `app/Television.kt:53`; `search.letters` is the on-screen
     * keyboard's alphabet, which has a Latin default in the code. Their absence from English is the
     * current behaviour and not a defect to fix from a test.
     */
    private val gaps = setOf("language.digits", "language.text_scale", "search.letters")

    @Test
    fun `every word the television asks for is in the strings file`() {
        val root = repoRoot()
        val book = Json.parseToJsonElement(File(root, "strings/en.json").readText()).jsonObject
        val asked = sortedSetOf<String>()
        val missing = sortedSetOf<String>()
        File(root, MAIN).walkTopDown()
            .filter { it.isFile && it.extension == "kt" }
            .forEach { source ->
                for (found in KEY.findAll(source.readText())) {
                    val key = found.groupValues[1]
                    asked += key
                    // `counted` asks for a family by its base and the book holds the forms.
                    if (key in book || "$key.one" in book) continue
                    missing += key
                }
            }
        assertTrue(
            "the scan found ${asked.size} keys: it has stopped reading the surface",
            asked.size > 150,
        )
        assertEquals(gaps, missing.toSet())
    }

    /**
     * Every key this surface asks for and English holds is held by all seventeen other languages
     * too, so a part that speaks on an English build speaks on every build.
     *
     * `check_strings.py` already compares each language against English as a whole. This is the
     * narrower question that matters to this surface: of the keys *this code* asks for, is any one
     * of them missing from a language a viewer might be on. A key present in English and absent in
     * Japanese draws nothing on a Japanese television and nothing in the repository says so today.
     */
    @Test
    fun `every word the television asks for is in every language`() {
        val root = repoRoot()
        val books = File(root, "strings").listFiles()
            .orEmpty()
            .filter { it.extension == "json" }
            .associate { it.nameWithoutExtension to Json.parseToJsonElement(it.readText()).jsonObject }
        assertTrue("there have to be eighteen language files, found ${books.size}", books.size == 18)
        val english = books.getValue("en")
        val asked = sortedSetOf<String>()
        File(root, MAIN).walkTopDown()
            .filter { it.isFile && it.extension == "kt" }
            .forEach { source -> KEY.findAll(source.readText()).forEach { asked += it.groupValues[1] } }
        val holes = sortedSetOf<String>()
        for (key in asked) {
            if (key !in english && "$key.one" !in english) continue
            for ((language, book) in books) {
                if (key !in book && "$key.one" !in book) holes += "$language:$key"
            }
        }
        assertEquals(emptySet<String>(), holes.toSet())
    }

    private companion object {
        const val MAIN = "apps/android/surfaces/tv/src/main/kotlin/streams/tv"

        /**
         * Every key this surface asks for is of this shape and no other literal in it is, which is
         * what lets the reading work with no list of key families: a path, a colour and a format
         * string all fail it. Taken unchanged from the phone's `WordsTest` so the two surfaces are
         * read the same way.
         */
        val KEY = Regex("\"([a-z][a-z0-9_]*(?:\\.[a-z0-9_]+)+)\"")
    }
}
