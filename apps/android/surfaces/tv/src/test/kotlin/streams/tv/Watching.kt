package streams.tv

import contract.Action
import contract.DriverCommand
import contract.DriverEvent
import contract.Profile
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import streams.ports.NoPictures
import streams.ports.Pictures
import streams.ports.Player
import streams.ports.Reel
import streams.ports.Seen
import streams.ports.Shell

/**
 * A shell a whole television screen can be composed against, and a record of what that screen did.
 *
 * The television twin of `phone/.../Showing.kt`, which cannot be imported: a test source set is not
 * published to another module's. Three deliberate differences, each for a thing a television screen
 * does that a phone screen does not.
 *
 * Pictures are [NoPictures], which refuses in the open, so a tile falls back to its words. That is
 * the state a screen reading test wants, because the words are the thing being read.
 *
 * Travel is recorded rather than refused. A screen draws controls that could travel, and a shell
 * that threw on one would be testing whichever control the test happened to touch first.
 *
 * [retry] is recorded too, and this is the difference that matters. On the phone it is
 * `error("a screen reading test is already connected")`, because no phone screen offers to reconnect.
 * The television's way out of an unreachable server is exactly that call, so a shell that refused it
 * would make the dead end untestable -- which is how it stayed unmeasured.
 *
 * Asks are held unanswered unless a test answers them, for the phone's reason: the gap between a
 * request and its answer is where screens are wrong, and a shell that closed it would pass a test
 * against broken code.
 */
internal class Watching(seen: Seen = Seen(), private val filmReel: Reel? = null) : Shell {
    private val state = MutableStateFlow(seen)
    override val seen: StateFlow<Seen> = state

    /** What the screen asked the server for, in order. */
    val asks = mutableListOf<String>()

    /** Where the screen was told to travel, in order. "back" for a step back. */
    val went = mutableListOf<String>()

    /** How many times the screen asked to retry the bound server. */
    var retries = 0

    /** Everything the screen asked the core to do, in order. */
    val acted = mutableListOf<Action>()

    /** Everything the screen asked the player to do, in order. */
    val commanded = mutableListOf<DriverCommand>()

    private val replies = mutableListOf<(Int, String) -> Unit>()

    /** Puts a new state in front of the screen, as the core would. */
    fun show(next: Seen) { state.value = next }

    override fun ask(path: String, reply: (status: Int, text: String) -> Unit) {
        asks += path
        replies += reply
    }

    /** Answers the oldest ask still waiting, as the server would. */
    fun answer(status: Int, text: String) {
        check(replies.isNotEmpty()) { "nothing was asked" }
        replies.removeAt(0)(status, text)
    }

    override val pictures: Pictures = NoPictures

    /**
     * A player that does nothing and refuses nothing. The player screen asks it for a picture and a
     * position while composing, so one that threw would stop a sweep at whichever screen reached it
     * first -- which is how a sweep comes to cover less than it claims.
     */
    override val player: Player = object : Player {
        override fun command(given: DriverCommand) { commanded += given }
        override fun listen(report: (Long, DriverEvent) -> Unit) = Unit
        // A real View, because that is what `Watch.kt` does with it:
        // `AndroidView({ shell.player.picture(it) as View })`. A fake returning `Any()` compiles,
        // satisfies the signature, and throws ClassCastException the moment the picture is drawn.
        override fun picture(context: Any): Any =
            android.view.View(context as android.content.Context)
        override fun now(): Long = 0
        override fun lift(share: Float) = Unit
        override fun wordsLook(size: Int, color: String, background: String) = Unit
    }

    /**
     * A reel that hands back a real `View` and plays nothing.
     *
     * It threw until 2026-11-14, on the stated ground that no screen reading test has a brand film.
     * That stopped being true the day a test composed the whole of `app/Television.kt` to prove a
     * route reaches its screen: the opening layer is drawn over every signed in screen and draws
     * the landing's film (`door/Door.kt`'s `Film`), so a reel that threw made every route in this
     * surface untestable -- which is how they came to be untested.
     *
     * A real `View` and not `Any()`, for the reason the player above gives: `Film` casts it.
     */
    private val viewReel: Reel = object : Reel {
        override fun picture(
            context: Any,
            url: String,
            loop: Boolean,
            ready: () -> Unit,
            ended: () -> Unit,
        ): Any = android.view.View(context as android.content.Context)
        override fun play(picture: Any, on: Boolean) = Unit
        override fun release(picture: Any) = Unit
        override fun hold(still: Boolean) = Unit
    }

    /** A test that brings its own reel gets it; every other gets the one above. */
    override val reel: Reel get() = filmReel ?: viewReel
    override fun go(address: String) { went += address }
    override fun swap(address: String) { went += address }
    override fun back() { went += "back" }
    override fun retry() { retries += 1 }
    override fun act(action: Action) { acted += action }
    override fun send(method: String, path: String, body: String, reply: (Int, String) -> Unit) =
        error("a screen reading test reads, it never sends")
    override fun keep(profile: Profile) = Unit
    override fun keepAll(profiles: List<Profile>) = Unit
    override fun speak(language: String?) = Unit
    override fun refreshNotifications() = Unit
}
