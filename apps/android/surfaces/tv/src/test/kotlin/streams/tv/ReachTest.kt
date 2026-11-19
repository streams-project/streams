package streams.tv

import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onRoot
import contract.ActionCommand
import contract.Address
import contract.CalendarDay
import contract.CommandRemind
import contract.NavItem
import contract.RouteTarget
import contract.Screen
import contract.ScreenName
import contract.Section
import contract.SectionKind
import contract.Target
import contract.Tile
import contract.TitleDetail
import contract.TitleKey
import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import streams.ports.Seen
import streams.ports.Shell
import streams.shared.R
import streams.shared.fill
import streams.shared.number
import streams.shared.titleAddress
import streams.tv.browse.BrowseView
import streams.tv.browse.Feed
import streams.tv.browse.Place
import streams.tv.notifications.NoticesView
import streams.tv.reminder.ReminderView
import streams.tv.views.Word

/**
 * Can a viewer get to the two screens that were built and left unrouted, and get back.
 *
 * Both were finished and tested on their own -- the notifications feed in `notifications/`, the
 * reminder panel in `reminder/` -- and neither could be opened from anywhere in the app. A screen
 * with tests and no way in is a screen no viewer has, and nothing in those tests could say so:
 * they press the state machine, and the state machine was never wired to anything a remote
 * reaches.
 *
 * The screens are composed as they ship and the keys are dispatched at the window, which is where a
 * remote's keys really arrive. That matters here more than usual: both screens are `View`s inside
 * an `AndroidView`, so a key sent through Compose's own test harness would never reach them and a
 * test written that way would pass for a panel wired to nothing.
 *
 * `@Config(sdk = 34)` for `views/Heard.java`'s reason: Robolectric cannot stand up this project's
 * `compileSdk 37`. `@GraphicsMode(NATIVE)` because the opening bakes a real bitmap
 * (`shared/Opening.kt`'s `bake`), and the legacy mode hands it null.
 */
@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class ReachTest {
    @get:Rule val compose = createAndroidComposeRule<ComponentActivity>()

    /** The real words, so a line is read as a viewer hears it and not as a literal written here. */
    private val book: Map<String, String> by lazy {
        Json.parseToJsonElement(File(repoRoot(), "strings/en.json").readText()).jsonObject
            .mapValues { (_, word) -> word.jsonPrimitive.content }
    }

    /** One key, dispatched at the window, which is where a remote's keys arrive on a device. */
    private fun press(code: Int) {
        val window = compose.activity.window
        window.decorView.dispatchKeyEvent(
            android.view.KeyEvent(android.view.KeyEvent.ACTION_DOWN, code),
        )
        compose.waitForIdle()
    }

    private fun down() = press(android.view.KeyEvent.KEYCODE_DPAD_DOWN)
    private fun up() = press(android.view.KeyEvent.KEYCODE_DPAD_UP)
    private fun right() = press(android.view.KeyEvent.KEYCODE_DPAD_RIGHT)
    private fun ok() = press(android.view.KeyEvent.KEYCODE_DPAD_CENTER)

    /** Everything a reader would stop on in the Compose half of the screen, by its words. */
    private fun heard(): List<String> =
        HeardScreen.words(compose.onRoot().fetchSemanticsNode()).map { it.label }

    /** The reminder panel, if it is on screen. It is a `View`, so it is looked for as one. */
    private fun panel(): ReminderView? = found(compose.activity.window.decorView)

    private fun found(view: View): ReminderView? {
        if (view is ReminderView) return view
        if (view is ViewGroup) {
            for (at in 0 until view.childCount) {
                found(view.getChildAt(at))?.let { return it }
            }
        }
        return null
    }

    // --- the notifications feed -----------------------------------------------------------------

    /**
     * A route to the feed draws the feed.
     *
     * Read through what the screen asks the server for, and not through the words on it: the feed
     * is drawn by a `View` and a Compose semantics tree does not reach into one.
     * `notifications/NoticesPage.kt` reads `GET /v1/notifications` the moment it is composed and
     * nothing else on this surface reads that path, so the ask is the screen.
     *
     * Without the case in `app/Television.kt` this route falls to `BrowseOrProblem`, which draws a
     * page of rows for an answer the server never sends for this screen: a bar, the empty-page
     * word, and no feed.
     */
    @Test
    fun `the route to the feed draws the feed`() {
        val shell = Watching(signedIn(ScreenName.NOTIFICATIONS))
        compose.setContent { Television(shell) }
        compose.waitForIdle()
        assertTrue(
            "nothing on the notifications route asked the server for the feed: ${shell.asks}",
            shell.asks.contains("/v1/notifications"),
        )
    }

    /**
     * And another screen's route does not, so the test above is about this route and not about
     * every route.
     */
    @Test
    fun `another route does not read the feed`() {
        val shell = Watching(signedIn(ScreenName.HELP))
        compose.setContent { Television(shell) }
        compose.waitForIdle()
        assertFalse(
            "a route that is not the feed read the feed: ${shell.asks}",
            shell.asks.contains("/v1/notifications"),
        )
    }

    // --- the reminder panel ---------------------------------------------------------------------

    /** A title that has not arrived offers a reminder. */
    @Test
    fun `a title still to come offers a reminder`() {
        titlePage(coming("2026-12-01"))
        assertTrue("a coming title does not offer a reminder: ${heard()}", heard().contains(book["remind.me"]))
    }

    /**
     * And a title already here does not. `available_from` is the server's own answer to "is it
     * here": the condition the phone's button hangs on and the one `reminder/Coming.known()`
     * states.
     */
    @Test
    fun `a title already here does not offer a reminder`() {
        titlePage(coming(null))
        assertFalse("a title already here offers a reminder: ${heard()}", heard().contains(book["remind.me"]))
    }

    /**
     * The line opens the panel, the panel tells the server which way it is turning, and the line
     * under it comes back to the title. Plainly: a viewer can get there and get back.
     */
    @Test
    fun `the reminder is reached from the title page and comes back`() {
        val shell = titlePage(coming("2026-12-01"))
        assertNull("the panel is on screen before anybody asked for it", panel())
        // The menu opens on Play, and the reminder is the last line: Play, My List, Remind Me.
        down()
        down()
        ok()
        assertNotNull("the line did not open the panel", panel())
        assertEquals("opening the panel told the server something", emptyList<Any>(), shell.acted)

        // The panel opens on the line that leaves, so Up reaches the one that turns.
        up()
        ok()
        assertEquals(
            "the panel did not tell the server to set the reminder",
            listOf(ActionCommand(CommandRemind(on = true, title = TitleKey("movie:1")))),
            shell.acted,
        )

        // Down again onto the line that leaves, which comes back to the title's own menu.
        down()
        ok()
        assertNull("the panel did not close", panel())
        assertTrue("the panel did not come back to the title: ${heard()}",
            heard().contains(book["common.play"]))
        assertEquals("leaving the panel left the title page as well", emptyList<String>(), shell.went)
    }

    /**
     * Back leaves the panel too, and leaves it without leaving the title page under it.
     *
     * Worth its own test because Back is the key with two owners here: the panel is a `View` that
     * answers it, and `app/Television.kt` puts a `BackHandler` over every title. A Back the panel
     * did not consume would close the panel and the title in one press.
     */
    @Test
    fun `back leaves the panel and not the title under it`() {
        val shell = titlePage(coming("2026-12-01"))
        down()
        down()
        ok()
        assertNotNull(panel())
        press(android.view.KeyEvent.KEYCODE_BACK)
        assertNull("Back did not close the panel", panel())
        assertEquals("Back left the title page as well", emptyList<String>(), shell.went)
    }

    /**
     * The line that opened the panel says what the viewer did there when they come back to it.
     *
     * Without this a viewer sets a reminder, leaves the panel, and reads "Remind Me" on the title
     * they have just asked about, because `seen.detail` is still the answer the server gave before
     * the command. The page keeps the answer the viewer gave, which is the shape `in_list` already
     * has on this page.
     */
    @Test
    fun `the line says the reminder is set once the viewer has set it`() {
        titlePage(coming("2026-12-01"))
        down()
        down()
        ok()
        up()
        ok()
        down()
        ok()
        assertTrue(
            "the line still offers to set a reminder that is already set: ${heard()}",
            heard().contains(book["remind.set"]),
        )
    }

    // --- the way in: the bell on the top bar ----------------------------------------------------

    /**
     * The bell is reachable with a remote and OK on it opens the feed.
     *
     * The whole way in, pressed on the page as it ships. `PlaceTest` presses the state machine and
     * proves it answers `NOTIFICATIONS`; this presses the page and proves the answer reaches the
     * shell. The two are a seam apart (`browse/Browse.kt`'s `chose`), and a mutation that emptied
     * that seam was invisible to every test until this one.
     */
    @Test
    fun `the bell is reached with a remote and opens the feed`() {
        val shell = browsePage(unread = 4)
        // Up from the first row to the bar, then right past both tabs onto the bell.
        up()
        right()
        right()
        ok()
        assertEquals(
            "the bell did not open the feed",
            listOf(Address.NOTIFICATIONS),
            shell.went,
        )
    }

    /**
     * And the count it says is the server's.
     *
     * Read off the page's own live region, which is `Place.says` and is what a person using a
     * screen reader hears as the remote arrives. The number is the one on `seen`, which
     * `core/Host.kt` fills from `GET /v1/notifications/count`; a page that passed its bar a count
     * of its own, or none, says something else here.
     */
    @Test
    fun `the count the bar says is the one the server sent`() {
        browsePage(unread = 4)
        up()
        right()
        right()
        val said = browseView()?.contentDescription?.toString().orEmpty()
        assertTrue(
            "the bar does not say the count the server sent, it says [$said]",
            said.contains(book["say.notifications_unread"].orEmpty().fill("n", 4)),
        )
    }

    /** With nothing waiting it says the screen's own name and no count at all. */
    @Test
    fun `with nothing waiting the bell says the screens own name`() {
        browsePage(unread = 0)
        up()
        right()
        right()
        val said = browseView()?.contentDescription?.toString().orEmpty()
        assertEquals("the bell said something other than the screen's name", book["notifications.title"], said)
    }

    /**
     * Leaving the bar and coming back to it lands on the bell, and not at the start of the bar.
     *
     * A page keeps where the viewer was for the run of the app (note 187), and the bell is a place
     * on that bar like any other. The round trip itself is the core's, so what is pressed here is
     * the half this surface owns: a fresh answer for the same page, which is what arrives while the
     * viewer is away.
     */
    @Test
    fun `a fresh answer leaves the remote on the bell`() {
        val place = Place(
            Feed(
                listOf(Feed.Tab("Home", "home"), Feed.Tab("Series", "series")),
                0, null, "", emptyList(), null, "", 4,
            ),
            0,
        )
        assertTrue(place.press(Word.RIGHT))
        assertTrue(place.press(Word.RIGHT))
        assertEquals(Feed.BELL_TAB, place.tab())
        place.arrived(
            Feed(
                listOf(Feed.Tab("Home", "home"), Feed.Tab("Series", "series")),
                0, null, "", emptyList(), null, "", 7,
            ),
        )
        assertEquals("a fresh answer moved the remote off the bell", Feed.BELL_TAB, place.tab())
    }

    /**
     * The bar of a page still loading carries the count too.
     *
     * `browse/Waiting.kt` builds its own `Feed`, and the line that hands it the count was invisible
     * to every test: `Waiting` is the page a viewer sees for as long as the server takes, and the
     * bar it draws is the one that will stay. Read off the drawn number itself, because a bar that
     * is only waiting has the remote on nothing and says nothing.
     */
    @Test
    fun `the bar of a page still loading carries the server's count`() {
        waitingPage(unread = 4)
        assertTrue(
            "the loading bar does not draw the count the server sent: ${drawnTexts()}",
            drawnTexts().contains(number(4)),
        )
    }

    /** And with nothing waiting it draws no number at all, not a nought. */
    @Test
    fun `a loading bar with nothing waiting draws no count`() {
        waitingPage(unread = 0)
        assertFalse(
            "the loading bar draws a count of none: ${drawnTexts()}",
            drawnTexts().contains(number(0)),
        )
    }

    // --- the feed, once it is open --------------------------------------------------------------

    /**
     * Back leaves the feed, which is the way back from the bell.
     *
     * `NoticesView` answers Back by asking its host to leave, and the host here is
     * `notifications/NoticesPage.kt`, whose `leave` is the one line that turns that into the
     * shell's own step back. The Java is tested on its own; this is the seam under it.
     */
    @Test
    fun `back leaves the feed`() {
        val shell = feedPage()
        press(android.view.KeyEvent.KEYCODE_BACK)
        assertEquals("Back did not step back from the feed", listOf("back"), shell.inner.went)
    }

    /**
     * OK on a notice opens its title, tells the server it was read, and asks for the count again
     * once the server has answered, so the bell on the bar behind agrees with the page.
     */
    @Test
    fun `a notice opens its title and is marked read`() {
        val shell = feedPage()
        ok()
        assertEquals(
            "OK on a notice did not open its title",
            listOf(titleAddress(TitleKey("movie:1"))),
            shell.inner.went,
        )
        assertEquals(
            "the server was not told which notice was read",
            listOf("POST /v1/notifications/read {\"up_to\":\"n1\"}"),
            shell.sent,
        )
        assertEquals("the count was asked for before the server had answered", 0, shell.refreshed)
        compose.runOnIdle { shell.replies.single()(200, "") }
        assertEquals("the count was not asked for again once the notice was read", 1, shell.refreshed)
    }

    /**
     * The feed reads its next page as the remote walks toward the end of what it holds, and a
     * notice marked read is still read when that page arrives.
     *
     * Two things the page keeps and the screen cannot: `moved` is what makes the paging read follow
     * the remote, and `onRead` marks the feed the page holds, because the screen marked only its
     * own copy and the next arrival would hand the unread notice straight back.
     */
    @Test
    fun `the next page follows the remote and does not unread what was read`() {
        val shell = feedPage()
        assertEquals("the feed read a second page before the remote went anywhere", 1, shell.inner.asks.size)
        ok()
        var presses = 0
        while (!shell.inner.asks.contains("/v1/notifications?cursor=c2") && presses < 12) {
            down()
            presses += 1
        }
        assertTrue(
            "walking down the feed never read the next page: ${shell.inner.asks}",
            shell.inner.asks.contains("/v1/notifications?cursor=c2"),
        )
        compose.runOnIdle { shell.inner.answer(200, secondPage) }
        compose.waitForIdle()
        val held = noticesView()!!.spot().feed()
        assertEquals("the next page was not added to the feed", 14, held.size())
        assertFalse("a notice that was read came back unread with the next page", held.at(0).unread)
    }

    // --- the reminder panel, the parts the title page keeps ---------------------------------------

    /** A title the viewer has already asked about says so the moment it opens. */
    @Test
    fun `a reminder already set is said on arrival`() {
        titlePage(coming("2026-12-01", reminded = true))
        assertTrue("the line does not say the reminder is set: ${heard()}", heard().contains(book["remind.set"]))
        assertFalse("the line still offers a reminder: ${heard()}", heard().contains(book["remind.me"]))
    }

    /** The tick once asked and the bell to ask: the picture is not words, so it is read as the pair. */
    @Test
    fun `the reminder line is a bell to ask and a tick once asked`() {
        assertEquals(R.drawable.icon_bell, remindLine(book, false).second)
        assertEquals(R.drawable.icon_check, remindLine(book, true).second)
        assertEquals(book["remind.me"], remindLine(book, false).first)
        assertEquals(book["remind.set"], remindLine(book, true).first)
    }

    /**
     * A reminder that is already set is offered to be turned off, and not on a second time.
     *
     * The panel is told the page's answer; a panel built with `false` for it sends the command the
     * wrong way for every title the viewer has already asked about.
     */
    @Test
    fun `the panel of a set reminder turns it off`() {
        val shell = titlePage(coming("2026-12-01", reminded = true))
        down()
        down()
        ok()
        up()
        ok()
        assertEquals(
            listOf(ActionCommand(CommandRemind(on = false, title = TitleKey("movie:1")))),
            shell.acted,
        )
    }

    /**
     * The remote stays on the line it turned.
     *
     * The answer that comes back from setting a reminder is this panel with its words the other
     * way, and a panel rebuilt for it would put the remote back on the line that leaves: a second
     * OK would close the panel instead of turning the reminder off again.
     */
    @Test
    fun `the remote stays on the line it turned`() {
        val shell = titlePage(coming("2026-12-01"))
        down()
        down()
        ok()
        up()
        ok()
        ok()
        assertEquals(
            listOf(
                ActionCommand(CommandRemind(on = true, title = TitleKey("movie:1"))),
                ActionCommand(CommandRemind(on = false, title = TitleKey("movie:1"))),
            ),
            shell.acted,
        )
        assertNotNull("the second OK closed the panel", panel())
    }

    // --- what the screens are made of -----------------------------------------------------------

    private fun signedIn(screen: ScreenName) = Seen(
        words = book,
        server = "http://box",
        active = "p1",
        route = RouteTarget(target = Target.SCREEN, screen = screen),
    )

    private fun coming(day: String?, reminded: Boolean? = null) = TitleDetail(
        name = "Dune",
        title = TitleKey("movie:1"),
        availableFrom = day?.let { CalendarDay(it) },
        reminderSet = reminded,
    )

    /**
     * What a shell can be asked and cannot answer in a screen reading test: `send`. Everything else
     * is [Watching]'s, so the feed's two reads are recorded where every other test looks for them.
     */
    private class Telling(val inner: Watching) : Shell by inner {
        val sent = mutableListOf<String>()
        val replies = mutableListOf<(Int, String) -> Unit>()
        var refreshed = 0

        override fun send(method: String, path: String, body: String, reply: (Int, String) -> Unit) {
            sent += "$method $path $body"
            replies += reply
        }

        override fun refreshNotifications() {
            refreshed += 1
        }
    }

    /** Twelve notices, the first unread, and a cursor for the rest: the feed as the server sends it. */
    private val firstPage: String by lazy {
        "{\"cursor\":\"c2\",\"has_more\":true,\"items\":[" + (1..12).joinToString(",") { n ->
            "{\"id\":\"n$n\",\"unread\":${n == 1},\"date\":\"2026-12-01\",\"kind\":\"coming\"," +
                "\"tile\":{\"title\":\"movie:$n\",\"name\":\"Title $n\"}}"
        } + "]}"
    }

    /** The rest of it, with no cursor, which is how the server says there is no more. */
    private val secondPage: String by lazy {
        "{\"cursor\":\"\",\"has_more\":false,\"items\":[" + (13..14).joinToString(",") { n ->
            "{\"id\":\"n$n\",\"unread\":false,\"date\":\"2026-11-01\",\"kind\":\"coming\"," +
                "\"tile\":{\"title\":\"movie:$n\",\"name\":\"Title $n\"}}"
        } + "]}"
    }

    /** The feed as it ships, with its first page answered. */
    private fun feedPage(): Telling {
        val watching = Watching(signedIn(ScreenName.NOTIFICATIONS))
        val shell = Telling(watching)
        compose.setContent { NoticesPage(shell, watching.seen.value) }
        compose.waitForIdle()
        compose.runOnIdle { watching.answer(200, firstPage) }
        compose.waitForIdle()
        return shell
    }

    private fun noticesView(): NoticesView? = viewOf(compose.activity.window.decorView, NoticesView::class.java)

    private fun <T : View> viewOf(view: View, kind: Class<T>): T? {
        if (kind.isInstance(view)) return kind.cast(view)
        if (view is ViewGroup) {
            for (at in 0 until view.childCount) {
                viewOf(view.getChildAt(at), kind)?.let { return it }
            }
        }
        return null
    }

    /** The loading page, with the bar it will have and the count the server gave. */
    private fun waitingPage(unread: Int) {
        val seen = Seen(
            words = book,
            server = "http://box",
            active = "p1",
            nav = listOf(
                NavItem(id = "home", label = "Home", screen = ScreenName.HOME),
                NavItem(id = "series", label = "Series", screen = ScreenName.SERIES),
            ),
            route = RouteTarget(target = Target.HOME, screen = ScreenName.HOME),
            unreadNotifications = unread,
        )
        compose.setContent { Waiting(Watching(), seen) }
        compose.waitForIdle()
    }

    /** Every word drawn on a part of the screen that is showing. */
    private fun drawnTexts(): List<String> {
        val found = mutableListOf<String>()
        fun walk(view: View) {
            if (view is TextView && view.isShown) found += view.text.toString()
            if (view is ViewGroup) for (at in 0 until view.childCount) walk(view.getChildAt(at))
        }
        walk(compose.activity.window.decorView)
        return found
    }

    /** The page of rows as it ships, with a count on the bar. */
    private fun browsePage(unread: Int): Watching {
        val shell = Watching()
        val seen = Seen(
            words = book,
            server = "http://box",
            active = "p1",
            nav = listOf(
                NavItem(id = "home", label = "Home", screen = ScreenName.HOME),
                NavItem(id = "series", label = "Series", screen = ScreenName.SERIES),
            ),
            screen = Screen(
                name = ScreenName.HOME,
                sections = listOf(
                    Section(
                        id = "r1",
                        kind = SectionKind.ROW,
                        tiles = listOf(Tile(name = "Dune", title = TitleKey("movie:1"))),
                        title = "Trending Now",
                    ),
                ),
                ttlS = 60,
            ),
            route = RouteTarget(target = Target.HOME, screen = ScreenName.HOME),
            unreadNotifications = unread,
        )
        compose.setContent { Browse(shell, seen) }
        compose.waitForIdle()
        return shell
    }

    /** The page of rows, as the `View` it is drawn by, so its live region can be read. */
    private fun browseView(): View? = firstBrowse(compose.activity.window.decorView)

    private fun firstBrowse(view: View): View? {
        if (view is BrowseView) return view
        if (view is ViewGroup) {
            for (at in 0 until view.childCount) {
                firstBrowse(view.getChildAt(at))?.let { return it }
            }
        }
        return null
    }

    private fun titlePage(data: TitleDetail): Watching {
        val shell = Watching()
        val seen = Seen(
            words = book,
            server = "http://box",
            active = "p1",
            detail = data,
            route = RouteTarget(target = Target.TITLE, title = data.title),
        )
        compose.setContent { TitlePage(shell, seen) }
        compose.waitForIdle()
        return shell
    }
}
