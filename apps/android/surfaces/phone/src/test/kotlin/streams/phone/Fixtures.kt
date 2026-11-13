package streams.phone

import contract.AgeMark
import contract.AgeTone
import contract.Badge
import contract.BadgeKind
import contract.CalendarDay
import contract.Episode
import contract.EpisodeRef
import contract.NavItem
import contract.NoticeItem
import contract.NoticeKind
import contract.Notifications
import contract.PlayState
import contract.PlayView
import contract.Profile
import contract.RouteTarget
import contract.Screen
import contract.ScreenName
import contract.Season
import contract.SeasonSummary
import contract.Section
import contract.SectionKind
import contract.Target
import contract.Tile
import contract.TitleDetail
import contract.TitleKey
import contract.contractJson
import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import streams.ports.Seen

/**
 * What a screen is handed so that it draws the thing it is for.
 *
 * A screen reading test measures what a viewer can hear. A screen handed nothing draws its waiting
 * state, and a sweep over waiting states measures the waiting states -- which is how `SweepTest`
 * came to report "Title: 0 heard" and read it as a fact about the title page rather than about the
 * fixture: `Title.kt:113` returns before it draws anything when the answer in hand is not the
 * answer for the route. Every screen here is given the answer it would have on a device with the
 * page loaded.
 *
 * The words are the real ones, read from `strings/en.json` as `WordsTest` reads them, so a label a
 * test asserts is a label a viewer hears rather than one written twice. The brand in them is a
 * made-up word for `HomeHeardTest`'s reason: a screen that drew a hardcoded name would pass a test
 * written with the real name and fail this one.
 */
internal const val BRAND = "Bramble"

/** The server a fixture names. No request leaves: `Showing` holds every ask unanswered. */
internal const val SERVER = "https://example.test"

/** Every word of the interface, from the one file that holds them, with the brand filled in. */
internal val words: Map<String, String> by lazy {
    Json.parseToJsonElement(File(repoRoot(), "strings/en.json").readText())
        .jsonObject
        .mapValues { (_, value) -> value.jsonPrimitive.content.replace("{brand}", BRAND) }
}

/** The menu, as the server builds it: the pages the chips offer and the ones the bar does. */
internal val menu: List<NavItem> = listOf(
    NavItem(id = "n-home", label = words.getValue("nav.home"), screen = ScreenName.HOME),
    NavItem(id = "n-series", label = words.getValue("nav.series"), screen = ScreenName.SERIES),
    NavItem(id = "n-films", label = words.getValue("nav.films"), screen = ScreenName.FILMS),
    NavItem(id = "n-games", label = words.getValue("nav.games"), screen = ScreenName.GAMES),
    NavItem(id = "n-new", label = words.getValue("nav.new"), screen = ScreenName.NEW),
    NavItem(id = "n-mine", label = words.getValue("nav.mine"), screen = ScreenName.MINE),
    NavItem(id = "g-drama", label = "Drama", screen = ScreenName.GENRE, genre = "drama"),
    NavItem(id = "g-comedy", label = "Comedy", screen = ScreenName.GENRE, genre = "comedy"),
)

/** The people on this account. Two, so the gate draws a grid rather than one face. */
internal val people: List<Profile> = listOf(
    Profile(hasPin = false, id = "1", kids = false, main = true, name = "Ada"),
    Profile(hasPin = false, id = "2", kids = true, main = false, name = "Grace"),
)

/**
 * A title in a row. No pictures on purpose: `Showing` plugs [streams.ports.NoPictures] in, a tile
 * with no art falls back to the words it is named by, and those words are what is being read.
 */
internal fun tile(number: Int, name: String, percent: Long? = null, badge: BadgeKind? = null) = Tile(
    name = name,
    title = TitleKey("movie:$number"),
    percent = percent,
    tags = listOf("Thriller", "Mystery"),
    badges = badge?.let { listOf(Badge(kind = it)) },
)

/** The billboard's title, which the hero section carries one of. */
internal val lead = tile(101, "Lantern Hill")

/** A row of titles, named, with more behind a cursor, so its name is a control that opens it. */
internal fun row(id: String, title: String, kind: SectionKind, from: Int) = Section(
    id = id,
    kind = kind,
    title = title,
    cursor = "page-$id-15",
    tiles = (0 until 6).map {
        tile(
            from + it,
            "$title ${it + 1}",
            percent = if (kind == SectionKind.CONTINUE) 40L else null,
        )
    },
)

/**
 * A built Home page: the hero the billboard draws, and one row of each kind this surface draws
 * differently, because a sweep that saw only the plain row would read neither the ranked numeral
 * nor the continue strip.
 */
internal val homeScreen = Screen(
    name = ScreenName.HOME,
    ttlS = 60,
    cursor = "page-home-12",
    sections = listOf(
        Section(id = "hero", kind = SectionKind.HERO, tiles = listOf(lead)),
        row("continue", "Continue Watching", SectionKind.CONTINUE, 200),
        row("top10", "Top 10 Films Today", SectionKind.RANKED, 300),
        row("trending", "Trending Now", SectionKind.ROW, 400),
    ),
)

/** A Home a viewer would meet with the page in hand: words, menu, people and the page itself. */
internal fun homeSeen(): Seen = Seen(
    brand = BRAND,
    server = SERVER,
    words = words,
    language = "en",
    route = RouteTarget(target = Target.HOME, screen = ScreenName.HOME),
    nav = menu,
    profiles = people,
    active = "1",
    screen = homeScreen,
    unreadNotifications = 3,
)

/** The facts of one film, as the title page is handed them. */
internal val filmDetail = TitleDetail(
    title = TitleKey("movie:101"),
    name = "Lantern Hill",
    overview = "A keeper of a failing light decides who is told the truth about the rocks.",
    year = 2024,
    runtimeMin = 104,
    genres = listOf("Drama", "Mystery"),
    age = AgeMark(label = "12", tone = AgeTone.YELLOW),
    inList = false,
    reminderSet = false,
    moreLikeThis = (0 until 4).map { tile(500 + it, "More like this ${it + 1}") },
)

/** The same for a series, which draws the season picker and the episode list. */
internal val seriesDetail = filmDetail.copy(
    title = TitleKey("tv:102"),
    name = "The Long Quiet",
    seasons = listOf(SeasonSummary(number = 1, episodeCount = 2)),
)

/** The episodes of that series' first season, as `/v1/titles/tv/102/seasons/1` answers. */
internal val firstSeason = Season(
    number = 1,
    name = "Season 1",
    episodes = listOf(
        Episode(number = 1, name = "The Arrival", overview = "She arrives.", runtimeMin = 48),
        Episode(number = 2, name = "The Letter", overview = "A letter arrives.", runtimeMin = 51),
    ),
)

/** That season as the body of an answer, for a test that answers the ask the page makes. */
internal fun seasonBody(): String = contractJson.encodeToString(Season.serializer(), firstSeason)

/** A title page whose answer is for the title the route names, so it draws rather than returns. */
internal fun titleSeen(of: TitleDetail = filmDetail): Seen = homeSeen().copy(
    route = RouteTarget(target = Target.TITLE, title = of.title),
    detail = of,
    screen = null,
)

/** A player with a film playing, so the controls draw and the clock says a real time. */
internal fun watchSeen(): Seen = homeSeen().copy(
    route = RouteTarget(target = Target.WATCH, title = filmDetail.title),
    screen = null,
    detail = filmDetail,
    play = PlayView(title = filmDetail.title, state = PlayState.PLAYING),
    positionMs = 1_800_000,
    lengthMs = 6_240_000,
)

/** The notifications page's route. The feed itself arrives as an answer, below. */
internal fun noticesSeen(): Seen = homeSeen().copy(
    route = RouteTarget(target = Target.SCREEN, screen = ScreenName.NOTIFICATIONS),
    screen = null,
)

/** The feed as `/v1/notifications` answers it: two notices, one unread. */
internal fun noticesBody(): String = contractJson.encodeToString(
    Notifications.serializer(),
    Notifications(
        hasMore = false,
        items = listOf(
            NoticeItem(
                date = CalendarDay("2026-11-12"),
                id = "notice-1",
                kind = NoticeKind.NEW_EPISODE,
                tile = tile(102, "The Long Quiet"),
                unread = true,
            ),
            NoticeItem(
                date = CalendarDay("2026-11-10"),
                id = "notice-2",
                kind = NoticeKind.NEW_ARRIVAL,
                tile = lead,
                unread = false,
            ),
        ),
    ),
)

/** An episode of that series, for a screen that takes one. */
internal val episodeRef = EpisodeRef(season = 1, episode = 1)
