package streams.tv

import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import contract.TitleDetail
import contract.TitleKey
import contract.contractJson
import kotlin.coroutines.resume
import kotlinx.coroutines.delay
import kotlinx.coroutines.suspendCancellableCoroutine
import streams.ports.Shell

private const val SETTLE_MS = 240L

/** What has been read already, for this run of the screen. */
class DetailStore {
    val kept = HashMap<TitleKey, TitleDetail>()
}

@Composable
fun rememberDetails() = remember { DetailStore() }

/** The details of whatever has focus, once the viewer settles on it. */
@Composable
fun detailOf(shell: Shell, store: DetailStore, title: TitleKey): State<TitleDetail?> =
    produceState(store.kept[title], title) {
        value = store.kept[title]
        if (value != null) return@produceState
        delay(SETTLE_MS)
        value = suspendCancellableCoroutine { waiting ->
            shell.ask("/v1/titles/" + title.value.replace(':', '/')) { status, text ->
                val detail = if (status in 200..299) {
                    contractJson.decodeFromString(TitleDetail.serializer(), text)
                } else {
                    null
                }
                detail?.let { store.kept[title] = it }
                if (waiting.isActive) waiting.resume(detail)
            }
        }
    }
