package streams.phone

import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsNode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull

/**
 * One thing a screen reader would stop on in a composed tree: the words it reads, what it reads
 * after them, what kind of thing the node says it is, and how deep it sits.
 *
 * The Compose twin of `surfaces/tv/src/test/java/streams/tv/views/Heard.java`, and the thing
 * `shared/src/test/kotlin/streams/shared/Heard.kt` says it cannot do. That file reads what a
 * `Modifier` chain declares without composing anything, and is honest that it proves nothing about
 * the tree: not whether a screen puts the right helper on the right box, not the order a reader
 * reaches things in, and not what `mergeDescendants` actually gathers. This walks the merged tree
 * the toolkit builds, which is the tree a reader is handed.
 *
 * `androidx.compose.ui:ui-test` resolved on 2026-11-13. It had been recorded as absent, along with
 * `org.robolectric`, in a sentence that also said gradle runs `--offline` here. No such setting
 * exists; see note 228.
 *
 * Read the **merged** tree on purpose. A node that gathers its descendants is one stop for a
 * reader however many boxes are drawn inside it, and a walk of the unmerged tree would report the
 * boxes -- which would pass a test named for what a reader hears while describing what it does not.
 */
internal data class HeardTree(
    /** The words read out. Never empty: a node with no words is not collected. */
    val label: String,
    /** What is read after the label: "on", "in my list". Empty when the node says no state. */
    val state: String,
    /** What the node says it is. Null means it does not say. */
    val role: Role?,
    /** Whether the node says it is the chosen one. Null means it never says either way. */
    val selected: Boolean?,
    /**
     * How deep under the root it sits in the **semantics** tree, which is not how deep the
     * composable sits. A `Column` declares no semantics, so two boxes nested one inside another's
     * column both land at depth 1. Measured 2026-11-13. Carried because a genuinely nested
     * semantics node does differ, and left unasserted where Compose flattens.
     */
    val depth: Int,
) {
    override fun toString() =
        "depth $depth ${role ?: "-"} \"$label\"" +
            (if (state.isEmpty()) "" else " ($state)") +
            (if (selected == true) " [on]" else "")

    companion object {
        /** Everything a reader would stop on under [root], in the order it would reach them. */
        fun under(root: SemanticsNode): List<HeardTree> {
            val found = mutableListOf<HeardTree>()
            walk(root, 0, found)
            return found
        }

        private fun walk(node: SemanticsNode, depth: Int, found: MutableList<HeardTree>) {
            val said = node.config.getOrNull(SemanticsProperties.ContentDescription)?.firstOrNull()
            // Every gathered word, joined, and not the first one. A node that gathers its
            // descendants carries a *list* here -- one entry per `Text` drawn inside it -- and a
            // walker that read `.first()` would report "Continue" for a tile a reader hears as
            // "Continue Watching". Found on 2026-11-13 by a test that expected both words.
            val drawn = node.config.getOrNull(SemanticsProperties.Text)
                ?.joinToString(" ") { it.text }
                ?.takeIf { it.isNotBlank() }
            // A description wins over drawn words, which is the order a reader takes them in.
            val label = said ?: drawn
            if (!label.isNullOrEmpty()) {
                found += HeardTree(
                    label = label,
                    state = node.config.getOrNull(SemanticsProperties.StateDescription).orEmpty(),
                    role = node.config.getOrNull(SemanticsProperties.Role),
                    selected = node.config.getOrNull(SemanticsProperties.Selected),
                    depth = depth,
                )
            }
            node.children.forEach { walk(it, depth + 1, found) }
        }
    }
}
