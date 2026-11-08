package streams.shared

import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight

/** The brand's colours. brand.json is the source, and `streams build android` will write these. */
val Back = Color(0xFF0B0B10)
val Plate = Color(0xFF16161D)
val Raised = Color(0xFF24242E)
val Muted = Color(0xFFA6A6B3)
val Accent = Color(0xFF2F6BFF)
val Join = Color(0xFFE5352B)

@Composable
fun Brand(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = darkColorScheme(
            background = Back,
            surface = Back,
            surfaceVariant = Raised,
            primary = Color.White,
            onPrimary = Color.Black,
            onBackground = Color.White,
            onSurface = Color.White,
        ),
    ) {
        CompositionLocalProvider(LocalContentColor provides Color.White, content = content)
    }
}

/** The brand's text. A variable font, so every weight comes from one file. */
@OptIn(ExperimentalTextApi::class)
private fun inter(weight: Int) = Font(
    R.font.inter,
    weight = FontWeight(weight),
    variationSettings = FontVariation.Settings(FontVariation.weight(weight)),
)

val BrandFont = FontFamily(inter(400), inter(500), inter(600), inter(700), inter(800))

/** The brand's corners from brand.json `shape`, in dp on a phone. Each family scales them for its screen. */
object Corner {
    const val CARD = BuildConfig.CARD_RADIUS
    const val PANEL = BuildConfig.PANEL_RADIUS
    const val FIELD = BuildConfig.FIELD_RADIUS
}

/**
 * A word's `{name}` place filled with a value, isolated (U+2068, U+2069) so a number, a title or an
 * address keeps its own order inside a line of either direction.
 */
fun String.fill(name: String, value: Any): String = replace("{$name}", "\u2068$value\u2069")
