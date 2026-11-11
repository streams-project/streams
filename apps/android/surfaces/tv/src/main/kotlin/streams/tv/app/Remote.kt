package streams.tv

import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.foundation.focusable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics

/** What a remote can say. A screen answers true when it used the word. */
enum class Word { UP, DOWN, LEFT, RIGHT, OK, BACK }

private fun wordOf(event: KeyEvent): Word? = when (event.key) {
    Key.DirectionUp -> Word.UP
    Key.DirectionDown -> Word.DOWN
    Key.DirectionLeft -> Word.LEFT
    Key.DirectionRight -> Word.RIGHT
    Key.DirectionCenter, Key.Enter, Key.NumPadEnter -> Word.OK
    Key.Back -> Word.BACK
    else -> null
}

/** Left and right swapped, for a right to left page. */
private fun turn(word: Word): Word = when (word) {
    Word.LEFT -> Word.RIGHT
    Word.RIGHT -> Word.LEFT
    else -> word
}

/** Gives a whole screen the remote. The screen keeps its own focus as plain state. */
@Composable
fun Modifier.remote(press: (Word) -> Boolean): Modifier = remote(true, "", press)

/** `on` goes false while something drawn over the screen holds the remote, and takes it back after. */
@Composable
fun Modifier.remote(on: Boolean, press: (Word) -> Boolean): Modifier = remote(on, "", press)

/** The remote, with what it is on now said aloud. */
@Composable
fun Modifier.remote(says: String, press: (Word) -> Boolean): Modifier = remote(true, says, press)

/**
 * The remote, and what a screen reader says.
 *
 * This box is the one thing on a television screen the toolkit really focuses: the screen paints
 * its own focus from plain state, so there is no per-control focus for a screen reader to follow
 * and nothing to announce when the viewer presses a key. `says` is what the remote is on now - a
 * tab's name, a title's name, a button's word - and it is a live region, so it is spoken again
 * each time it changes, which is each time the remote moves. Without it a person using a screen
 * reader hears one thing when a screen opens and nothing ever again.
 *
 * The parts themselves still carry their own words (`Spoken.kt`), so the tree can be read and
 * checked; this is what is heard without reading it.
 */
@Composable
fun Modifier.remote(on: Boolean, says: String, press: (Word) -> Boolean): Modifier {
    val focus = remember { FocusRequester() }
    // A key always reaches the newest answer, with what the screen holds now.
    val latest by rememberUpdatedState(press)
    // Under the opening a screen keeps its place but takes no keys, and takes them back after.
    val live = on && !LocalCovered.current
    // Pages say Left for towards the start of a line. In a right to left language the start is on
    // the right, so the remote's arrows swap here, once, for every page. The player keeps its own
    // left to right way, so its arrows stay as they are.
    val turned = LocalLayoutDirection.current == LayoutDirection.Rtl
    LaunchedEffect(live) {
        if (live) focus.requestFocus()
    }
    return focusRequester(focus)
        .focusable()
        .semantics {
            if (says.isNotEmpty()) {
                contentDescription = says
                liveRegion = LiveRegionMode.Polite
            }
        }
        .onPreviewKeyEvent { event ->
            val word = wordOf(event)?.let { if (turned) turn(it) else it }
            live && word != null && event.type == KeyEventType.KeyDown && latest(word)
        }
}
