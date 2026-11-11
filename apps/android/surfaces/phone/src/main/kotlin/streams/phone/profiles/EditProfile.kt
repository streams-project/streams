package streams.phone.profiles

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import contract.AvatarSets
import contract.Confirm
import contract.Hello
import contract.Language
import contract.LanguageList
import contract.PinSet
import contract.Profile
import contract.ProfileList
import contract.ProfileUpdate
import contract.contractJson
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Accent
import streams.shared.Art
import streams.shared.Back
import streams.shared.BrandFont
import streams.shared.Corner
import streams.shared.Face
import streams.shared.Join
import streams.shared.Muted
import streams.shared.Plate
import streams.shared.R
import streams.shared.Raised
import streams.shared.read
import streams.shared.write

private const val DEFAULT_NAME_MAX = 32
private const val DEFAULT_PIN_DIGITS = 4

/** The avatar tiles of the picture chooser, which is the size their picture is asked for. */
private val AvatarBox = 72.dp

private enum class LockStep { PASSWORD, FIRST, SECOND }

/** Edit Profile page on mobile phones. */
@Composable
fun EditProfile(
    shell: Shell,
    seen: Seen,
    start: Profile,
    leave: () -> Unit,
) {
    var person by remember { mutableStateOf(start) }
    var name by remember { mutableStateOf(start.name) }
    var avatar by remember { mutableStateOf(start.avatar) }
    var kids by remember { mutableStateOf(start.kids) }
    var language by remember { mutableStateOf(start.language ?: seen.language) }
    var hasPin by remember { mutableStateOf(start.hasPin) }

    var showAvatarPicker by remember { mutableStateOf(false) }
    var showLanguagePicker by remember { mutableStateOf(false) }
    var showLockFlow by remember { mutableStateOf(false) }
    var showDeleteConfirm by remember { mutableStateOf(false) }
    var nameError by remember { mutableStateOf<String?>(null) }
    var saving by remember { mutableStateOf(false) }

    val scope = rememberCoroutineScope()
    val words = seen.words

    val hello by produceState<Hello?>(null) {
        value = shell.read("/v1/hello")?.let { contractJson.decodeFromString(Hello.serializer(), it) }
    }
    val nameMax = hello?.limits?.profileNameMax?.toInt() ?: DEFAULT_NAME_MAX
    val pinDigits = hello?.limits?.pinDigits?.toInt() ?: DEFAULT_PIN_DIGITS

    val sets by produceState<AvatarSets?>(null, person.id) {
        value = shell.read("/v1/profiles/${person.id}/avatars")
            ?.let { contractJson.decodeFromString(AvatarSets.serializer(), it) }
    }

    val languages by produceState<List<Language>>(emptyList()) {
        value = shell.read("/v1/languages")
            ?.let { contractJson.decodeFromString(LanguageList.serializer(), it).languages }
            .orEmpty()
    }

    BackHandler {
        when {
            showAvatarPicker -> showAvatarPicker = false
            showLanguagePicker -> showLanguagePicker = false
            showLockFlow -> showLockFlow = false
            showDeleteConfirm -> showDeleteConfirm = false
            else -> leave()
        }
    }

    fun save() {
        val trimmed = name.trim().take(nameMax)
        if (trimmed.isEmpty()) {
            nameError = words["error.name"]
            return
        }
        if (saving) return
        saving = true
        scope.launch {
            val change = ProfileUpdate(
                name = trimmed,
                avatar = avatar,
                kids = kids,
                language = language,
            )
            val body = contractJson.encodeToString(ProfileUpdate.serializer(), change)
            val answer = shell.write("PATCH", "/v1/profiles/${person.id}", body)
            if (answer != null) {
                val updated = contractJson.decodeFromString(Profile.serializer(), answer)
                shell.keep(updated)
                shell.read("/v1/profiles")?.let { listJson ->
                    val list = contractJson.decodeFromString(ProfileList.serializer(), listJson).profiles
                    shell.keepAll(list)
                }
                leave()
            } else {
                saving = false
                nameError = words["error.try_again"]
            }
        }
    }

    fun deleteProfile() {
        scope.launch {
            if (shell.write("DELETE", "/v1/profiles/${person.id}", "") != null) {
                val updated = shell.read("/v1/profiles")
                    ?.let { contractJson.decodeFromString(ProfileList.serializer(), it).profiles }
                    ?: seen.profiles.filter { it.id != person.id }
                shell.keepAll(updated)
                leave()
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
            // Header: Cancel / Back, Title, Save
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
                    words["profiles.edit"] ?: words["profile.edit"].orEmpty(),
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

            Spacer(Modifier.height(16.dp))

            // Large centered avatar tile with pencil badge
            Box(
                Modifier
                    .align(Alignment.CenterHorizontally)
                    .size(100.dp)
                    .clickable { showAvatarPicker = true },
            ) {
                val previewProfile = person.copy(avatar = avatar, kids = kids, name = name)
                Face(
                    person = previewProfile,
                    server = seen.server,
                    modifier = Modifier.fillMaxSize(),
                    shape = RoundedCornerShape(Corner.CARD.dp),
                    fontSize = 44.sp,
                )

                // Pencil badge overlay
                Box(
                    Modifier
                        .align(Alignment.BottomEnd)
                        .offset(x = 4.dp, y = 4.dp)
                        .size(32.dp)
                        .background(Raised, CircleShape)
                        .border(2.dp, Back, CircleShape),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painterResource(R.drawable.icon_pencil),
                        words["profile.icon"],
                        Modifier.size(16.dp),
                        tint = Color.White,
                    )
                }
            }

            Spacer(Modifier.height(28.dp))

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
                        words["profile.age.kids.help"].orEmpty(),
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

            Spacer(Modifier.height(12.dp))

            // Display Language row
            val currentLangName = languages.firstOrNull { it.code == language }?.name ?: language
            Row(
                Modifier
                    .fillMaxWidth()
                    .clickable { showLanguagePicker = true }
                    .padding(horizontal = 24.dp, vertical = 16.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        words["profile.language"].orEmpty(),
                        color = Color.White,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        currentLangName,
                        color = Muted,
                        fontSize = 13.sp,
                    )
                }
                Icon(
                    painterResource(R.drawable.icon_caret_down),
                    null,
                    Modifier.size(20.dp),
                    tint = Muted,
                )
            }

            Spacer(Modifier.height(12.dp))

            // Profile Lock row
            Row(
                Modifier
                    .fillMaxWidth()
                    .clickable { showLockFlow = true }
                    .padding(horizontal = 24.dp, vertical = 16.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        words["profile.lock"].orEmpty(),
                        color = Color.White,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        if (hasPin) words["profile.lock.remove"].orEmpty() else words["profile.lock"].orEmpty(),
                        color = Muted,
                        fontSize = 13.sp,
                    )
                }
                Icon(
                    painterResource(if (hasPin) R.drawable.icon_locked else R.drawable.icon_lock_open),
                    null,
                    Modifier.size(22.dp),
                    tint = if (hasPin) Accent else Muted,
                )
            }

            // Delete Profile button at bottom (only for non-main profiles)
            if (!person.main) {
                Spacer(Modifier.height(36.dp))
                Box(
                    Modifier
                        .fillMaxWidth()
                        .clickable { showDeleteConfirm = true }
                        .padding(vertical = 16.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        words["profile.delete"].orEmpty(),
                        color = Join,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }

            Spacer(Modifier.height(32.dp))
        }

        // Avatar Picker Bottom Sheet
        if (showAvatarPicker) {
            AvatarPickerSheet(
                seen = seen,
                sets = sets,
                currentAvatar = avatar,
                onSelect = { selected ->
                    avatar = selected
                    showAvatarPicker = false
                },
                onDismiss = { showAvatarPicker = false },
            )
        }

        // Language Picker Bottom Sheet
        if (showLanguagePicker) {
            LanguagePickerSheet(
                words = words,
                languages = languages,
                current = language,
                onSelect = { selected ->
                    language = selected
                    shell.speak(selected)
                    showLanguagePicker = false
                },
                onDismiss = { showLanguagePicker = false },
            )
        }

        // Profile Lock Flow Sheet
        if (showLockFlow) {
            LockFlowSheet(
                shell = shell,
                seen = seen,
                person = person,
                hasPin = hasPin,
                pinDigits = pinDigits,
                onComplete = { newHasPin ->
                    hasPin = newHasPin
                    showLockFlow = false
                },
                onDismiss = { showLockFlow = false },
            )
        }

        // Delete Confirmation Sheet
        if (showDeleteConfirm) {
            DeleteConfirmSheet(
                words = words,
                onConfirm = {
                    showDeleteConfirm = false
                    deleteProfile()
                },
                onDismiss = { showDeleteConfirm = false },
            )
        }
    }
}

/** Avatar picker bottom sheet showing picture rows. */
@Composable
private fun AvatarPickerSheet(
    seen: Seen,
    sets: AvatarSets?,
    currentAvatar: String?,
    onSelect: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    val words = seen.words
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
        Column(
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
                .padding(vertical = 24.dp),
        ) {
            Row(
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    words["profile.icon"].orEmpty(),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                )
                Box(
                    Modifier
                        .size(48.dp)
                        .clickable(onClick = onDismiss),
                    contentAlignment = Alignment.CenterEnd,
                ) {
                    Text(
                        words["common.cancel"].orEmpty(),
                        color = Muted,
                        fontSize = 15.sp,
                    )
                }
            }

            Spacer(Modifier.height(12.dp))

            LazyColumn(
                Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(bottom = 16.dp),
                verticalArrangement = Arrangement.spacedBy(20.dp),
            ) {
                // Currently worn picture row (History) if applicable
                val worn = currentAvatar?.takeIf { it.startsWith("/") }
                if (worn != null) {
                    item {
                        Column(Modifier.fillMaxWidth()) {
                            Text(
                                words["profile.icon.history"].orEmpty(),
                                color = Color.White,
                                fontSize = 15.sp,
                                fontWeight = FontWeight.SemiBold,
                                modifier = Modifier.padding(horizontal = 24.dp, vertical = 6.dp),
                            )
                            LazyRow(
                                contentPadding = PaddingValues(horizontal = 24.dp),
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                            ) {
                                item {
                                    val shape = RoundedCornerShape(Corner.CARD.dp)
                                    Box(
                                        Modifier
                                            .size(72.dp)
                                            .clip(shape)
                                            .border(2.dp, Color.White, shape)
                                            .clickable { onSelect(worn) },
                                    ) {
                                        Art(seen.server + worn, Modifier.fillMaxSize(), AvatarBox, AvatarBox)
                                    }
                                }
                            }
                        }
                    }
                }

                // Theme sets
                val allSets = sets?.sets.orEmpty()
                items(allSets, key = { it.name }) { set ->
                    Column(Modifier.fillMaxWidth()) {
                        Text(
                            set.name,
                            color = Color.White,
                            fontSize = 15.sp,
                            fontWeight = FontWeight.SemiBold,
                            modifier = Modifier.padding(horizontal = 24.dp, vertical = 6.dp),
                        )
                        LazyRow(
                            contentPadding = PaddingValues(horizontal = 24.dp),
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            items(set.pictures, key = { it.path }) { picture ->
                                val isSelected = picture.path == currentAvatar
                                val shape = RoundedCornerShape(Corner.CARD.dp)
                                Box(
                                    Modifier
                                        .size(72.dp)
                                        .clip(shape)
                                        .then(
                                            if (isSelected) Modifier.border(2.dp, Color.White, shape)
                                            else Modifier,
                                        )
                                        .clickable { onSelect(picture.path) },
                                ) {
                                    Art(seen.server + picture.path, Modifier.fillMaxSize(), AvatarBox, AvatarBox)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/** Language selection bottom sheet displaying native language names. */
@Composable
private fun LanguagePickerSheet(
    words: Map<String, String>,
    languages: List<Language>,
    current: String,
    onSelect: (String) -> Unit,
    onDismiss: () -> Unit,
) {
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
        Column(
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
                .padding(vertical = 24.dp),
        ) {
            Row(
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 24.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    words["profile.language"].orEmpty(),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                )
                Box(
                    Modifier
                        .size(48.dp)
                        .clickable(onClick = onDismiss),
                    contentAlignment = Alignment.CenterEnd,
                ) {
                    Text(
                        words["common.cancel"].orEmpty(),
                        color = Muted,
                        fontSize = 15.sp,
                    )
                }
            }

            Spacer(Modifier.height(8.dp))

            LazyColumn(
                Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(vertical = 8.dp),
            ) {
                items(languages, key = { it.code }) { lang ->
                    val isChosen = lang.code == current
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .clickable { onSelect(lang.code) }
                            .padding(horizontal = 24.dp, vertical = 14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text(
                            lang.name,
                            color = if (isChosen) Accent else Color.White,
                            fontSize = 16.sp,
                            fontWeight = if (isChosen) FontWeight.Bold else FontWeight.Normal,
                        )
                        if (isChosen) {
                            Icon(
                                painterResource(R.drawable.icon_check),
                                null,
                                Modifier.size(20.dp),
                                tint = Accent,
                            )
                        }
                    }
                }
            }
        }
    }
}

/** Profile PIN setup or removal flow bottom sheet. */
@Composable
private fun LockFlowSheet(
    shell: Shell,
    seen: Seen,
    person: Profile,
    hasPin: Boolean,
    pinDigits: Int,
    onComplete: (Boolean) -> Unit,
    onDismiss: () -> Unit,
) {
    val words = seen.words
    val scope = rememberCoroutineScope()
    var step by remember { mutableStateOf(LockStep.PASSWORD) }
    var password by remember { mutableStateOf("") }
    var firstPin by remember { mutableStateOf("") }
    var secondPin by remember { mutableStateOf("") }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }

    val passwordFocus = remember { FocusRequester() }
    val pinFocus = remember { FocusRequester() }

    LaunchedEffect(step) {
        if (step == LockStep.PASSWORD) {
            passwordFocus.requestFocus()
        } else {
            pinFocus.requestFocus()
        }
    }

    fun submitPin(pinToSet: String?) {
        if (busy) return
        busy = true
        scope.launch {
            val body = contractJson.encodeToString(PinSet.serializer(), PinSet(pin = pinToSet))
            val answer = shell.write("PUT", "/v1/profiles/${person.id}/pin", body)
            busy = false
            if (answer == null) {
                errorMessage = words["error.try_again"]
            } else {
                shell.read("/v1/profiles")?.let { listJson ->
                    val list = contractJson.decodeFromString(ProfileList.serializer(), listJson).profiles
                    shell.keepAll(list)
                }
                onComplete(pinToSet != null)
            }
        }
    }

    fun verifyPassword() {
        if (password.isBlank() || busy) return
        busy = true
        errorMessage = null
        scope.launch {
            val body = contractJson.encodeToString(Confirm.serializer(), Confirm(password = password))
            val answer = shell.write("POST", "/v1/auth/confirm", body)
            busy = false
            password = ""
            if (answer == null) {
                errorMessage = words["error.sign_in"]
            } else if (hasPin) {
                submitPin(null)
            } else {
                step = LockStep.FIRST
                errorMessage = null
            }
        }
    }

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
                .padding(horizontal = 24.dp, vertical = 28.dp),
        ) {
            Column(
                Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                val titleKey = when {
                    hasPin -> "profile.lock.remove"
                    else -> "profile.lock"
                }
                Text(
                    words[titleKey].orEmpty(),
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                )

                Spacer(Modifier.height(8.dp))

                val helpKey = when (step) {
                    LockStep.PASSWORD -> "profile.lock.password.help"
                    LockStep.FIRST -> "profile.lock.pin.help"
                    LockStep.SECOND -> "profile.lock.confirm.help"
                }
                Text(
                    words[helpKey].orEmpty(),
                    color = Muted,
                    fontSize = 14.sp,
                    textAlign = TextAlign.Center,
                )

                Spacer(Modifier.height(24.dp))

                when (step) {
                    LockStep.PASSWORD -> {
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .background(Raised, RoundedCornerShape(Corner.FIELD.dp))
                                .padding(horizontal = 16.dp, vertical = 14.dp),
                        ) {
                            BasicTextField(
                                value = password,
                                onValueChange = { password = it; errorMessage = null },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .focusRequester(passwordFocus),
                                textStyle = TextStyle(
                                    color = Color.White,
                                    fontSize = 16.sp,
                                    fontFamily = BrandFont,
                                ),
                                singleLine = true,
                                visualTransformation = PasswordVisualTransformation(),
                                keyboardOptions = KeyboardOptions(
                                    keyboardType = KeyboardType.Password,
                                    imeAction = ImeAction.Done,
                                ),
                                keyboardActions = KeyboardActions(onDone = { verifyPassword() }),
                                cursorBrush = SolidColor(Color.White),
                            )
                        }

                        if (errorMessage != null) {
                            Spacer(Modifier.height(10.dp))
                            Text(
                                errorMessage.orEmpty(),
                                color = Join,
                                fontSize = 13.sp,
                                textAlign = TextAlign.Center,
                            )
                        }

                        Spacer(Modifier.height(24.dp))

                        Row(
                            Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(16.dp),
                        ) {
                            Box(
                                Modifier
                                    .weight(1f)
                                    .height(48.dp)
                                    .background(Raised, RoundedCornerShape(Corner.BUTTON.dp))
                                    .clickable(onClick = onDismiss),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    words["common.cancel"].orEmpty(),
                                    color = Color.White,
                                    fontSize = 16.sp,
                                    fontWeight = FontWeight.SemiBold,
                                )
                            }
                            Box(
                                Modifier
                                    .weight(1f)
                                    .height(48.dp)
                                    .background(Accent, RoundedCornerShape(Corner.BUTTON.dp))
                                    .clickable(enabled = password.isNotBlank() && !busy, onClick = ::verifyPassword),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    words["common.next"].orEmpty(),
                                    color = Color.White,
                                    fontSize = 16.sp,
                                    fontWeight = FontWeight.Bold,
                                )
                            }
                        }
                    }

                    LockStep.FIRST, LockStep.SECOND -> {
                        val currentPin = if (step == LockStep.FIRST) firstPin else secondPin
                        val fieldShape = RoundedCornerShape(Corner.FIELD.dp)

                        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            for (i in 0 until pinDigits) {
                                val entered = i < currentPin.length
                                val isCurrent = i == currentPin.length
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

                        // Invisible input capturing digits
                        BasicTextField(
                            value = currentPin,
                            onValueChange = { input ->
                                val digits = input.filter { it.isDigit() }.take(pinDigits)
                                errorMessage = null
                                if (step == LockStep.FIRST) {
                                    firstPin = digits
                                    if (digits.length == pinDigits) {
                                        step = LockStep.SECOND
                                        secondPin = ""
                                    }
                                } else {
                                    secondPin = digits
                                    if (digits.length == pinDigits) {
                                        if (digits == firstPin) {
                                            submitPin(digits)
                                        } else {
                                            errorMessage = words["profile.lock.mismatch"]
                                            firstPin = ""
                                            secondPin = ""
                                            step = LockStep.FIRST
                                        }
                                    }
                                }
                            },
                            modifier = Modifier
                                .focusRequester(pinFocus)
                                .size(1.dp)
                                .alpha(0f),
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
                            cursorBrush = SolidColor(Color.Transparent),
                        )

                        if (errorMessage != null) {
                            Spacer(Modifier.height(14.dp))
                            Text(
                                errorMessage.orEmpty(),
                                color = Join,
                                fontSize = 13.sp,
                                textAlign = TextAlign.Center,
                            )
                        }

                        Spacer(Modifier.height(24.dp))

                        Box(
                            Modifier
                                .fillMaxWidth()
                                .height(48.dp)
                                .background(Raised, RoundedCornerShape(Corner.BUTTON.dp))
                                .clickable(onClick = onDismiss),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                words["common.cancel"].orEmpty(),
                                color = Color.White,
                                fontSize = 16.sp,
                                fontWeight = FontWeight.SemiBold,
                            )
                        }
                    }
                }
            }
        }
    }
}

/** Delete confirmation bottom sheet. */
@Composable
private fun DeleteConfirmSheet(
    words: Map<String, String>,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
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
        Column(
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
                .padding(horizontal = 24.dp, vertical = 28.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                words["profile.delete.ask"].orEmpty(),
                color = Color.White,
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
            )

            Spacer(Modifier.height(24.dp))

            Box(
                Modifier
                    .fillMaxWidth()
                    .height(48.dp)
                    .background(Join, RoundedCornerShape(Corner.BUTTON.dp))
                    .clickable(onClick = onConfirm),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    words["profile.delete"].orEmpty(),
                    color = Color.White,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold,
                )
            }

            Spacer(Modifier.height(12.dp))

            Box(
                Modifier
                    .fillMaxWidth()
                    .height(48.dp)
                    .background(Raised, RoundedCornerShape(Corner.BUTTON.dp))
                    .clickable(onClick = onDismiss),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    words["common.cancel"].orEmpty(),
                    color = Color.White,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }
    }
}
