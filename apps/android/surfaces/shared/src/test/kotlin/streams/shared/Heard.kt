package streams.shared

import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsConfiguration
import androidx.compose.ui.semantics.SemanticsModifier
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.SemanticsPropertyKey

/**
 * One thing a screen reader would be handed: the words it reads, what is read after them, what kind
 * of thing the node says it is, and whether the node gathers its children or clears them away.
 *
 * This is the Android answer to `apps/apple/Tests/Heard/Heard.swift`, and it is a weaker one. Heard
 * on Apple hosts a real view in a real window and walks the accessibility tree UIKit built. This
 * reads the semantics a `Modifier` chain declares, without composing anything, which is still
 * worth having: it says what each helper puts on the node it is applied to, cheaply, with no
 * toolkit runner.
 *
 * When this was written on 2026-11-12 it also said the tree could not be read here at all, because
 * `androidx.compose.ui:ui-test` and `org.robolectric` were absent from `~/.gradle` and gradle ran
 * `--offline`. There is no such setting, and both resolved on 2026-11-13 in one line each. The
 * tree is now read by `surfaces/tv/.../views/HeardTest.java` for Views and
 * `surfaces/phone/.../HeardTreeTest.kt` for Compose, each proved by mutating the helper it tests.
 * Note 228.
 *
 * So what these tests prove and what they do not:
 *  * proved -- what each helper in `Says.kt` puts on the node it is applied to, read off the real
 *    `SemanticsConfiguration` the toolkit would build from it. Not a count, and not a copy of the
 *    helper's body: `AppendedSemanticsElement.semanticsConfiguration` runs the helper's own
 *    properties lambda into a fresh configuration, which is what the composition does.
 *  * not proved -- the tree. Whether a screen applies the right helper to the right box, what order
 *    a reader reaches things in, and what `mergeDescendants` actually gathers from the children all
 *    need a composed tree. `apps/android/surfaces/phone/src/test/.../SpokenTest.kt` covers the
 *    "right box" half by reading the sources, and the other two are covered by `HeardTest.java`
 *    for Views and `HeardTreeTest.kt` for Compose.
 */
internal data class Heard(
    /** The words read out. Empty means the node is there and has no words of its own. */
    val label: String,
    /** What is read after the label: "on", "in my list". Empty when the node has no state. */
    val state: String,
    /** What the node says it is. Null means it does not say. */
    val role: Role?,
    /** Whether the node says the remote is on it. Null means it never said either way. */
    val selected: Boolean?,
    /** `mergeDescendants`: the words drawn inside this node become its label. */
    val gathers: Boolean,
    /** `clearAndSetSemantics`: the node and everything under it go away. */
    val cleared: Boolean,
    /** Every semantics key the node sets, by name, so a test can say what was *not* set. */
    val keys: Set<String>,
) {
    val isButton: Boolean get() = role == Role.Button
    val isSwitch: Boolean get() = role == Role.Switch
    val silent: Boolean get() = label.isEmpty()
}

/**
 * Everything a reader would be handed by one modifier chain, one entry per semantics node in it.
 *
 * A chain with no semantics on it gives an empty list, which is the case worth having: a helper that
 * decided to do nothing is indistinguishable from a helper nobody called, and that is exactly what
 * `saysSwitch("")` does on purpose.
 */
internal fun heard(modifier: Modifier): List<Heard> =
    modifier.foldIn(mutableListOf<Heard>()) { found, element ->
        if (element is SemanticsModifier) found.add(read(element.semanticsConfiguration))
        found
    }

/**
 * The same chain read as the one node it is applied to.
 *
 * Every semantics modifier in a chain lands on the same layout node, so a reader meets their sum and
 * never the pieces. This sums them by hand -- later wins for a value, any wins for a flag -- rather
 * than through the toolkit's own `collapsePeer`, which is internal to `ui`. For the shapes these
 * helpers build (a value is set once, in one place) the two agree; a chain that set the same key
 * twice would be a question for a composed tree and not for this.
 */
internal fun one(modifier: Modifier): Heard {
    val nodes = heard(modifier)
    require(nodes.isNotEmpty()) { "nothing in this chain says anything" }
    return Heard(
        label = nodes.lastOrNull { it.label.isNotEmpty() }?.label.orEmpty(),
        state = nodes.lastOrNull { it.state.isNotEmpty() }?.state.orEmpty(),
        role = nodes.firstNotNullOfOrNull { it.role },
        selected = nodes.firstNotNullOfOrNull { it.selected },
        gathers = nodes.any { it.gathers },
        cleared = nodes.any { it.cleared },
        keys = nodes.flatMap { it.keys }.toSet(),
    )
}

private fun read(config: SemanticsConfiguration) = Heard(
    label = config.at(SemanticsProperties.ContentDescription)?.joinToString("\n").orEmpty(),
    state = config.at(SemanticsProperties.StateDescription).orEmpty(),
    role = config.at(SemanticsProperties.Role),
    selected = config.at(SemanticsProperties.Selected),
    gathers = config.isMergingSemanticsOfDescendants,
    cleared = config.isClearingSemantics,
    keys = config.map { it.key.name }.toSet(),
)

/** The value under a key, or null when the node never set it. The two are not the same thing. */
private fun <T> SemanticsConfiguration.at(key: SemanticsPropertyKey<T>): T? =
    if (contains(key)) get(key) else null
