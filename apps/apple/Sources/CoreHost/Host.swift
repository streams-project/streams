import Contract
import CoreBridge
import Foundation
import Ports

public enum NativeCore {
    public static func abi() -> Int {
        Int(core_abi())
    }

    public static func open(config: Data) -> OpaquePointer? {
        config.withUnsafeBytes { buf in
            guard let base = buf.bindMemory(to: UInt8.self).baseAddress else { return nil }
            return core_open(base, buf.count)
        }
    }

    public static func step(handle: OpaquePointer, input: Data) -> Data? {
        var outPtr: UnsafeMutablePointer<UInt8>? = nil
        var outLen: Int = 0
        let status = input.withUnsafeBytes { inBuf in
            core_step(handle, inBuf.bindMemory(to: UInt8.self).baseAddress, inBuf.count, &outPtr, &outLen)
        }
        guard status == CORE_OK, let ptr = outPtr else { return nil }
        let data = Data(bytes: ptr, count: outLen)
        core_free(ptr, outLen)
        return data
    }

    public static func close(handle: OpaquePointer) {
        core_close(handle)
    }
}

/// Steps the core and performs what it asks, through the ports.
public final class Host: Shell, @unchecked Sendable {
    private static let serverKey = "app:server"
    private static let defaultLanguage = "en"
    private static let wakeCapMs: Int64 = 60_000
    private static let touchEveryMs: Int64 = 10_000
    private static let restartLimit = 3
    // Said to the viewer, not logged: these two are the only moments the host has words of its own.
    private static let plainRefused =
        "This address must start with https://. Plain http reaches only a server on your own network."
    private static let coreStopped = "This app could not start. Close it and open it again."

    private let network: Network
    private let secrets: Secrets
    private let store: Store
    public let player: Player
    public let reel: Reel
    public let pictures: Pictures
    private let deviceClass: DeviceClass
    private let form: Form
    private let version: String
    private let os: String

    private let queue = DispatchQueue(label: "core", qos: .userInteractive)
    @MainActor public let seen: Seen = Seen()
    private var handle: OpaquePointer? = nil
    private var base: String = ""
    private var matched: String? = nil
    private var eventQueue: [Event] = []
    private var asks: Int = 0
    private var waiting: [String: (Int, String) -> Void] = [:]
    private var trail: [String] = []
    private var here: String = "/"
    private var wakeWorkItem: DispatchWorkItem? = nil
    private var lastTouch: Int64 = 0
    private var restarts: Int = 0

    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    /// Nothing here has a default, `pictures` included: a port with a stand-in behind it lets an
    /// entry point ship without an adapter and fail at run time, and this one draws every screen.
    /// Three core policies key on the class, the form and the version,
    /// and one on the operating system, so an entry point that left any of them out would inherit
    /// another device's rules in silence: a phone that called itself a television was never asked
    /// who is watching after time away.
    public init(
        network: Network,
        secrets: Secrets,
        store: Store,
        player: Player,
        reel: Reel,
        pictures: Pictures,
        deviceClass: DeviceClass,
        form: Form,
        version: String,
        os: String
    ) {
        self.network = network
        self.secrets = secrets
        self.store = store
        self.player = player
        self.reel = reel
        self.pictures = pictures
        self.deviceClass = deviceClass
        self.form = form
        self.version = version
        self.os = os

        network.watch { [weak self] online in
            self?.sendEvents([Event(id: nil, body: .network(EventNetwork(online: online)))])
        }

        player.listen { [weak self] generation, event in
            guard let self = self else { return }
            switch event {
            case .position(let pos):
                Task { @MainActor in
                    self.seen.positionMs = Int64(pos.timeMs)
                    if let dur = pos.durationMs { self.seen.lengthMs = Int64(dur) }
                }
            case .tracks(let tr):
                Task { @MainActor in
                    self.seen.audio = tr.audio
                    self.seen.text = tr.text
                }
            default:
                break
            }
            self.sendEvents([Event(id: nil, body: .driver(EventDriver(event: event, generation: Int(generation))))])
        }

        if let kept = secrets.get(Self.serverKey) {
            connect(kept)
        } else if let envServer = ProcessInfo.processInfo.environment["SERVER_URL"], !envServer.isEmpty {
            connect(envServer)
        }
    }

    /// The one gate every server address passes, typed or kept. Plain http carries the session's
    /// bearer and refresh tokens in the clear, so it may only reach the viewer's own network;
    /// anything else must be https. A refused address is said, not swallowed.
    public func connect(_ server: String) {
        let trimmed = server
            .trimmingCharacters(in: .whitespaces)
            .trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        guard let plain = Self.encryption(of: trimmed) else {
            Task { @MainActor in self.said(Self.plainRefused) }
            return
        }
        // The screens see a kept server from the first frame, so a run with a server never looks
        // like a first run without one.
        if Thread.isMainThread {
            MainActor.assumeIsolated {
                seen.server = trimmed
                seen.serverPlain = plain
                // A notice is one time. This address was taken, so a refusal still sitting in the
                // app wide slot is not news any more and must not be read as a screen's own error.
                seen.notice = nil
            }
        }
        queue.async {
            self.base = trimmed
            _ = self.secrets.put(Self.serverKey, self.base)
            Task { @MainActor in
                self.seen.server = self.base
                self.seen.serverPlain = plain
                self.seen.notice = nil
            }
            if self.handle == nil {
                self.open()
            }
        }
    }

    /// True when this address is reached in the clear, false when it is encrypted, and nil when it
    /// may not be used at all.
    static func encryption(of server: String) -> Bool? {
        let lower = server.lowercased()
        if lower.hasPrefix("https://") { return false }
        guard lower.hasPrefix("http://") else { return nil }
        return ownNetwork(host(of: lower)) ? true : nil
    }

    /// The host of an address, with the port, the path and an IPv6 address's brackets taken off.
    private static func host(of address: String) -> String {
        let after = address.dropFirst("http://".count)
        let upTo = after.prefix { $0 != "/" }
        if upTo.hasPrefix("[") {
            return String(upTo.dropFirst().prefix { $0 != "]" })
        }
        return String(upTo.prefix { $0 != ":" })
    }

    /// The viewer's own machine or their own network: a hop no stranger is on. Names are here
    /// because a server is usually found by its name on the network, not by its number.
    private static func ownNetwork(_ host: String) -> Bool {
        if host == "localhost" || host.hasSuffix(".local") || host == "::1" { return true }
        let parts = host.split(separator: ".").map { Int($0) }
        guard parts.count == 4, let first = parts[0], let second = parts[1],
              parts.allSatisfy({ $0.map { (0...255).contains($0) } == true }) else {
            return false
        }
        switch (first, second) {
        case (127, _), (10, _), (192, 168), (169, 254): return true
        case (172, 16...31): return true
        default: return false
        }
    }

    private func open() {
        let config = try! CoreConfig(
            app: "apple",
            class: deviceClass,
            contract: WireDate(wireVersion),
            form: form,
            mode: .local,
            os: os,
            seed: seed(),
            type: .value,
            version: version
        )
        guard NativeCore.abi() == abiVersion else {
            fatalError("this core has another ABI")
        }
        guard let cfgData = try? encoder.encode(config),
              let h = NativeCore.open(config: cfgData) else {
            fatalError("the core did not open")
        }
        self.handle = h
        sendEvents([Event(id: nil, body: .address(EventAddress(address: "/")))])
    }

    private func seed() -> String {
        var bytes = [UInt8](repeating: 0, count: 16)
        _ = SecRandomCopyBytes(kSecRandomDefault, bytes.count, &bytes)
        return bytes.map { String(format: "%02x", $0) }.joined()
    }

    public func go(_ address: String) {
        queue.async {
            self.trail.append(self.here)
            self.here = address
            self.sendEvents([Event(id: nil, body: .address(EventAddress(address: address)))])
        }
    }

    public func shown(_ visible: Bool) {
        reel.hold(!visible)
        let pause: [Event] = visible ? [] : [Event(id: nil, body: .action(EventAction(action: .pause(ActionPause()))))]
        sendEvents(pause + [Event(id: nil, body: .visibility(EventVisibility(visible: visible)))])
    }

    public func touched() {
        let now = Int64(ProcessInfo.processInfo.systemUptime * 1000)
        if now - lastTouch < Self.touchEveryMs { return }
        lastTouch = now
        sendEvents([Event(id: nil, body: .activity(EventActivity()))])
    }

    public func swap(_ address: String) {
        queue.async {
            self.here = address
            self.sendEvents([Event(id: nil, body: .address(EventAddress(address: address)))])
        }
    }

    public func back() {
        queue.async {
            self.here = self.trail.popLast() ?? "/"
            self.sendEvents([Event(id: nil, body: .address(EventAddress(address: self.here)))])
        }
    }

    public func act(_ action: Action) {
        sendEvents([Event(id: nil, body: .action(EventAction(action: action)))])
    }

    public func ask(_ path: String, reply: @escaping (Int, String) -> Void) {
        queue.async {
            self.asks += 1
            let id = "ask-\(self.asks)"
            self.waiting[id] = reply
            let action = ActionRequest(method: .gET, path: path)
            self.sendEvents([Event(id: id, body: .action(EventAction(action: .request(action))))])
        }
    }

    public func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) {
        queue.async {
            self.asks += 1
            let id = "ask-\(self.asks)"
            self.waiting[id] = reply
            let action = ActionRequest(body: body, method: HttpMethod(rawValue: method), path: path)
            self.sendEvents([Event(id: id, body: .action(EventAction(action: .request(action))))])
        }
    }

    /// The one place a notice is said. Every notice moves `notices`, so a screen that acted can
    /// tell the answer to its own action from words another screen left in this shared slot.
    @MainActor private func said(_ notice: String?) {
        seen.notice = notice
        seen.notices += 1
    }

    public func keep(_ profile: Profile) {
        Task { @MainActor in
            self.seen.profiles = self.seen.profiles.map { $0.id == profile.id ? profile : $0 }
        }
    }

    public func keepAll(_ profiles: [Profile]) {
        Task { @MainActor in
            self.seen.profiles = profiles
        }
    }

    public func speak(_ language: String?) {
        Task { @MainActor in
            let brand = self.seen.brand
            self.readWords(language ?? Self.defaultLanguage, brand)
        }
    }

    public func refreshNotifications() {
        Task { @MainActor in
            guard self.seen.active != nil else { return }
            self.ask("/v1/notifications/count") { [weak self] status, text in
                guard let self = self, (200...299).contains(status),
                      let count = try? self.decoder.decode(NotificationCount.self, from: Data(text.utf8)) else { return }
                Task { @MainActor in
                    self.seen.unreadNotifications = count.unread
                }
            }
        }
    }

    private func sendEvents(_ events: [Event]) {
        queue.async {
            guard let h = self.handle else { return }
            self.eventQueue.append(contentsOf: events)
            var rounds = 0
            repeat {
                let batch = self.eventQueue
                self.eventQueue.removeAll()
                let mono = Int(ProcessInfo.processInfo.systemUptime * 1000)
                let now = Int(Date().timeIntervalSince1970 * 1000)
                let input = CoreIn(
                    contract: try! WireDate(wireVersion),
                    events: batch,
                    monoMs: mono,
                    nowMs: now,
                    type: .value
                )
                guard let inData = try? self.encoder.encode(input),
                      let outData = NativeCore.step(handle: h, input: inData),
                      let out = try? self.decoder.decode(CoreOut.self, from: outData) else {
                    // A refused step lost its whole batch, so the core may be waiting on answers it
                    // will never get, and it keeps no timer of its own. Dropping it quietly leaves
                    // the app sitting there with nothing moving, so start the core again.
                    self.restart()
                    return
                }
                self.apply(out)
                rounds += 1
            } while !self.eventQueue.isEmpty && rounds < 64
        }
    }

    /// Opens a fresh core and gives every waiting ask its no answer, up to a few tries.
    private func restart() {
        if let h = handle { NativeCore.close(handle: h) }
        handle = nil
        eventQueue.removeAll()
        wakeWorkItem?.cancel()
        wakeWorkItem = nil
        let pending = waiting
        waiting.removeAll()
        for reply in pending.values { reply(0, "") }
        restarts += 1
        if restarts > Self.restartLimit {
            Task { @MainActor in self.said(Self.coreStopped) }
            return
        }
        open()
    }

    private func readWords(_ language: String, _ brand: String) {
        ask("/v1/strings/\(language)") { [weak self] status, text in
            guard let self = self else { return }
            // A language this server does not have falls back to English, so a screen always has words.
            if !(200...299).contains(status) && language != Self.defaultLanguage {
                Task { @MainActor in self.readWords(Self.defaultLanguage, brand) }
                return
            }
            if (200...299).contains(status),
               let read = try? self.decoder.decode(Strings.self, from: Data(text.utf8)) {
                // A right to left language's words start with a right to left mark, so a line that
                // opens with the brand's Latin name still reads right to left.
                let rtl = read.dir == .rtl
                var mapped: [String: String] = [:]
                for (key, text) in read.texts {
                    let mark = rtl && !key.hasPrefix("language.") ? "\u{200F}" : ""
                    mapped[key] = mark + text.replacingOccurrences(of: "{brand}", with: brand)
                }
                Task { @MainActor in
                    self.seen.words = mapped
                    self.seen.language = read.lang
                    self.seen.rtl = rtl
                }
            }
        }
    }

    private func matchLanguage() {
        Task { @MainActor in
            let now = self.seen
            guard let wanted = now.profiles.first(where: { $0.id == now.active })?.language else { return }
            if wanted != now.language && wanted != self.matched {
                self.matched = wanted
                self.readWords(wanted, now.brand)
            }
        }
    }

    private func apply(_ out: CoreOut) {
        if let hello = out.view?.hello {
            Task { @MainActor in
                // Before any profile, the screens speak the device's language when the server has it.
                if self.seen.words.isEmpty {
                    let device = Locale.current.language.languageCode?.identifier ?? Self.defaultLanguage
                    self.readWords(device, hello.brand.name)
                }
            }
        }
        if let view = out.view {
            Task { @MainActor in
                if let r = view.route { self.seen.route = r }
                if let d = view.deviceSignIn { self.seen.device = d }
                if let p = view.profiles?.profiles { self.seen.profiles = p }
                if let pl = view.play { self.seen.play = pl }
                if let b = view.hello?.brand.name { self.seen.brand = b }
                if let s = view.session?.profile {
                    let prev = self.seen.active
                    self.seen.active = s
                    if s != prev {
                        self.refreshNotifications()
                    }
                }
                self.matchLanguage()
            }
        }

        self.wakeWorkItem?.cancel()
        if let at = out.nextWakeMs {
            let mono = Int(ProcessInfo.processInfo.systemUptime * 1000)
            let after = max(0, min(Int64(at - mono), Self.wakeCapMs))
            let item = DispatchWorkItem { [weak self] in
                self?.sendEvents([])
            }
            self.wakeWorkItem = item
            queue.asyncAfter(deadline: .now() + .milliseconds(Int(after)), execute: item)
        }

        for effect in out.effects {
            perform(effect)
        }
    }

    private func perform(_ effect: Effect) {
        switch effect.body {
        case .http(let httpEffect):
            http(effect.id, httpEffect)
        case .secret(let secretEffect):
            eventQueue.append(secret(effect.id, secretEffect))
        case .store(let storeEffect):
            eventQueue.append(kept(effect.id, storeEffect))
        case .player(let playerEffect):
            player.command(playerEffect.command)
        case .view(let viewEffect):
            if let n = viewEffect.notice {
                Task { @MainActor in
                    self.said(n.text.isEmpty ? self.seen.words[n.label] : n.text)
                }
            }
        case .open, .log:
            break
        case .timer, .unknown:
            eventQueue.append(Event(id: effect.id, body: .unsupported(EventUnsupported())))
        }
    }

    private func kept(_ id: String, _ effect: EffectStore) -> Event {
        let key = effect.key ?? ""
        switch effect.op {
        case .get:
            return Event(id: id, body: .storeDone(EventStoreDone(value: store.get(key))))
        case .put:
            store.put(key, effect.value ?? "")
            return Event(id: id, body: .storeDone(EventStoreDone()))
        case .delete:
            store.delete(key)
            return Event(id: id, body: .storeDone(EventStoreDone()))
        case .list:
            return Event(id: id, body: .storeDone(EventStoreDone(keys: store.keys(key))))
        default:
            return Event(id: id, body: .unsupported(EventUnsupported()))
        }
    }

    private func secret(_ id: String, _ effect: EffectSecret) -> Event {
        let name = "core:\(effect.name)"
        switch effect.op {
        case .get:
            return Event(id: id, body: .secretDone(EventSecretDone(value: secrets.get(name))))
        case .put:
            // A refused write is not a write. Saying it was done would sign the viewer out at the
            // next launch with no signal, where the core keeps the tokens in memory instead.
            guard secrets.put(name, effect.value ?? "") else {
                return Event(id: id, body: .unsupported(EventUnsupported()))
            }
            return Event(id: id, body: .secretDone(EventSecretDone()))
        case .delete:
            secrets.drop(name)
            return Event(id: id, body: .secretDone(EventSecretDone()))
        default:
            return Event(id: id, body: .unsupported(EventUnsupported()))
        }
    }

    private func http(_ id: String, _ effect: EffectHttp) {
        let call = Call(
            method: effect.method.rawValue,
            url: base + effect.path,
            headers: effect.headers ?? [:],
            body: effect.body,
            deadlineMs: Int64(effect.deadlineMs)
        )
        network.send(call) { [weak self] answer in
            guard let self = self else { return }
            switch answer {
            case .noReply(let reason):
                self.queue.async {
                    self.waiting.removeValue(forKey: id)?(0, "")
                }
                self.sendEvents([Event(id: id, body: .httpFailed(EventHttpFailed(reason: NoAnswerReason(rawValue: reason))))])
            case .reply(let status, let text, let etag):
                let good = (200...299).contains(status)
                let toScreen = effect.to == .screen && good
                if toScreen {
                    self.keepScreen(effect.path, text)
                }
                self.queue.async {
                    self.waiting.removeValue(forKey: id)?(status, text)
                }
                let bodyToSend = (!toScreen && !text.isEmpty) ? text : nil
                var headersMap: [String: String]? = nil
                if let tag = etag { headersMap = ["etag": tag] }
                let done = EventHttpDone(body: bodyToSend, headers: headersMap, status: status)
                self.sendEvents([Event(id: id, body: .httpDone(done))])
            }
        }
    }

    private func keepScreen(_ path: String, _ text: String) {
        let data = Data(text.utf8)
        Task { @MainActor in
            if path.hasPrefix("/v1/nav"),
               let nav = try? self.decoder.decode(Nav.self, from: data) {
                self.seen.nav = nav.items
            } else if path.hasPrefix("/v1/screens/"),
                      let screen = try? self.decoder.decode(Screen.self, from: data) {
                self.seen.screen = screen
            } else if path.hasPrefix("/v1/notifications/count"),
                      let count = try? self.decoder.decode(NotificationCount.self, from: data) {
                self.seen.unreadNotifications = count.unread
            } else if path.contains("/titles/") && !path.contains("/seasons/"),
                      let detail = try? self.decoder.decode(TitleDetail.self, from: data) {
                self.seen.detail = detail
            }
        }
    }
}
