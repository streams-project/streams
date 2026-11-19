package streams.ports

import contract.Hello
import contract.contractJson
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject

/**
 * How `features.accounts_required` in the server's hello becomes `Seen.accountsRequired`, which is
 * what both doors read to decide whether a visitor is shown a way to Home.
 *
 * The hello is the contract's own valid fixture with only that one feature changed, so what is
 * decoded is what the generated type reads off the wire, not a value built by hand.
 */
class AccountsRequiredTest {
    private fun fixture(): JsonObject {
        var here: File? = File("").absoluteFile
        while (here != null && !File(here, "platform/contract/fixtures").isDirectory) here = here.parentFile
        val file = File(requireNotNull(here) { "no platform/contract/fixtures above ${File("").absolutePath}" },
            "platform/contract/fixtures/2026-11-05/valid/Hello.basic.json")
        return contractJson.parseToJsonElement(file.readText()).jsonObject
    }

    /** The server's hello with `accounts_required` set to [flag], or left out of it when [flag] is null. */
    private fun hello(flag: Boolean?): Hello {
        val whole = fixture()
        val features = buildJsonObject {
            whole.getValue("features").jsonObject.forEach { (name, value) ->
                if (name != "accounts_required") put(name, value)
            }
            if (flag != null) put("accounts_required", JsonPrimitive(flag))
        }
        val edited = buildJsonObject {
            whole.forEach { (name, value) -> put(name, if (name == "features") features else value) }
        }
        return contractJson.decodeFromString(Hello.serializer(), edited.toString())
    }

    @Test
    fun `a hello that says accounts are required leaves them required`() {
        assertEquals(true, accountsRequiredAfter(hello(true), before = true))
        assertEquals(true, accountsRequiredAfter(hello(true), before = false))
    }

    @Test
    fun `a hello that says accounts are off lets a visitor in`() {
        assertEquals(false, accountsRequiredAfter(hello(false), before = true))
    }

    /** The contract: with no answer from the server, accounts are required. */
    @Test
    fun `a hello that does not say makes accounts required, whatever was known before`() {
        assertEquals(true, accountsRequiredAfter(hello(null), before = true))
        assertEquals(true, accountsRequiredAfter(hello(null), before = false), "an earlier hello's visitor access is not kept")
    }

    @Test
    fun `a view with no hello keeps what was known`() {
        assertEquals(true, accountsRequiredAfter(null, before = true))
        assertEquals(false, accountsRequiredAfter(null, before = false))
    }

    /** Before any hello a screen is drawn from `Seen()`, and the secure default is the one it gets. */
    @Test
    fun `a viewer who has heard nothing is shown the sign in door`() {
        assertEquals(true, Seen().accountsRequired)
    }
}
