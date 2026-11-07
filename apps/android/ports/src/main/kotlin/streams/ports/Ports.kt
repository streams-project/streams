// The hexagon of the Android family. The core sits in the middle, and everything it needs from a
// device goes through one of these. They name the same ports as every other family.
package streams.ports

import contract.Action
import contract.DeviceSignInView
import contract.DriverCommand
import contract.DriverEvent
import contract.NavItem
import contract.PlayView
import contract.Profile
import contract.RouteTarget
import contract.Screen
import contract.TitleDetail
import kotlinx.coroutines.flow.StateFlow

/** One request. `done` gets a [Reply] or a [NoReply], from any thread. */
data class Call(
    val method: String,
    val url: String,
    val headers: Map<String, String>,
    val body: String?,
    val deadlineMs: Long,
)

sealed interface Answer
data class Reply(val status: Int, val text: String, val etag: String?) : Answer
data class NoReply(val reason: String) : Answer

interface Network {
    fun send(call: Call, done: (Answer) -> Unit)
}

/** What the core keeps between runs. */
interface Secrets {
    fun get(name: String): String?
    fun put(name: String, value: String)
    fun drop(name: String)
}

/** Plays films, reports facts, and decides nothing. */
interface Player {
    fun command(given: DriverCommand)
    fun listen(report: (generation: Long, event: DriverEvent) -> Unit)
    /** The picture, as an object only the surface's own toolkit understands. */
    fun picture(context: Any): Any
}

/** All a surface knows. It reads state and hands back what the viewer asked for. */
data class Seen(
    val route: RouteTarget? = null,
    val device: DeviceSignInView? = null,
    val profiles: List<Profile> = emptyList(),
    val play: PlayView? = null,
    val screen: Screen? = null,
    val nav: List<NavItem> = emptyList(),
    val active: String? = null,
    /** The words of the interface, with the brand filled in. */
    val words: Map<String, String> = emptyMap(),
    /** The language those words are in. */
    val language: String = "",
    val detail: TitleDetail? = null,
    val notice: String? = null,
    val brand: String = "",
    val server: String = "",
    val positionMs: Long = 0,
    val lengthMs: Long = 0,
)

interface Shell {
    val seen: StateFlow<Seen>
    val player: Player
    fun connect(server: String)
    fun go(address: String)
    fun back()
    fun act(action: Action)
    fun ask(path: String, reply: (status: Int, text: String) -> Unit)
    /** A change the viewer made. The core adds what the route needs. */
    fun send(method: String, path: String, body: String, reply: (status: Int, text: String) -> Unit)
    /** A saved profile, so every screen shows the change. */
    fun keep(profile: Profile)
    /** Read the words of this language, or the default when there is none. */
    fun speak(language: String?)
}
