package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
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
import contract.Profile
import contract.ProfileEnter
import contract.contractJson
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Muted
import streams.shared.Raised
import streams.shared.ref
import streams.shared.refType

private const val DEFAULT_DIGITS = 4

/** A row of digit boxes for entering or choosing a PIN. */
@Composable
fun PinBoxes(digits: List<Int>, at: Int) {
    Row(horizontalArrangement = Arrangement.spacedBy(16.ref)) {
        digits.forEachIndexed { index, digit ->
            val on = index == at
            Box(
                Modifier.size(96.ref).background(if (on) Color.White else Raised, RoundedCornerShape(16.ref)),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    digit.toString(),
                    color = if (on) Back else Color.White,
                    fontSize = 48.refType,
                    fontWeight = FontWeight.Bold,
                )
            }
        }
    }
}

/** The PIN goes to the server and is never kept. */
@Composable
fun PinPanel(
    shell: Shell,
    seen: Seen,
    person: Profile,
    digitsCount: Int = DEFAULT_DIGITS,
    unlocked: (String) -> Unit,
    leave: () -> Unit,
) {
    val count = digitsCount.coerceAtLeast(1)
    val digits = remember(count) { mutableStateListOf(*Array(count) { 0 }) }
    var at by remember { mutableIntStateOf(0) }
    var refused by remember { mutableStateOf(false) }
    var busy by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    fun check() {
        if (busy) return
        busy = true
        val pin = digits.joinToString("")
        scope.launch {
            val body = contractJson.encodeToString(ProfileEnter.serializer(), ProfileEnter(pin = pin))
            val answer = shell.write("POST", "/v1/profiles/${person.id}/enter", body)
            busy = false
            if (answer == null) {
                refused = true
            } else {
                unlocked(pin)
            }
        }
    }

    Column(
        Modifier.fillMaxSize().padding(top = 200.ref).remote(@DontMemoize { word ->
            when (word) {
                Word.LEFT -> if (at > 0) at -= 1
                Word.RIGHT -> if (at < count - 1) at += 1
                Word.UP -> {
                    digits[at] = (digits[at] + 1) % 10
                    refused = false
                }
                Word.DOWN -> {
                    digits[at] = (digits[at] + 9) % 10
                    refused = false
                }
                Word.OK -> check()
                Word.BACK -> leave()
            }
            true
        }),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Face(person, FACE_LARGE, 18, seen.server, Modifier.size(FACE_LARGE.ref))
        Spacer(Modifier.height(24.ref))
        Text(person.name, fontSize = 56.refType, fontWeight = FontWeight.ExtraBold)
        Spacer(Modifier.height(40.ref))
        PinBoxes(digits, at)
        if (refused) {
            Spacer(Modifier.height(28.ref))
            Text(seen.words["error.pin_wrong"].orEmpty(), color = Muted, fontSize = 30.refType)
        }
    }
}
