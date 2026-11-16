package streams.tv

import android.content.Context
import android.view.View
import androidx.compose.runtime.mutableStateOf
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.NativeKeyEvent
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performKeyPress
import contract.Address
import contract.PublicTitles
import contract.TitleDetail
import contract.TitleKey
import contract.contractJson
import java.io.File
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import streams.ports.Reel
import streams.ports.Seen

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34], qualifiers = "w960dp-h540dp-xhdpi")
@GraphicsMode(GraphicsMode.Mode.NATIVE)
class DoorTest {
    @get:Rule val compose = createComposeRule()

    private val book: Map<String, String> by lazy {
        Json.parseToJsonElement(File(repoRoot(), "strings/en.json").readText()).jsonObject
            .mapValues { (_, word) -> word.jsonPrimitive.content }
    }

    private val stubReel = object : Reel {
        override fun picture(context: Any, url: String, loop: Boolean, ready: () -> Unit, ended: () -> Unit): Any =
            View(context as Context)
        override fun play(picture: Any, on: Boolean) = Unit
        override fun release(picture: Any) = Unit
        override fun hold(still: Boolean) = Unit
    }

    private fun press(code: Int) {
        compose.onRoot().performKeyPress(KeyEvent(NativeKeyEvent(android.view.KeyEvent.ACTION_DOWN, code)))
        compose.waitForIdle()
    }

    private fun right() = press(android.view.KeyEvent.KEYCODE_DPAD_RIGHT)
    private fun left() = press(android.view.KeyEvent.KEYCODE_DPAD_LEFT)
    private fun ok() = press(android.view.KeyEvent.KEYCODE_DPAD_CENTER)

    private fun wordsOnScreen(): List<String> =
        HeardScreen.words(compose.onRoot().fetchSemanticsNode()).map { it.label }

    @Test
    fun `door shows home pill and opens home on OK when accounts are not required`() {
        val seen = Seen(server = "https://example.test", words = book, accountsRequired = false)
        val shell = Watching(seen, stubReel)
        val opening = OpeningState().apply { over() }
        compose.setContent { Door(shell, seen, opening) }
        compose.waitForIdle()

        val homeWord = book.getValue("nav.home")
        assertTrue("nav.home pill missing when accountsRequired = false: ${wordsOnScreen()}", homeWord in wordsOnScreen())

        right()
        ok()
        assertEquals(listOf(Address.HOME), shell.went)
    }

    @Test
    fun `door omits home pill and opens sign in when accounts are required`() {
        val seen = Seen(server = "https://example.test", words = book, accountsRequired = true)
        val shell = Watching(seen, stubReel)
        val opening = OpeningState().apply { over() }
        compose.setContent { Door(shell, seen, opening) }
        compose.waitForIdle()

        val homeWord = book.getValue("nav.home")
        assertFalse("nav.home pill must be omitted when accountsRequired = true", homeWord in wordsOnScreen())

        right()
        ok()
        assertEquals(listOf(Address.SIGN_IN), shell.went)
    }

    @Test
    fun `door navigates from LOOK_ROUND to JOIN on Left and clamps when accountsRequired flips true`() {
        val state = mutableStateOf(Seen(server = "https://example.test", words = book, accountsRequired = false))
        val shell = Watching(state.value, stubReel)
        val opening = OpeningState().apply { over() }
        compose.setContent { Door(shell, state.value, opening) }
        compose.waitForIdle()

        right() // JOIN -> LOOK_ROUND
        left() // LOOK_ROUND -> JOIN
        ok()
        assertEquals(listOf(Address.SIGN_IN), shell.went)

        right() // JOIN -> LOOK_ROUND again
        state.value = state.value.copy(accountsRequired = true)
        compose.waitForIdle()
        assertFalse(book.getValue("nav.home") in wordsOnScreen())

        ok() // clamped to JOIN -> opens SIGN_IN, not HOME
        assertEquals(listOf(Address.SIGN_IN, Address.SIGN_IN), shell.went)
    }

    @Test
    fun `door navigates past LOOK_ROUND to the first trending card on Right`() {
        val seen = Seen(server = "https://example.test", words = book, accountsRequired = false)
        val shell = Watching(seen, stubReel)
        val opening = OpeningState().apply { over() }
        compose.setContent { Door(shell, seen, opening) }
        compose.waitForIdle()

        val titles = PublicTitles(
            titles = listOf(TitleDetail(title = TitleKey("movie:1"), name = "Lantern Hill")),
        )
        shell.answer(200, contractJson.encodeToString(PublicTitles.serializer(), titles))
        compose.waitForIdle()

        right() // JOIN -> LOOK_ROUND
        right() // LOOK_ROUND -> TitleCard 1
        assertTrue("expected trending card name on screen: ${wordsOnScreen()}", "Lantern Hill" in wordsOnScreen())
        ok()
        assertEquals(listOf(Address.SIGN_IN), shell.went)
    }
}
