package streams.surfaces

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

/** The brand's colours. brand.json is the source, and `streams build android` will write these. */
val Back = Color(0xFF0B0B10)
val Raised = Color(0xFF24242E)
val Muted = Color(0xFFA6A6B3)
val Accent = Color(0xFF2F6BFF)

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
        content = content,
    )
}
