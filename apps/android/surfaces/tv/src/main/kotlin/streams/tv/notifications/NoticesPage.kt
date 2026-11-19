package streams.tv

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import contract.TitleKey
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.titleAddress
import streams.tv.notifications.Notices
import streams.tv.notifications.NoticesView
import streams.tv.notifications.Spot

/**
 * The notifications feed.
 *
 * ## What is in this file, and what is not
 * The screen is `notifications/NoticesView.java`: the layout, the key map, the focus memory, the
 * measured geometry, what a screen reader is handed and the drawing. `notifications/Notices.java`
 * reads `GET /v1/notifications` itself -- it needs no Kotlin seam, which `browse/Browse.kt`'s does,
 * because `contract.Contract.Notifications` is a plain Java class (note 240).
 *
 * So what is left here is smaller than `Browse.kt`'s share and is the same two things: the
 * `AndroidView` that holds the screen, and the suspending work. The suspending work is a shell that
 * speaks in callbacks and coroutines -- `shell.read` is a `suspend fun` and `shell.send` takes a
 * reply -- and neither can be called from Java against the Kotlin arm of the contract. `Browse.kt`
 * states the end this wrapper has: wave 3 turns `app/Main.kt`'s `setContent` into `setContentView`
 * and every one of these becomes an `addView`.
 *
 * ## Where the viewer is, across a visit
 * `Spot` is kept in `LocalPages` under this screen's own key, for the run of the app, exactly as a
 * page of rows keeps its `Place`. Leaving the feed and coming back therefore lands on the notice
 * that was under the remote, and not on the newest one -- which matters more here than on a page of
 * rows, because this is the one list in the app that grows at the top.
 */
@Composable
fun NoticesPage(shell: Shell, seen: Seen) {
    val memory = LocalPages.current
    val spot = remember(seen.active) {
        memory.getOrPut("notifications ${seen.active}") { Spot(Notices.NOTHING) } as Spot
    }
    val pictures = remember(shell) { ShellArt(shell) }
    val screen = remember { mutableStateOf<NoticesView?>(null) }
    // Bumped by the Java screen when the remote has gone somewhere, so the paging read below
    // follows it. The explicit redraw note 210 asks for, in the one direction Compose cannot be
    // told about: without it the page would draw correctly and never read another page again.
    var moved by remember { mutableIntStateOf(0) }
    var feed by remember { mutableStateOf(spot.feed()) }
    var reading by remember { mutableStateOf(false) }

    // The first page. Keyed on the profile, so a change of viewer reads that viewer's feed.
    LaunchedEffect(seen.active) {
        val body = shell.read(FEED)
        if (body != null) feed = Notices.read(body)
    }
    // The rest of it. A remote has no moment where a line comes into view, so the read hangs off
    // where the focus is: `Spot.wantsMore()` carries the reach and its derivation.
    LaunchedEffect(moved, feed) {
        if (reading || !spot.wantsMore()) return@LaunchedEffect
        val cursor = feed.cursor
        if (cursor.isEmpty() || !feed.more) return@LaunchedEffect
        reading = true
        try {
            val body = shell.read("$FEED?cursor=$cursor")
            if (body != null) feed = feed.and(Notices.read(body))
        } finally {
            reading = false
        }
    }

    Box(Modifier.fillMaxSize()) {
        AndroidView(
            factory = { context -> NoticesView(context, pictures).also { screen.value = it } },
            modifier = Modifier.fillMaxSize(),
        ) { view ->
            view.mirror(seen.rtl)
            view.speak(seen.words)
            view.asks(asking(shell, { moved += 1 }) { id -> feed = feed.readTo(id) })
            view.show(spot, feed)
        }
    }
}

/** The feed, written as the server's own path, as every other screen here writes the paths it reads. */
private const val FEED = "/v1/notifications"

/** And the call that marks a notice read. */
private const val READ = "/v1/notifications/read"

/**
 * What the screen asks of the shell: an address to open, a notice to mark read, and the way out.
 *
 * The address is built by `shared/Addresses.kt`'s own `titleAddress`, which takes a `TitleKey` --
 * a `@JvmInline value class` the Java cannot name -- so the key is wrapped here rather than the
 * address being spelled out. One statement of how an address is built, which is what
 * `check_patterns.py` protects.
 */
private fun asking(
    shell: Shell,
    onMoved: () -> Unit,
    onRead: (String) -> Unit,
): NoticesView.Asked = object : NoticesView.Asked {
    override fun chose(notice: Notices.Notice) {
        // The key is a string by the time it reaches here, and `TitleKey`'s own `require` is what
        // decides whether it is one. A key this build cannot name opens nothing rather than
        // throwing: a notice from a newer server must not be able to kill the screen it is on.
        val key = runCatching { TitleKey(notice.key) }.getOrNull() ?: return
        shell.go(titleAddress(key))
    }

    override fun read(notice: Notices.Notice) {
        // The screen has already drawn it read, which is what the phone does
        // (`phone/notifications/Notifications.kt:189-200`): the ink must not wait on the network.
        // Three things follow from here and none of them is the drawing. The server is told; the
        // count is asked for again, so the bell on the bar behind this screen agrees with the page;
        // and the feed this file holds is marked too, because the screen marked the `Spot`'s copy
        // and the next arrival would hand the unread one straight back.
        shell.send("POST", READ, """{"up_to":"${notice.id}"}""") { _, _ ->
            shell.refreshNotifications()
        }
        onRead(notice.id)
    }

    override fun leave() = shell.back()

    override fun moved() = onMoved()
}
