package streams.tv

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import contract.Account
import contract.ActionCommand
import contract.ActionSignOut
import contract.CommandClearHistory
import contract.CommandSetting
import contract.DeviceList
import contract.DeviceSession
import contract.Language
import contract.LanguageList
import contract.ProfileSettings
import contract.SettingKey
import contract.contractJson
import kotlinx.coroutines.launch
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.ref
import streams.shared.refType

private const val MOVE_MS = 240
private const val SCREEN = 1080
private const val FIRST_LINE = 262
private const val LINE = 78

private enum class LanguageTarget { AUDIO, SUBTITLE }

/** Settings: seven sections on the left and their rows on the right. */
@Composable
fun Settings(shell: Shell, seen: Seen) {
    val words = seen.words
    val scope = rememberCoroutineScope()

    var autoplayNext by remember { mutableStateOf(true) }
    var autoplayPreviews by remember { mutableStateOf(true) }
    var audioLanguage by remember { mutableStateOf<String?>(null) }
    var subtitleLanguage by remember { mutableStateOf<String?>(null) }
    var subSize by remember { mutableStateOf("1") }
    var subColor by remember { mutableStateOf("#ffffff") }
    var subBg by remember { mutableStateOf("none") }
    var historyCleared by remember { mutableStateOf(false) }
    var devices by remember { mutableStateOf<List<DeviceSession>>(emptyList()) }
    var account by remember { mutableStateOf<Account?>(null) }

    val languages by produceState<List<Language>>(emptyList()) {
        value = shell.read("/v1/languages")
            ?.let { contractJson.decodeFromString(LanguageList.serializer(), it).languages }
            .orEmpty()
    }

    LaunchedEffect(seen.active) {
        shell.read("/v1/profiles/me/settings")?.let { text ->
            val loaded = contractJson.decodeFromString(ProfileSettings.serializer(), text)
            autoplayNext = loaded.autoplayNext
            autoplayPreviews = loaded.autoplayPreviews
            audioLanguage = loaded.audioLanguage
            subtitleLanguage = loaded.subtitleLanguage
            loaded.subtitleStyle?.let { style ->
                subSize = style["size"] ?: "1"
                subColor = style["color"] ?: "#ffffff"
                subBg = style["background"] ?: "none"
            }
        }
        shell.read("/v1/devices")?.let { text ->
            devices = contractJson.decodeFromString(DeviceList.serializer(), text).devices
        }
        shell.read("/v1/account")?.let { text ->
            account = contractJson.decodeFromString(Account.serializer(), text)
        }
    }

    var sectionIndex by remember { mutableIntStateOf(0) }
    var right by remember { mutableStateOf(false) }
    var line by remember { mutableIntStateOf(0) }
    var langPicker by remember { mutableStateOf<LanguageTarget?>(null) }
    var pendingConfirm by remember { mutableStateOf<SettingsAction?>(null) }
    var confirmAt by remember { mutableIntStateOf(1) }

    fun sendSetting(key: SettingKey, value: String) {
        shell.act(ActionCommand(command = CommandSetting(key = key, value = value)))
    }

    fun sendSubtitleStyle(size: String, color: String, bg: String) {
        val json = """{"size":"$size","color":"$color","background":"$bg"}"""
        sendSetting(SettingKey.SUBTITLE_STYLE, json)
    }

    fun refreshDevices() {
        scope.launch {
            shell.read("/v1/devices")?.let { text ->
                devices = contractJson.decodeFromString(DeviceList.serializer(), text).devices
            }
        }
    }

    langPicker?.let { target ->
        val list = if (target == LanguageTarget.SUBTITLE) {
            listOf(Language(code = "", name = words["settings.off"].orEmpty())) + languages
        } else {
            languages
        }
        val current = if (target == LanguageTarget.SUBTITLE) {
            subtitleLanguage.orEmpty()
        } else {
            audioLanguage?.takeIf { it.isNotEmpty() } ?: seen.language
        }
        EditLanguage(words, list, current) { chosen ->
            langPicker = null
            if (chosen != null) {
                if (target == LanguageTarget.AUDIO) {
                    audioLanguage = chosen
                    sendSetting(SettingKey.AUDIO_LANGUAGE, chosen)
                } else {
                    subtitleLanguage = chosen.ifEmpty { null }
                    sendSetting(SettingKey.SUBTITLE_LANGUAGE, chosen)
                }
            }
        }
        return
    }

    val sections = SettingsSection.entries
    val section = sections[sectionIndex]
    val rows = buildSettingsRows(
        section = section,
        words = words,
        autoplayNext = autoplayNext,
        autoplayPreviews = autoplayPreviews,
        audioLanguage = audioLanguage,
        subtitleLanguage = subtitleLanguage,
        languages = languages,
        uiLanguage = seen.language.ifEmpty { "en" },
        subSize = subSize,
        subColor = subColor,
        subBg = subBg,
        historyCleared = historyCleared,
        devices = devices,
        account = account,
        server = seen.server,
    )

    if (line > rows.lastIndex && rows.isNotEmpty()) {
        line = rows.lastIndex
    }

    fun executeConfirmed(action: SettingsAction) {
        pendingConfirm = null
        when (action) {
            AskClearHistory -> {
                historyCleared = true
                shell.act(ActionCommand(command = CommandClearHistory))
            }
            is AskRemoveDevice -> {
                scope.launch {
                    shell.write("DELETE", "/v1/devices?id=${action.id}", "")
                    refreshDevices()
                }
            }
            AskRemoveOtherDevices -> {
                scope.launch {
                    shell.write("DELETE", "/v1/devices", "")
                    refreshDevices()
                }
            }
            AskSignOutEverywhere -> {
                shell.act(ActionSignOut(everywhere = true))
            }
            else -> Unit
        }
    }

    fun activateRow(action: SettingsAction?) {
        when (action) {
            null -> Unit
            FlipAutoplayNext -> {
                autoplayNext = !autoplayNext
                sendSetting(SettingKey.AUTOPLAY_NEXT, autoplayNext.toString())
            }
            FlipAutoplayPreviews -> {
                autoplayPreviews = !autoplayPreviews
                sendSetting(SettingKey.AUTOPLAY_PREVIEWS, autoplayPreviews.toString())
            }
            PickAudioLanguage -> langPicker = LanguageTarget.AUDIO
            PickSubtitleLanguage -> langPicker = LanguageTarget.SUBTITLE
            CycleSubtitleSize -> {
                subSize = when (subSize) {
                    "0" -> "1"
                    "1" -> "2"
                    else -> "0"
                }
                sendSubtitleStyle(subSize, subColor, subBg)
            }
            CycleSubtitleColor -> {
                subColor = if (subColor.lowercase() == "#ffff00") "#ffffff" else "#ffff00"
                sendSubtitleStyle(subSize, subColor, subBg)
            }
            CycleSubtitleBackground -> {
                subBg = if (subBg == "black") "none" else "black"
                sendSubtitleStyle(subSize, subColor, subBg)
            }
            AskClearHistory, is AskRemoveDevice, AskRemoveOtherDevices, AskSignOutEverywhere -> {
                pendingConfirm = action
                confirmAt = 1
            }
        }
    }

    val scrolled by animateFloatAsState(maxOf(0, line - 7).toFloat(), tween(MOVE_MS, easing = Glide), label = "list")
    val step = with(LocalDensity.current) { LINE.ref.toPx() }

    fun press(word: Word): Boolean {
        val asking = pendingConfirm
        if (asking != null) {
            when (word) {
                Word.UP -> if (confirmAt > 0) confirmAt -= 1
                Word.DOWN -> if (confirmAt < 1) confirmAt += 1
                Word.OK -> if (confirmAt == 0) executeConfirmed(asking) else pendingConfirm = null
                Word.BACK, Word.LEFT -> pendingConfirm = null
                else -> Unit
            }
            return true
        }
        when {
            word == Word.BACK && right -> right = false
            word == Word.BACK -> shell.back()
            right && word == Word.LEFT -> right = false
            // On the sections, the left edge of the page, Left goes back.
            word == Word.LEFT -> shell.back()
            right && word == Word.UP -> if (line > 0) line -= 1
            right && word == Word.DOWN -> if (line < rows.lastIndex) line += 1
            right && word == Word.OK -> activateRow(rows.getOrNull(line)?.action)
            word == Word.UP -> if (sectionIndex > 0) {
                sectionIndex -= 1
                line = 0
                historyCleared = false
            }
            word == Word.DOWN -> if (sectionIndex < sections.lastIndex) {
                sectionIndex += 1
                line = 0
                historyCleared = false
            }
            (word == Word.RIGHT || word == Word.OK) && rows.isNotEmpty() -> right = true
        }
        return true
    }

    val menuLines = sections.map { MenuLine(words[it.key].orEmpty()) }
    Box(Modifier.fillMaxSize().background(Color.Black).remote(@DontMemoize { press(it) })) {
        PageHead(words["settings.title"].orEmpty(), words["settings.intro"].orEmpty())
        Menu(
            menuLines,
            if (right) -1 else sectionIndex,
            Modifier.offset(127.ref, FIRST_LINE.ref),
            width = 480,
            kept = sectionIndex,
        )
        Box(Modifier.offset(700.ref, FIRST_LINE.ref).size(1100.ref, (SCREEN - FIRST_LINE - 60).ref).clipToBounds()) {
            val asking = pendingConfirm
            if (asking != null) {
                val (askPrompt, yesWord) = when (asking) {
                    AskClearHistory -> words["settings.clear_history.ask"].orEmpty() to words["settings.clear_history"].orEmpty()
                    is AskRemoveDevice -> words["settings.device_remove.ask"].orEmpty() to words["settings.device_remove"].orEmpty()
                    AskRemoveOtherDevices -> words["settings.devices_sign_out_others.ask"].orEmpty() to words["settings.devices_sign_out_others"].orEmpty()
                    AskSignOutEverywhere -> words["settings.sign_out_everywhere.ask"].orEmpty() to words["settings.sign_out_everywhere"].orEmpty()
                    else -> "" to ""
                }
                Column(Modifier.padding(start = 16.ref, top = 8.ref)) {
                    Text(askPrompt, fontSize = 32.refType, color = Color.White)
                    Spacer(Modifier.height(24.ref))
                    Menu(
                        listOf(MenuLine(yesWord), MenuLine(words["common.cancel"].orEmpty())),
                        confirmAt,
                        width = 520,
                    )
                }
            } else {
                Column(
                    Modifier
                        .wrapContentHeight(Alignment.Top, unbounded = true)
                        .graphicsLayer { translationY = -scrolled * step },
                ) {
                    rows.forEachIndexed { index, row ->
                        SettingsRowItem(row, right && index == line)
                    }
                    if (section == SettingsSection.SUBTITLES) {
                        SubtitlePreview(words, subSize, subColor, subBg)
                    }
                }
            }
        }
    }
}
