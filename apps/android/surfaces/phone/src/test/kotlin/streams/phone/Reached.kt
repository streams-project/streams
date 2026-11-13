package streams.phone

import android.view.accessibility.AccessibilityManager
import androidx.compose.ui.platform.ViewRootForTest
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsNode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.core.view.ViewCompat
import org.robolectric.RuntimeEnvironment
import org.robolectric.Shadows

/**
 * The order a screen reader is walked through a screen, taken from the toolkit's own accessibility
 * delegate rather than from the shape of the semantics tree.
 *
 * Why this exists. [HeardTree] walks the merged tree and proves that a control carries words. Note
 * 228 built it and it is sound. On 2026-11-13 two screen readers on two phones, Google's and
 * Samsung's, showed that the words are not the half that was broken: the chip row, the row headers
 * and the tiles were **never reached** by the next-item gesture, which is the gesture every blind
 * Android user navigates with (note 239, section 3). A tree walk cannot see that, because the order
 * a reader is handed is not the tree's order.
 *
 * It is the delegate's: `AndroidComposeViewAccessibilityDelegateCompat` sorts every reachable node
 * by geometry and by `traversalIndex`, and writes the result into each node as
 * `AccessibilityNodeInfo.setTraversalBefore`, which is what TalkBack walks. The same sort fills the
 * delegate's own `idToBeforeMap`, and that map is what this reads. So this is not a model of the
 * device's order written here a second time -- it is the device's order, computed by the toolkit's
 * own code, in a unit test.
 *
 * Two things it still cannot see, and a device is needed for both: whether a screen reader's
 * **gesture** is recognised at all (note 239 watched an injected swipe activate the billboard
 * instead of moving focus, and flags that `input swipe` is not a finger), and what the reader's
 * voice actually says.
 *
 * The one reflective call is `getIdToBeforeMap$ui`, a method Kotlin's `internal` publishes into the
 * class file under that name. A toolkit upgrade that renames it fails this with a
 * `NoSuchMethodException` naming the method, which is a test breaking loudly rather than a test
 * quietly measuring nothing.
 */
internal data class Stop(
    /** The semantics node this stop is, so a failure can be traced back to a node in the tree. */
    val id: Int,
    /** The words read out at this stop. */
    val label: String,
    /** What is read after them. Empty when the node says no state. */
    val state: String,
    /** What the node says it is. */
    val role: Role?,
) {
    override fun toString() = "\"$label\"" + (if (state.isEmpty()) "" else " ($state)")
}

// Named for what it answers rather than for the reading, because `streams.phone.Reading` is
// already taken by the paging state in `browse/Paging.kt`: `private` keeps a Kotlin class out of
// other files, not out of the package, and two of them in one package is a class that shadows the
// other at run time. Found here on 2026-11-13 by a NoSuchMethodError naming the paging one.
internal object Reached {
    /**
     * Every stop a screen reader would make under [root], in the order it would make them.
     *
     * A node the delegate left out is a node a reader cannot reach from here: that is how an
     * off-screen tile, a node with no size and a node covered by another one drop out, and leaving
     * them out is the point rather than a loss.
     */
    fun of(root: SemanticsNode): List<Stop> {
        val owner = root.root as ViewRootForTest
        // Robolectric starts with the accessibility manager off. The delegate does not sort while
        // nothing is listening, and its event loop throws "Accessibility off" the moment it is
        // forced on without the manager agreeing, so both are set.
        val manager = RuntimeEnvironment.getApplication()
            .getSystemService(AccessibilityManager::class.java)
        Shadows.shadowOf(manager).setEnabled(true)
        Shadows.shadowOf(manager).setTouchExplorationEnabled(true)
        owner.forceAccessibilityForTesting(true)

        val view = owner.view
        val provider = requireNotNull(view.accessibilityNodeProvider) {
            "the toolkit handed no node provider, so nothing here is what a reader is handed"
        }
        val drawn = mutableMapOf<Int, Stop>()
        collect(root, drawn)
        // The sort runs while a node is built. Before that the delegate's map is empty, and a
        // reading taken from it would be an empty list reported as "nothing to reach".
        //
        // The answer is also the question this is asked for: the toolkit builds nothing for a node
        // it does not hand a reader, so a null here is a thing drawn on the screen that a reader
        // cannot reach. Three tiles of the second row answered null on Home on 2026-11-13, laid
        // out at no size because the row had not been measured yet, and an earlier reading here
        // listed them as stops -- a reachability test that counted the unreachable.
        val said = drawn.filterKeys { provider.createAccessibilityNodeInfo(it) != null }

        val delegate = requireNotNull(ViewCompat.getAccessibilityDelegate(view)) {
            "the toolkit's accessibility delegate is not on the view"
        }
        val map = delegate.javaClass.getMethod("getIdToBeforeMap\$ui").invoke(delegate)
        val lookUp = map.javaClass.getMethod(
            "getOrDefault",
            Int::class.javaPrimitiveType,
            Int::class.javaPrimitiveType,
        )
        fun after(id: Int): Int = lookUp.invoke(map, id, NONE) as Int

        val next = said.keys.associateWith { after(it) }.filterValues { it != NONE }
        val reached = next.values.toSet()
        val order = mutableListOf<Int>()
        val seen = mutableSetOf<Int>()
        // Several chains are possible -- a popup sorts apart from the page under it -- so every
        // head is followed, in the order the tree gives the heads.
        for (head in said.keys) {
            if (head in reached || head in seen) continue
            var at: Int? = head
            while (at != null && seen.add(at)) {
                order += at
                at = next[at]
            }
        }
        return order.mapNotNull { said[it] }
    }

    /** What every node of the merged tree says, by id, so an order can be read back as words. */
    private fun collect(node: SemanticsNode, into: MutableMap<Int, Stop>) {
        val description = node.config.getOrNull(SemanticsProperties.ContentDescription)?.firstOrNull()
        val drawn = node.config.getOrNull(SemanticsProperties.Text)
            ?.joinToString(" ") { it.text }
            ?.takeIf { it.isNotBlank() }
        val label = description ?: drawn
        if (!label.isNullOrEmpty()) {
            into[node.id] = Stop(
                id = node.id,
                label = label,
                state = node.config.getOrNull(SemanticsProperties.StateDescription).orEmpty(),
                role = node.config.getOrNull(SemanticsProperties.Role),
            )
        }
        node.children.forEach { collect(it, into) }
    }

    /** The answer for a node the delegate gave no place in the order. */
    private const val NONE = -1
}

/** The words of a reading, in order, for a message a person can read. */
internal fun List<Stop>.words(): String = joinToString(" | ") { it.label }

/** Where a reading first stops on something whose words hold [part], or -1. */
internal fun List<Stop>.firstWith(part: String): Int = indexOfFirst { it.label.contains(part) }

/**
 * Every control of a screen that answers a tap and says nothing, as `node id at bounds`.
 *
 * The class of defect this is for was found on a device, not here: note 239's `uiautomator` dump of
 * phone Home has one control that is "silent everywhere in its subtree", a twelve pixel wide
 * clickable sliver at the end of the chip row. A reader stops on it and reads out nothing. Neither
 * a label test nor a reading order can see it -- a label test asks the controls it knows the names
 * of, and an order lists what has words -- so it is asked here, of every control there is.
 */
internal fun silentControls(root: SemanticsNode): List<String> {
    val found = mutableListOf<String>()
    fun walk(node: SemanticsNode) {
        val taps = node.config.getOrNull(SemanticsActions.OnClick) != null
        val said = node.config.getOrNull(SemanticsProperties.ContentDescription)?.firstOrNull()
        val drawn = node.config.getOrNull(SemanticsProperties.Text)?.joinToString(" ") { it.text }
        if (taps && said.isNullOrBlank() && drawn.isNullOrBlank()) {
            found += "node ${node.id} at ${node.boundsInRoot}"
        }
        node.children.forEach { walk(it) }
    }
    walk(root)
    return found
}
