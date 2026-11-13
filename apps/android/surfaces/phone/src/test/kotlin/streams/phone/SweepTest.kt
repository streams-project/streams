package streams.phone

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import contract.RouteTarget
import contract.Target
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
 * Every screen a route can land on, composed and read as a screen reader would read it.
 *
 * MEASURING PASS -- each test prints what it heard and asserts nothing yet. The frozen list of
 * screens that legitimately say nothing goes in once this has run, because a placeholder body
 * draws no words on purpose and a sweep that assumed otherwise would be wrong rather than strict.
 */
@RunWith(RobolectricTestRunner::class)
// Native graphics: the legacy shadow returns null bitmaps, and a screen that draws one
// dies before anything is read. Found by `Waiting` on the first sweep.
@Config(sdk = [34])
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class SweepTest {
    @get:Rule val compose = createComposeRule()

    private fun heardOn(name: String, screen: @Composable () -> Unit): List<HeardTree> {
        compose.setContent { Brand { screen() } }
        val heard = HeardTree.under(compose.onRoot().fetchSemanticsNode())
        println("SWEEP $name: ${heard.size} heard -> ${heard.joinToString(" | ") { it.label }}")
        return heard
    }

    private fun seen(target: Target, server: String = "https://example.test") =
        Seen(brand = "Bramble", server = server, route = RouteTarget(target = target))

    @Test fun server() { heardOn("Server") { Server(Showing(), seen(Target.START, "")) } }
    @Test fun waiting() { heardOn("Waiting") { Waiting() } }
    @Test fun door() { heardOn("Door") { Door(Showing(), seen(Target.START)) } }
    @Test fun signIn() { heardOn("SignIn") { SignIn(Showing(), seen(Target.SIGN_IN)) } }
    @Test fun code() { heardOn("Code") { Code(Showing(), seen(Target.SIGN_IN)) } }
    @Test fun gate() {
        val people = listOf(
            contract.Profile(hasPin = false, id = "1", kids = false, main = true, name = "Ada"),
            contract.Profile(hasPin = false, id = "2", kids = true, main = false, name = "Grace"),
        )
        heardOn("Gate") { Gate(Showing(), seen(Target.PROFILES).copy(profiles = people)) }
    }
    @Test fun title() { heardOn("Title") { Title(Showing(), seen(Target.TITLE), tv = false) } }
    @Test fun watch() { heardOn("Watch") { Watch(Showing(), seen(Target.WATCH), tv = false) } }
    @Test fun notifications() {
        heardOn("Notifications") { NotificationsScreen(Showing(), seen(Target.SCREEN)) }
    }
    @Test fun home() { heardOn("Home") { Home(Showing(), seen(Target.HOME)) } }
}
