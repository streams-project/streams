package streams.phone

import contract.Action
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
 * A shell a whole screen can be composed against, as against [Asked] which only answers paging.
 *
 * Two deliberate differences from [Asked]. Pictures are [NoPictures], which refuses in the open, so
 * a tile falls back to its words -- which is the state a screen reader test wants, because the
 * words are the thing being read. And travel is recorded rather than refused, because a screen
 * draws controls that could travel and a test that threw on one would be testing the control it
 * happened to touch first.
 *
 * Asks are still held unanswered unless a test answers them, for [Asked]'s reason: the gap between
 * a request and its answer is where screens are wrong, and a shell that closed it would pass a test
 * against broken code.
 */
internal class Showing(seen: Seen = Seen()) : Shell {
    private val state = MutableStateFlow(seen)
    override val seen: StateFlow<Seen> = state

    /** What the screen asked the network for, in order. */
    val asks = mutableListOf<String>()
    /** Where the screen was told to travel, in order. */
    val went = mutableListOf<String>()

    private val waiting = mutableListOf<(Int, String) -> Unit>()

    override fun ask(path: String, reply: (status: Int, text: String) -> Unit) {
        asks += path
        waiting += reply
    }

    /** Answers the oldest ask still waiting, as the network would. */
    fun answer(status: Int, text: String) {
        check(waiting.isNotEmpty()) { "nothing was asked" }
        waiting.removeAt(0)(status, text)
    }

    override val pictures: Pictures = NoPictures
    /**
     * A player that does nothing and refuses nothing. The screens that draw controls ask it for a
     * picture and a position while composing, so a player that threw would stop the sweep at
     * whichever screen reached it first, which is how a sweep comes to cover less than it claims.
     */
    override val player: Player = object : Player {
        override fun command(given: contract.DriverCommand) = Unit
        override fun listen(report: (Long, contract.DriverEvent) -> Unit) = Unit
        // A real View, because that is what the port's contract means by "an object only the
        // surface's own toolkit understands" on Android: `Watch.kt:85` does
        // `AndroidView({ shell.player.picture(it) as View })`. A fake returning `Any()` compiles,
        // satisfies the signature, and throws ClassCastException the moment a screen draws the
        // picture -- a fake that lies about the port it stands in for.
        override fun picture(context: Any): Any =
            android.view.View(context as android.content.Context)
        override fun now(): Long = 0
        override fun lift(share: Float) = Unit
        override fun wordsLook(size: Int, color: String, background: String) = Unit
    }
    override val reel: Reel get() = error("a screen reading test has no brand film")
    override fun go(address: String) { went += address }
    override fun swap(address: String) { went += address }
    override fun back() { went += "back" }
    override fun connect(server: String) = error("a screen reading test is already connected")
    override fun act(action: Action) = Unit
    override fun send(method: String, path: String, body: String, reply: (Int, String) -> Unit) =
        error("a screen reading test reads, it never sends")
    override fun keep(profile: Profile) = Unit
    override fun keepAll(profiles: List<Profile>) = Unit
    override fun speak(language: String?) = Unit
    override fun refreshNotifications() = Unit
}
