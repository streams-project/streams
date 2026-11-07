package streams.tv

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
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
    val door = remember { DoorState() }
    Brand {
        ProvideTextStyle(TextStyle(fontFamily = BrandFont)) {
            Box(Modifier.fillMaxSize().background(Color.Black)) {
                val target = seen.route?.target
                when {
                    seen.server.isEmpty() -> Server(shell)
                    target == null || target == Target.WAIT -> Unit
                    target == Target.START -> Door(shell, seen, door)
                    target == Target.SIGN_IN -> SignIn(shell, seen)
                    target == Target.PROFILES -> Profiles(shell, seen)
                    target == Target.TITLE -> TitlePage(shell, seen)
                    target == Target.WATCH -> Watch(shell, seen)
                    target == Target.NOT_FOUND -> NotFoundProblem(shell, seen)
                    target == Target.UPGRADE -> UpgradeProblem(seen)
                    target == Target.SCREEN && seen.route?.screen?.value == "search" -> Search(shell, seen, search)
                    target == Target.SCREEN && seen.route?.screen?.value == "help" -> HelpPage(shell, seen)
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
    }
}
