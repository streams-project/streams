package streams.tv

import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.SemanticsNode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull

/**
 * One stop a screen reader is handed on a composed television screen: the words it reads, what it
 * reads after them, what the node says it is, and whether the toolkit can put focus on it.
 *
 * Why this exists beside `Heard.kt`. That file reads what a `Modifier` chain declares without
 * composing anything, and says so plainly -- it cannot prove that a screen put the right helper on
 * the right box, nor what a reader reaches, nor in what order. Its reason was that
 * `androidx.compose.ui:ui-test` could not be resolved here. **That sentence is wrong as of
 * 2026-11-13**: the phone module resolved `ui-test-junit4` the same day (note 228,
 * `phone/.../HeardTree.kt`), and the television module resolves it too now that its build asks for
 * it. `Heard.kt` is left where it is -- its tests are real -- with this beside it for the half it
 * cannot reach.
 *
 * Why it carries [focusable] and `HeardTree` does not. A phone can be explored by touch, so a
 * labelled control is findable even when focus never lands on it. A remote has nothing to explore
 * with: a screen with words on it and nothing focusable is still a dead end, because there is no
 * way to put the reader's cursor anywhere. Both halves are measured here, and both are asserted
 * separately, because on 2026-11-13 a Google TV Streamer showed a screen with **0 words and 0
 * focusable nodes** (note 239) and a test that checked only one of them would have called that
 * screen half-fixed.
 *
 * Read the **merged** tree, as `HeardTree` does and for its reason: a node that gathers its
 * descendants is one stop for a reader however many boxes are drawn inside it.
 */
internal data class HeardScreen(
    /** The words read out. Empty when the node is a stop with nothing to say, which is a defect. */
    val label: String,
    /** What is read after the label: "Paused". Empty when the node says no state. */
    val state: String,
    /** What the node says it is. Null means it does not say. */
    val role: Role?,
    /** Whether the node says the remote is on it. Null means it never says either way. */
    val selected: Boolean?,
    /** Whether the toolkit can put focus here: the thing a remote needs and a finger does not. */
    val focusable: Boolean,
) {
    override fun toString() =
        (if (focusable) "focusable " else "") + "${role ?: "-"} \"$label\"" +
            (if (state.isEmpty()) "" else " ($state)") +
            (if (selected == true) " [on]" else "")

    companion object {
        /**
         * Every stop under [root], in the order a reader would reach them.
         *
         * A node is a stop when it has words or when focus can land on it. The second half is the
         * one that matters here: an unnamed focusable node is exactly what the television player
         * was measured to be, and a walker that only collected labelled nodes would report it as
         * nothing at all rather than as a stop with no name.
         */
        fun under(root: SemanticsNode): List<HeardScreen> {
            val found = mutableListOf<HeardScreen>()
            walk(root, found)
            return found
        }

        /** The stops that have words. What a reader actually hears. */
        fun words(root: SemanticsNode): List<HeardScreen> = under(root).filter { it.label.isNotEmpty() }

        private fun walk(node: SemanticsNode, found: MutableList<HeardScreen>) {
            val said = node.config.getOrNull(SemanticsProperties.ContentDescription)?.firstOrNull()
            // Every gathered word, joined, and not the first one: a node that gathers its
            // descendants carries one entry per `Text` drawn inside it, and reading `.first()`
            // would report "Audio" for a chip a reader hears as "Audio English".
            val drawn = node.config.getOrNull(SemanticsProperties.Text)
                ?.joinToString(" ") { it.text }
                ?.takeIf { it.isNotBlank() }
            // A description wins over drawn words, which is the order a reader takes them in.
            val label = said ?: drawn
            // `Modifier.focusable()` declares `Focused` whether it holds focus or not, and the
            // toolkit adds a `RequestFocus` action to anything focus can be moved to. Either one
            // means a remote can get a reader's cursor here.
            val canFocus = node.config.contains(SemanticsProperties.Focused) ||
                node.config.contains(SemanticsActions.RequestFocus)
            if (!label.isNullOrEmpty() || canFocus) {
                found += HeardScreen(
                    label = label.orEmpty(),
                    state = node.config.getOrNull(SemanticsProperties.StateDescription).orEmpty(),
                    role = node.config.getOrNull(SemanticsProperties.Role),
                    selected = node.config.getOrNull(SemanticsProperties.Selected),
                    focusable = canFocus,
                )
            }
            node.children.forEach { walk(it, found) }
        }
    }
}
