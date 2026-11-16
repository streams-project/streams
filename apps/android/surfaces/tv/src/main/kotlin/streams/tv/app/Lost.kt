package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import kotlinx.coroutines.delay
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.Back
import streams.shared.ref
import streams.shared.refType

/**
 * How long a television waits for its server before it says, on screen, that it cannot reach it.
 *
 * Derived rather than chosen. The opening stops waiting for the first screen at [WAIT_AT_MOST] and
 * lifts whatever is underneath; from that moment the viewer is looking at a screen rather than at
 * the opening. This is that wait again, so the server gets as long with the viewer watching as it
 * got behind the opening, and the message arrives about [WAIT_AT_MOST] seconds after the opening
 * lifts rather than the instant a slow server is slow.
 *
 * Against what was measured: on a Google TV Streamer with the server unreachable, the app drew a
 * flat `#0b0b10` rectangle with 9 nodes, 0 focusable and 0 words, held across 8 probes and across a
 * force-stop and cold start (notes/239, and the same thing in notes/206 on 2026-11-11). There was
 * no number to be too long or too short, because nothing was ever drawn.
 */
const val LOST_AFTER_MS: Long = 2 * (WAIT_AT_MOST * 1000).toLong()

/**
 * A screen that has a server and has heard nothing back from it.
 *
 * Until the wait is up this is [Waiting], which is the measured reference behaviour for a page that
 * cannot yet say where it is. After it, the viewer is told, and given the one thing that can
 * actually change the situation from a television: ask the same address again.
 *
 * **The hard part of this screen is that it may have no words.** Every word this app draws arrives
 * from `GET /v1/strings/{lang}` (`core/Host.kt`), so a television that cannot reach its server has
 * no vocabulary at all -- `seen.words` is empty, `seen.brand` is empty, and every part that draws
 * `words[...]` draws nothing. That is why the measured screen had 0 words rather than a wrong
 * message: there was nothing to say it with. `parts/Pill.kt` states the same thing from the other
 * end -- a pill with no word draws nothing rather than an empty white shape.
 *
 * So this screen is built to say something anyway. The server's address is the viewer's own data,
 * not an interface word, so it is there whether the strings arrived or not, and it is drawn and
 * read aloud. A reader is handed the address and lands on a focusable control; when the strings did
 * arrive it is handed a sentence and a named button as well. What it cannot do with no strings is
 * explain *what* to press, and no amount of work inside this surface can fix that: the fix is for
 * the app to carry words it did not have to ask the server for.
 */
@Composable
fun Reaching(shell: Shell, seen: Seen) {
    var lost by remember(seen.server) { mutableStateOf(false) }
    LaunchedEffect(seen.server) {
        delay(LOST_AFTER_MS)
        lost = true
    }
    if (lost) Lost(shell, seen) else Waiting(shell, seen)
}

/** The television cannot reach the address it was given, said out loud, with a way to try again. */
@Composable
fun Lost(shell: Shell, seen: Seen) {
    val words = seen.words
    val trouble = words["error.no_server"]
    val again = words["say.try_again"]
    // What the remote is on, said when the screen opens and again whenever it changes. The address
    // is last and never missing, so this is never the empty string -- an empty `says` leaves the
    // one focusable node on the screen unnamed, which is the defect next door in the player.
    val saying = sayBoth(sayBoth(trouble, again), seen.server)
    Box(
        Modifier
            .fillMaxSize()
            .background(Color.Black)
            .remote(saying) { word ->
                if (word == Word.OK) {
                    shell.retry()
                    true
                } else {
                    false
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
        ) {
            trouble?.takeIf { it.isNotEmpty() }?.let {
                Text(
                    it,
                    Modifier.width(1100.ref),
                    fontSize = 44.refType,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                )
            }
            // The address, always. It is the one thing on this screen that did not have to be
            // fetched to be known, and with no strings it is the whole of what a reader is handed.
            Text(
                seen.server,
                Modifier.padding(top = 24.ref).width(1100.ref),
                fontSize = 32.refType,
                color = Color.White.copy(alpha = 0.75f),
                textAlign = TextAlign.Center,
            )
            // The way out: the same address, asked again. `Pill` is not used here because a pill
            // with no word draws nothing, and this control has to stay on the screen in the case
            // the screen exists for.
            Box(
                Modifier
                    .padding(top = 56.ref)
                    .background(Color.White, CircleShape)
                    .padding(horizontal = 56.ref, vertical = 20.ref)
                    .says(sayBoth(again, seen.server), on = true),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    again.orEmpty(),
                    color = Back,
                    fontSize = 32.refType,
                    fontWeight = FontWeight.Bold,
                )
            }
        }
    }
}
