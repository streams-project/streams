package streams.phone

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedButton
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
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.ActionDeviceSignIn
import contract.ActionSignIn
import contract.Address
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Join
import streams.shared.Muted

@Composable
fun Door(shell: Shell, seen: Seen) {
    Column(
        Modifier
            .fillMaxSize()
            .statusBarsPadding()
            .navigationBarsPadding()
            .padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(seen.brand, fontSize = 56.sp, fontWeight = FontWeight.Black)
        Spacer(Modifier.height(32.dp))
        val first = remember { FocusRequester() }
        if (!seen.accountsRequired) {
            Button(
                onClick = { shell.go(Address.HOME) },
                modifier = Modifier.defaultMinSize(minHeight = 48.dp).focusRequester(first),
            ) {
                Text(seen.words["nav.home"].orEmpty())
            }
            Spacer(Modifier.height(12.dp))
            OutlinedButton(
                onClick = { shell.go(Address.SIGN_IN) },
                modifier = Modifier.defaultMinSize(minHeight = 48.dp),
            ) {
                Text(seen.words["door.sign_in"].orEmpty())
            }
        } else {
            Button(
                onClick = { shell.go(Address.SIGN_IN) },
                modifier = Modifier.defaultMinSize(minHeight = 48.dp).focusRequester(first),
            ) {
                Text(seen.words["door.sign_in"].orEmpty())
            }
        }
        LaunchedEffect(Unit) { first.requestFocus() }
    }
}

@Composable
fun SignIn(shell: Shell, seen: Seen) {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    Column(
        Modifier
            .fillMaxSize()
            .statusBarsPadding()
            .imePadding()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 32.dp, vertical = 48.dp),
    ) {
        Text(seen.words["door.sign_in"].orEmpty(), fontSize = 32.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(24.dp))
        OutlinedTextField(
            email,
            { email = it },
            Modifier.fillMaxWidth(),
            label = { Text(seen.words["signin.email"].orEmpty()) },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email, imeAction = ImeAction.Next),
        )
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            password,
            { password = it },
            Modifier.fillMaxWidth(),
            label = { Text(seen.words["signin.password"].orEmpty()) },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password, imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { shell.act(ActionSignIn(email = email, password = password)) }),
        )
        Spacer(Modifier.height(24.dp))
        Button(
            onClick = { shell.act(ActionSignIn(email = email, password = password)) },
            modifier = Modifier.defaultMinSize(minHeight = 48.dp),
        ) {
            Text(seen.words["door.sign_in"].orEmpty())
        }
        seen.notice?.let {
            Spacer(Modifier.height(16.dp))
            Text(it, color = Join)
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
