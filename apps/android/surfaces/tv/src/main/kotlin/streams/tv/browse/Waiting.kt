package streams.tv

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import streams.ports.Seen

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
 */
@Composable
fun Waiting(seen: Seen) {
    val tabs = tabsOf(seen.nav)
    val current = seen.route?.screen?.value ?: "home"
    val person = seen.profiles.firstOrNull { it.id == seen.active } ?: seen.profiles.firstOrNull()
    Box(Modifier.fillMaxSize().background(Color.Black)) {
        // The destination, lit, as the reference lights it. Nothing is lit when nothing is known.
        if (tabs.isNotEmpty()) {
            TopBar(
                tabs = tabs,
                active = tabs.indexOfFirst { it.screen.value == current },
                focus = -1,
                person = person,
                server = seen.server,
                words = seen.words,
            )
        }
    }
}
