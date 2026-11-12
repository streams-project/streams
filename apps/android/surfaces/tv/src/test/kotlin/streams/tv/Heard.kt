package streams.tv

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
 * reads the semantics a `Modifier` chain declares, without composing anything, because the two
 * libraries that would compose a tree in a unit test are not here: `androidx.compose.ui:ui-test`
 * and `org.robolectric` are both absent from `~/.gradle/caches/modules-2/files-2.1` and gradle runs
 * `--offline` on this machine (verified 2026-11-12, `ls ~/.gradle/caches/modules-2/files-2.1`).
 *
 * A literal copy of `surfaces/shared/src/test/kotlin/streams/shared/Heard.kt` but for its package.
 * A test source set is not published to another module's, and this surface's helpers are its own
 * (`app/Spoken.kt`, five of them, not `shared/Says.kt`'s six), so the reader has to exist on both
 * sides. The two files are meant to stay identical; if one gains a property the other should.
 *
 * So what these tests prove and what they do not:
 *  * proved -- what each helper in `app/Spoken.kt` puts on the node it is applied to, read off the real
 *    `SemanticsConfiguration` the toolkit would build from it. Not a count, and not a copy of the
 *    helper's body: `AppendedSemanticsElement.semanticsConfiguration` runs the helper's own
 *    properties lambda into a fresh configuration, which is what the composition does.
 *  * not proved -- the tree. Whether a screen applies the right helper to the right box, what order
 *    a reader reaches things in, and what `mergeDescendants` actually gathers from the children all
 *    need a composed tree. `SpokenTest`'s census covers the "is it applied at all"
 *    half by reading the sources; nothing here covers the other two, and the accessibility rows of
 *    note 215 stay open until `ui-test` can be resolved.
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
