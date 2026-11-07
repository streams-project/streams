package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import contract.ActionDeviceSignIn
import contract.ActionSignIn
import contract.DeviceSignInState
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Accent
import streams.shared.Back
import streams.shared.Muted
import streams.shared.ref
import streams.shared.refType

private enum class Way { PHONE, REMOTE }
private enum class Ask { EMAIL, PASSWORD }

private val Wash = Brush.verticalGradient(0f to Accent.copy(alpha = 0.45f), 0.6f to Color.Black)

private fun Map<String, String>.say(key: String) = this[key].orEmpty()

@Composable
private fun Pills(words: Map<String, String>, way: Way, on: Boolean) {
    Row(
        Modifier
            .fillMaxWidth()
            .padding(top = 118.ref),
        horizontalArrangement = Arrangement.Center,
    ) {
        Row(Modifier.background(Color.White.copy(alpha = 0.16f), CircleShape).padding(4.ref)) {
            listOf(Way.PHONE to "signin.phone", Way.REMOTE to "signin.remote").forEach { (each, key) ->
                val here = each == way
                Box(
                    Modifier
                        .width(240.ref)
                        .height(54.ref)
                        .background(if (here && on) Color.White else if (here) Color.White.copy(alpha = 0.3f) else Color.Transparent, CircleShape),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(words.say(key), color = if (here && on) Back else Color.White, fontSize = 26.refType, fontWeight = FontWeight.Medium)
                }
            }
        }
    }
}

@Composable
private fun Step(number: Int, left: Int) {
    Box(
        Modifier.offset(left.ref, 214.ref).size(52.ref).background(Color.White.copy(alpha = 0.2f), CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Text(number.toString(), fontSize = 28.refType, fontWeight = FontWeight.Bold)
    }
}

/** The code a phone approves. The core keeps it alive, so this only draws it. */
@Composable
private fun UsePhone(seen: Seen) {
    val words = seen.words
    val device = seen.device
    Step(1, 170)
    Text(
        words.say("signin.step1").replace("{address}", device?.verificationUri.orEmpty()),
        Modifier.offset(240.ref, 214.ref).width(680.ref),
        fontSize = 32.refType,
        lineHeight = 42.refType,
    )
    device?.verificationUriComplete?.let { QrCode(it, Modifier.offset(240.ref, 410.ref).size(440.ref)) }
    Step(2, 1030)
    Text(
        words.say("signin.step2"),
        Modifier.offset(1100.ref, 214.ref).width(680.ref),
        fontSize = 32.refType,
        lineHeight = 42.refType,
    )
    Text(
        when {
            device?.state == DeviceSignInState.OFF -> words.say("signin.off")
            device?.userCode != null -> device.userCode.orEmpty()
            else -> words.say("signin.getting")
        },
        Modifier.offset(1100.ref, 410.ref).width(760.ref),
        fontSize = if (device?.userCode != null) 88.refType else 36.refType,
        fontWeight = FontWeight.ExtraBold,
        letterSpacing = if (device?.userCode != null) 3.sp else 0.sp,
    )
}

@Composable
private fun Field(label: String, value: String, hidden: Boolean) {
    Box(
        Modifier
            .offset(960.ref, 280.ref)
            .size(800.ref, 76.ref)
            .border(2.ref, Color.White.copy(alpha = 0.5f), RoundedCornerShape(12.ref))
            .padding(horizontal = 26.ref),
        contentAlignment = Alignment.CenterStart,
    ) {
        Text(
            if (hidden) "•".repeat(value.length) else value.ifEmpty { label },
            color = if (value.isEmpty()) Muted else Color.White,
            fontSize = 32.refType,
            maxLines = 1,
        )
    }
}

/** Choose how to sign in: a phone and a code, or the remote and a keyboard. */
@Composable
fun SignIn(shell: Shell, seen: Seen) {
    LaunchedEffect(Unit) { shell.act(ActionDeviceSignIn(deviceName = "Android TV")) }
    val words = seen.words
    var way by remember { mutableStateOf(Way.PHONE) }
    var onPills by remember { mutableStateOf(true) }
    var ask by remember { mutableStateOf(Ask.EMAIL) }
    var email by remember { mutableStateOf("") }
    val keys = remember { KeyboardState("") }
    val back = words.say("common.back")
    val next = words.say("common.next")

    fun finish(text: String?) {
        when {
            text == null && ask == Ask.PASSWORD -> {
                ask = Ask.EMAIL
                keys.text = email
            }
            text == null -> onPills = true
            ask == Ask.EMAIL -> {
                email = text.trim()
                ask = Ask.PASSWORD
                keys.text = ""
            }
            else -> {
                shell.act(ActionSignIn(email = email, password = text))
                keys.text = ""
            }
        }
    }

    fun press(word: Word): Boolean {
        if (!onPills) {
            if (!keys.press(word, back, next, ::finish)) onPills = true
            return true
        }
        when (word) {
            Word.LEFT -> way = Way.PHONE
            Word.RIGHT -> way = Way.REMOTE
            Word.DOWN, Word.OK -> if (way == Way.REMOTE) onPills = false
            Word.BACK -> return false
            Word.UP -> Unit
        }
        return true
    }

    Box(Modifier.fillMaxSize().background(Wash).remote(@DontMemoize { press(it) })) {
        AsyncImage("${seen.server}/mark.svg", null, Modifier.offset(1796.ref, 40.ref).size(64.ref))
        Text(
            words.say("signin.choose"),
            Modifier.fillMaxWidth().padding(top = 48.ref),
            textAlign = TextAlign.Center,
            fontSize = 44.refType,
            fontWeight = FontWeight.Bold,
        )
        Pills(words, way, onPills)
        if (way == Way.PHONE) {
            UsePhone(seen)
        } else {
            Keyboard(keys, back, next, Modifier.offset(160.ref, 250.ref))
            Field(
                words.say(if (ask == Ask.EMAIL) "signin.email" else "signin.password"),
                keys.text,
                ask == Ask.PASSWORD,
            )
            seen.notice?.let {
                Text(it, Modifier.offset(960.ref, 380.ref).width(800.ref), fontSize = 28.refType, color = Muted)
            }
        }
    }
}
