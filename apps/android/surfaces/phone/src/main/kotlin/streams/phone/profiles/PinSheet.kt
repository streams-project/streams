package streams.phone.profiles

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.ActionEnterProfile
import contract.Hello
import contract.Profile
import contract.contractJson
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Corner
import streams.shared.Face
import streams.shared.Join
import streams.shared.Muted
import streams.shared.Plate
import streams.shared.Raised
import streams.shared.read

private const val DEFAULT_PIN_DIGITS = 4

/** Bottom sheet for entering a profile's PIN. */
@Composable
fun PinSheet(
    shell: Shell,
    seen: Seen,
    person: Profile,
    onDismiss: () -> Unit,
) {
    val hello by produceState<Hello?>(null) {
        value = shell.read("/v1/hello")?.let { contractJson.decodeFromString(Hello.serializer(), it) }
    }
    val count = hello?.limits?.pinDigits?.toInt() ?: DEFAULT_PIN_DIGITS
    var pin by remember { mutableStateOf("") }
    // How many notices had been said when this sheet submitted a PIN. The notice slot is app wide,
    // so a notice is this sheet's answer only once the count has moved past this.
    var noticesAtSubmit by remember { mutableStateOf<Int?>(null) }
    val focusRequester = remember { FocusRequester() }
    val words = seen.words

    LaunchedEffect(Unit) {
        focusRequester.requestFocus()
    }

    LaunchedEffect(pin) {
        if (pin.length == count) {
            noticesAtSubmit = seen.notices
            shell.speak(person.language)
            shell.act(ActionEnterProfile(profile = person.id, pin = pin))
            onDismiss()
        }
    }

    // Dismiss overlay on backdrop tap
    Box(
        Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.6f))
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
                onClick = onDismiss,
            ),
        contentAlignment = Alignment.BottomCenter,
    ) {
        Box(
            Modifier
                .fillMaxWidth()
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null,
                    onClick = {},
                )
                .background(
                    Plate,
                    RoundedCornerShape(topStart = Corner.PANEL.dp, topEnd = Corner.PANEL.dp),
                )
                .navigationBarsPadding()
                .padding(horizontal = 24.dp, vertical = 32.dp),
        ) {
            Column(
                Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Face(
                    person = person,
                    server = seen.server,
                    modifier = Modifier.size(64.dp),
                    shape = RoundedCornerShape(10.8.dp),
                    fontSize = 28.sp,
                )
                Spacer(Modifier.height(12.dp))
                Text(
                    person.name,
                    color = Color.White,
                    fontSize = 20.sp,
                    fontWeight = FontWeight.Bold,
                )
                Spacer(Modifier.height(8.dp))
                Text(
                    words["profile.pin.enter"].orEmpty(),
                    color = Muted,
                    fontSize = 14.sp,
                    textAlign = TextAlign.Center,
                )
                Spacer(Modifier.height(24.dp))

                // Digit display boxes
                val fieldShape = RoundedCornerShape(Corner.FIELD.dp)
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    for (i in 0 until count) {
                        val entered = i < pin.length
                        val isCurrent = i == pin.length
                        Box(
                            Modifier
                                .size(width = 52.dp, height = 56.dp)
                                .background(Raised, fieldShape)
                                .then(
                                    if (isCurrent) Modifier.border(2.dp, Color.White, fieldShape)
                                    else Modifier,
                                ),
                            contentAlignment = Alignment.Center,
                        ) {
                            if (entered) {
                                Text(
                                    "•",
                                    color = Color.White,
                                    fontSize = 28.sp,
                                    fontWeight = FontWeight.Bold,
                                )
                            }
                        }
                    }
                }

                // Hidden text field capturing input
                BasicTextField(
                    value = pin,
                    onValueChange = { input ->
                        val digits = input.filter { it.isDigit() }.take(count)
                        pin = digits
                    },
                    modifier = Modifier
                        .focusRequester(focusRequester)
                        .size(1.dp)
                        .alpha(0f),
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
                    cursorBrush = SolidColor(Color.Transparent),
                )

                val said = noticesAtSubmit
                if (said != null && seen.notices > said) {
                    Spacer(Modifier.height(16.dp))
                    Text(
                        words["error.pin_wrong"].orEmpty(),
                        color = Join,
                        fontSize = 14.sp,
                        textAlign = TextAlign.Center,
                    )
                }
            }
        }
    }
}
