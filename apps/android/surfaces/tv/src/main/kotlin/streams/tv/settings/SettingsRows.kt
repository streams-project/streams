package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import contract.Account
import contract.DeviceSession
import contract.Language
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import streams.shared.Back
import streams.shared.Join
import streams.shared.Muted
import streams.shared.Raised
import streams.shared.ref
import streams.shared.refType

enum class SettingsSection(val key: String) {
    PLAYBACK("settings.playback"),
    LANGUAGES("settings.languages"),
    SUBTITLES("settings.subtitles"),
    HISTORY("settings.history"),
    DEVICES("settings.devices"),
    ACCOUNT("settings.account"),
    ABOUT("settings.about"),
}

sealed interface SettingsAction
data object FlipAutoplayNext : SettingsAction
data object FlipAutoplayPreviews : SettingsAction
data object PickAudioLanguage : SettingsAction
data object PickSubtitleLanguage : SettingsAction
data object CycleSubtitleSize : SettingsAction
data object CycleSubtitleColor : SettingsAction
data object CycleSubtitleBackground : SettingsAction
data object AskClearHistory : SettingsAction
data class AskRemoveDevice(val id: String, val name: String) : SettingsAction
data object AskRemoveOtherDevices : SettingsAction
data object AskSignOutEverywhere : SettingsAction

data class SettingsRow(
    val label: String,
    val value: String = "",
    val action: SettingsAction? = null,
    /** Drawn in the warning colour, for a value the viewer should not pass over. */
    val warn: Boolean = false,
)

private val DateFormat: DateTimeFormatter =
    DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm").withZone(ZoneId.systemDefault())

private fun Map<String, String>.say(key: String) = this[key].orEmpty()

fun formatSeenMs(ms: Long): String =
    runCatching { DateFormat.format(Instant.ofEpochMilli(ms)) }.getOrDefault("")

/** Builds the right-hand rows for the active settings section. */
fun buildSettingsRows(
    section: SettingsSection,
    words: Map<String, String>,
    autoplayNext: Boolean,
    autoplayPreviews: Boolean,
    audioLanguage: String?,
    subtitleLanguage: String?,
    languages: List<Language>,
    uiLanguage: String,
    subSize: String,
    subColor: String,
    subBg: String,
    historyCleared: Boolean,
    devices: List<DeviceSession>,
    account: Account?,
    server: String,
    serverPlain: Boolean,
): List<SettingsRow> {
    val onWord = words.say("settings.on")
    val offWord = words.say("settings.off")
    return when (section) {
        SettingsSection.PLAYBACK -> listOf(
            SettingsRow(
                words.say("settings.autoplay_next"),
                if (autoplayNext) onWord else offWord,
                FlipAutoplayNext,
            ),
            SettingsRow(
                words.say("settings.autoplay_previews"),
                if (autoplayPreviews) onWord else offWord,
                FlipAutoplayPreviews,
            ),
        )
        SettingsSection.LANGUAGES -> {
            val audioCode = audioLanguage?.takeIf { it.isNotEmpty() } ?: uiLanguage
            val audioName = languages.firstOrNull { it.code == audioCode }?.name ?: audioCode
            val subName = if (subtitleLanguage.isNullOrEmpty()) {
                offWord
            } else {
                languages.firstOrNull { it.code == subtitleLanguage }?.name ?: subtitleLanguage
            }
            listOf(
                SettingsRow(words.say("settings.audio_language"), audioName, PickAudioLanguage),
                SettingsRow(words.say("settings.subtitle_language"), subName, PickSubtitleLanguage),
            )
        }
        SettingsSection.SUBTITLES -> {
            val sizeWord = when (subSize) {
                "0" -> words.say("settings.size_small")
                "2" -> words.say("settings.size_large")
                else -> words.say("settings.size_medium")
            }
            val colorWord = if (subColor.lowercase() == "#ffff00") {
                words.say("settings.color_yellow")
            } else {
                words.say("settings.color_white")
            }
            val bgWord = if (subBg == "black") {
                words.say("settings.bg_black")
            } else {
                words.say("settings.bg_none")
            }
            listOf(
                SettingsRow(words.say("settings.subtitle_size"), sizeWord, CycleSubtitleSize),
                SettingsRow(words.say("settings.subtitle_color"), colorWord, CycleSubtitleColor),
                SettingsRow(words.say("settings.subtitle_background"), bgWord, CycleSubtitleBackground),
            )
        }
        SettingsSection.HISTORY -> listOf(
            SettingsRow(
                words.say("settings.clear_history"),
                if (historyCleared) words.say("settings.cleared") else "",
                AskClearHistory,
            ),
        )
        SettingsSection.DEVICES -> {
            val deviceRows = devices.map { device ->
                val tag = if (device.current) " · ${words.say("settings.device_current")}" else ""
                val label = "${device.name} (${device.`class`.value})$tag"
                val seenText = formatSeenMs(device.lastSeenMs)
                val act = if (device.current) null else AskRemoveDevice(device.id, device.name)
                SettingsRow(label, seenText, act)
            }
            deviceRows + SettingsRow(
                words.say("settings.devices_sign_out_others"),
                "",
                AskRemoveOtherDevices,
            )
        }
        SettingsSection.ACCOUNT -> {
            val fullName = listOfNotNull(account?.firstName, account?.lastName)
                .joinToString(" ")
                .trim()
            listOf(
                SettingsRow(words.say("settings.account_email"), account?.email.orEmpty()),
                SettingsRow(words.say("settings.account_name"), fullName),
            )
        }
        SettingsSection.ABOUT -> listOf(
            // An unencrypted server is marked here, because this is the only screen that says
            // which server this device talks to.
            SettingsRow(words.say("settings.server"), server, warn = serverPlain),
            SettingsRow(words.say("settings.sign_out_everywhere"), "", AskSignOutEverywhere),
        )
    }
}

/** Draws one row in the right-hand settings column, with its label on the left and value on the right. */
@Composable
fun SettingsRowItem(row: SettingsRow, on: Boolean) {
    val labelColor = if (on) Back else Color.White
    val valueColor = if (row.warn) Join else if (on) Back else Muted
    Row(
        Modifier
            .width(1080.ref)
            .padding(bottom = 6.ref)
            .background(if (on) Color.White else Color.Transparent, CircleShape)
            .padding(horizontal = 32.ref, vertical = 14.ref),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            row.label,
            Modifier.weight(1f, fill = false).padding(end = 24.ref),
            color = labelColor,
            fontSize = 30.refType,
            fontWeight = FontWeight.Normal,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (row.value.isNotEmpty()) {
            Text(
                row.value,
                color = valueColor,
                fontSize = 30.refType,
                fontWeight = FontWeight.Medium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

/** A live preview of the chosen subtitle size, colour, and background. */
@Composable
fun SubtitlePreview(words: Map<String, String>, size: String, color: String, background: String) {
    val fontSize = when (size) {
        "0" -> 26.refType
        "2" -> 42.refType
        else -> 34.refType
    }
    val ink = if (color.lowercase() == "#ffff00") Color(0xFFFFFF00) else Color.White
    val boxBg = if (background == "black") Color.Black.copy(alpha = 0.9f) else Color.Transparent
    Column(Modifier.padding(top = 28.ref, start = 16.ref)) {
        Box(
            Modifier
                .width(1040.ref)
                .height(140.ref)
                .background(Raised, shapeOf(Round.field)),
            contentAlignment = Alignment.Center,
        ) {
            Box(
                Modifier
                    .background(boxBg, shapeOf(Round.chip))
                    .padding(horizontal = 20.ref, vertical = 8.ref),
            ) {
                Text(
                    words.say("settings.subtitle_preview"),
                    color = ink,
                    fontSize = fontSize,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        Spacer(Modifier.height(8.ref))
    }
}
