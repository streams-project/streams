package streams.phone

import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import contract.Address
import contract.RouteTarget
import contract.Target
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import streams.ports.Seen
import streams.shared.Brand

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34], qualifiers = "w411dp-h914dp-xxhdpi")
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class DoorSignInTest {
    @get:Rule val compose = createComposeRule()

    @Test
    fun `door offers home button when accounts are not required`() {
        val seen = Seen(
            brand = BRAND,
            server = SERVER,
            words = words,
            route = RouteTarget(target = Target.START),
            accountsRequired = false,
        )
        val shell = Showing(seen)
        compose.setContent { Brand { Door(shell, seen) } }
        statusBar(compose)

        val homeLabel = words.getValue("nav.home")
        val signInLabel = words.getValue("door.sign_in")
        compose.onNodeWithText(homeLabel).assertExists()
        compose.onNodeWithText(signInLabel).assertExists()
        assertEquals(emptyList<String>(), silentControls(compose.onRoot().fetchSemanticsNode()))

        compose.onNodeWithText(homeLabel).performClick()
        assertEquals(listOf(Address.HOME), shell.went)
    }

    @Test
    fun `door omits home button when accounts are required`() {
        val seen = Seen(
            brand = BRAND,
            server = SERVER,
            words = words,
            route = RouteTarget(target = Target.START),
            accountsRequired = true,
        )
        val shell = Showing(seen)
        compose.setContent { Brand { Door(shell, seen) } }
        statusBar(compose)

        val homeLabel = words.getValue("nav.home")
        val signInLabel = words.getValue("door.sign_in")
        assertTrue(
            "nav.home must not appear when accounts are required",
            compose.onAllNodesWithText(homeLabel).fetchSemanticsNodes().isEmpty(),
        )
        compose.onNodeWithText(signInLabel).performClick()
        assertEquals(listOf(Address.SIGN_IN), shell.went)
    }

    /** The server has not said (no hello yet, or none that says): the contract's answer is that accounts are required. */
    @Test
    fun `door omits home button when the server has not said whether accounts are required`() {
        val seen = Seen(
            brand = BRAND,
            server = SERVER,
            words = words,
            route = RouteTarget(target = Target.START),
        )
        val shell = Showing(seen)
        compose.setContent { Brand { Door(shell, seen) } }
        statusBar(compose)

        assertTrue(
            "nav.home must not appear until the server says accounts are off",
            compose.onAllNodesWithText(words.getValue("nav.home")).fetchSemanticsNodes().isEmpty(),
        )
        compose.onNodeWithText(words.getValue("door.sign_in")).performClick()
        assertEquals(listOf(Address.SIGN_IN), shell.went)
    }

    @Test
    fun `sign in places heading, fields, and button in top half of screen`() {
        val seen = Seen(
            brand = BRAND,
            server = SERVER,
            words = words,
            route = RouteTarget(target = Target.SIGN_IN),
        )
        val shell = Showing(seen)
        compose.setContent { Brand { SignIn(shell, seen) } }
        statusBar(compose)

        val root = compose.onRoot().fetchSemanticsNode()
        val rootHeight = root.size.height.toFloat()
        val signInMatches = compose.onAllNodesWithText(words.getValue("door.sign_in")).fetchSemanticsNodes()
        assertEquals("expected heading and submit button with door.sign_in text", 2, signInMatches.size)
        val heading = signInMatches[0]
        val button = signInMatches[1]
        val emailField = compose.onNodeWithText(words.getValue("signin.email")).fetchSemanticsNode()

        assertTrue(
            "heading top (${heading.boundsInRoot.top}) must sit below status bar ($STATUS_BAR)",
            heading.boundsInRoot.top >= STATUS_BAR,
        )
        assertTrue(
            "heading top (${heading.boundsInRoot.top}) must be top-aligned (< 25% of $rootHeight)",
            heading.boundsInRoot.top < rootHeight * 0.25f,
        )
        assertTrue(
            "email field top (${emailField.boundsInRoot.top}) must be near the top (< 35% of $rootHeight)",
            emailField.boundsInRoot.top < rootHeight * 0.35f,
        )
        assertTrue(
            "sign in button bottom (${button.boundsInRoot.bottom}) must stay above keyboard half (< 55% of $rootHeight)",
            button.boundsInRoot.bottom < rootHeight * 0.55f,
        )
    }
}
