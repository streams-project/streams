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
import contract.Track
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

    /** Says when the device gains or loses its connection. */
    fun watch(changed: (online: Boolean) -> Unit)
}

/** What the core keeps between runs. */
interface Secrets {
    fun get(name: String): String?

    /**
     * False when the store refused the write. The caller must say so rather than pretend: a session
     * token that was never written signs the viewer out at the next launch with no signal.
     */
    fun put(name: String, value: String): Boolean

    fun drop(name: String)
}

/** What the core's offline layer keeps between runs: waiting commands and kept pages. */
interface Store {
    fun get(key: String): String?
    fun put(key: String, value: String)
    fun delete(key: String)
    fun keys(prefix: String): List<String>
}

/** Plays films, reports facts, and decides nothing. */
interface Player {
    fun command(given: DriverCommand)
    fun listen(report: (generation: Long, event: DriverEvent) -> Unit)
    /** The picture, as an object only the surface's own toolkit understands. */
    fun picture(context: Any): Any

    /** Where the picture is now, read fresh for drawing subtitles. */
    fun now(): Long

    /** Raises subtitles by this share of the picture's height, to clear the controls. */
    fun lift(share: Float)

    /** The look a profile keeps for subtitles: size 0, 1 or 2, and colours as #rrggbb or none. */
    fun wordsLook(size: Int, color: String, background: String)
}

/** Brand films on the way in. The title player belongs to the core, so these stay apart from it. */
interface Reel {
    /**
     * A picture of this address. It starts still until `play` says so. `ready` runs once, when it can
     * show and play; `ended` runs when it finishes or cannot play.
     */
    fun picture(context: Any, url: String, loop: Boolean, ready: () -> Unit, ended: () -> Unit): Any
    /** Starts or stops one film. */
    fun play(picture: Any, on: Boolean)
    /** Hands back the decoder, because a television has few and the title player needs one. */
    fun release(picture: Any)
    /** Holds every film still while the app is off screen, and lets them go on after. */
    fun hold(still: Boolean)
}

/** One picture wanted. `done` gets a [Painted] or an [Unpainted]. */
data class Want(
    val url: String,
    /**
     * The box it will fill, in pixels, or 0 for the picture's own size. The size is asked for here
     * and not left to the layout for two reasons: the read starts before the first measure instead
     * of a frame after it, and a picture read ahead of the viewer lands under the same key as the
     * one that is drawn, so reading ahead is worth something. The nearest sample size is taken, so
     * a w342 poster is not re-encoded to fit a box exactly.
     */
    val wide: Int = 0,
    val high: Int = 0,
    /**
     * True when the caller will read the pixels back, which is what taking a tint off a billboard
     * or a profile gate does. The picture then has to be decoded into ordinary memory: the pixels
     * of one handed to the graphics card cannot be read, and asking for them throws.
     */
    val pixels: Boolean = false,
)

sealed interface Picture

/**
 * A picture, as an object only the surface's own toolkit understands -- the bargain
 * [Player.picture] already makes, and for the same reason: this file may not name a type that
 * belongs to one device. The family's adapter documents which object it hands over.
 */
data class Painted(val picture: Any) : Picture

/** Why there is no picture. The caller draws something else; it never waits for one that is not coming. */
data class Unpainted(val reason: String) : Picture

/** The right to stop one read. Letting it go stops nothing. */
fun interface Stop {
    fun stop()
}

/**
 * Pictures from the network: artwork, logos and the brand's mark.
 *
 * This was the one device capability of this family with no port. The network, the secrets, the
 * store, the player and the reel are all above; image loading went from a surface straight into a
 * library, and that library brought a second HTTP client in with it -- 78 `okhttp3` and 89 `okio`
 * classes ship in the APK -- under a ring check whose ban on the word `okhttp3` passed every time,
 * because no surface ever typed it.
 *
 * What is here is what the surfaces' own call sites ask for and nothing more: a url, the box it
 * will fill, whether the caller means to read the pixels back, a way to stop, and a read ahead of
 * the viewer. Drawing is not here, because drawing is what a surface is for.
 */
interface Pictures {
    /**
     * Reads one picture. `done` runs once, on the thread that asked, with a [Painted] or an
     * [Unpainted]. A read that was stopped answers neither.
     */
    fun fetch(want: Want, done: (Picture) -> Unit): Stop

    /**
     * Reads a picture into the cache before anything draws it, at the size the box will ask for, so
     * a row is already painted when the viewer reaches it.
     *
     * `done` runs whether the picture arrived or not: the one call site that passes it uses it to
     * stop waiting, and a failed read has to stop the wait as surely as a good one.
     */
    fun warm(want: Want, done: () -> Unit = {})
}

/**
 * What [Shell.pictures] answers while no picture adapter is plugged in.
 *
 * It refuses in the open and at once, because the alternative is worse: a port that quietly never
 * answers leaves a screen waiting on a read that cannot finish, and that reads to a person as a
 * slow network rather than as an app that was wired wrong.
 */
object NoPictures : Pictures {
    override fun fetch(want: Want, done: (Picture) -> Unit): Stop {
        done(Unpainted("no picture adapter is plugged into this app"))
        return Stop {}
    }

    override fun warm(want: Want, done: () -> Unit) = done()
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
    /** True when that language runs right to left: the interface mirrors, the player never does. */
    val rtl: Boolean = false,
    val detail: TitleDetail? = null,
    val notice: String? = null,
    /**
     * How many notices have been said in this run. The slot is app wide and a notice is one time,
     * so a screen that acts records this count first and reads the slot as its own answer only once
     * the count has moved. Without it a screen shows words another screen left behind. Every write
     * of `notice` with words in it moves this.
     */
    val notices: Int = 0,
    val brand: String = "",
    val server: String = "",
    /**
     * True when that server is reached over plain http. The screen that shows the address marks it,
     * so nobody runs on an unencrypted connection without being told.
     */
    val serverPlain: Boolean = false,
    val positionMs: Long = 0,
    val lengthMs: Long = 0,
    /** The sound and subtitle tracks of what is playing. */
    val audio: List<Track> = emptyList(),
    val text: List<Track> = emptyList(),
    val unreadNotifications: Int = 0,
)

interface Shell {
    val seen: StateFlow<Seen>
    val player: Player
    val reel: Reel

    /**
     * Where a surface's pictures come from.
     *
     * It carries a default because the family's picture adapter is not plugged in yet: `app/Main.kt`
     * builds the other five ports and `core/Host.kt` carries them, and both are one line short of
     * carrying this one. Until that line lands every call is refused in the open rather than
     * answered with a blank that reads as a slow network.
     */
    val pictures: Pictures get() = NoPictures

    fun connect(server: String)
    fun go(address: String)
    /** Goes to this address in place of the one here, so Back passes over it. */
    fun swap(address: String)
    fun back()
    fun act(action: Action)
    fun ask(path: String, reply: (status: Int, text: String) -> Unit)
    /** A change the viewer made. The core adds what the route needs. */
    fun send(method: String, path: String, body: String, reply: (status: Int, text: String) -> Unit)
    /** A saved profile, so every screen shows the change. */
    fun keep(profile: Profile)
    /** Replace the kept profile list after adding or deleting a profile. */
    fun keepAll(profiles: List<Profile>)
    /** Read the words of this language, or the default when there is none. */
    fun speak(language: String?)
    /** Refresh the unread notification count. */
    fun refreshNotifications()
}
