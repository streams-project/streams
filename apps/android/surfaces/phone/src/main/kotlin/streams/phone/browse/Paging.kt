package streams.phone

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import contract.More
import contract.Screen
import contract.Section
import contract.Tile
import contract.contractJson
import streams.ports.Shell

/**
 * How a page grows. The answer a screen arrives with carries the first part only -- twelve rows of
 * fifteen titles -- and the rest is read back through the one field the contract already has for it:
 * a screen's `cursor` for more rows, a section's `cursor` for more titles of that row, and
 * `/v1/more/{cursor}`, the route `apps/web` had been the only caller of. Nothing here caps the
 * reading: a page grows until the server stops handing out a cursor, so how many rows a phone ends
 * up with is the server's answer and not a shape measured off the reference.
 *
 * Nothing here is new shape: no route, no field, no control on the screen. A page grows as the
 * viewer travels, the way the reference's does (`netflix-reference/00-gaps.md:23`).
 */

/**
 * Rows read ahead of the viewer, vertically. The phone is 936.6 dp tall and a row is 217.0 dp from
 * its name to the next, so a screen holds 4.3 rows; two screens of scroll left is 8.6 rows, so nine.
 *
 * The two-screen trigger is measured on the reference's *web* page, not its phone
 * (`netflix-reference/00-gaps.md:29`: "trigger about 2 viewports of remaining scroll (web)"). The
 * phone column of that same line records only that 23 rows arrive progressively; when the next
 * block is asked for there has not been measured. This number is therefore a borrowed one.
 */
private const val ROWS_AHEAD = 9

/**
 * Titles read ahead of the viewer, sideways. A row shows 3.29 tiles, so two rows' worth of travel
 * is 6.6 tiles, so seven. Netflix's own sideways trigger has not been measured.
 */
private const val TILES_AHEAD = 7

/**
 * Where a page's reading is up to: the cursor it would ask for next, and whether it is asking now.
 *
 * The two are one value on purpose. Held apart, the end of a read wrote them one after the other,
 * and a draw that fell between the two writes saw "not asking" beside a cursor that had already been
 * spent and asked the server for it a second time. That was watched happening on a device: two
 * requests for `page-...-36` and two for `-48` in one scroll to the bottom.
 */
private data class Reading(val cursor: String?, val asking: Boolean = false)

/**
 * Which answer a page's read parts belong to: the route, the cursor that answer carried and how many
 * sections it carried. A page built again gets a new stamp and starts its paging again.
 *
 * Both halves of the reading are keyed on it -- the page's own parts and each row's depth -- because
 * they have to be thrown away together. Keyed apart, a refreshed Home kept every row's old cursor
 * while the rows themselves went back to the fifteen tiles the answer carries, so a row either
 * stopped growing or asked again for a cursor whose tiles it was already showing.
 */
fun pageStamp(key: String, screen: Screen?): String =
    "$key|${screen?.cursor}|${screen?.sections?.size ?: 0}"

/** The sections of this page as the viewer can reach them: the answer's, plus every part read since. */
@Composable
fun rememberPage(shell: Shell, screen: Screen?, stamp: String, list: LazyListState): List<Section> {
    val base = screen?.sections.orEmpty()
    var parts by remember(stamp) { mutableStateOf(emptyList<Section>()) }
    var reading by remember(stamp) { mutableStateOf(Reading(screen?.cursor)) }

    val near by remember(list) {
        derivedStateOf {
            val held = list.layoutInfo.totalItemsCount
            val last = list.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0
            held > 0 && last >= held - ROWS_AHEAD
        }
    }

    LaunchedEffect(stamp, near, reading) {
        val next = reading.cursor
        if (!near || next == null || reading.asking) return@LaunchedEffect
        reading = Reading(next, asking = true)
        shell.ask("/v1/more/$next") { status, text ->
            val more = read(status, text)
            if (more != null) parts = parts + more.sections.orEmpty()
            // A part that does not arrive stops this page. Asking the same cursor again on the next
            // frame would be a loop against a server that has already said no.
            reading = Reading(more?.cursor)
        }
    }

    // Two parts could name one section if a page were built again mid-read, and a lazy list with
    // the same key twice throws, so the id decides.
    return remember(base, parts) { (base + parts).distinctBy { it.id } }
}

/**
 * How deep each row of this page has been read. A row is drawn inside the page's lazy column and is
 * disposed when it scrolls off, so its depth is held here, at the page: a row the viewer comes back
 * to is as deep as they left it.
 */
@Stable
class Depth internal constructor(private val shell: Shell) {
    private val parts = mutableStateMapOf<String, List<Tile>>()

    /**
     * Where each row's reading is up to. One value a row, as a page's is, and every touch of the map
     * is locked: a row asks from the screen's own thread and is answered from the network's.
     */
    private val reading = mutableMapOf<String, Reading>()

    /** Every title of this row the viewer can reach. */
    fun tiles(section: Section): List<Tile> {
        val got = parts[section.id] ?: return section.tiles.orEmpty()
        return (section.tiles.orEmpty() + got).distinctBy { it.title }
    }

    /** The row says how far the viewer has flung; the next part is read once the end is near. */
    fun reach(section: Section, held: Int, lastVisible: Int) {
        if (held <= 0 || lastVisible < held - TILES_AHEAD) return
        val id = section.id
        val next = synchronized(reading) {
            val now = reading[id] ?: Reading(section.cursor)
            if (now.asking || now.cursor == null) return
            reading[id] = now.copy(asking = true)
            now.cursor
        }
        shell.ask("/v1/more/$next") { status, text ->
            val more = read(status, text)
            if (more != null) parts[id] = parts[id].orEmpty() + more.tiles.orEmpty()
            // As with a page: what did not arrive is not asked for twice.
            synchronized(reading) { reading[id] = Reading(more?.cursor) }
        }
    }
}

/**
 * One depth per answer. Leaving a page and coming back reads it again, and a page rebuilt under the
 * viewer drops the depths with the parts, since the stamp names the answer both were read from.
 */
@Composable
fun rememberDepth(shell: Shell, stamp: String): Depth = remember(shell, stamp) { Depth(shell) }

/** A part, or nothing at all: a refusal and a body that does not parse are the same answer here. */
private fun read(status: Int, text: String): More? {
    if (status !in 200..299) return null
    return runCatching { contractJson.decodeFromString(More.serializer(), text) }.getOrNull()
}
