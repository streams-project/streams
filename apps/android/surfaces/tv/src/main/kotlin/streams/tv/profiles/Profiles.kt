package streams.tv

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import coil3.compose.AsyncImage
import contract.ActionEnterProfile
import contract.Hello
import contract.Profile
import contract.contractJson
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.ref
import streams.shared.refType

private const val DEFAULT_PROFILES_MAX = 5
private const val DEFAULT_NAME_MAX = 32
private const val DEFAULT_PIN_DIGITS = 4

/** Who is watching: a column of profiles on the left, and a title chosen for the one in focus. */
@Composable
fun Profiles(shell: Shell, seen: Seen, fromInside: Boolean = false) {
    val list = seen.profiles
    val hello by produceState<Hello?>(null) {
        value = shell.read("/v1/hello")?.let { contractJson.decodeFromString(Hello.serializer(), it) }
    }
    val profilesMax = hello?.limits?.profilesMax?.toInt() ?: DEFAULT_PROFILES_MAX
    val nameMax = hello?.limits?.profileNameMax?.toInt() ?: DEFAULT_NAME_MAX
    val pinDigits = hello?.limits?.pinDigits?.toInt() ?: DEFAULT_PIN_DIGITS
    val canAdd = list.size < profilesMax

    var at by remember { mutableIntStateOf(0) }
    var pencil by remember { mutableStateOf(false) }
    var locked by remember { mutableStateOf<Profile?>(null) }
    var editing by remember { mutableStateOf<Profile?>(null) }
    var adding by remember { mutableStateOf(false) }
    var focusProfileId by remember { mutableStateOf<String?>(null) }

    focusProfileId?.let { wantedId ->
        val found = list.indexOfFirst { it.id == wantedId }
        if (found >= 0) {
            at = found
            focusProfileId = null
        }
    }

    val maxIndex = if (canAdd) list.size else list.lastIndex
    if (maxIndex >= 0 && at > maxIndex) {
        at = maxIndex
    }

    if (adding) {
        AddProfile(shell, seen, nameMax) { createdId ->
            adding = false
            pencil = false
            if (createdId != null) {
                val idx = seen.profiles.indexOfFirst { it.id == createdId }
                if (idx >= 0) at = idx else focusProfileId = createdId
            }
        }
        return
    }
    editing?.let { person ->
        EditProfile(shell, seen, person, pinDigits) {
            editing = null
            pencil = false
        }
        return
    }
    locked?.let { person ->
        PinPanel(
            shell,
            seen,
            person,
            pinDigits,
            unlocked = { pin ->
                val toEdit = pencil
                locked = null
                if (toEdit) {
                    editing = person
                } else {
                    shell.speak(person.language)
                    shell.act(ActionEnterProfile(profile = person.id, pin = pin))
                }
            },
            leave = { locked = null },
        )
        return
    }
    val featurePerson = list.getOrNull(at) ?: list.lastOrNull() ?: return
    val person = list.getOrNull(at)

    fun press(word: Word): Boolean {
        when (word) {
            Word.UP -> if (at > 0) {
                at -= 1
                pencil = false
            } else {
                return false
            }
            Word.DOWN -> if (at < maxIndex) {
                at += 1
                pencil = false
            } else {
                return false
            }
            Word.LEFT -> if (person == null) return false else pencil = true
            Word.RIGHT -> pencil = false
            Word.OK -> when {
                person == null -> adding = true
                person.hasPin -> locked = person
                pencil -> editing = person
                else -> {
                    shell.speak(person.language)
                    shell.act(ActionEnterProfile(profile = person.id))
                }
            }
            // Opened with Switch Profiles, Back returns to the page. At the start of the app it stops
            // here and the app stays open: Netflix's own gate answered four Back presses without
            // leaving (netflix-reference/tv/00-netflix-on-the-tv-measured.md, section 7, n=4), and it
            // carries no way out of its own either - the way out is the television's own Home.
            Word.BACK -> if (fromInside) shell.back()
        }
        return true
    }

    Box(Modifier.fillMaxSize().remote(@DontMemoize { press(it) })) {
        ProfileFeature(shell, featurePerson.id)
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
        Column(
            Modifier.offset(0.ref, 168.ref).width(720.ref).wrapContentHeight(Alignment.Top, unbounded = true),
            verticalArrangement = Arrangement.spacedBy(26.ref),
        ) {
            list.forEachIndexed { index, profile ->
                ProfileRow(profile, index == at, pencil && index == at, seen.server)
            }
            if (canAdd) {
                AddProfileRow(seen.words["profile.add"].orEmpty(), at == list.size)
            }
        }
    }
}
