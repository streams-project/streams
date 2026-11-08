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
    const val BUTTON = BuildConfig.BUTTON_RADIUS

    /** A chip inside a chip row, and the row's two outer ends (a value past half the height reads as round). */
    const val CHIP = BuildConfig.CHIP_RADIUS
    const val CHIP_END = BuildConfig.CHIP_END_RADIUS
}

/**
 * A word's `{name}` place filled with a value, isolated (U+2068, U+2069) so a number, a title or an
 * address keeps its own order inside a line of either direction.
 */
fun String.fill(name: String, value: Any): String =
    replace("{$name}", "\u2068${if (value is Number) number(value) else value}\u2069")

/**
 * How the interface's language writes numbers and counts: its code (for plural forms) and its
 * digits (`language.digits` in its strings, `arab` for Arabic-Indic). The app sets both as the
 * words arrive.
 */
object Speaking {
    @Volatile
    var code: String = "en"

    @Volatile
    var arabicDigits: Boolean = false
}

/** A number as the interface's language writes it. */
fun number(value: Any): String {
    val plain = value.toString()
    return if (Speaking.arabicDigits) plain.map { if (it in '0'..'9') '\u0660' + (it - '0') else it }.joinToString("") else plain
}

/** The plural form a count takes in a language, as CLDR names them: zero, one, two, few, many, other. */
fun pluralOf(n: Int, lang: String): String = when (lang.substringBefore('-')) {
    "ar" -> when {
        n == 0 -> "zero"
        n == 1 -> "one"
        n == 2 -> "two"
        n % 100 in 3..10 -> "few"
        n % 100 in 11..99 -> "many"
        else -> "other"
    }
    "ru", "uk" -> when {
        n % 10 == 1 && n % 100 != 11 -> "one"
        n % 10 in 2..4 && n % 100 !in 12..14 -> "few"
        else -> "many"
    }
    "pl" -> when {
        n == 1 -> "one"
        n % 10 in 2..4 && n % 100 !in 12..14 -> "few"
        else -> "many"
    }
    "he", "iw" -> when (n) {
        1 -> "one"
        2 -> "two"
        else -> "other"
    }
    "ja", "ko", "zh" -> "other"
    "fr", "pt", "hi" -> if (n <= 1) "one" else "other"
    else -> if (n == 1) "one" else "other"
}

/** A counted word: `{base}.{form}` for the count's plural form, else `other`, `many`, then `one`. */
fun Map<String, String>.counted(base: String, n: Int): String {
    val form = pluralOf(n, Speaking.code)
    return listOf(form, "other", "many", "one").firstNotNullOfOrNull { this["$base.$it"] }.orEmpty().fill("n", n)
}
