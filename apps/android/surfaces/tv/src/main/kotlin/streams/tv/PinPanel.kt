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
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import contract.ActionEnterProfile
import contract.Profile
import streams.ports.Shell
import streams.shared.Back
import streams.shared.Raised
import streams.shared.ref
import streams.shared.refType

private const val DIGITS = 4

/** The PIN goes to the core and is never kept. */
@Composable
fun PinPanel(shell: Shell, person: Profile, server: String, leave: () -> Unit) {
    val digits = remember { mutableStateListOf(*Array(DIGITS) { 0 }) }
    var at by remember { mutableIntStateOf(0) }
    Column(
        Modifier.fillMaxSize().padding(top = 200.ref).remote { word ->
            when (word) {
                Word.LEFT -> if (at > 0) at -= 1
                Word.RIGHT -> if (at < DIGITS - 1) at += 1
                Word.UP -> digits[at] = (digits[at] + 1) % 10
                Word.DOWN -> digits[at] = (digits[at] + 9) % 10
                Word.OK -> shell.act(ActionEnterProfile(profile = person.id, pin = digits.joinToString("")))
                Word.BACK -> leave()
            }
            true
        },
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Face(person, FACE_LARGE, 18, server, Modifier.size(FACE_LARGE.ref))
        Spacer(Modifier.height(24.ref))
        Text(person.name, fontSize = 56.refType, fontWeight = FontWeight.ExtraBold)
        Spacer(Modifier.height(40.ref))
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
}
