package streams.shared

import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription

/**
 * What a screen reader is handed. Not a parity question: our own capture says Netflix's television
 * app exposes one node for its whole interface, and a control a person cannot hear is a defect
 * whatever anyone else ships.
 *
 * Measured on emulator-5554 on 2026-11-11 before any of this existed: Home drew 61 nodes, 19 of
 * them reachable, and **all 19 carried neither text nor a description**. Eleven nodes did carry
 * text -- "Play", "My List", "Home" -- but every one of them was a sibling of the control, not the
 * control. `Modifier.clickable` makes a node that answers a tap; it does not gather what is drawn
 * inside it. So a reader landing on the Play button said nothing, with the word "Play" sitting one
 * node away where nothing would ever focus it.
 *
 * Hence [saysItsWords], which is most of the work: the control keeps the words it already draws and
 * merely gathers them. [says] is for a control that draws no words at all, and its label is read
 * from `strings/` like every other word in this project -- never written here.
 */

/**
 * A control whose own words are drawn inside it. Everything under this node is read as one thing,
 * so the word beside the icon becomes the button's name.
 *
 * Use this and not [says] wherever a word is already on screen: the label then cannot drift from
 * what the viewer sees, and it needs no key of its own.
 */
fun Modifier.saysItsWords(): Modifier = semantics(mergeDescendants = true) { role = Role.Button }

/**
 * The same, for a control that also stands one of two ways and says so with something that is not
 * words -- a dot, a tick, a filled bar. [state] is those words, from `strings/`, read after the
 * control's own.
 */
fun Modifier.saysItsWords(state: String): Modifier = saysItsWords().then(
    if (state.isEmpty()) Modifier else Modifier.semantics { stateDescription = state },
)

/**
 * A control with no words on it: an icon, a picture, a tile. [label] is words a person hears, so it
 * comes from `strings/` or from the title's own name, and is never an English literal in the code.
 *
 * An empty label is left alone rather than set: a control described as "" is worse than one a reader
 * falls through to its contents for, and `words[...]` is empty for as long as a run has not read its
 * language yet.
 */
fun Modifier.says(label: String): Modifier =
    if (label.isEmpty()) {
        saysItsWords()
    } else {
        semantics(mergeDescendants = true) {
            role = Role.Button
            contentDescription = label
        }
    }

/**
 * A control that is on or off -- in My List or not, a reminder set or not. [state] is the words for
 * which of the two it is now, again from `strings/`, and is read after the label.
 */
fun Modifier.says(label: String, state: String): Modifier = says(label).then(
    if (state.isEmpty()) Modifier else Modifier.semantics { stateDescription = state },
)

/**
 * A switch. The toolkit's own `Switch` already says which way it stands and that it is a switch; it
 * does not say what it switches, because the words for that are drawn beside it as a row of text
 * the switch is no part of. [label] is those words, from `strings/`.
 */
fun Modifier.saysSwitch(label: String): Modifier =
    if (label.isEmpty()) {
        this
    } else {
        semantics {
            role = Role.Switch
            contentDescription = label
        }
    }

/**
 * A picture that carries no meaning: the art inside a tile that already says the title's name, a
 * gradient, a plate, a brand mark beside the page's own name. Marking it is as much the work as
 * labelling a control -- an unmarked decoration is one more thing a reader stops on and reads out
 * as nothing.
 *
 * This clears the subtree rather than setting an empty description, so the node goes away instead of
 * becoming a silent stop.
 */
fun Modifier.decoration(): Modifier = clearAndSetSemantics {}
