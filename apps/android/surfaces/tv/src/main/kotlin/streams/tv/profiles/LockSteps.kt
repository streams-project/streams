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
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import contract.Confirm
import contract.PinSet
import contract.Profile
import contract.ProfileList
import contract.contractJson
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Muted
import streams.shared.ref
import streams.shared.refType

private enum class LockStep { PASSWORD, FIRST, SECOND }

/** Step-up password check, followed by choosing and confirming a PIN (or clearing an existing one). */
@Composable
fun EditLock(
    shell: Shell,
    seen: Seen,
    person: Profile,
    pinDigits: Int,
    finish: (Profile?) -> Unit,
) {
    val words = seen.words
    val scope = rememberCoroutineScope()
    val keys = remember { KeyboardState("") }
    var step by remember { mutableStateOf(LockStep.PASSWORD) }
    var refused by remember { mutableStateOf(false) }
    var mismatch by remember { mutableStateOf(false) }
    var firstPin by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }

    fun savePin(pin: String?) {
        if (busy) return
        busy = true
        scope.launch {
            val body = contractJson.encodeToString(PinSet.serializer(), PinSet(pin = pin))
            val saved = shell.write("PUT", "/v1/profiles/${person.id}/pin", body)
            if (saved == null) {
                refused = true
                busy = false
            } else {
                val hasPin = pin != null
                val updated = shell.read("/v1/profiles")
                    ?.let { contractJson.decodeFromString(ProfileList.serializer(), it).profiles }
                    ?: seen.profiles.map { if (it.id == person.id) it.copy(hasPin = hasPin) else it }
                shell.keepAll(updated)
                finish(updated.firstOrNull { it.id == person.id } ?: person.copy(hasPin = hasPin))
            }
        }
    }

    fun confirmPassword(password: String) {
        if (busy) return
        busy = true
        scope.launch {
            val body = contractJson.encodeToString(Confirm.serializer(), Confirm(password = password))
            val answer = shell.write("POST", "/v1/auth/confirm", body)
            // The password is done with, whatever the answer was.
            keys.text = ""
            if (answer == null) {
                refused = true
                busy = false
            } else if (person.hasPin) {
                busy = false
                savePin(null)
            } else {
                refused = false
                busy = false
                step = LockStep.FIRST
            }
        }
    }

    when (step) {
        LockStep.PASSWORD -> {
            val cancel = words["common.cancel"].orEmpty()
            val next = words["common.next"].orEmpty()
            val titleKey = if (person.hasPin) "profile.lock.remove" else "profile.lock"
            Box(
                Modifier.fillMaxSize().remote(@DontMemoize { word ->
                    keys.press(word, cancel, next) { chosen ->
                        when {
                            chosen == null -> {
                                keys.text = ""
                                finish(null)
                            }
                            chosen.isEmpty() -> refused = true
                            else -> {
                                keys.text = ""
                                confirmPassword(chosen)
                            }
                        }
                    }
                }),
            ) {
                PageHead(words[titleKey].orEmpty(), words["profile.lock.password.help"].orEmpty())
                Keyboard(keys, cancel, next, Modifier.offset(SUB_EDGE.ref, 302.ref))
                ProfileCard(
                    person,
                    seen.server,
                    {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            CardName("•".repeat(keys.text.length))
                            Box(Modifier.padding(start = 3.ref).width(3.ref).height(44.ref).background(Color.White))
                        }
                    },
                    if (refused) words["error.sign_in"].orEmpty() else null,
                )
            }
        }
        LockStep.FIRST, LockStep.SECOND -> {
            val count = pinDigits.coerceAtLeast(1)
            val digits = remember(step, count) { mutableStateListOf(*Array(count) { 0 }) }
            var at by remember(step) { mutableIntStateOf(0) }
            val helpKey = if (step == LockStep.FIRST) "profile.lock.pin.help" else "profile.lock.confirm.help"
            Box(
                Modifier.fillMaxSize().remote(@DontMemoize { word ->
                    when (word) {
                        Word.LEFT -> if (at > 0) at -= 1
                        Word.RIGHT -> if (at < count - 1) at += 1
                        Word.UP -> {
                            digits[at] = (digits[at] + 1) % 10
                            mismatch = false
                        }
                        Word.DOWN -> {
                            digits[at] = (digits[at] + 9) % 10
                            mismatch = false
                        }
                        Word.OK -> {
                            val entered = digits.joinToString("")
                            if (step == LockStep.FIRST) {
                                firstPin = entered
                                mismatch = false
                                step = LockStep.SECOND
                            } else if (entered != firstPin) {
                                firstPin = ""
                                mismatch = true
                                step = LockStep.FIRST
                            } else {
                                savePin(entered)
                            }
                        }
                        Word.BACK -> finish(null)
                    }
                    true
                }),
            ) {
                PageHead(words["profile.lock"].orEmpty(), words[helpKey].orEmpty())
                Column(
                    Modifier.fillMaxSize().padding(top = 240.ref),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Face(person, FACE_LARGE, seen.server, Modifier.size(FACE_LARGE.ref))
                    Spacer(Modifier.height(24.ref))
                    Text(person.name, fontSize = 56.refType, fontWeight = FontWeight.ExtraBold)
                    Spacer(Modifier.height(40.ref))
                    PinBoxes(digits, at)
                    if (mismatch) {
                        Spacer(Modifier.height(28.ref))
                        Text(words["profile.lock.mismatch"].orEmpty(), color = Muted, fontSize = 30.refType)
                    }
                }
            }
        }
    }
}
