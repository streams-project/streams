package streams.tv

import contract.ImageRole
import contract.NavItem
import contract.Profile
import contract.Section
import contract.Tile
import contract.TitleDetail
import streams.ports.Painted
import streams.ports.Picture
import streams.ports.Shell
import streams.ports.Want
import streams.shared.tall
import streams.shared.wide
import streams.tv.browse.Art
import streams.tv.browse.Feed

/**
 * The one place a browse page reads the contract, and the only Kotlin left in this package's
 * drawing path.
 *
 * ## Why it exists, which is a measurement and not a preference
 * Java source cannot read the contract. Checked with `javap` against
 * `apps/android/ports/build/classes/kotlin/main` on 2026-11-12: `contract.Tile`'s title getter is
 * `getTitle-syqQ2Kk()`, `contract.Section`'s kind is `getKind-TNS5Z4k()`, `contract.NavItem`'s
 * screen is `getScreen-mbV6UQU()` and `contract.Image`'s role is `getRole-pIcKOd0()`, because
 * `gen/src/kotlin.rs:95-114` emits every open word type as a `@JvmInline value class`. A hyphen is
 * not a legal character in a Java identifier.
 *
 * Note 210 section 4 measured the whole of it -- 69 classes, 184 mangled getters -- called it the
 * gate on the entire programme, and made it wave 0's package S1. S1 has not landed in the Android
 * build: `platform/contract/generated/types/Contract.java` is generated and on disk with protobuf
 * shaped accessors, and `ports/build.gradle.kts:14` adds that folder as a *Kotlin* source directory,
 * so the only class built from it is `ContractKt`. Until a module compiles that file, a Java screen
 * reads the contract through Kotlin or not at all.
 *
 * ## What that buys and what it costs
 * It buys everything that can be wrong without looking wrong: the key map, the focus memory, the
 * measured geometry, what a screen reader is handed and the drawing are all Java, all on wave 1's
 * foundation, and all under unit tests that need no device. It costs this file, which is 1 of the 6
 * files this package holds, and which is deleted when S1 lands -- at which point `Feed` is filled in
 * Java from `Contract.Tile.getTitle()` with no line of this file surviving.
 *
 * ## It constructs no contract value
 * Note 210 section 4's rule for making note 204's protobuf swap mechanical is that a surface reads a
 * contract value through an accessor and never constructs one. Nothing here constructs: every value
 * is read off what the core published, and what is handed to Java is `String`, `Int` and `Boolean`.
 */

/**
 * The pictures of a browse page, from the `Pictures` port.
 *
 * This is that port's first consumer anywhere in the application. Note 210 measured zero surface
 * calls to `shell.pictures` and recorded two consequences: `check_rings.py:117`'s exemption for
 * `io.coil-kt.coil3:coil-compose` stays load bearing while the port has no consumers, and the
 * television surface has no disk cache at all, because `shared/Loader.kt:21-23` never installs the
 * process loader on a television and the port's adapter is what owns the 64 MB cache.
 *
 * The adaptation is three lines and exists because `Pictures.fetch` takes a Kotlin function type and
 * `Want` has four parameters with defaults; wave 0's package S2 turns both into Java callable
 * shapes, and it owns `apps/android/ports`, which this package does not.
 */
internal class ShellArt(private val shell: Shell) : Art.Source {
    override fun fetch(url: String, wide: Int, high: Int, done: Art.Done): Art.Stop {
        val stop = shell.pictures.fetch(Want(url, wide, high, false)) { answer: Picture ->
            // `Painted.picture` is an `android.graphics.Bitmap`, which `CoilPictures.kt:86-87`
            // states as the port's bargain for this family. Anything else is no picture, and the
            // caller draws its plate instead of waiting for one that is not coming.
            done.painted((answer as? Painted)?.picture as? android.graphics.Bitmap)
        }
        return Art.Stop { stop.stop() }
    }
}

/** The picture for a tall place and for a wide one, as `shared/Art.kt:19-22` chooses them. */
private fun cardOf(tile: Tile, words: Map<String, String>): Feed.Card = Feed.Card(
    tile.title.value,
    tile.name,
    tall(tile.images).orEmpty(),
    wide(tile.images).orEmpty(),
    tile.badges.orEmpty().map { badgeLabel(it, words) }.filter { it.isNotEmpty() }.toTypedArray(),
)

private fun rowOf(section: Section, words: Map<String, String>): Feed.Row = Feed.Row(
    section.id,
    section.title.orEmpty(),
    section.tiles.orEmpty().map { cardOf(it, words) },
)

/**
 * Everything a page of rows draws, flattened.
 *
 * The three answers that decide what a page may draw at all stay where they were -- `rowsOn`,
 * `heroOn` and `answerIsThisPage` in `browse/Page.kt`, which `src/test/kotlin/streams/tv/PageTest.kt`
 * checks -- because all three read `SectionKind`, which is one of the mangled types. They are passed
 * in rather than called here so that this function has one job.
 */
internal fun feedOf(
    tabs: List<NavItem>,
    active: Int,
    person: Profile?,
    server: String,
    rows: List<Section>,
    lead: Tile?,
    words: Map<String, String>,
    empty: String,
): Feed = Feed(
    tabs.map { Feed.Tab(it.label, it.screen.value) },
    active,
    person?.let {
        Feed.Who(
            it.name,
            // `shared/Face.kt:35`: a picture is a path on the server or a whole address, and
            // anything else is no picture. Made absolute here, once, so the Java holds no rule
            // about what a server address looks like.
            it.avatar?.let { a ->
                when {
                    a.startsWith("/") -> server + a
                    a.startsWith("http") -> a
                    else -> ""
                }
            }.orEmpty(),
            it.kids,
        )
    },
    server,
    rows.map { rowOf(it, words) },
    lead?.let { cardOf(it, words) },
    empty,
)

/**
 * What is known about one title: its facts, its story, and its name drawn as a picture.
 *
 * The facts line is built by `parts/Facts.kt`'s own `facts`, not restated here, and that is note 210
 * section 3's rule: the CLDR plural table (`shared/Theme.kt:100-128`), the `{name}` filling with its
 * bidirectional isolates (`:66-67`) and the Arabic-Indic digits (`:86-96`) are called and never
 * copied, because a second statement of a language's plural forms is a second place for them to be
 * wrong. `platform/tools/check_patterns.py` exists to protect exactly that.
 */
internal fun storyOf(detail: TitleDetail?, lead: Tile?, words: Map<String, String>): Feed.Story? {
    if (detail == null) return null
    val logo = (lead?.images.orEmpty() + detail.images.orEmpty())
        .firstOrNull { it.role == ImageRole.LOGO }
        ?.url
    return Feed.Story(facts(detail, words).toTypedArray(), detail.overview.orEmpty(), logo.orEmpty())
}
