package streams.phone

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import contract.ActionStop
import contract.Target
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.LayoutDirection
import streams.shared.Back
import streams.shared.Brand
import streams.shared.Speaking
import streams.ports.Shell

/** The one place that picks a surface. `tv` chooses the remote's layouts over the phone's. */
@Composable
fun Root(shell: Shell, tv: Boolean) {
    val seen by shell.seen.collectAsStateWithLifecycle()
    // The language decides direction, digits, plural forms and text size, as on the TV; the
    // system's own text size is kept and multiplied, never replaced.
    Speaking.arabicDigits = seen.words["language.digits"] == "arab"
    Speaking.code = seen.language.ifEmpty { "en" }
    val scale = seen.words["language.text_scale"]?.toFloatOrNull()?.coerceIn(0.8f, 1.4f) ?: 1f
    val density = LocalDensity.current
    Brand {
        CompositionLocalProvider(
            LocalLayoutDirection provides if (seen.rtl) LayoutDirection.Rtl else LayoutDirection.Ltr,
            LocalDensity provides Density(density.density, density.fontScale * scale),
        ) {
        Box(Modifier.fillMaxSize().background(Back)) {
            val target = seen.route?.target
            when {
                seen.server.isEmpty() -> Server(shell)
                target == null || target == Target.WAIT -> Unit
                target == Target.START -> Door(shell, seen)
                target == Target.SIGN_IN -> if (tv) Code(shell, seen) else SignIn(shell, seen)
                target == Target.PROFILES -> People(shell, seen, tv)
                target == Target.TITLE -> Title(shell, seen, tv = false)
                target == Target.WATCH -> Watch(shell, seen, tv)
                target == Target.HOME || target == Target.KIDS_HOME || target == Target.SCREEN ->
                    Home(shell, seen)
                else -> Text("That page is not here.", Modifier.align(Alignment.Center))
            }
            if (target == Target.TITLE) BackHandler { shell.back() }
            if (target == Target.WATCH) {
                BackHandler {
                    shell.act(ActionStop)
                    shell.back()
                }
            }
        }
        }
    }
}
