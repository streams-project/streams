package streams.phone

import contract.Action
import contract.Profile
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import streams.ports.Pictures
import streams.ports.Player
import streams.ports.Reel
import streams.ports.Seen
import streams.ports.Shell

/**
 * A shell that writes down what was asked of the network and answers only when a test says to.
 *
 * Holding the answer is the whole point. The defect these tests are about happens in the gap
 * between a request going out and its answer coming back, and a shell that answered inside `ask`
 * would close that gap and the test would pass against the broken code.
 *
 * Everything a screen does that is not asking fails loudly rather than quietly doing nothing: a
 * test that reached one of these would be testing something it did not mean to.
 */
internal class Asked : Shell {
    /** Every path asked for, in order. */
    val asks = mutableListOf<String>()

    private val waiting = mutableListOf<(Int, String) -> Unit>()

    override fun ask(path: String, reply: (status: Int, text: String) -> Unit) {
        asks += path
        waiting += reply
    }

    /** How many asks have gone out and not been answered. */
    val unanswered: Int get() = waiting.size

    /** Answers the oldest ask that is still waiting, as the network would from its own thread. */
    fun answer(status: Int, text: String) {
        check(waiting.isNotEmpty()) { "nothing was asked" }
        waiting.removeAt(0)(status, text)
    }

    override val seen: StateFlow<Seen> = MutableStateFlow(Seen())
    override val player: Player get() = error("a paging test does not play")
    override val reel: Reel get() = error("a paging test has no brand film")
    override val pictures: Pictures get() = error("a paging test draws no pictures")
    override fun connect(server: String) = error("not asked for by paging")
    override fun go(address: String) = error("not asked for by paging")
    override fun swap(address: String) = error("not asked for by paging")
    override fun back() = error("not asked for by paging")
    override fun act(action: Action) = error("not asked for by paging")
    override fun send(method: String, path: String, body: String, reply: (Int, String) -> Unit) =
        error("paging reads, it never sends")
    override fun keep(profile: Profile) = error("not asked for by paging")
    override fun keepAll(profiles: List<Profile>) = error("not asked for by paging")
    override fun speak(language: String?) = error("not asked for by paging")
    override fun refreshNotifications() = error("not asked for by paging")
}
