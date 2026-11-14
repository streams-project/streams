package streams.phone

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import contract.DeviceSignInState
import contract.DeviceSignInView
import contract.RouteTarget
import contract.Target
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import streams.phone.notifications.NotificationsScreen
import streams.phone.profiles.Gate
import streams.ports.Seen
import streams.shared.Brand

/**
 * Every screen a route can land on, composed with the answer it would have on a device, and read
 * in the order a screen reader is walked through it.
 *
 * Each screen says what a viewer sees on it; the sweep holds the reading against that list and
 * fails on anything missing. It also asks every screen the question a tree walk cannot: whether a
 * control that answers a tap says nothing at all.
 *
 * Two traps this was walked into before and is written against. The first: a screen handed nothing
 * draws its waiting state, so a sweep run with no fixtures measures the waiting states -- this
 * reported "Title: 0 heard" on 2026-11-13, which was `Title.kt:113` returning before it drew
 * anything because the answer in hand was not the answer for the route. The second: freezing that
 * nothing as the expected answer, which would leave the sweep covering the screen forever while
 * checking none of it. So every screen below is given its answer, and every screen is asserted by
 * words a person can read off the screen.
 */
@RunWith(RobolectricTestRunner::class)
// Native graphics: the legacy shadow returns null bitmaps, and a screen that draws one
// dies before anything is read. Found by `Waiting` on the first sweep.
// A phone's screen, not the toolkit's 320 x 470 dp default, in which the billboard alone fills
// Home and not one row is composed -- a sweep run at that size measures the window.
@Config(sdk = [34], qualifiers = "w411dp-h914dp-xxhdpi")
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class SweepTest {
    @get:Rule val compose = createComposeRule()

    /**
     * Composes [screen], reads it as a screen reader would, and holds it against [says].
     *
     * [says] is what a viewer can read off the screen, so a failure is a sentence anyone can check
     * by looking at the device. The reading is printed whether it passes or not: a sweep is also a
     * record of what each screen hands a reader.
     */
    private fun sweep(
        name: String,
        says: List<String>,
        /** Milliseconds to let pass before reading, for a screen that draws after a wait. */
        after: Long = 0,
        screen: @Composable () -> Unit,
    ): List<Stop> {
        compose.setContent { Brand { screen() } }
        // Idling waits for the drawing to settle; it does not move the clock past a `delay`, so a
        // screen that shows itself after one shows nothing until the clock is moved by hand.
        if (after > 0) compose.mainClock.advanceTimeBy(after)
        // With a status bar, as a phone has. Read with none, an order decided by a tie on where a
        // box starts comes out right here and wrong on a device: see [STATUS_BAR].
        statusBar(compose)
        val root = compose.onRoot().fetchSemanticsNode()
        val reading = Reached.of(root)
        println("SWEEP $name: ${reading.size} stops -> ${reading.words()}")
        val missing = says.filter { want -> reading.none { it.label.contains(want) } }
        assertEquals("$name does not say this to a reader: ${reading.words()}", emptyList<String>(), missing)
        assertEquals(
            "$name has a control that answers a tap and says nothing",
            emptyList<String>(),
            silentControls(root),
        )
        return reading
    }

    private fun seen(target: Target, server: String = SERVER) =
        Seen(brand = BRAND, server = server, words = words, route = RouteTarget(target = target))

    @Test fun server() {
        // The one screen with no words from `strings/`: it is drawn before a server is known, and
        // the words of the interface come from the server. Those three are written in `Doors.kt`.
        sweep("Server", listOf("Your server", "Address", "Connect")) {
            Server(Showing(), seen(Target.START, ""))
        }
    }

    @Test fun waiting() {
        // The wait only draws after `LATER_WAIT_MS`: a wait that is over in a blink shows nothing
        // on purpose. Swept before the clock was moved, this screen reported nothing heard and the
        // nothing was almost written down as correct.
        val reading = sweep("Waiting", listOf(words.getValue("say.loading")), after = 1_000) {
            Waiting(words)
        }
        assertEquals("a wait is one thing to hear, not several: ${reading.words()}", 1, reading.size)
    }

    @Test fun door() {
        sweep("Door", listOf(BRAND, words.getValue("door.sign_in"))) {
            Door(Showing(), seen(Target.START))
        }
    }

    @Test fun signIn() {
        sweep(
            "SignIn",
            listOf(words.getValue("door.sign_in"), words.getValue("signin.email"), words.getValue("signin.password")),
        ) { SignIn(Showing(), seen(Target.SIGN_IN)) }
    }

    @Test fun code() {
        // With the code in hand, which is the state this screen exists for. Without one it says
        // "Getting a code...", and a sweep that only ever saw that would never read the code out.
        val device = DeviceSignInView(
            state = DeviceSignInState.WAITING,
            userCode = "KHTP-2291",
            verificationUri = "$SERVER/link",
            expiresIn = 600,
        )
        sweep("Code", listOf("Sign in on your phone", "KHTP-2291", "$SERVER/link")) {
            Code(Showing(), seen(Target.SIGN_IN).copy(device = device))
        }
    }

    @Test fun gate() {
        sweep("Gate", listOf(people[0].name, people[1].name)) {
            Gate(Showing(), seen(Target.PROFILES).copy(profiles = people))
        }
    }

    @Test fun title() {
        // The fixture `Title.kt:113` wanted: the answer in hand is the answer for the route. With
        // any other, the whole page returns before it draws and a reader is handed an empty screen.
        sweep(
            "Title",
            listOf(
                filmDetail.name,
                words.getValue("common.play"),
                words.getValue("list.add"),
                filmDetail.moreLikeThis.orEmpty().first().name,
            ),
        ) { Title(Showing(), titleSeen(), tv = false) }
    }

    @Test fun series() {
        // The same page for a series, which draws the season picker and asks for its episodes.
        sweep("Series", listOf(seriesDetail.name, words.getValue("common.play"))) {
            Title(Showing(), titleSeen(seriesDetail), tv = false)
        }
    }

    @Test fun watch() {
        // Half an hour into a film of one hour forty-four, so the clock says a real time rather
        // than the 0:00 / 0:00 a player with nothing playing draws.
        sweep("Watch", listOf("30:00", "1:44:00")) { Watch(Showing(), watchSeen(), tv = false) }
    }

    @Test fun notifications() {
        val shell = Showing(noticesSeen())
        compose.setContent { Brand { NotificationsScreen(shell, noticesSeen()) } }
        // The feed arrives as an answer to the ask the screen makes while it composes. Held back,
        // the screen draws its spinner, and a sweep over that measures the spinner.
        compose.runOnIdle { shell.answer(200, noticesBody()) }
        statusBar(compose)
        val root = compose.onRoot().fetchSemanticsNode()
        val reading = Reached.of(root)
        println("SWEEP Notifications: ${reading.size} stops -> ${reading.words()}")
        val says = listOf(words.getValue("notifications.title"), "The Long Quiet", lead.name)
        assertEquals(
            "Notifications does not say this to a reader: ${reading.words()}",
            emptyList<String>(),
            says.filter { want -> reading.none { it.label.contains(want) } },
        )
        assertEquals(
            "Notifications has a control that answers a tap and says nothing",
            emptyList<String>(),
            silentControls(root),
        )
    }

    @Test fun home() {
        val seen = homeSeen()
        val reading = sweep(
            "Home",
            listOf(
                words.getValue("nav.home"),
                words.getValue("nav.series"),
                words.getValue("notifications.title"),
                lead.name,
                words.getValue("common.play"),
                homeScreen.sections[1].title.orEmpty(),
            ),
        ) { Home(Showing(seen), seen) }
        // The order, which is the thing that was broken: a reader is walked down the page, so the
        // page's own name comes before the billboard in the middle of it. `HomeReachTest` holds
        // the rest of this.
        val name = reading.firstWith(words.getValue("nav.home"))
        val billboard = reading.firstWith(lead.name)
        assertEquals("Home is read out of order: ${reading.words()}", true, name < billboard)
    }
}
