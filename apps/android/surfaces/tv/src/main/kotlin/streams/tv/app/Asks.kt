package streams.tv

import kotlin.coroutines.resume
import kotlinx.coroutines.suspendCancellableCoroutine
import streams.ports.Shell

private fun good(status: Int, text: String) = if (status in 200..299) text else null

/** The text of a read, or nothing when it did not work. */
suspend fun Shell.read(path: String): String? = suspendCancellableCoroutine { waiting ->
    ask(path) { status, text -> if (waiting.isActive) waiting.resume(good(status, text)) }
}

/** The text of a change, or nothing when it was refused. */
suspend fun Shell.write(method: String, path: String, body: String): String? =
    suspendCancellableCoroutine { waiting ->
        send(method, path, body) { status, text -> if (waiting.isActive) waiting.resume(good(status, text)) }
    }
