package streams.tv

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import contract.AvatarSets
import contract.Language
import contract.LanguageList
import contract.Profile
import contract.ProfileLimits
import contract.ProfileUpdate
import contract.contractJson
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell

/** The pages of Edit Profile. */
enum class EditStep { MENU, NAME, ICON, LANGUAGE }

private fun rowsOf(sets: AvatarSets?, history: String, person: Profile): List<PictureRow> {
    val rows = sets?.sets.orEmpty().map { set -> PictureRow(set.name, set.pictures.map { it.path }) }
    val worn = person.avatar?.takeIf { it.startsWith("/") } ?: return rows
    return rows.take(1) + PictureRow(history, listOf(worn)) + rows.drop(1)
}

/** Edit Profile. It opens from the picker and from Settings, and takes the same profile either way. */
@Composable
fun EditProfile(shell: Shell, seen: Seen, start: Profile, leave: () -> Unit) {
    var person by remember { mutableStateOf(start) }
    var step by remember { mutableStateOf(EditStep.MENU) }
    var refused by remember { mutableStateOf(false) }
    var menuAt by remember { mutableIntStateOf(0) }
    val scope = rememberCoroutineScope()
    val words = seen.words
    val sets by produceState<AvatarSets?>(null, person.id) {
        value = shell.read("/v1/profiles/${person.id}/avatars")
            ?.let { contractJson.decodeFromString(AvatarSets.serializer(), it) }
    }
    val languages by produceState<List<Language>>(emptyList()) {
        value = shell.read("/v1/languages")
            ?.let { contractJson.decodeFromString(LanguageList.serializer(), it).languages }
            .orEmpty()
    }
    val limit by produceState("", person.id) {
        value = shell.read("/v1/profiles/${person.id}/limits")
            ?.let { contractJson.decodeFromString(ProfileLimits.serializer(), it).ageRating }
            .orEmpty()
    }
    LaunchedEffect(person.language) { shell.speak(person.language) }

    fun save(change: ProfileUpdate) {
        scope.launch {
            val body = contractJson.encodeToString(ProfileUpdate.serializer(), change)
            val answer = shell.write("PATCH", "/v1/profiles/${person.id}", body)
            refused = answer == null
            if (answer != null) {
                person = contractJson.decodeFromString(Profile.serializer(), answer)
                shell.keep(person)
                step = EditStep.MENU
            }
        }
    }

    val server = seen.server
    when (step) {
        EditStep.MENU -> EditMenu(
            words,
            person,
            limit,
            languages.firstOrNull { it.code == (person.language ?: seen.language) }?.name,
            sets?.sets.orEmpty().isNotEmpty(),
            server,
            menuAt,
            { menuAt = it },
            { step = it },
            leave,
        )
        EditStep.NAME -> EditName(words, person, server, refused) { name ->
            if (name == null) step = EditStep.MENU else save(ProfileUpdate(name = name))
        }
        EditStep.ICON -> EditIcon(words, person, rowsOf(sets, words["profile.icon.history"].orEmpty(), person), server) {
            if (it == null) step = EditStep.MENU else save(ProfileUpdate(avatar = it))
        }
        EditStep.LANGUAGE -> EditLanguage(words, languages, person.language ?: seen.language) { code ->
            if (code == null) step = EditStep.MENU else save(ProfileUpdate(language = code))
        }
    }
}
