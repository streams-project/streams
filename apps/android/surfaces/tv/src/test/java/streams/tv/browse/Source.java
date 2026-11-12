package streams.tv.browse;

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
 * The third copy of this helper, and the reason is a language boundary each time:
 * {@code src/test/kotlin/streams/tv/Sources.kt}'s functions are {@code internal}, which is not a
 * name Java source can depend on, and {@code src/test/java/streams/tv/views/Repo.java} is package
 * private in {@code streams.tv.views}. Both say the same thing about the cost, and it is worth
 * repeating here because this file's tests are the ones the cost was predicted for:
 *
 * "What a source test buys: it can hold a number against the derivation written beside it, and
 * against a number in a different file, which is the only kind of check worth having on a number
 * ({@code assertEquals(328, STEP)} passes forever and catches nothing). What it costs: the tests are
 * coupled to the written form, so porting this surface to Java and Views will break them. That is
 * intended." ({@code Sources.kt:29-38})
 *
 * This is the port those lines were written about. So these tests read <b>both</b> forms -- the
 * Kotlin that is still there and the Java that replaced it -- and hold them equal. A number that did
 * not come across fails; a number changed on one side only fails; and when the lead deletes the dead
 * Kotlin, the Kotlin half of each assertion has to be deleted deliberately, in the diff, which is
 * the whole point of {@code Sources.kt}'s rule.
 */
final class Source {
    /** The television surface's Kotlin, which these tests read the measurements out of. */
    static final String KOTLIN = "apps/android/surfaces/tv/src/main/kotlin/streams/tv";

    /** And its Java, which they read them out of again. */
    static final String JAVA = "apps/android/surfaces/tv/src/main/java/streams/tv";

    /** What every surface shares, for the brand's colours and the half-dp scale. */
    static final String SHARED = "apps/android/surfaces/shared/src/main/kotlin/streams/shared";

    private Source() {
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
     * The same text with every line comment and block comment replaced by spaces, so a scan for code
     * cannot match the prose about the code and every offset still points where it did.
     *
     * {@code Sources.kt:29-37} found this necessary on 14 Compose chains, and it is more necessary
     * here: the Java in this package cites the Kotlin line every number came from, so every number
     * appears twice in the file -- once in a javadoc citation and once as the constant. A scan that
     * did not blank the comments would find each one twice and {@link #theOne} would fail on all of
     * them.
     */
    static String bare(String text) {
        char[] out = text.toCharArray();
        int at = 0;
        while (at < text.length()) {
            char here = text.charAt(at);
            if (here == '"') {
                at++;
                while (at < text.length()) {
                    if (text.charAt(at) == '\\') {
                        at += 2;
                        continue;
                    }
                    if (text.charAt(at) == '"') {
                        at++;
                        break;
                    }
                    at++;
                }
                continue;
            }
            if (here == '/' && at + 1 < text.length() && text.charAt(at + 1) == '/') {
                while (at < text.length() && text.charAt(at) != '\n') {
                    out[at] = ' ';
                    at++;
                }
                continue;
            }
            if (here == '/' && at + 1 < text.length() && text.charAt(at + 1) == '*') {
                while (at < text.length()) {
                    if (text.startsWith("*/", at)) {
                        out[at] = ' ';
                        out[at + 1] = ' ';
                        at += 2;
                        break;
                    }
                    if (text.charAt(at) != '\n') {
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

    /** One of this project's files with its comments blanked out. */
    static String code(String path) {
        return bare(text(path));
    }

    /**
     * The one match of a pattern in a text, or a failure that says how many there were instead.
     *
     * Every number these tests hold is written once in the code. A pattern that matches twice means
     * the number has been copied, which is the drift the test is for. {@code Sources.kt:104-114}.
     */
    static Matcher theOne(String pattern, String text, String what) {
        Matcher found = Pattern.compile(pattern).matcher(text);
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
    static int number(String pattern, String text, String what) {
        return Integer.parseInt(theOne(pattern, text, what).group(1));
    }

    /** Every file of a folder with a suffix, relative to that folder, in a settled order. */
    static List<String> filesUnder(String folder, final String suffix) {
        List<String> found = new ArrayList<>();
        File base = new File(root(), folder);
        assertTrue(folder + " has moved", base.isDirectory());
        walk(base, base.getPath().length() + 1, suffix, found);
        java.util.Collections.sort(found);
        return found;
    }

    private static void walk(File here, int cut, String suffix, List<String> found) {
        File[] inside = here.listFiles();
        if (inside == null) {
            return;
        }
        for (File one : inside) {
            if (one.isDirectory()) {
                walk(one, cut, suffix, found);
            } else if (one.getName().endsWith(suffix)) {
                found.add(one.getPath().substring(cut));
            }
        }
    }

    /** How many times a pattern appears. */
    static int count(String pattern, String text) {
        Matcher found = Pattern.compile(pattern).matcher(text);
        int seen = 0;
        while (found.find()) {
            seen++;
        }
        return seen;
    }
}
