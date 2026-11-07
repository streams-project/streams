package streams.tv

import androidx.compose.foundation.focusable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type

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

/** Gives a whole screen the remote. The screen keeps its own focus as plain state. */
@Composable
fun Modifier.remote(press: (Word) -> Boolean): Modifier {
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { focus.requestFocus() }
    return focusRequester(focus)
        .focusable()
        .onPreviewKeyEvent { event ->
            val word = wordOf(event)
            word != null && event.type == KeyEventType.KeyDown && press(word)
        }
}
