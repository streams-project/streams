package streams.core

import android.os.Handler
import android.os.HandlerThread
import android.os.SystemClock
import contract.ABI_VERSION
import contract.Action
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
import contract.Effect
import contract.EffectHttp
import contract.EffectLog
import contract.EffectOpen
import contract.EffectPlayer
import contract.EffectSecret
import contract.EffectView
import contract.Event
import contract.EventAction
import contract.EventAddress
import contract.EventDriver
import contract.EventHttpDone
import contract.EventHttpFailed
import contract.EventSecretDone
import contract.EventUnsupported
import contract.Form
import contract.HttpMethod
import contract.Nav
import contract.NoAnswerReason
import contract.Profile
import contract.Screen
import contract.SecretOp
import contract.Strings
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

/**
 * Steps the core and performs what it asks, through the ports. It holds no rules and no screens.
 * Everything runs on one thread, because a core handle is used from one thread at a time.
 */
class Host(
    private val network: Network,
    private val secrets: Secrets,
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

    override val seen: StateFlow<Seen> = state

    init {
        player.listen { generation, event ->
            if (event is DriverEventPosition) {
                state.update { it.copy(positionMs = event.timeMs, lengthMs = event.durationMs ?: it.lengthMs) }
            }
            send(listOf(EventDriver(event = event, generation = generation)))
        }
        val kept = secrets.get(SERVER)
        if (kept != null) connect(kept)
    }

    override fun connect(server: String) {
        loop.post {
            base = server.trimEnd('/')
            secrets.put(SERVER, base)
            state.update { it.copy(server = base) }
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
            if (status in 200..299) {
                val read = contractJson.decodeFromString(Strings.serializer(), text)
                state.update { now ->
                    now.copy(
                        words = read.texts.mapValues { (_, word) -> word.replace("{brand}", brand) },
                        language = read.lang,
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
            if (state.value.words.isEmpty()) readWords(DEFAULT_LANGUAGE, hello.brand.name)
        }
        out.view?.let { view ->
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
            is EffectPlayer -> player.command(effect.command)
            is EffectView -> effect.notice?.let { notice ->
                state.update { now -> now.copy(notice = notice.text.ifEmpty { now.words[notice.label] }) }
            }
            is EffectOpen, is EffectLog -> Unit
            else -> queue.addLast(EventUnsupported(id = effect.id))
        }
    }

    private fun secret(effect: EffectSecret): Event {
        val name = "core:${effect.name}"
        return when (effect.op) {
            SecretOp.GET -> EventSecretDone(id = effect.id, value = secrets.get(name))
            SecretOp.PUT -> {
                secrets.put(name, effect.value.orEmpty())
                EventSecretDone(id = effect.id)
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
            path.contains("/titles/") && !path.contains("/seasons/") ->
                state.update { it.copy(detail = contractJson.decodeFromString(TitleDetail.serializer(), text)) }
        }
    }
}
