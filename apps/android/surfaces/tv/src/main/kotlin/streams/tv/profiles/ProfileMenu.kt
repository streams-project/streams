package streams.tv

import android.app.Activity
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import contract.ActionEnterProfile
import contract.Address
import contract.Profile
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Muted
import streams.shared.Plate
import streams.shared.R
import streams.shared.ref
import streams.shared.refType

/** The card's padding, and a line's padding before its face. */
private const val MENU_INSET = 12
private const val LINE_PAD = 16
private const val LINE_HIGH = 84
private const val MENU_FACE = 60

/**
 * The card opens from the face in the top bar: its first face is centred where that face is
 * (the bar's face is 48 at the page edge, centre 68), so the menu seems to grow out of it.
 */
private const val MENU_X = PAGE_EDGE + 24 - MENU_FACE / 2 - LINE_PAD - MENU_INSET
private const val MENU_Y = 68 - LINE_HIGH / 2 - MENU_INSET

private sealed interface Pick
private data object Switch : Pick
private data class Other(val person: Profile) : Pick
private data object Help : Pick
private data object Leave : Pick

/** One line of the card: a face or an icon, a word, and what it does. */
@Composable
private fun Line(on: Boolean, word: String, under: String? = null, lead: @Composable (Color) -> Unit) {
    val ink = if (on) Back else Color.White
    Row(
        Modifier
            .fillMaxWidth()
            .height(LINE_HIGH.ref)
            .background(if (on) Color.White else Color.Transparent, shapeOf(Round.inside(Round.panel, MENU_INSET)))
            .padding(horizontal = LINE_PAD.ref),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(MENU_FACE.ref), contentAlignment = Alignment.Center) { lead(ink) }
        Spacer(Modifier.width(20.ref))
        Column {
            Text(word, color = ink, fontSize = 28.refType, lineHeight = 34.refType, fontWeight = if (under != null) FontWeight.Bold else null, maxLines = 1)
            under?.let { Text(it, color = if (on) Back else Muted, fontSize = 22.refType, lineHeight = 28.refType, maxLines = 1) }
        }
    }
}

/** The card behind the face: who is watching, the other profiles, and the ways out. */
@Composable
fun ProfileMenu(shell: Shell, seen: Seen, person: Profile?, leave: () -> Unit) {
    val words = seen.words
    val context = LocalContext.current
    val others = seen.profiles.filter { it.id != person?.id }
    val picks = listOf(Switch) + others.map(::Other) + listOf(Help, Leave)
    var at by remember { mutableIntStateOf(0) }

    fun choose() {
        when (val pick = picks[at]) {
            Switch -> shell.go(Address.PROFILES)
            is Other -> if (pick.person.hasPin) {
                shell.go(Address.PROFILES)
            } else {
                shell.speak(pick.person.language)
                shell.act(ActionEnterProfile(profile = pick.person.id))
            }
            Help -> shell.go(Address.HELP)
            Leave -> (context as? Activity)?.finish()
        }
    }

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> if (at > 0) at -= 1
            Word.DOWN -> if (at < picks.lastIndex) at += 1
            Word.OK -> choose()
            // The card hangs from the face at the left, so Left closes it as Back does.
            Word.BACK, Word.LEFT -> leave()
            else -> Unit
        }
        return true
    }

    Box(Modifier.fillMaxSize().remote(@DontMemoize { press(it) })) {
        Column(
            Modifier
                .offset(MENU_X.ref, MENU_Y.ref)
                .width(470.ref)
                .background(Plate, shapeOf(Round.panel))
                .padding(MENU_INSET.ref),
        ) {
            picks.forEachIndexed { index, pick ->
                val on = index == at
                when (pick) {
                    Switch -> Line(on, person?.name.orEmpty(), words["menu.switch"].orEmpty()) {
                        Face(person, MENU_FACE, seen.server, Modifier.size(MENU_FACE.ref))
                    }
                    is Other -> Line(on, pick.person.name) {
                        Face(pick.person, MENU_FACE, seen.server, Modifier.size(MENU_FACE.ref))
                    }
                    Help -> Line(on, words["menu.help"].orEmpty()) { ink ->
                        Icon(painterResource(R.drawable.icon_question), null, Modifier.size(38.ref), tint = ink)
                    }
                    Leave -> Line(on, words["menu.exit"].orEmpty()) { ink ->
                        Icon(painterResource(R.drawable.icon_leave), null, Modifier.size(38.ref), tint = ink)
                    }
                }
            }
        }
    }
}
