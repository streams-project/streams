package streams.tv

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.text.font.FontWeight
import coil3.compose.AsyncImage
import contract.ActionEnterProfile
import contract.Profile
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.ref
import streams.shared.refType

/** Who is watching: a column of profiles on the left, and a title chosen for the one in focus. */
@Composable
fun Profiles(shell: Shell, seen: Seen) {
    val list = seen.profiles
    var at by remember { mutableIntStateOf(0) }
    var pencil by remember { mutableStateOf(false) }
    var locked by remember { mutableStateOf<Profile?>(null) }
    var editing by remember { mutableStateOf<Profile?>(null) }
    editing?.let { person ->
        EditProfile(shell, seen, person) {
            editing = null
            pencil = false
        }
        return
    }
    locked?.let { person ->
        PinPanel(shell, person, seen.server) { locked = null }
        return
    }
    val person = list.getOrNull(at) ?: return

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> if (at > 0) at -= 1 else return false
            Word.DOWN -> if (at < list.lastIndex) at += 1 else return false
            Word.LEFT -> if (person.hasPin) return false else pencil = true
            Word.RIGHT -> pencil = false
            Word.OK -> when {
                pencil -> editing = person
                person.hasPin -> locked = person
                else -> {
                    shell.speak(person.language)
                    shell.act(ActionEnterProfile(profile = person.id))
                }
            }
            Word.BACK -> return false
        }
        return true
    }

    Box(Modifier.fillMaxSize().remote(@DontMemoize { press(it) })) {
        ProfileFeature(shell, person.id)
        Column(Modifier.offset(190.ref, 44.ref).width(200.ref), horizontalAlignment = Alignment.CenterHorizontally) {
            AsyncImage("${seen.server}/mark.svg", null, Modifier.size(68.ref))
            Text(
                seen.words["profiles.choose"].orEmpty(),
                Modifier.padding(top = 8.ref),
                fontSize = 24.refType,
                fontWeight = FontWeight.Medium,
                softWrap = false,
            )
        }
        Column(Modifier.offset(0.ref, 168.ref).width(720.ref), verticalArrangement = Arrangement.spacedBy(26.ref)) {
            list.forEachIndexed { index, profile ->
                ProfileRow(profile, index == at, pencil && index == at, seen.server)
            }
        }
    }
}
