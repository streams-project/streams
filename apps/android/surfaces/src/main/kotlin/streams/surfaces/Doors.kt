package streams.surfaces

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.ActionDeviceSignIn
import contract.ActionEnterProfile
import contract.ActionSignIn
import contract.Profile
import streams.ports.Seen
import streams.ports.Shell

/** Where this app finds its server. Everyone runs their own, so nothing is built in. */
@Composable
fun Server(shell: Shell) {
    var address by remember { mutableStateOf("http://") }
    Column(
        Modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("Your server", fontSize = 32.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(24.dp))
        OutlinedTextField(address, { address = it }, Modifier.fillMaxWidth(), label = { Text("Address") })
        Spacer(Modifier.height(24.dp))
        Button({ shell.connect(address) }) { Text("Connect") }
    }
}

@Composable
fun Door(shell: Shell, seen: Seen) {
    Column(
        Modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(seen.brand, fontSize = 56.sp, fontWeight = FontWeight.Black)
        Spacer(Modifier.height(32.dp))
        val first = remember { FocusRequester() }
        Button({ shell.go("/sign-in") }, Modifier.focusRequester(first)) { Text("Sign in") }
        LaunchedEffect(Unit) { first.requestFocus() }
    }
}

@Composable
fun SignIn(shell: Shell, seen: Seen) {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    Column(
        Modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.Center,
    ) {
        Text("Sign in", fontSize = 32.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(24.dp))
        OutlinedTextField(email, { email = it }, Modifier.fillMaxWidth(), label = { Text("Email") })
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            password, { password = it }, Modifier.fillMaxWidth(), label = { Text("Password") },
            visualTransformation = PasswordVisualTransformation(),
        )
        Spacer(Modifier.height(24.dp))
        Button({ shell.act(ActionSignIn(email = email, password = password)) }) { Text("Sign in") }
        seen.notice?.let {
            Spacer(Modifier.height(16.dp))
            Text(it, color = Muted)
        }
    }
}

/** A remote has no keyboard to speak of, so a television signs in with a code a phone approves. */
@Composable
fun Code(shell: Shell, seen: Seen) {
    LaunchedEffect(Unit) { shell.act(ActionDeviceSignIn(deviceName = "Android TV")) }
    Column(Modifier.fillMaxSize().padding(64.dp), verticalArrangement = Arrangement.Center) {
        Text("Sign in on your phone", fontSize = 40.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(24.dp))
        val device = seen.device
        if (device?.userCode == null) {
            Text("Getting a code...", color = Muted, fontSize = 24.sp)
        } else {
            Text("Go to", color = Muted, fontSize = 24.sp)
            Text(device.verificationUri.orEmpty(), fontSize = 28.sp, fontWeight = FontWeight.Bold)
            Spacer(Modifier.height(12.dp))
            Text("and enter", color = Muted, fontSize = 24.sp)
            Text(device.userCode.orEmpty(), fontSize = 72.sp, fontWeight = FontWeight.Black)
        }
    }
}

@Composable
fun People(shell: Shell, seen: Seen, tv: Boolean) {
    Column(
        Modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("Who's watching?", fontSize = if (tv) 44.sp else 30.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(32.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(24.dp)) {
            seen.profiles.forEachIndexed { index, person -> Face(shell, person, index == 0) }
        }
    }
}

@Composable
private fun Face(shell: Shell, person: Profile, first: Boolean) {
    val focus = remember { FocusRequester() }
    var on by remember { mutableStateOf(false) }
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .focusRequester(focus)
            .focusRing { on = it }
            .focusable()
            .clickable { shell.act(ActionEnterProfile(profile = person.id)) },
    ) {
        Text(
            person.name.take(1).uppercase(),
            Modifier
                .size(120.dp)
                .clip(RoundedCornerShape(20.dp))
                .background(if (on) Color.White else Accent)
                .padding(top = 30.dp),
            color = if (on) Color.Black else Color.White,
            textAlign = TextAlign.Center,
            fontSize = 44.sp,
            fontWeight = FontWeight.Black,
        )
        Spacer(Modifier.height(10.dp))
        Text(person.name)
    }
    if (first) LaunchedEffect(Unit) { focus.requestFocus() }
}
