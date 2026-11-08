package streams.tv

import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import contract.ActionStop
import contract.Target
import streams.ports.Shell
import streams.shared.Brand
import streams.shared.BrandFont

/** The one place that picks a television screen. */
@Composable
fun Television(shell: Shell) {
    val seen by shell.seen.collectAsStateWithLifecycle()
    val search = remember { SearchState() }
    val pages = remember { HashMap<String, Any>() }
    val opening = remember { OpeningState() }
    // Whether this run has been past the profile chooser, so Back there returns rather than leaves.
    var inside by remember { mutableStateOf(false) }
    Brand {
        ProvideTextStyle(TextStyle(fontFamily = BrandFont)) {
            Box(Modifier.fillMaxSize().background(Color.Black)) {
                val target = seen.route?.target
                // A run of the app with no server yet starts at the address screen, with no opening.
                if (seen.server.isEmpty() && !opening.opened) opening.over()
                // Under the opening the first screen is built and loading, and drawn once the first stage is over.
                Box(Modifier.fillMaxSize().graphicsLayer { alpha = if (opening.shown || opening.opened) 1f else 0f }) {
                // A right to left language mirrors every screen; the player keeps its own way (Watch).
                val way = if (seen.rtl) LayoutDirection.Rtl else LayoutDirection.Ltr
                CompositionLocalProvider(
                    LocalCovered provides !opening.opened,
                    LocalPages provides pages,
                    LocalLayoutDirection provides way,
                ) {
                if (target != null && target != Target.WAIT && target != Target.PROFILES && target != Target.START &&
                    target != Target.SIGN_IN
                ) {
                    inside = true
                }
                when {
                    seen.server.isEmpty() -> Server(shell)
                    target == null || target == Target.WAIT -> Unit
                    target == Target.START -> Door(shell, seen, opening)
                    target == Target.SIGN_IN -> SignIn(shell, seen)
                    target == Target.PROFILES -> Profiles(shell, seen, inside)
                    target == Target.TITLE -> TitlePage(shell, seen)
                    target == Target.WATCH -> Watch(shell, seen)
                    target == Target.NOT_FOUND -> NotFoundProblem(shell, seen)
                    target == Target.UPGRADE -> UpgradeProblem(seen)
                    target == Target.SCREEN && seen.route?.screen?.value == "search" -> Search(shell, seen, search)
                    target == Target.SCREEN && seen.route?.screen?.value == "help" -> GetHelp(shell, seen)
                    target == Target.SCREEN && seen.route?.screen?.value == "settings" -> Settings(shell, seen)
                    target == Target.HOME || target == Target.KIDS_HOME || target == Target.SCREEN ->
                        BrowseOrProblem(shell, seen)
                    else -> Text("", Modifier.align(Alignment.Center))
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
                if (seen.server.isNotEmpty()) OpeningLayer(shell, seen, opening)
            }
        }
    }
}
