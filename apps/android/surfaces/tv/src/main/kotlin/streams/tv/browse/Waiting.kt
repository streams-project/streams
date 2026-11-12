package streams.tv

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.foundation.layout.fillMaxSize
import streams.ports.Seen
import streams.ports.Shell
import streams.tv.browse.WaitingView

/**
 * What a television page shows while it has nothing to show.
 *
 * Until this, nothing: `Television` drew `Unit` for a route with no screen behind it yet, and
 * `Browse` returned without drawing. On a Google TV Streamer with no server reachable that was a
 * black screen for about fourteen seconds and then one white rounded shape in the middle of it,
 * measured on 2026-11-11 and written up in
 * notes/206-what-each-surface-does-with-no-server-2026-11-11.md.
 *
 * What is drawn instead is the measured reference behaviour and nothing more: Netflix's television
 * app, between tabs, draws the top bar with the destination tab already lit and the body pure
 * black, for 1.08-2.38 s going Home to Series and 3.96-4.67 s going Films to Games, with no
 * skeleton, no shimmer and no old content held (netflix-reference/00-gaps.md:133). The touch
 * surface of this project settled the same question for itself by drawing placeholder rows, which
 * is its own reference's behaviour on a phone (00-gaps.md:132) and not this one's.
 *
 * There is no wording here, invented or otherwise. A page that cannot say where it is - no tabs
 * yet, which is what a run with no server looks like - draws the black body alone rather than a
 * bar with nothing in it.
 *
 * The drawing is `browse/WaitingView.java`. What is left here is the same data seam `Browse` keeps,
 * for the same reason: `it.screen.value` on a `NavItem` is `getScreen-mbV6UQU()` in the bytecode and
 * cannot be written in Java (`browse/Reading.kt`).
 */
@Composable
fun Waiting(shell: Shell, seen: Seen) {
    val tabs = tabsOf(seen.nav)
    val current = seen.route?.screen?.value ?: "home"
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    // The face and the mark are pictures, so this screen needs the port even though it has nothing
    // else on it. `shell` was not passed to the Compose version and both its call sites had one;
    // without it the bar draws a plate where the profile's picture goes, which is a behaviour this
    // port is not allowed to lose.
    val pictures = remember(shell) { ShellArt(shell) }
    val feed = feedOf(
        tabs = tabs,
        active = tabs.indexOfFirst { it.screen.value == current },
        person = person,
        server = seen.server,
        rows = emptyList(),
        lead = null,
        words = seen.words,
        empty = "",
    )
    AndroidView(
        factory = { context -> WaitingView(context, pictures) },
        modifier = Modifier.fillMaxSize(),
    ) { view ->
        view.mirror(seen.rtl)
        view.show(feed, seen.words)
    }
}
