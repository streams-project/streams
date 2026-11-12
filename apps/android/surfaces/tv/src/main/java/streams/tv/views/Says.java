package streams.tv.views;

import android.os.Build;
import android.view.View;
import android.view.accessibility.AccessibilityNodeInfo;

/**
 * What a screen reader is handed, in Views.
 *
 * The twin of {@code shared/Says.kt}'s six helpers and {@code tv/app/Spoken.kt}'s five, which
 * together carry 97 production call sites across 27 files -- 71 on the phone and 26 on the
 * television, counted 2026-11-12 with
 * {@code rg -o "\.(says|saysItsWords|saysSwitch|decoration)\(" phone/src/main tv/src/main}. Not a parity question: our own capture says Netflix's
 * television app exposes one node for its whole interface, and a control a person cannot hear is a
 * defect whatever anyone else ships. The defect this answers was measured on 2026-11-11 before any
 * of it existed -- Home drew 61 nodes, 19 of them reachable, and all 19 carried neither text nor a
 * description ({@code Says.kt:16-21}).
 *
 * <h2>The eleven jobs, and which helper each one is</h2>
 * <table>
 * <tr><th>Kotlin</th><th>here</th><th>what it does</th></tr>
 * <tr><td>{@code Says.kt:35} saysItsWords()</td><td>{@link #saysItsWords()}</td>
 *     <td>the control's own drawn words are its label</td></tr>
 * <tr><td>{@code Says.kt:42} saysItsWords(state)</td><td>{@link #saysItsWords(String)}</td>
 *     <td>the same, and it stands one of two ways</td></tr>
 * <tr><td>{@code Says.kt:54} says(label)</td><td>{@link #says(String)}</td>
 *     <td>a control that draws no words: an icon, a tile</td></tr>
 * <tr><td>{@code Says.kt:68} says(label, state)</td><td>{@link #says(String, String)}</td>
 *     <td>the same, on or off</td></tr>
 * <tr><td>{@code Says.kt:77} saysSwitch(label)</td><td>{@link #saysSwitch(String)}</td>
 *     <td>a switch whose words are drawn beside it</td></tr>
 * <tr><td>{@code Says.kt:96} decoration()</td><td>{@link #decoration()}</td>
 *     <td>a picture that carries no meaning</td></tr>
 * <tr><td>{@code Spoken.kt:41} says(words)</td><td>{@link #means(String)}</td>
 *     <td>what a drawn part means, with no role claimed</td></tr>
 * <tr><td>{@code Spoken.kt:49} says(words, on)</td><td>{@link #stop(String, boolean)}</td>
 *     <td>a part the remote can stop on, and whether it is on it</td></tr>
 * <tr><td>{@code Spoken.kt:60} saysItsWords(on)</td><td>{@link #saysItsWords(boolean)}</td>
 *     <td>its own words, and whether the remote is on it</td></tr>
 * <tr><td>{@code Spoken.kt:72} decoration()</td><td>{@link #decoration()}</td><td>the same one</td></tr>
 * <tr><td>{@code Spoken.kt:79} sayBoth(a, b)</td><td>{@link #sayBoth(String, String)}</td>
 *     <td>two things read as one label</td></tr>
 * </table>
 * The two surfaces name two different helpers {@code says}: the phone's sets a role and gathers the
 * subtree, the television's sets a description and nothing else. They are kept apart here by name
 * instead of being merged, because merging them would change one surface's reading.
 *
 * <h2>Why a value and then an application</h2>
 * {@link Said} is what a part says; {@link #onto(View, Said)} puts it on a view. Split for the same
 * reason {@code Heard.kt} exists in the test source sets: the decisions -- an empty label is left
 * unset, an empty state is not a state, a decoration takes its children with it -- are the part that
 * can be wrong, and they can be read off a value without a device. What a real screen reader then
 * does with the node needs {@code uiautomator dump} on the Streamer, which is note 210's wave 2
 * check and is not claimed here.
 *
 * <h2>What Views gives for free, and where it needs help</h2>
 * {@code mergeDescendants = true} is already the behaviour of a Views leaf, so most call sites are
 * one description with nothing to merge. For a group of text it is
 * {@code setScreenReaderFocusable}, which is API 28, and {@code setStateDescription}, which is API
 * 30, against this app's minSdk of 26. Below each one this writes the same extra that
 * {@code androidx.core}'s compat class writes, so the behaviour is the one Compose already has on
 * those two releases and nothing is added to the module's dependencies. Verified by
 * {@code javap -c} on {@code androidx.core:core:1.18.0} from this machine's gradle cache on
 * 2026-11-12: {@code setStateDescription} writes
 * {@code androidx.view.accessibility.AccessibilityNodeInfoCompat.STATE_DESCRIPTION_KEY} as a
 * {@code CharSequence} below API 30, and {@code setScreenReaderFocusable} sets bit 1 of the int
 * extra {@code androidx.view.accessibility.AccessibilityNodeInfoCompat.BOOLEAN_PROPERTY_KEY} below
 * API 28.
 */
public final class Says {
    /** What a node says it is. {@code NONE} is a node that does not say, which is not {@code Button}. */
    public enum Role {
        NONE,
        BUTTON,
        SWITCH
    }

    private static final String STATE_KEY =
            "androidx.view.accessibility.AccessibilityNodeInfoCompat.STATE_DESCRIPTION_KEY";
    private static final String BOOLEAN_KEY =
            "androidx.view.accessibility.AccessibilityNodeInfoCompat.BOOLEAN_PROPERTY_KEY";
    private static final int SCREEN_READER_FOCUSABLE = 1;

    private Says() {
    }

    /**
     * One thing a screen reader would be handed. The same six readings as {@code Heard} in the test
     * source sets, so a test can say what was <em>not</em> set as easily as what was.
     */
    public static final class Said {
        /** The words read out. Empty means the node has no words of its own. */
        public final String label;

        /** What is read after the label. Empty means the node has no state. */
        public final String state;

        /** What the node says it is. */
        public final Role role;

        /** Whether the node says the remote is on it. Null means it never says either way. */
        public final Boolean selected;

        /** The words drawn inside this node become its label. */
        public final boolean gathers;

        /** This node and everything under it go away. */
        public final boolean cleared;

        /** Nothing is said at all, and a view is left exactly as it was. */
        public final boolean nothing;

        private Said(String label, String state, Role role, Boolean selected, boolean gathers,
                boolean cleared, boolean nothing) {
            this.label = label == null ? "" : label;
            this.state = state == null ? "" : state;
            this.role = role;
            this.selected = selected;
            this.gathers = gathers;
            this.cleared = cleared;
            this.nothing = nothing;
        }

        /** The same reading with a state after it, or the same reading when there is no state. */
        public Said and(String state) {
            if (state == null || state.isEmpty() || nothing) {
                return this;
            }
            return new Said(label, state, role, selected, gathers, cleared, false);
        }

        /** The same reading, saying whether the remote is on this part now. */
        public Said on(boolean on) {
            if (nothing) {
                return this;
            }
            return new Said(label, state, role, Boolean.valueOf(on), gathers, cleared, false);
        }

        /** True when a reader lands here and hears nothing: the case {@code Says.kt:46-52} is about. */
        public boolean silent() {
            return label.isEmpty();
        }
    }

    private static final Said NOTHING = new Said("", "", Role.NONE, null, false, false, true);

    /**
     * A control whose own words are drawn inside it. Everything under this node is read as one
     * thing, so the word beside the icon becomes the button's name.
     *
     * Used wherever a word is already on screen, so the label cannot drift from what the viewer
     * sees and needs no key of its own.
     */
    public static Said saysItsWords() {
        return new Said("", "", Role.BUTTON, null, true, false, false);
    }

    /** The same, for a control that also stands one of two ways and says so with something that is not words. */
    public static Said saysItsWords(String state) {
        return saysItsWords().and(state);
    }

    /** The television's: its own words, and whether the remote is on it now. {@code Spoken.kt:60}. */
    public static Said saysItsWords(boolean on) {
        return saysItsWords().on(on);
    }

    /**
     * A control with no words on it: an icon, a picture, a tile. The label is words a person hears,
     * so it comes from the words of the interface or from the title's own name, and is never an
     * English literal in the code.
     *
     * An empty label is left alone rather than set: a control described as nothing is worse than one
     * a reader falls through to its contents for, and the words are empty for as long as a run has
     * not read its language yet. {@code Says.kt:54-62}, exactly.
     */
    public static Said says(String label) {
        if (label == null || label.isEmpty()) {
            return saysItsWords();
        }
        return new Said(label, "", Role.BUTTON, null, true, false, false);
    }

    /** A control that is on or off, with the words for which of the two it is now read after the label. */
    public static Said says(String label, String state) {
        return says(label).and(state);
    }

    /**
     * A switch. The toolkit's own switch already says which way it stands and that it is a switch;
     * it does not say what it switches, because those words are drawn beside it as a row of text the
     * switch is no part of.
     *
     * Empty words say nothing at all -- not an empty label, nothing -- which is
     * {@code Says.kt:77-85} returning the chain unchanged.
     */
    public static Said saysSwitch(String label) {
        if (label == null || label.isEmpty()) {
            return NOTHING;
        }
        return new Said(label, "", Role.SWITCH, null, false, false, false);
    }

    /**
     * A picture that carries no meaning: the art inside a tile that already says the title's name, a
     * gradient, a plate, a mark beside the page's own name. Marking it is as much the work as
     * labelling a control -- an unmarked decoration is one more thing a reader stops on and reads out
     * as nothing.
     *
     * This takes the subtree with it, which is what {@code clearAndSetSemantics {}} does
     * ({@code Says.kt:87-96}): the node goes away instead of becoming a silent stop. Note 210 names
     * {@code IMPORTANT_FOR_ACCESSIBILITY_NO} for this job, and that constant is the weaker one -- it
     * hides the view and leaves its children reachable, so a decoration with anything drawn inside
     * it would still be a stop. {@code NO_HIDE_DESCENDANTS} is the twin.
     */
    public static Said decoration() {
        return new Said("", "", Role.NONE, null, false, true, false);
    }

    /**
     * What a drawn part means, said in the viewer's language, with no role claimed and nothing said
     * when there are no words for it. {@code Spoken.kt:41-42}.
     */
    public static Said means(String words) {
        if (words == null || words.isEmpty()) {
            return NOTHING;
        }
        return new Said(words, "", Role.NONE, null, false, false, false);
    }

    /**
     * A part the remote can stop on: what it means, and whether the remote is on it now.
     *
     * {@code on} is what a screen reader says as "selected", which is the only thing that tells a
     * person where they are on a screen whose focus is painted rather than held by the toolkit. It
     * is said either way, even with no words, because where the remote is matters more than what it
     * is on. {@code Spoken.kt:49-53}.
     */
    public static Said stop(String words, boolean on) {
        String said = words == null ? "" : words;
        return new Said(said, "", Role.BUTTON, Boolean.valueOf(on), false, false, false);
    }

    /**
     * Two things a person hears one after the other, as one label: a title and the row it sits in, a
     * name and its state.
     *
     * The break is a line end rather than a word or a comma, so no language's punctuation or word
     * order is decided in the code. {@code Spoken.kt:79-80}.
     */
    public static String sayBoth(String first, String second) {
        boolean one = first != null && !first.isEmpty();
        boolean two = second != null && !second.isEmpty();
        if (one && two) {
            return first + "\n" + second;
        }
        if (one) {
            return first;
        }
        return two ? second : "";
    }

    /** Puts a reading on a view. */
    public static void onto(View view, Said said) {
        if (said.nothing) {
            return;
        }
        if (said.cleared) {
            view.setAccessibilityDelegate(null);
            view.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS);
            view.setContentDescription(null);
            return;
        }
        view.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_YES);
        // Unset and not empty: the rule at `Says.kt:46-52`.
        view.setContentDescription(said.label.isEmpty() ? null : said.label);
        if (said.role == Role.NONE && said.selected == null && said.state.isEmpty() && !said.gathers) {
            view.setAccessibilityDelegate(null);
            return;
        }
        view.setAccessibilityDelegate(new Node(said));
    }

    /** What the rest of a reading needs a delegate for: the role, the state, selected, and gathering. */
    private static final class Node extends View.AccessibilityDelegate {
        private final Said said;

        Node(Said said) {
            this.said = said;
        }

        @Override
        public void onInitializeAccessibilityNodeInfo(View host, AccessibilityNodeInfo info) {
            super.onInitializeAccessibilityNodeInfo(host, info);
            if (said.role == Role.BUTTON) {
                info.setClassName("android.widget.Button");
            } else if (said.role == Role.SWITCH) {
                info.setClassName("android.widget.Switch");
            }
            if (said.selected != null) {
                info.setSelected(said.selected.booleanValue());
            }
            if (!said.state.isEmpty()) {
                if (Build.VERSION.SDK_INT >= 30) {
                    info.setStateDescription(said.state);
                } else if (info.getExtras() != null) {
                    info.getExtras().putCharSequence(STATE_KEY, said.state);
                }
            }
            if (said.gathers) {
                if (Build.VERSION.SDK_INT >= 28) {
                    info.setScreenReaderFocusable(true);
                } else if (info.getExtras() != null) {
                    int properties = info.getExtras().getInt(BOOLEAN_KEY, 0) | SCREEN_READER_FOCUSABLE;
                    info.getExtras().putInt(BOOLEAN_KEY, properties);
                }
            }
        }
    }
}
