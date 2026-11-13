package streams.tv.parts;

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
 * Reading this project's own source as text, for the test that has nothing else to read.
 *
 * <h2>The fourth copy of this helper, and it should be the last</h2>
 * {@code src/test/kotlin/streams/tv/Sources.kt} is first and says why a source test is worth its
 * cost. {@code src/test/java/streams/tv/views/Repo.java} is the second, because that file's
 * functions are {@code internal} and an internal Kotlin top-level function is not a name Java can
 * depend on. {@code src/test/java/streams/tv/browse/Source.java} is the third, because {@code Repo}
 * is package private in {@code streams.tv.views}. This is the fourth, for the same reason again.
 *
 * Four copies of one helper is a value with four homes, which is what this migration is supposed to
 * be reducing. It is written rather than avoided because the alternative inside this package's
 * ownership is to make a file in another agent's package public, and the honest fix is one change
 * by the lead: now that {@code Says} and {@code Words} have moved to {@code surfaces/shared}, a
 * {@code shared/src/test/java/streams/shared/Repo.java} can hold this once and every surface's
 * tests can read it. That is named in this package's report and not done here.
 *
 * Kept to what this package's one source test needs, so the copy is as small as a copy can be.
 */
final class Text {
    private Text() {
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
     * The same text with every comment replaced by spaces, so a scan for code cannot match the
     * prose about the code and every offset still points where it did. {@code Sources.kt:29-37}
     * found this necessary on 14 Compose chains; it is more necessary here, where the Java cites
     * the Kotlin line of every number it carries.
     */
    static String code(String path) {
        String said = text(path);
        char[] out = said.toCharArray();
        int at = 0;
        while (at < said.length()) {
            char here = said.charAt(at);
            if (here == '"') {
                at++;
                while (at < said.length()) {
                    if (said.charAt(at) == '\\') {
                        at += 2;
                        continue;
                    }
                    if (said.charAt(at) == '"') {
                        at++;
                        break;
                    }
                    at++;
                }
                continue;
            }
            if (here == '/' && at + 1 < said.length() && said.charAt(at + 1) == '/') {
                while (at < said.length() && said.charAt(at) != '\n') {
                    out[at] = ' ';
                    at++;
                }
                continue;
            }
            if (here == '/' && at + 1 < said.length() && said.charAt(at + 1) == '*') {
                while (at < said.length()) {
                    if (said.startsWith("*/", at)) {
                        out[at] = ' ';
                        out[at + 1] = ' ';
                        at += 2;
                        break;
                    }
                    if (said.charAt(at) != '\n') {
                        out[at] = ' ';
                    }
                    at++;
                }
                continue;
            }
            at++;
        }
        return new String(out);
    }

    /**
     * The one match of a pattern, or a failure saying how many there were instead. Every number
     * these tests hold is written once; a pattern matching twice means it has been copied, which is
     * the drift the test is for. {@code Sources.kt:104-114}.
     */
    static Matcher theOne(String pattern, String said, String what) {
        Matcher found = Pattern.compile(pattern).matcher(said);
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

    /** The one integer a pattern's first group holds. */
    static int number(String pattern, String said, String what) {
        return Integer.parseInt(theOne(pattern, said, what).group(1));
    }

    /** The one hundredths an alpha is written as: {@code 0.72f} is 72. */
    static int hundredths(String pattern, String said, String what) {
        String decimal = theOne(pattern, said, what).group(1);
        return Math.round(Float.parseFloat(decimal) * 100f);
    }
}
