package streams.tv.views;

import android.view.View;
import android.view.ViewGroup;
import android.view.accessibility.AccessibilityNodeInfo;
import java.util.ArrayList;
import java.util.List;

/**
 * One thing a screen reader would stop on in a Views tree: the words it reads, what it reads after
 * them, what kind of thing the node says it is, and how deep in the screen it sits.
 *
 * <p>The Views answer to {@code apps/apple/Tests/Heard/Heard.swift}, and unlike
 * {@code surfaces/shared/src/test/kotlin/streams/shared/Heard.kt} it is not weaker than the Apple
 * one. That file reads the semantics a {@code Modifier} chain declares without composing anything,
 * and says so, because {@code androidx.compose.ui:ui-test} and {@code org.robolectric} were absent
 * from {@code ~/.gradle} on 2026-11-12. Robolectric was added on 2026-11-13 and this walks the real
 * {@link AccessibilityNodeInfo} tree the toolkit builds from an attached window, which is the thing
 * note 215 recorded as missing: without it a surface could drop any of its 93 accessibility call
 * sites and every check in the repository would stay green.
 *
 * <p><b>Two conditions this found the hard way, both of which make a reading empty rather than
 * wrong.</b> A detached view answers {@code null} to everything -- {@code createAccessibilityNodeInfo}
 * populates against a window, so a test that forgets {@code setContentView} reads nothing from a
 * correctly labelled view. And Robolectric cannot stand up this project's {@code compileSdk 37}:
 * at that level {@code AndroidTestEnvironment.setUpApplicationState} reaches
 * {@code ApplicationSharedMemory.create}, whose {@code FileDescriptor} reflection JDK 25 refuses,
 * and the run dies before any test body. Every test here therefore pins {@code @Config(sdk = 34)}.
 * What that costs is real and is named in note 228: behaviour the toolkit changed after API 34 is
 * not covered by these.
 */
final class Heard {
    /** The words read out. Empty means the node is there and has no words of its own. */
    final String label;
    /** What is read after the label: "on", "in my list". Empty when the node says no state. */
    final String state;
    /** What the node says it is, by the class name a reader maps to a kind. */
    final String kind;
    /** Whether the node says the remote is on it. */
    final boolean selected;
    /** How deep under the root it sits, so a test can say what order a reader reaches things in. */
    final int depth;

    private Heard(String label, String state, String kind, boolean selected, int depth) {
        this.label = label;
        this.state = state;
        this.kind = kind;
        this.selected = selected;
        this.depth = depth;
    }

    @Override
    public String toString() {
        return "depth " + depth + " " + kind + " \"" + label + "\""
                + (state.isEmpty() ? "" : " (" + state + ")") + (selected ? " [on]" : "");
    }

    /**
     * Everything a reader would stop on under {@code root}, in the order it would reach them.
     *
     * <p>A node is stopped on when it carries words of its own, the same rule the platform uses to
     * decide a view is worth announcing. A subtree marked {@code NO_HIDE_DESCENDANTS} is not
     * entered, because that is what the flag means and a walk that ignored it would report nodes no
     * reader can reach -- which is how a test comes to assert the opposite of its name.
     */
    static List<Heard> under(View root) {
        List<Heard> found = new ArrayList<>();
        walk(root, 0, found);
        return found;
    }

    private static void walk(View view, int depth, List<Heard> found) {
        if (view.getImportantForAccessibility() == View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS) {
            return;
        }
        AccessibilityNodeInfo node = view.createAccessibilityNodeInfo();
        if (node != null && view.getImportantForAccessibility() != View.IMPORTANT_FOR_ACCESSIBILITY_NO) {
            String label = text(node.getContentDescription(), node.getText());
            if (!label.isEmpty()) {
                found.add(new Heard(label, say(node.getStateDescription()), say(node.getClassName()),
                        node.isSelected(), depth));
            }
        }
        if (view instanceof ViewGroup group) {
            for (int i = 0; i < group.getChildCount(); i++) {
                walk(group.getChildAt(i), depth + 1, found);
            }
        }
    }

    /** A description wins over drawn text, which is the order a reader takes them in. */
    private static String text(CharSequence described, CharSequence drawn) {
        String first = say(described);
        return first.isEmpty() ? say(drawn) : first;
    }

    private static String say(CharSequence value) {
        return value == null ? "" : value.toString();
    }
}
