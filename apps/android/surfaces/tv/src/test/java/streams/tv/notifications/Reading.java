package streams.tv.notifications;

import static org.junit.Assert.assertTrue;

import android.view.View;
import android.view.ViewGroup;
import android.view.accessibility.AccessibilityNodeInfo;
import contract.Contract;
import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import streams.shared.Words;

/**
 * What these two screens' tests read: the real words of the interface, and the real
 * {@link AccessibilityNodeInfo} tree the toolkit builds.
 *
 * <h2>Why it is not {@code streams.tv.views.Heard}</h2>
 * {@code views/HeardTest.java} is the pattern these tests follow and {@code views/Heard.java} is
 * the walker it follows it with, but that class is package private, so no test outside
 * {@code streams.tv.views} can name it. This is one copy of a walk, in the package that needs it,
 * and it should collapse into {@code Heard} the moment that class is made public -- which is a
 * change to another hand's file and is reported rather than made here.
 *
 * <h2>And it asks a different question</h2>
 * {@code Heard.under} answers "what would a reader stop on". {@link #unnamed(View)} answers the
 * opposite one, which is the defect actually measured on hardware on 2026-11-13: the television's
 * player, search, profile menu and Get Help each draw a node that says it is a control and carries
 * no words, so a screen reader can see a button there and cannot say what it is. A test that only
 * lists what speaks cannot fail when a sixth silent control arrives beside it.
 *
 * <p>A node is counted silent when it claims to be a control -- a class name a reader maps to a
 * kind, or a node that says the remote is selected on it -- and neither it nor anything beneath it
 * carries words. Anything beneath counts, because this surface labels a group and lets the words
 * inside it be the label ({@code Says.says} sets {@code screenReaderFocusable}, and the gathering
 * is the reader's work, not the toolkit's, so the node itself can be legitimately textless).
 */
public final class Reading {
    /** The words a reader is handed at one stop. */
    public static final class Stop {
        /** The words read out. */
        public final String label;

        /** What is read after them. Empty when the node says no state. */
        public final String state;

        /** What the node says it is, by the class name a reader maps to a kind. */
        public final String kind;

        /** Whether the node says the remote is on it. */
        public final boolean on;

        Stop(String label, String state, String kind, boolean on) {
            this.label = label;
            this.state = state;
            this.kind = kind;
            this.on = on;
        }

        @Override
        public String toString() {
            return kind + " \"" + label + "\"" + (state.isEmpty() ? "" : " (" + state + ")")
                    + (on ? " [on]" : "");
        }
    }

    private Reading() {
    }

    /**
     * Everything a reader would stop on under a view, in the order it would reach them.
     *
     * The same rule the platform uses: a node with words of its own is worth announcing. A subtree
     * marked {@code NO_HIDE_DESCENDANTS} is not entered, because that is what the flag means.
     */
    public static List<Stop> stops(View root) {
        List<Stop> found = new ArrayList<Stop>();
        walk(root, found);
        return found;
    }

    /** Just the words, which is what most of an assertion is about. */
    public static List<String> labels(View root) {
        List<Stop> found = stops(root);
        List<String> words = new ArrayList<String>(found.size());
        for (int at = 0; at < found.size(); at++) {
            words.add(found.get(at).label);
        }
        return words;
    }

    /**
     * Every node under a view that says it is a control and hands a reader no words at all, by the
     * class name it claims. Empty is the only passing answer.
     */
    public static List<String> unnamed(View root) {
        List<String> found = new ArrayList<String>();
        silent(root, found);
        return found;
    }

    /**
     * A part that is not on the screen is not read out, whatever it still holds.
     *
     * {@code views/Heard.java} does not ask this, because nothing in its tests is ever hidden. These
     * screens hide a part and keep its words -- the line that says there is nothing here keeps
     * saying it while a loaded screen is drawn over it -- and a walk that read hidden words would
     * report a screen saying two contradictory things.
     */
    private static boolean onScreen(View view) {
        return view.getVisibility() == View.VISIBLE;
    }

    private static void walk(View view, List<Stop> found) {
        if (!onScreen(view)
                || view.getImportantForAccessibility() == View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS) {
            return;
        }
        AccessibilityNodeInfo node = view.createAccessibilityNodeInfo();
        if (node != null && view.getImportantForAccessibility() != View.IMPORTANT_FOR_ACCESSIBILITY_NO) {
            String label = words(node);
            if (!label.isEmpty()) {
                found.add(new Stop(label, say(node.getStateDescription()), say(node.getClassName()),
                        node.isSelected()));
            }
        }
        if (view instanceof ViewGroup) {
            ViewGroup group = (ViewGroup) view;
            for (int at = 0; at < group.getChildCount(); at++) {
                walk(group.getChildAt(at), found);
            }
        }
    }

    private static void silent(View view, List<String> found) {
        if (!onScreen(view)
                || view.getImportantForAccessibility() == View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS) {
            return;
        }
        AccessibilityNodeInfo node = view.createAccessibilityNodeInfo();
        if (node != null && view.getImportantForAccessibility() != View.IMPORTANT_FOR_ACCESSIBILITY_NO
                && claimsAControl(node) && !anyWords(view)) {
            found.add(say(node.getClassName()) + " at " + view.getClass().getName());
        }
        if (view instanceof ViewGroup) {
            ViewGroup group = (ViewGroup) view;
            for (int at = 0; at < group.getChildCount(); at++) {
                silent(group.getChildAt(at), found);
            }
        }
    }

    private static boolean claimsAControl(AccessibilityNodeInfo node) {
        String kind = say(node.getClassName());
        return kind.equals("android.widget.Button") || kind.equals("android.widget.Switch")
                || node.isSelected();
    }

    /** Whether this view, or anything a reader can reach under it, carries words. */
    private static boolean anyWords(View view) {
        if (!onScreen(view)
                || view.getImportantForAccessibility() == View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS) {
            return false;
        }
        AccessibilityNodeInfo node = view.createAccessibilityNodeInfo();
        if (node != null && !words(node).isEmpty()) {
            return true;
        }
        if (view instanceof ViewGroup) {
            ViewGroup group = (ViewGroup) view;
            for (int at = 0; at < group.getChildCount(); at++) {
                if (anyWords(group.getChildAt(at))) {
                    return true;
                }
            }
        }
        return false;
    }

    /** A description wins over drawn text, which is the order a reader takes them in. */
    private static String words(AccessibilityNodeInfo node) {
        String described = say(node.getContentDescription());
        return described.isEmpty() ? say(node.getText()) : described;
    }

    private static String say(CharSequence value) {
        return value == null ? "" : value.toString();
    }

    /**
     * The real words of the interface, from {@code strings/en.json}.
     *
     * Not a map written in the test: a test that invents its own words proves the screen can read a
     * map and proves nothing about whether the key it asks for exists. These read the file the
     * server ships, so a key spelled wrong here answers empty and the assertion fails.
     */
    public static Words words() {
        Words words = new Words();
        words.read(said());
        return words;
    }

    /** The same, as the map a surface is handed in {@code Seen.words}. */
    public static Map<String, String> said() {
        Object parsed = Contract.Json.parse(text("strings/en.json"));
        Map<String, String> said = new HashMap<String, String>();
        for (Map.Entry<?, ?> entry : ((Map<?, ?>) parsed).entrySet()) {
            if (entry.getValue() instanceof String) {
                said.put((String) entry.getKey(), (String) entry.getValue());
            }
        }
        return said;
    }

    /** Where the repository is, found by walking up: a test's working folder is not promised. */
    public static File root() {
        File here = new File("").getAbsoluteFile();
        while (here != null && !new File(here, "strings/en.json").isFile()) {
            here = here.getParentFile();
        }
        assertTrue("no strings/en.json above " + new File("").getAbsolutePath(), here != null);
        return here;
    }

    /** One of this project's files, as text. */
    public static String text(String path) {
        File file = new File(root(), path);
        assertTrue(path + " has gone", file.isFile());
        try {
            return new String(Files.readAllBytes(file.toPath()), StandardCharsets.UTF_8);
        } catch (IOException broken) {
            throw new IllegalStateException("could not read " + path, broken);
        }
    }
}
