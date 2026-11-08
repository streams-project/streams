package streams.phone.profiles

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.AvatarSets
import contract.Hello
import contract.Profile
import contract.ProfileCreate
import contract.ProfileList
import contract.contractJson
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Accent
import streams.shared.Art
import streams.shared.Back
import streams.shared.BrandFont
import streams.shared.Corner
import streams.shared.Join
import streams.shared.Muted
import streams.shared.R
import streams.shared.Raised
import streams.shared.read
import streams.shared.write

private const val DEFAULT_NAME_MAX = 32

/** Add Profile page on mobile phones. */
@Composable
fun AddProfile(
    shell: Shell,
    seen: Seen,
    leave: () -> Unit,
) {
    var name by remember { mutableStateOf("") }
    var kids by remember { mutableStateOf(false) }
    var nameError by remember { mutableStateOf<String?>(null) }
    var saving by remember { mutableStateOf(false) }

    val scope = rememberCoroutineScope()
    val words = seen.words

    val hello by produceState<Hello?>(null) {
        value = shell.read("/v1/hello")?.let { contractJson.decodeFromString(Hello.serializer(), it) }
    }
    val nameMax = hello?.limits?.profileNameMax?.toInt() ?: DEFAULT_NAME_MAX

    val defaultAvatar by produceState<String?>(null) {
        val firstId = seen.profiles.firstOrNull()?.id
        if (firstId != null) {
            val sets = shell.read("/v1/profiles/$firstId/avatars")
                ?.let { contractJson.decodeFromString(AvatarSets.serializer(), it) }
            value = sets?.sets?.firstOrNull()?.pictures?.firstOrNull()?.path
        }
    }

    BackHandler { leave() }

    fun save() {
        val trimmed = name.trim().take(nameMax)
        if (trimmed.isEmpty()) {
            nameError = words["error.name"]
            return
        }
        if (saving) return
        saving = true
        scope.launch {
            val body = contractJson.encodeToString(
                ProfileCreate.serializer(),
                ProfileCreate(kids = kids, name = trimmed),
            )
            val answer = shell.write("POST", "/v1/profiles", body)
            if (answer != null) {
                val created = contractJson.decodeFromString(Profile.serializer(), answer)
                val listJson = shell.read("/v1/profiles")
                val updated = listJson
                    ?.let { contractJson.decodeFromString(ProfileList.serializer(), it).profiles }
                    ?: (seen.profiles + created)
                shell.keepAll(updated)
                leave()
            } else {
                saving = false
                nameError = words["error.try_again"]
            }
        }
    }

    Box(Modifier.fillMaxSize().background(Back)) {
        Column(
            Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .navigationBarsPadding()
                .verticalScroll(rememberScrollState()),
        ) {
            // Header: Cancel, Title, Save
            Row(
                Modifier
                    .fillMaxWidth()
                    .height(56.dp)
                    .padding(horizontal = 16.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Box(
                    Modifier
                        .size(48.dp)
                        .clickable(onClick = leave),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    Icon(
                        painterResource(R.drawable.icon_back),
                        words["common.cancel"],
                        Modifier.size(24.dp),
                        tint = Color.White,
                    )
                }

                Text(
                    words["profile.add"].orEmpty(),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                )

                Box(
                    Modifier
                        .size(width = 64.dp, height = 48.dp)
                        .clickable(enabled = !saving, onClick = ::save),
                    contentAlignment = Alignment.CenterEnd,
                ) {
                    Text(
                        words["common.done"].orEmpty(),
                        color = if (saving) Muted else Color.White,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }

            Spacer(Modifier.height(24.dp))

            // Centered avatar preview
            Box(
                Modifier
                    .align(Alignment.CenterHorizontally)
                    .size(100.dp)
                    .clip(RoundedCornerShape(Corner.CARD.dp))
                    .background(if (kids) Join else Accent),
                contentAlignment = Alignment.Center,
            ) {
                if (defaultAvatar != null) {
                    Art(seen.server + defaultAvatar, Modifier.fillMaxSize())
                } else {
                    Text(
                        if (name.isBlank()) "+" else name.take(1).uppercase(),
                        color = Color.White,
                        fontSize = 44.sp,
                        fontWeight = FontWeight.ExtraBold,
                    )
                }
            }

            Spacer(Modifier.height(32.dp))

            // Name field
            Column(Modifier.fillMaxWidth().padding(horizontal = 24.dp)) {
                Text(
                    words["profile.name"].orEmpty(),
                    color = Muted,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Medium,
                    modifier = Modifier.padding(bottom = 6.dp),
                )
                Box(
                    Modifier
                        .fillMaxWidth()
                        .background(Raised, RoundedCornerShape(Corner.FIELD.dp))
                        .padding(horizontal = 16.dp, vertical = 14.dp),
                ) {
                    BasicTextField(
                        value = name,
                        onValueChange = {
                            if (it.length <= nameMax) {
                                name = it
                                nameError = null
                            }
                        },
                        modifier = Modifier.fillMaxWidth(),
                        textStyle = TextStyle(
                            color = Color.White,
                            fontSize = 16.sp,
                            fontWeight = FontWeight.Normal,
                            fontFamily = BrandFont,
                        ),
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(
                            capitalization = KeyboardCapitalization.Words,
                            imeAction = ImeAction.Done,
                        ),
                        cursorBrush = SolidColor(Color.White),
                    )
                    if (name.isEmpty()) {
                        Text(
                            words["profile.name"].orEmpty(),
                            color = Muted,
                            fontSize = 16.sp,
                        )
                    }
                }
                if (nameError != null) {
                    Text(
                        nameError.orEmpty(),
                        color = Join,
                        fontSize = 12.sp,
                        modifier = Modifier.padding(top = 6.dp, start = 4.dp),
                    )
                }
            }

            Spacer(Modifier.height(24.dp))

            // Kids toggle switch row
            Row(
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Column(Modifier.weight(1f).padding(end = 16.dp)) {
                    Text(
                        words["profile.age.kids"].orEmpty(),
                        color = Color.White,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        words["profile.add.kids"].orEmpty(),
                        color = Muted,
                        fontSize = 13.sp,
                    )
                }
                Switch(
                    checked = kids,
                    onCheckedChange = { kids = it },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = Color.White,
                        checkedTrackColor = Accent,
                        uncheckedThumbColor = Muted,
                        uncheckedTrackColor = Raised,
                    ),
                )
            }
        }
    }
}
