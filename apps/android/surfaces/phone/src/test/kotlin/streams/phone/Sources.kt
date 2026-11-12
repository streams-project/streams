package streams.phone

import java.io.File

/**
 * Reading this project's own source as text, for the tests that have nothing else to read.
 *
 * Some of the measured numbers in this surface are `private const val`, and all of its derivations
 * are written as prose beside the number they produced, so a test in this source set cannot name
 * them. `WordsTest` in this same source set set the precedent on 2026-11-12: a unit test that walks
 * the repository and reads it. The same route is taken here, and it costs something worth stating
 * plainly.
 *
 * A literal copy of `surfaces/tv/src/test/kotlin/streams/tv/Sources.kt` but for its package and the
 * surface it points at, plus [chains], which only the phone needs because only the phone has
 * controls that answer a tap. A test source set is not published to another module's.
 *
 * What a source test buys: it can hold a number against the derivation written beside it, and
 * against a number in a different file, which is the only kind of check worth having on a number
 * (`assertEquals(328, STEP)` passes forever and catches nothing).
 *
 * What it costs: the tests are coupled to the written form, so porting this surface to Java and
 * Views will break them. That is intended. Note 215's rule is the owner's -- "do not sacrifice
 * working features simply to make the migration easier" -- and a check that survives a rewrite
 * without being looked at is a check the rewrite was free to ignore. Re-pointing each of these at
 * the Java form is how a rewriter states, in the diff, that the number came across.
 */
internal fun repoRoot(): File {
    var here: File? = File("").absoluteFile
    while (here != null && !File(here, "strings/en.json").isFile) here = here.parentFile
    return requireNotNull(here) { "no strings/en.json above ${File("").absolutePath}" }
}

/**
 * The same text with every comment replaced by spaces, so a scan for code cannot match the prose
 * about the code, and every offset still points where it did.
 *
 * Needed for a reason found while writing these: the surfaces put a comment between the links of a
 * `Modifier` chain, explaining why a part speaks the way it does. A scan that does not blank those
 * stops reading the chain at the comment and reports the rest of it missing -- which it did, on 14
 * chains, before this existed.
 */
internal fun withoutComments(text: String): String {
    val out = text.toCharArray()
    var i = 0
    while (i < text.length) {
        val c = text[i]
        if (c == '"') {
            if (text.startsWith("\"\"\"", i)) {
                val end = text.indexOf("\"\"\"", i + 3)
                i = if (end < 0) text.length else end + 3
                continue
            }
            i++
            while (i < text.length) {
                if (text[i] == '\\') { i += 2; continue }
                if (text[i] == '"') { i++; break }
                i++
            }
            continue
        }
        if (c == '/' && i + 1 < text.length && text[i + 1] == '/') {
            while (i < text.length && text[i] != '\n') { out[i] = ' '; i++ }
            continue
        }
        if (c == '/' && i + 1 < text.length && text[i + 1] == '*') {
            var depth = 0
            while (i < text.length) {
                if (text.startsWith("/*", i)) { depth++; out[i] = ' '; out[i + 1] = ' '; i += 2; continue }
                if (text.startsWith("*/", i)) {
                    depth--; out[i] = ' '; out[i + 1] = ' '; i += 2
                    if (depth == 0) break
                    continue
                }
                if (text[i] != '\n') out[i] = ' '
                i++
            }
            continue
        }
        i++
    }
    return String(out)
}

/**
 * The first argument of the call whose opening bracket is at [open], as written. Brackets inside it
 * are counted, so `says(words["say.close"].orEmpty())` comes back whole.
 */
internal fun firstArgument(text: String, open: Int): String {
    var depth = 0
    var i = open
    while (i < text.length) {
        when (text[i]) {
            '(', '[', '{' -> depth++
            ')', ']', '}' -> {
                depth--
                if (depth == 0) return text.substring(open + 1, i)
            }
            ',' -> if (depth == 1) return text.substring(open + 1, i)
        }
        i++
    }
    return ""
}

/** The 1-based line a position falls on, so a failure names a place a person can open. */
internal fun lineAt(text: String, at: Int): Int = text.take(at).count { it == '\n' } + 1

/**
 * The one match of [pattern] in [text], or a failure that says how many there were instead.
 *
 * Every number these tests hold is written once. A pattern that matches twice means the number has
 * been copied, which is the drift the test is for, so finding two is a failure and not a choice of
 * which to read.
 */
internal fun theOne(pattern: Regex, text: String, what: String): MatchResult {
    val all = pattern.findAll(text).toList()
    check(all.size == 1) { "$what: expected one `${pattern.pattern}`, found ${all.size}" }
    return all.single()
}

/** A `Modifier` chain, as the names of the calls on it and the text it was written as. */
internal data class Chain(val at: Int, val calls: List<String>, val text: String)

/**
 * Every `Modifier` chain in [text], which has to have had its comments blanked by [withoutComments]
 * first: the surfaces put a comment between the links of a chain to say why a part speaks the way
 * it does, and a scan that does not blank those stops reading at the comment.
 *
 * Found by hand rather than by one regular expression because a chain is not a regular language:
 * `.clickable(onClick = { ... })` and `.graphicsLayer { ... }` carry brackets of their own, and a
 * chain runs over as many lines as it likes. So this walks from each `Modifier` through as many
 * `.name(...)` and `.name { ... }` links as follow it, counting brackets.
 */
internal fun chains(text: String): List<Chain> {
    val found = mutableListOf<Chain>()
    for (start in Regex("""\bModifier\b""").findAll(text)) {
        var i = start.range.last + 1
        val calls = mutableListOf<String>()
        while (true) {
            var j = i
            while (j < text.length && text[j].isWhitespace()) j++
            if (j >= text.length || text[j] != '.') break
            j++
            var k = j
            while (k < text.length && (text[k].isLetterOrDigit() || text[k] == '_')) k++
            if (k == j) break
            calls += text.substring(j, k)
            i = k
            // A link may carry a bracketed argument list, a trailing lambda, or both.
            while (true) {
                var b = i
                while (b < text.length && text[b].isWhitespace()) b++
                if (b >= text.length || (text[b] != '(' && text[b] != '{')) break
                i = past(text, b)
            }
        }
        if (calls.isNotEmpty()) found += Chain(start.range.first, calls, text.substring(start.range.first, i))
    }
    return found
}

/** One past the bracket group that opens at [open], counting nested brackets of the same pair. */
private fun past(text: String, open: Int): Int {
    val close = if (text[open] == '(') ')' else '}'
    var depth = 0
    var i = open
    while (i < text.length) {
        if (text[i] == text[open]) depth++
        else if (text[i] == close) {
            depth--
            if (depth == 0) return i + 1
        }
        i++
    }
    return text.length
}
