package streams.tv

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import contract.ActionCommand
import contract.CommandRemind
import contract.TitleDetail
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.tall
import streams.tv.reminder.Ask
import streams.tv.reminder.Coming
import streams.tv.reminder.ReminderView

/**
 * The reminder panel, over a title that has not arrived yet.
 *
 * ## What is in this file, and what is not
 * The panel is `reminder/ReminderView.java` and the question it asks is `reminder/Ask.java`: the
 * two stops, the key map, what a screen reader hears and the drawing. What is left here is the seam
 * `reminder/Coming.java` names in its own words -- `Shell.act` takes `contract.Action`, the Kotlin
 * arm of the generated types, and note 240 measured the two arms as two classes in one jar with no
 * cast between them, so the command cannot be sent from Java at all.
 *
 * ## It reads nothing
 * `Coming.read(String)` exists for a caller that has only a body. This one has the answer already:
 * the title page put `GET /v1/titles/{key}` into `seen.detail` before it drew the line that opens
 * this, so the panel asks the server for nothing on the way in. The same choice the Apple
 * television's `Coming(TitleDetail)` makes, for the same reason.
 *
 * ## Why it is a panel of the title page and not a screen of its own
 * `contract/src/address.rs` has no reminder address and `screen.rs` no reminder screen, so there is
 * no route to give it; adding one is eighteen generated languages and another owner's. It is the
 * same answer the Apple television reached, and it is the better one either way: the question is
 * about the title the viewer is looking at, and a route would let it be opened with no title behind
 * it.
 */
@Composable
fun ReminderPage(
    shell: Shell,
    seen: Seen,
    data: TitleDetail,
    reminded: Boolean,
    turned: (Boolean) -> Unit,
    close: () -> Unit,
) {
    // A title already here has no `available_from` and nothing to be reminded of. `Coming.known()`
    // answers false for it and the panel draws no question, which is the same condition the phone's
    // button hangs on (`phone/title/Title.kt:300-301`).
    val coming = Coming(
        data.title.value,
        data.name,
        tall(data.images).orEmpty(),
        data.availableFrom?.value.orEmpty(),
        reminded,
    )
    // The question keeps which line the remote is on across a turn, which is `Ask.arrived`'s whole
    // point: the answer that comes back from setting a reminder is this panel with the words on the
    // first line the other way, and a focus reset by it would move under the viewer's hand.
    val ask = remember(data.title) { Ask(coming) }
    val pictures = remember(shell) { ShellArt(shell) }

    Box(Modifier.fillMaxSize()) {
        AndroidView(
            factory = { context -> ReminderView(context, pictures) },
            modifier = Modifier.fillMaxSize(),
        ) { view ->
            view.mirror(seen.rtl)
            view.speak(seen.words)
            view.asks(object : ReminderView.Asked {
                override fun turn(title: Coming, on: Boolean) {
                    // The panel has already drawn it turned. What is left is the command, and
                    // telling the page behind, so the line that opened this panel says the same
                    // thing when the viewer comes back to it.
                    turned(on)
                    shell.act(ActionCommand(CommandRemind(on = on, title = data.title)))
                }

                override fun leave() = close()
            })
            view.show(ask, coming)
        }
    }
}
