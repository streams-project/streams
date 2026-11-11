package streams.tv

import contract.Section
import contract.SectionKind
import contract.Tile

/**
 * Which rows a page of the television surface draws, as plain functions of what it was given.
 *
 * This is the one piece of `Browse` that can be wrong without anything looking wrong: the screen
 * the core holds is not always the screen the route asked for, and when it is not, a page draws
 * another page's rows under its own name. A reviewer caught exactly that on the Games tab on
 * 2026-11-11 - Games has no rows, the core never asks for it and never clears what it holds
 * (platform/core/src/lib.rs:410-417), so the tab drew whatever the tab before it was showing.
 *
 * It lives apart from the drawing so it can be checked without a screen, a device or a toolkit.
 * `src/test/kotlin/streams/tv/PageTest.kt` is that check.
 */

/** The tab that has nothing to show yet. The server builds no rows for it. */
private const val GAMES = "games"

/** The section kinds this surface draws as a row of tiles. Anything else is not a row. */
private val RowKinds = setOf(SectionKind.ROW, SectionKind.RANKED, SectionKind.CONTINUE, SectionKind.GRID)

/**
 * Whether the answer in hand belongs to the page the viewer is on.
 *
 * Home and the kids home name no screen on the route, so there the answer's own name stands for it.
 * Games is never the answer's name, so it never owns one.
 */
fun answerIsThisPage(current: String, asked: String?, answerName: String): Boolean =
    current != GAMES && (asked == null || asked == answerName)

/**
 * The rows a page draws: the answer's own rows, and the rows read past the first twelve when the
 * answer is this page's. A page with nothing of its own draws nothing, whatever is held.
 */
fun rowsOn(
    current: String,
    asked: String?,
    answerName: String,
    answer: List<Section>,
    extra: List<Section>,
): List<Section> {
    val held = filled(if (current == GAMES) emptyList() else answer)
    val all = if (answerIsThisPage(current, asked, answerName)) held + filled(extra) else held
    return all.filter { it.kind in RowKinds }
}

/** The title the top of a page opens with, or nothing when the page has no hero of its own. */
fun heroOn(current: String, answer: List<Section>): Tile? =
    filled(if (current == GAMES) emptyList() else answer)
        .firstOrNull { it.kind == SectionKind.HERO }
        ?.tiles
        ?.firstOrNull()

/** A section with no tiles is not drawn: a row with no titles in it is an empty line under a name. */
private fun filled(sections: List<Section>): List<Section> = sections.filter { !it.tiles.isNullOrEmpty() }
