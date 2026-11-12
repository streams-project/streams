package streams.tv.views;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Reading this project's own source as text, for the tests that have nothing else to read.
 *
 * The Java twin of {@code src/test/kotlin/streams/tv/Sources.kt}, written rather than reused for one
 * reason: that file's helpers are {@code internal}, and an internal Kotlin top-level function is not
 * a name Java source can depend on.
 *
 * What a source test buys, in that file's own words: it can hold a number against the derivation
 * written beside it, and against a number in a different file, which is the only kind of check
 * worth having on a number -- {@code assertEquals(328, STEP)} passes forever and catches nothing.
 * What it costs: these tests are coupled to the written form of both languages, so they fail when
 * either side is rewritten. That is intended during a migration. A check that survives a rewrite
 * without being looked at is a check the rewrite was free to ignore.
 */
final class Repo {
    private Repo() {
    }

    /** Where the repository is, found by walking up: a test's working folder is not promised. */
    static File root() {
        File here = new File("").getAbsoluteFile();
        while (here != null && !new File(here, "strings/en.json").isFile()) {
            here = here.getParentFile();
        }
        assertTrue("no strings/en.json above " + new File("").getAbsolutePath(), here != null);
        return here;
    }

    /** One of this project's files, as text. */
    static String text(String path) {
        File file = new File(root(), path);
        assertTrue(path + " has gone", file.isFile());
        try {
            return new String(Files.readAllBytes(file.toPath()), StandardCharsets.UTF_8);
        } catch (IOException broken) {
            throw new IllegalStateException("could not read " + path, broken);
        }
    }

    /**
     * The one match of a pattern in a text, or a failure that says how many there were instead.
     *
     * Every number these tests hold is written once. A pattern that matches twice means the number
     * has been copied, which is the drift the test is for, so finding two is a failure and not a
     * choice of which to read. Taken from {@code Sources.kt:111-114}.
     */
    static Matcher theOne(String pattern, int flags, String text, String what) {
        Matcher found = Pattern.compile(pattern, flags).matcher(text);
        List<String> all = new ArrayList<>();
        while (found.find()) {
            all.add(found.group());
        }
        assertEquals(what + ": expected one `" + pattern + "`, found " + all.size() + " " + all,
                1, all.size());
        found.reset();
        assertTrue(found.find());
        return found;
    }

    /** Every match of one capturing group, in the order they are written. */
    static List<String> every(String pattern, String text) {
        Matcher found = Pattern.compile(pattern).matcher(text);
        List<String> all = new ArrayList<>();
        while (found.find()) {
            all.add(found.group(1));
        }
        return all;
    }
}
