package streams.tv

import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics

/**
 * Words a screen reader can read, on a surface that draws its own focus.
 *
 * Measured on a Google TV Streamer on 2026-11-11: `uiautomator dump` on this surface returned a
 * tree of nested `android.view.View` nodes with `text=""` and `content-desc=""` on every one, and
 * one leaf of 48 x 48 dp (notes/206-what-each-surface-does-with-no-server-2026-11-11.md). A person
 * using a screen reader heard nothing, and no automated check could read a screen either. This is
 * not a parity question: our own capture found Netflix's television app exposes one node for the
 * whole interface (netflix-reference/tv/00-netflix-on-the-tv-measured.md), and a control a person
 * cannot hear is a defect whatever anyone else ships.
 *
 * A television makes this worse than a phone does. A phone can be explored by touch, so an
 * unlabelled control is at least findable; a remote has nothing to explore with, so an unlabelled
 * control is unreachable.
 *
 * Why the surface needs this file at all: every screen here keeps its focus as plain state and
 * paints it (`Remote.kt`), so the toolkit's one real focusable is the whole-screen box. There is no
 * per-control focus for a screen reader to follow. So two things are done:
 *  * every drawn part that means something carries its words ([says]), and the part the remote is
 *    on carries `selected`, so the tree can be read and explored;
 *  * the screen box says what the remote is on and re-says it as the remote moves
 *    (`Modifier.remote`'s `says`), which is what a person actually hears while pressing a key.
 *
 * The words themselves are never written here. They come from `strings/{lang}.json` through
 * `seen.words`, like every other word in this project, or from the catalogue (a title's name is
 * already the viewer's language). A key this surface asks for and the strings do not hold leaves
 * the part unlabelled rather than putting English in the code.
 */

/** What a part means, said in the viewer's language. Nothing is said when there are no words for it. */
fun Modifier.says(words: String?): Modifier =
    if (words.isNullOrEmpty()) this else semantics { contentDescription = words }

/**
 * A part the remote can stop on: what it means, and whether the remote is on it now. `on` is what
 * a screen reader says as "selected", which is the only thing that tells a person where they are
 * on a screen whose focus is painted rather than held by the toolkit.
 */
fun Modifier.says(words: String?, on: Boolean): Modifier = semantics {
    role = Role.Button
    selected = on
    if (!words.isNullOrEmpty()) contentDescription = words
}

/**
 * A part whose own drawn words are its label: a pill, a tab, a key, a line of a menu. The words
 * inside it are read as one label instead of as loose text, and `on` says whether the remote is on
 * it. Nothing is written here, so the part keeps saying whatever the strings gave it to draw.
 */
fun Modifier.saysItsWords(on: Boolean): Modifier = semantics(mergeDescendants = true) {
    role = Role.Button
    selected = on
}

/**
 * A picture that repeats what is already said beside it: artwork behind a name, a mark, an arrow
 * drawn next to the thing it points at. A screen reader skips it instead of announcing an unnamed
 * image. The toolkit's own way of saying this is `contentDescription = null` on an `Icon` or an
 * `AsyncImage`, and the call sites that pass `null` mean exactly this; this is for the boxes and
 * rows that hold them.
 */
fun Modifier.decoration(): Modifier = clearAndSetSemantics {}

/**
 * Two things a person hears one after the other, as one label: a title and the row it sits in, a
 * name and its state. The break is a line end rather than a word or a comma, so no language's
 * punctuation or word order is decided in the code.
 */
fun sayBoth(first: String?, second: String?): String =
    listOf(first, second).filter { !it.isNullOrEmpty() }.joinToString("\n")
