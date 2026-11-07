package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import contract.ActionSignOut
import contract.Profile
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Plate
import streams.shared.ref
import streams.shared.refType

private enum class Pick { SWITCH, SETTINGS, HELP, SIGN_OUT }

/** The menu behind the avatar: who is watching, and the ways out of the page. */
@Composable
fun ProfileMenu(shell: Shell, seen: Seen, person: Profile?, leave: () -> Unit) {
    val words = seen.words
    var at by remember { mutableIntStateOf(0) }
    var asking by remember { mutableStateOf(false) }
    val picks = buildList {
        add(Pick.SWITCH)
        if (person?.kids != true) add(Pick.SETTINGS)
        add(Pick.HELP)
        add(Pick.SIGN_OUT)
    }
    val lines = if (asking) {
        listOf(MenuLine(words["menu.sign_out"].orEmpty()), MenuLine(words["common.cancel"].orEmpty()))
    } else {
        picks.map { pick ->
            val key = when (pick) {
                Pick.SWITCH -> "menu.switch"
                Pick.SETTINGS -> "menu.settings"
                Pick.HELP -> "menu.help"
                Pick.SIGN_OUT -> "menu.sign_out"
            }
            MenuLine(words[key].orEmpty())
        }
    }

    fun stay() {
        asking = false
        at = picks.indexOf(Pick.SIGN_OUT).coerceAtLeast(0)
    }

    fun choose() {
        when {
            asking && at == 0 -> shell.act(ActionSignOut(everywhere = false))
            asking -> stay()
            picks[at] == Pick.SWITCH -> shell.go("/profiles")
            picks[at] == Pick.SETTINGS -> shell.go("/settings")
            picks[at] == Pick.HELP -> shell.go("/help")
            else -> {
                asking = true
                at = 1
            }
        }
    }

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> if (at > 0) at -= 1
            Word.DOWN -> if (at < lines.lastIndex) at += 1
            Word.OK -> choose()
            Word.BACK -> if (asking) stay() else leave()
            else -> Unit
        }
        return true
    }

    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.7f)).remote(@DontMemoize { press(it) })) {
        Column(
            Modifier
                .offset(60.ref, 120.ref)
                .width(480.ref)
                .background(Plate, RoundedCornerShape(24.ref))
                .padding(32.ref),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Face(person, 72, 12, seen.server, Modifier.size(72.ref))
                Spacer(Modifier.width(20.ref))
                Text(person?.name.orEmpty(), fontSize = 32.refType, fontWeight = FontWeight.Bold, maxLines = 1)
            }
            Spacer(Modifier.height(24.ref))
            if (asking) {
                Text(words["menu.sign_out.ask"].orEmpty(), fontSize = 30.refType)
                Spacer(Modifier.height(20.ref))
            }
            Menu(lines, at, width = 416)
        }
    }
}
