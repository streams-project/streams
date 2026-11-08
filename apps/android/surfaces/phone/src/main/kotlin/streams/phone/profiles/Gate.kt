package streams.phone.profiles

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.ActionEnterProfile
import contract.Hello
import contract.Profile
import contract.contractJson
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.read

private const val DEFAULT_PROFILES_MAX = 5

/** The profile chooser screen with manage mode, edit, add, and PIN flows. */
@Composable
fun Gate(shell: Shell, seen: Seen) {
    var manageMode by remember { mutableStateOf(false) }
    var editingProfile by remember { mutableStateOf<Profile?>(null) }
    var addingProfile by remember { mutableStateOf(false) }
    var pinProfile by remember { mutableStateOf<Profile?>(null) }

    val hello by produceState<Hello?>(null) {
        value = shell.read("/v1/hello")?.let { contractJson.decodeFromString(Hello.serializer(), it) }
    }
    val profilesMax = hello?.limits?.profilesMax?.toInt() ?: DEFAULT_PROFILES_MAX
    val words = seen.words

    BackHandler(enabled = pinProfile != null || editingProfile != null || addingProfile || manageMode) {
        when {
            pinProfile != null -> pinProfile = null
            editingProfile != null -> editingProfile = null
            addingProfile -> addingProfile = false
            manageMode -> manageMode = false
        }
    }

    when {
        editingProfile != null -> {
            EditProfile(
                shell = shell,
                seen = seen,
                start = editingProfile!!,
                leave = { editingProfile = null },
            )
        }
        addingProfile -> {
            AddProfile(
                shell = shell,
                seen = seen,
                leave = { addingProfile = false },
            )
        }
        else -> {
            Box(Modifier.fillMaxSize().background(Back)) {
                Column(
                    Modifier
                        .fillMaxSize()
                        .statusBarsPadding()
                        .navigationBarsPadding()
                        .verticalScroll(rememberScrollState()),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Spacer(Modifier.height(60.dp))

                    val headerKey = if (manageMode) "profiles.manage" else "profiles.choose.phone"
                    val fallbackKey = if (manageMode) "profiles.manage" else "profiles.choose"
                    Text(
                        words[headerKey] ?: words[fallbackKey].orEmpty(),
                        color = if (manageMode) Color.White else Color.White.copy(alpha = 0.7f),
                        fontSize = if (manageMode) 22.sp else 14.sp,
                        fontWeight = if (manageMode) FontWeight.Bold else FontWeight.Normal,
                        textAlign = TextAlign.Center,
                        modifier = Modifier.fillMaxWidth().padding(horizontal = 24.dp),
                    )

                    Spacer(Modifier.height(48.dp))

                    GateGrid(
                        seen = seen,
                        profiles = seen.profiles,
                        manageMode = manageMode,
                        profilesMax = profilesMax,
                        onEnterManage = { manageMode = true },
                        onExitManage = { manageMode = false },
                        onEditProfile = { person -> editingProfile = person },
                        onAddProfile = { addingProfile = true },
                        onPickPin = { person -> pinProfile = person },
                        onEnterProfile = { person ->
                            shell.speak(person.language)
                            shell.act(ActionEnterProfile(profile = person.id))
                        },
                    )

                    Spacer(Modifier.height(48.dp))
                }

                if (pinProfile != null) {
                    PinSheet(
                        shell = shell,
                        seen = seen,
                        person = pinProfile!!,
                        onDismiss = { pinProfile = null },
                    )
                }
            }
        }
    }
}
