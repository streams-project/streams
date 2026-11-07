package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DontMemoize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import contract.Address
import contract.Target
import kotlinx.coroutines.delay
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Muted
import streams.shared.ref
import streams.shared.refType

private const val OFFLINE_WAIT_MS = 8_000L

@Composable
private fun ProblemView(
    heading: String,
    line: String,
    buttonWord: String? = null,
    onPress: (() -> Unit)? = null,
    onBack: (() -> Unit)? = null,
) {
    fun press(word: Word): Boolean = when (word) {
        Word.OK -> {
            onPress?.invoke()
            true
        }
        Word.BACK -> {
            if (onBack != null) {
                onBack()
                true
            } else {
                false
            }
        }
        else -> true
    }

    Box(
        Modifier.fillMaxSize().background(Color.Black).remote(@DontMemoize { press(it) }),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            Modifier.width(1000.ref),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                heading,
                fontSize = 52.refType,
                fontWeight = FontWeight.Bold,
                color = Color.White,
                textAlign = TextAlign.Center,
            )
            Spacer(Modifier.height(20.ref))
            Text(
                line,
                fontSize = 32.refType,
                color = Muted,
                textAlign = TextAlign.Center,
            )
            if (buttonWord != null) {
                Spacer(Modifier.height(44.ref))
                Pill(buttonWord, on = true)
            }
        }
    }
}

/** Shown when the viewer navigates to an unknown route (`Target.NOT_FOUND`). */
@Composable
fun NotFoundProblem(shell: Shell, seen: Seen) {
    ProblemView(
        heading = seen.words["problem.not_found"].orEmpty(),
        line = seen.words["problem.not_found.help"].orEmpty(),
        buttonWord = seen.words["nav.home"].orEmpty(),
        onPress = { shell.go(Address.HOME) },
        onBack = { shell.go(Address.HOME) },
    )
}

/** Shown when the server requires a newer client version (`Target.UPGRADE`). */
@Composable
fun UpgradeProblem(seen: Seen) {
    ProblemView(
        heading = seen.words["problem.upgrade"].orEmpty(),
        line = seen.words["problem.upgrade.help"].orEmpty(),
    )
}

/** Draws `Browse`, or the offline problem screen when the target screen stays missing for 8 seconds. */
@Composable
fun BrowseOrProblem(shell: Shell, seen: Seen) {
    keptWords = seen.words
    val target = seen.route?.target
    val wanted = when (target) {
        Target.HOME -> "home"
        Target.KIDS_HOME -> "kids"
        else -> seen.route?.screen?.value
    }
    val hasScreen = seen.screen != null && (wanted == null || seen.screen?.name?.value == wanted)
    var attempts by remember { mutableIntStateOf(0) }
    var timedOut by remember(seen.route, hasScreen, attempts) { mutableStateOf(false) }

    LaunchedEffect(seen.route, hasScreen, attempts) {
        if (!hasScreen) {
            delay(OFFLINE_WAIT_MS)
            timedOut = true
        }
    }

    if (timedOut && !hasScreen) {
        val line = seen.notice?.takeIf { it.isNotEmpty() } ?: seen.words["problem.offline.help"].orEmpty()
        ProblemView(
            heading = seen.words["problem.offline"].orEmpty(),
            line = line,
            buttonWord = seen.words["problem.retry"].orEmpty(),
            onPress = {
                attempts += 1
                shell.go("/")
            },
            onBack = { shell.back() },
        )
    } else {
        Browse(shell, seen)
    }
}
