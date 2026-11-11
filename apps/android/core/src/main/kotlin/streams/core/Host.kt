package streams.core

import android.os.Handler
import android.os.HandlerThread
import android.os.SystemClock
import contract.ABI_VERSION
import contract.Action
import contract.ActionPause
import contract.ActionRequest
import contract.BodyTarget
import contract.CoreConfig
import contract.CoreConfigTag
import contract.CoreIn
import contract.CoreInTag
import contract.CoreMode
import contract.CoreOut
import contract.DeviceClass
import contract.DriverEvent
import contract.DriverEventPosition
import contract.DriverEventTracks
import contract.Effect
import contract.EffectHttp
import contract.EffectLog
import contract.EffectOpen
import contract.EffectPlayer
import contract.EffectSecret
import contract.EffectStore
import contract.EffectView
import contract.Event
import contract.EventAction
import contract.EventActivity
import contract.EventAddress
import contract.EventDriver
import contract.EventHttpDone
import contract.EventHttpFailed
import contract.EventNetwork
import contract.EventSecretDone
import contract.EventStoreDone
import contract.EventUnsupported
import contract.EventVisibility
import contract.Form
import contract.HttpMethod
import contract.Nav
import contract.NoAnswerReason
import contract.NotificationCount
import contract.Profile
import contract.Screen
import contract.SecretOp
import contract.StoreOp
import contract.Strings
import contract.TextDirection
import contract.TitleDetail
import contract.WIRE_VERSION
import contract.WireDate
import contract.contractJson
import java.security.SecureRandom
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import streams.ports.Call
import streams.ports.Network
import streams.ports.NoReply
import streams.ports.Player
import streams.ports.Reel
import streams.ports.Reply
import streams.ports.Secrets
import streams.ports.Seen
import streams.ports.Shell
import streams.ports.Store

/**
 * Steps the core and performs what it asks, through the ports. It holds no rules and no screens.
 * Everything runs on one thread, because a core handle is used from one thread at a time.
 */
class Host(
    private val network: Network,
    private val secrets: Secrets,
    private val store: Store,
    override val player: Player,
    override val reel: Reel,
    private val deviceClass: DeviceClass,
    private val form: Form,
    private val version: String,
) : Shell {
    private companion object {
        const val SERVER = "app:server"
        const val DEFAULT_LANGUAGE = "en"
        const val WAKE_CAP_MS = 60_000L
        const val TOUCH_EVERY_MS = 10_000L

        // Said to the viewer, not logged: the only words this host has of its own.
        const val PLAIN_REFUSED =
            "This address must start with https://. Plain http reaches only a server on your own network."

        /**
         * True when this address is reached in the clear, false when it is encrypted, and null when
         * it may not be used at all.
         */
        fun encryptionOf(server: String): Boolean? {
            val lower = server.lowercase()
            if (lower.startsWith("https://")) return false
            if (!lower.startsWith("http://")) return null
            return if (ownNetwork(hostOf(lower))) true else null
        }

        /** The host of an address, with the port, the path and an IPv6 address's brackets taken off. */
        private fun hostOf(address: String): String {
            val after = address.removePrefix("http://").takeWhile { it != '/' }
            if (after.startsWith("[")) return after.drop(1).takeWhile { it != ']' }
            return after.takeWhile { it != ':' }
        }

        /**
         * The viewer's own machine or their own network: a hop no stranger is on. Names are here
         * because a server is usually found by its name on the network, not by its number.
         */
        private fun ownNetwork(host: String): Boolean {
            if (host == "localhost" || host.endsWith(".local") || host == "::1") return true
            val parts = host.split(".")
            if (parts.size != 4) return false
            val numbers = parts.map { it.toIntOrNull() ?: return false }
            if (numbers.any { it < 0 || it > 255 }) return false
            return when {
                numbers[0] == 127 || numbers[0] == 10 -> true
                numbers[0] == 192 && numbers[1] == 168 -> true
                numbers[0] == 169 && numbers[1] == 254 -> true
                numbers[0] == 172 && numbers[1] in 16..31 -> true
                else -> false
            }
        }
    }

    private val thread = HandlerThread("core").apply { start() }
    private val loop = Handler(thread.looper)
    private val state = MutableStateFlow(Seen())
    private var handle = 0L
    private var base = ""
    private var matched: String? = null
    private val queue = ArrayDeque<Event>()
    private var asks = 0
    private val waiting = HashMap<String, (Int, String) -> Unit>()
    private val trail = ArrayDeque<String>()
    private var here = "/"
    private val wake = Runnable { send(emptyList()) }
    private var lastTouch = 0L

    override val seen: StateFlow<Seen> = state

    init {
        network.watch { online -> send(listOf(EventNetwork(online = online))) }
        player.listen { generation, event ->
            if (event is DriverEventPosition) {
                state.update { it.copy(positionMs = event.timeMs, lengthMs = event.durationMs ?: it.lengthMs) }
            }
            if (event is DriverEventTracks) state.update { it.copy(audio = event.audio, text = event.text) }
            send(listOf(EventDriver(event = event, generation = generation)))
        }
        val kept = secrets.get(SERVER)
        if (kept != null) connect(kept)
    }

    /**
     * The one gate every server address passes, typed or kept. Plain http carries the session's
     * bearer and refresh tokens in the clear, so it may only reach the viewer's own network;
     * anything else must be https. A refused address is said, not swallowed.
     */
    override fun connect(server: String) {
        val typed = server.trim().trimEnd('/')
        val plain = encryptionOf(typed)
        if (plain == null) {
            state.update { it.copy(notice = PLAIN_REFUSED, notices = it.notices + 1) }
            return
        }
        loop.post {
            base = typed
            secrets.put(SERVER, base)
            // A notice is one time. This address was taken, so a refusal still sitting in the app
            // wide slot is not news any more and must not be read as some other screen's error.
            state.update { it.copy(server = base, serverPlain = plain, notice = null) }
            if (handle == 0L) open()
        }
    }

    private fun open() {
        val config = CoreConfig(
            app = "android",
            `class` = deviceClass,
            contract = WireDate(WIRE_VERSION),
            form = form,
            mode = CoreMode.LOCAL,
            os = "android",
            seed = seed(),
            type = CoreConfigTag.VALUE,
            version = version,
        )
        check(NativeCore.abi() == ABI_VERSION) { "this core has another ABI" }
        handle = NativeCore.open(contractJson.encodeToString(CoreConfig.serializer(), config).toByteArray())
        check(handle != 0L) { "the core did not open" }
        send(listOf(EventAddress(address = "/")))
    }

    private fun seed(): String {
        val bytes = ByteArray(16).also { SecureRandom().nextBytes(it) }
        return bytes.joinToString("") { "%02x".format(it) }
    }

    override fun go(address: String) {
        loop.post {
            trail.addLast(here)
            here = address
            send(listOf(EventAddress(address = address)))
        }
    }

    /** The app left the screen or came back. Nobody can see a film off screen, so it pauses. */
    fun shown(visible: Boolean) {
        reel.hold(!visible)
        val pause = if (visible) emptyList() else listOf(EventAction(action = ActionPause))
        send(pause + EventVisibility(visible = visible))
    }

    /** A key or a touch. The core only needs to hear this now and then. */
    fun touched() {
        val now = SystemClock.elapsedRealtime()
        if (now - lastTouch < TOUCH_EVERY_MS) return
        lastTouch = now
        send(listOf(EventActivity()))
    }

    override fun swap(address: String) {
        loop.post {
            here = address
            send(listOf(EventAddress(address = address)))
        }
    }

    override fun back() {
        loop.post {
            here = trail.removeLastOrNull() ?: "/"
            send(listOf(EventAddress(address = here)))
        }
    }

    override fun act(action: Action) = send(listOf(EventAction(action = action)))

    override fun ask(path: String, reply: (Int, String) -> Unit) {
        loop.post {
            asks += 1
            val id = "ask-$asks"
            waiting[id] = reply
            val action = ActionRequest(method = HttpMethod.GET, path = path)
            send(listOf(EventAction(id = id, action = action)))
        }
    }

    override fun send(method: String, path: String, body: String, reply: (Int, String) -> Unit) {
        loop.post {
            asks += 1
            val id = "ask-$asks"
            waiting[id] = reply
            val action = ActionRequest(method = HttpMethod(method), path = path, body = body)
            send(listOf(EventAction(id = id, action = action)))
        }
    }

    override fun keep(profile: Profile) {
        state.update { now ->
            now.copy(profiles = now.profiles.map { if (it.id == profile.id) profile else it })
        }
    }

    override fun keepAll(profiles: List<Profile>) = state.update { it.copy(profiles = profiles) }

    override fun speak(language: String?) = readWords(language ?: DEFAULT_LANGUAGE, state.value.brand)

    override fun refreshNotifications() {
        if (state.value.active == null) return
        ask("/v1/notifications/count") { status, text ->
            if (status in 200..299) {
                val count = contractJson.decodeFromString(NotificationCount.serializer(), text)
                state.update { it.copy(unreadNotifications = count.unread.toInt()) }
            }
        }
    }

    private fun send(events: List<Event>) {
        loop.post {
            if (handle == 0L) return@post
            queue.addAll(events)
            var rounds = 0
            do {
                val batch = queue.toList()
                queue.clear()
                val input = CoreIn(
                    contract = WireDate(WIRE_VERSION),
                    events = batch,
                    monoMs = SystemClock.elapsedRealtime(),
                    nowMs = System.currentTimeMillis(),
                    type = CoreInTag.VALUE,
                )
                val text = contractJson.encodeToString(CoreIn.serializer(), input)
                val out = NativeCore.step(handle, text.toByteArray())
                apply(contractJson.decodeFromString(CoreOut.serializer(), String(out)))
                rounds += 1
            } while (queue.isNotEmpty() && rounds < 64)
        }
    }

    /** The words of the interface, with the brand's name in them. */
    private fun readWords(language: String, brand: String) {
        ask("/v1/strings/$language") { status, text ->
            // A language this server does not have falls back to English, so a screen always has words.
            if (status !in 200..299 && language != DEFAULT_LANGUAGE) {
                readWords(DEFAULT_LANGUAGE, brand)
                return@ask
            }
            if (status in 200..299) {
                val read = contractJson.decodeFromString(Strings.serializer(), text)
                state.update { now ->
                    now.copy(
                        words = read.texts.mapValues { (_, word) -> word.replace("{brand}", brand) },
                        language = read.lang,
                        rtl = read.dir == TextDirection.RTL,
                    )
                }
            }
        }
    }

    /** A profile kept from an earlier run speaks its own language from the start. */
    private fun matchLanguage() {
        val now = state.value
        val wanted = now.profiles.firstOrNull { it.id == now.active }?.language ?: return
        if (wanted != now.language && wanted != matched) {
            matched = wanted
            readWords(wanted, now.brand)
        }
    }

    private fun apply(out: CoreOut) {
        out.view?.hello?.let { hello ->
            // Before any profile, the screens speak the device's language when the server has it.
            if (state.value.words.isEmpty()) {
                readWords(java.util.Locale.getDefault().language.ifEmpty { DEFAULT_LANGUAGE }, hello.brand.name)
            }
        }
        out.view?.let { view ->
            val prevActive = state.value.active
            state.update { now ->
                now.copy(
                    route = view.route ?: now.route,
                    device = view.deviceSignIn ?: now.device,
                    profiles = view.profiles?.profiles ?: now.profiles,
                    play = view.play ?: now.play,
                    brand = view.hello?.brand?.name ?: now.brand,
                    active = view.session?.profile ?: now.active,
                )
            }
            matchLanguage()
            if (state.value.active != null && state.value.active != prevActive) {
                refreshNotifications()
            }
        }
        loop.removeCallbacks(wake)
        out.nextWakeMs?.let { at ->
            val after = (at - SystemClock.elapsedRealtime()).coerceIn(0, WAKE_CAP_MS)
            loop.postDelayed(wake, after)
        }
        out.effects.forEach(::perform)
    }

    private fun perform(effect: Effect) {
        when (effect) {
            is EffectHttp -> http(effect)
            is EffectSecret -> queue.addLast(secret(effect))
            is EffectStore -> queue.addLast(kept(effect))
            is EffectPlayer -> player.command(effect.command)
            is EffectView -> effect.notice?.let { notice ->
                state.update { now ->
                    now.copy(
                        notice = notice.text.ifEmpty { now.words[notice.label] },
                        notices = now.notices + 1,
                    )
                }
            }
            is EffectOpen, is EffectLog -> Unit
            else -> queue.addLast(EventUnsupported(id = effect.id))
        }
    }

    private fun kept(effect: EffectStore): Event {
        val key = effect.key.orEmpty()
        return when (effect.op) {
            StoreOp.GET -> EventStoreDone(id = effect.id, value = store.get(key))
            StoreOp.PUT -> {
                store.put(key, effect.value.orEmpty())
                EventStoreDone(id = effect.id)
            }
            StoreOp.DELETE -> {
                store.delete(key)
                EventStoreDone(id = effect.id)
            }
            StoreOp.LIST -> EventStoreDone(id = effect.id, keys = store.keys(key))
            else -> EventUnsupported(id = effect.id)
        }
    }

    private fun secret(effect: EffectSecret): Event {
        val name = "core:${effect.name}"
        return when (effect.op) {
            SecretOp.GET -> EventSecretDone(id = effect.id, value = secrets.get(name))
            SecretOp.PUT -> {
                // A refused write is not a write. Saying it was done would sign the viewer out at
                // the next launch with no signal, where the core keeps the tokens in memory instead.
                if (secrets.put(name, effect.value.orEmpty())) {
                    EventSecretDone(id = effect.id)
                } else {
                    EventUnsupported(id = effect.id)
                }
            }
            else -> {
                secrets.drop(name)
                EventSecretDone(id = effect.id)
            }
        }
    }

    private fun http(effect: EffectHttp) {
        val call = Call(effect.method.value, base + effect.path, effect.headers.orEmpty(), effect.body, effect.deadlineMs)
        network.send(call) { answer ->
            when (answer) {
                is NoReply -> {
                    loop.post { waiting.remove(effect.id)?.invoke(0, "") }
                    send(listOf(EventHttpFailed(id = effect.id, reason = NoAnswerReason(answer.reason))))
                }
                is Reply -> {
                    val good = answer.status in 200..299
                    val toScreen = effect.to == BodyTarget.SCREEN && good
                    if (toScreen) keep(effect.path, answer.text)
                    loop.post { waiting.remove(effect.id)?.invoke(answer.status, answer.text) }
                    val done = EventHttpDone(
                        id = effect.id,
                        body = answer.text.takeIf { !toScreen && it.isNotEmpty() },
                        headers = answer.etag?.let { mapOf("etag" to it) },
                        status = answer.status.toLong(),
                    )
                    send(listOf(done))
                }
            }
        }
    }

    private fun keep(path: String, text: String) {
        when {
            path.startsWith("/v1/nav") ->
                state.update { it.copy(nav = contractJson.decodeFromString(Nav.serializer(), text).items) }
            path.startsWith("/v1/screens/") ->
                state.update { it.copy(screen = contractJson.decodeFromString(Screen.serializer(), text)) }
            path.startsWith("/v1/notifications/count") ->
                state.update { it.copy(unreadNotifications = contractJson.decodeFromString(NotificationCount.serializer(), text).unread.toInt()) }
            path.contains("/titles/") && !path.contains("/seasons/") ->
                state.update { it.copy(detail = contractJson.decodeFromString(TitleDetail.serializer(), text)) }
        }
    }
}
