import AVFoundation
import CoreMedia
import Contract
import CryptoKit
import Foundation
import Network
import Ports
import Security
import UIKit

/// What this build and this device are. One place reads it, because three core policies key on
/// exactly these fields: the client version floor, the per class start deadline and how long a
/// profile may be left before the gate asks again. A device that calls itself a television is
/// never asked who is watching.
public enum ThisDevice {
    /// From the build target, not from the device: one binary is one operating system.
    public static var os: String {
        #if os(tvOS)
        return "tvos"
        #else
        return "ios"
        #endif
    }

    public static var deviceClass: DeviceClass {
        switch UIDevice.current.userInterfaceIdiom {
        case .phone: return .phone
        case .pad: return .tablet
        case .tv: return .tv
        // A headset or a Mac running this app is held at arm's length on a big picture, and it is
        // nobody's television, so it takes the desktop rules rather than the television's.
        default: return .desktop
        }
    }

    public static var form: Form {
        switch UIDevice.current.userInterfaceIdiom {
        case .phone: return .phone
        case .pad: return .tablet
        case .tv: return .tv
        default: return .desktop
        }
    }

    /// The version this build was stamped with. A build with none cannot be told apart from an old
    /// one, and the server's version floor would read it as 0, so the build is wrong, not the run.
    public static var version: String {
        guard let text = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String,
              !text.isEmpty else {
            fatalError("this bundle has no CFBundleShortVersionString")
        }
        return text
    }
}

/// The network port, using URLSession and NWPathMonitor. It carries the contract, which is text;
/// pictures have a port of their own (`Pictures.swift`), because a picture is bytes with a cache, a
/// decode and a size behind it and none of that belongs to a surface either.
public final class UrlNetwork: Network, @unchecked Sendable {
    private let session: URLSession
    private let monitor = NWPathMonitor()
    private let monitorQueue = DispatchQueue(label: "network.monitor")
    private var isOnline: Bool = true

    public init() {
        let config = URLSessionConfiguration.default
        self.session = URLSession(configuration: config)
    }

    public func watch(_ changed: @escaping @Sendable (Bool) -> Void) {
        monitor.pathUpdateHandler = { path in
            let online = path.status == .satisfied
            changed(online)
        }
        monitor.start(queue: monitorQueue)
    }

    public func send(_ call: Call, done: @escaping @Sendable (Answer) -> Void) {
        guard let url = URL(string: call.url) else {
            done(.noReply(reason: "bad_url"))
            return
        }

        var request = URLRequest(url: url)
        request.httpMethod = call.method
        request.timeoutInterval = Double(call.deadlineMs) / 1000.0
        for (key, value) in call.headers {
            request.setValue(value, forHTTPHeaderField: key)
        }
        if let body = call.body {
            request.setValue("application/json", forHTTPHeaderField: "content-type")
            request.httpBody = body.data(using: .utf8)
        }

        let task = session.dataTask(with: request) { data, response, error in
            if let error = error as? URLError {
                if error.code == .timedOut {
                    done(.noReply(reason: "timeout"))
                } else {
                    done(.noReply(reason: "offline"))
                }
                return
            }

            guard let http = response as? HTTPURLResponse else {
                done(.noReply(reason: "no_response"))
                return
            }

            let text = data.flatMap { String(data: $0, encoding: .utf8) } ?? ""
            let etag = http.value(forHTTPHeaderField: "etag")
            done(.reply(status: http.statusCode, text: text, etag: etag))
        }
        task.resume()
    }

}

/// The secrets port, using the system Keychain.
public final class KeychainSecrets: Secrets, @unchecked Sendable {
    private let service = "streams.secrets"

    public init() {}

    public func get(_ name: String) -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: name,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]
        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        guard status == errSecSuccess, let data = item as? Data else { return nil }
        return String(data: data, encoding: .utf8)
    }

    public func put(_ name: String, _ value: String) -> Bool {
        let data = Data(value.utf8)
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: name,
        ]
        // This device only, and only while it is unlocked: a session token belongs to the device
        // the viewer signed in on, and must not ride a backup or a transfer onto another one. It is
        // set on the update as well, so an item written by an older build is moved over too.
        let update: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
        ]
        let status = SecItemUpdate(query as CFDictionary, update as CFDictionary)
        if status != errSecItemNotFound {
            return status == errSecSuccess
        }
        var attributes = query
        attributes[kSecValueData as String] = data
        attributes[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        return SecItemAdd(attributes as CFDictionary, nil) == errSecSuccess
    }

    public func drop(_ name: String) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: name,
        ]
        SecItemDelete(query as CFDictionary)
    }
}

/// The store port, as one file per key in Application Support.
public final class FileStore: Store, @unchecked Sendable {
    private let folder: URL
    private let partExtension = ".part"

    public init() {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        self.folder = base.appendingPathComponent("store", isDirectory: true)
        try? FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    }

    private func fileOf(_ key: String) -> URL {
        let digest = SHA256.hash(data: Data(key.utf8))
        let hex = digest.map { String(format: "%02x", $0) }.joined()
        return folder.appendingPathComponent(hex)
    }

    public func get(_ key: String) -> String? {
        let file = fileOf(key)
        guard let content = try? String(contentsOf: file, encoding: .utf8) else { return nil }
        guard let newline = content.firstIndex(of: "\n") else { return nil }
        return String(content[content.index(after: newline)...])
    }

    public func put(_ key: String, _ value: String) {
        let file = fileOf(key)
        let part = folder.appendingPathComponent(file.lastPathComponent + partExtension)
        let payload = "\(key)\n\(value)"
        guard (try? payload.write(to: part, atomically: true, encoding: .utf8)) != nil else { return }
        _ = try? FileManager.default.removeItem(at: file)
        _ = try? FileManager.default.moveItem(at: part, to: file)
    }

    public func delete(_ key: String) {
        let file = fileOf(key)
        try? FileManager.default.removeItem(at: file)
    }

    public func keys(_ prefix: String) -> [String] {
        guard let items = try? FileManager.default.contentsOfDirectory(at: folder, includingPropertiesForKeys: nil) else {
            return []
        }
        var found: [String] = []
        for item in items where !item.lastPathComponent.hasSuffix(partExtension) {
            guard let content = try? String(contentsOf: item, encoding: .utf8) else { continue }
            let firstLine = content.components(separatedBy: "\n").first ?? ""
            if firstLine.hasPrefix(prefix) {
                found.append(firstLine)
            }
        }
        return found
    }
}

/// Subtitle sizes against the system's own, for size 0, 1 and 2.
private let wordSizes: [Double] = [80, 100, 130]
/// Where the subtitle line sits, as a share of the picture from the top, when nothing covers it.
private let wordsHome: Double = 88

/// The player port on AVFoundation. It reports facts, and the core picks every next step.
public final class AvPlayer: Player, @unchecked Sendable {
    private let engine = AVPlayer()
    private var report: (Int64, DriverEvent) -> Void = { _, _ in }
    private var generation: Int64 = 0
    private var firstFrame = false
    private var screen: PlayerScreen?
    private var watching: [NSKeyValueObservation] = []
    private var itemWatching: [NSKeyValueObservation] = []
    private var itemNotes: [NSObjectProtocol] = []
    private var tick: Any?
    private var audioGroup: AVMediaSelectionGroup?
    private var textGroup: AVMediaSelectionGroup?
    private var look = (size: 1, color: "#ffffff", background: "none")
    private var lifted: Float = 0

    public init() {
        tick = engine.addPeriodicTimeObserver(forInterval: CMTime(value: 1, timescale: 1), queue: .main) { [weak self] time in
            guard let self, self.engine.timeControlStatus == .playing else { return }
            let length = self.engine.currentItem?.duration.seconds ?? .nan
            self.report(self.generation, .position(DriverEventPosition(
                durationMs: length.isFinite ? Int(length * 1000) : nil,
                timeMs: Int(time.seconds * 1000)
            )))
        }
        watching.append(engine.observe(\.timeControlStatus) { [weak self] player, _ in
            DispatchQueue.main.async {
                guard let self else { return }
                switch player.timeControlStatus {
                case .waitingToPlayAtSpecifiedRate where player.reasonForWaitingToPlay == .toMinimizeStalls:
                    self.report(self.generation, .stalled(DriverEventStalled()))
                case .playing where self.screen == nil:
                    // With no picture on screen, playing is the first sign that it works.
                    self.saw()
                default: break
                }
            }
        })
    }

    private func landed() {
        let at = engine.currentTime().seconds
        let length = engine.currentItem?.duration.seconds ?? .nan
        guard at.isFinite else { return }
        report(generation, .position(DriverEventPosition(durationMs: length.isFinite ? Int(length * 1000) : nil, timeMs: Int(at * 1000))))
    }

    private func saw() {
        guard !firstFrame else { return }
        firstFrame = true
        report(generation, .firstFrame(DriverEventFirstFrame()))
    }

    public func listen(_ report: @escaping (Int64, DriverEvent) -> Void) {
        self.report = report
    }

    public func picture(_ context: Any) -> Any {
        let view = PlayerScreen()
        view.screen.player = engine
        view.screen.videoGravity = .resizeAspect
        view.ready = view.screen.observe(\.isReadyForDisplay) { [weak self] layer, _ in
            if layer.isReadyForDisplay { DispatchQueue.main.async { self?.saw() } }
        }
        screen = view
        return view
    }

    public func now() -> Int64 {
        let seconds = engine.currentTime().seconds
        return seconds.isFinite ? Int64(seconds * 1000) : 0
    }

    public func lift(_ share: Float) {
        DispatchQueue.main.async {
            self.lifted = share
            self.dress()
        }
    }

    public func wordsLook(size: Int, color: String, background: String) {
        DispatchQueue.main.async {
            self.look = (size, color, background)
            self.dress()
        }
    }

    /// The subtitles the system draws, in the look the viewer chose and above anything covering them.
    private func dress() {
        guard let item = engine.currentItem else { return }
        var marks: [String: Any] = [
            kCMTextMarkupAttribute_RelativeFontSize as String: wordSizes[min(max(look.size, 0), wordSizes.count - 1)],
            kCMTextMarkupAttribute_OrthogonalLinePositionPercentageRelativeToWritingDirection as String:
                max(10, wordsHome - Double(lifted) * 100),
        ]
        if let ink = argb(look.color) { marks[kCMTextMarkupAttribute_ForegroundColorARGB as String] = ink }
        marks[kCMTextMarkupAttribute_CharacterBackgroundColorARGB as String] = argb(look.background) ?? [0, 0, 0, 0]
        item.textStyleRules = AVTextStyleRule(textMarkupAttributes: marks).map { [$0] }
    }

    public func command(_ given: DriverCommand) {
        DispatchQueue.main.async {
            switch given {
            case .open(let open): self.open(open)
            case .play: self.engine.play()
            case .pause: self.engine.pause()
            case .seek(let seek):
                let near = CMTime(value: 1, timescale: 4)
                self.engine.seek(to: CMTime(value: Int64(seek.toMs), timescale: 1000), toleranceBefore: near, toleranceAfter: near) { _ in
                    // A paused picture reports no time, so a finished seek says where it landed.
                    DispatchQueue.main.async { self.landed() }
                }
            case .volume(let volume): self.engine.volume = Float(volume.percent) / 100
            case .rate(let rate):
                self.engine.defaultRate = Float(rate.percent) / 100
                if self.engine.timeControlStatus == .playing { self.engine.rate = self.engine.defaultRate }
            case .selectAudio(let audio): self.choose(self.audioGroup, audio.track)
            case .selectText(let text): self.choose(self.textGroup, text.track)
            case .close:
                self.engine.pause()
                self.forget()
                self.engine.replaceCurrentItem(with: nil)
            default: break
            }
        }
    }

    private func forget() {
        itemWatching = []
        itemNotes.forEach(NotificationCenter.default.removeObserver)
        itemNotes = []
        audioGroup = nil
        textGroup = nil
    }

    private func open(_ given: DriverCommandOpen) {
        forget()
        generation = Int64(given.generation)
        firstFrame = false
        guard let address = URL(string: given.url) else {
            report(generation, .error(DriverEventError(class: .network)))
            return
        }
        let options: [String: Any] = given.headers.isEmpty ? [:] : ["AVURLAssetHTTPHeaderFieldsKey": given.headers]
        let item = AVPlayerItem(asset: AVURLAsset(url: address, options: options))
        if let bits = given.maxBitrate { item.preferredPeakBitRate = Double(bits) }
        if let tall = given.maxHeight { item.preferredMaximumResolution = CGSize(width: Double(tall) * 16 / 9, height: Double(tall)) }
        let run = generation
        itemWatching.append(item.observe(\.status) { [weak self] item, _ in
            DispatchQueue.main.async {
                guard let self, run == self.generation else { return }
                switch item.status {
                case .readyToPlay: Task { await self.readTracks(item, run) }
                case .failed: self.report(run, .error(DriverEventError(class: Self.kind(of: item.error))))
                default: break
                }
            }
        })
        itemNotes.append(NotificationCenter.default.addObserver(
            forName: .AVPlayerItemDidPlayToEndTime, object: item, queue: .main
        ) { [weak self] _ in self?.report(run, .ended(DriverEventEnded())) })
        itemNotes.append(NotificationCenter.default.addObserver(
            forName: .AVPlayerItemFailedToPlayToEndTime, object: item, queue: .main
        ) { [weak self] note in
            let error = note.userInfo?[AVPlayerItemFailedToPlayToEndTimeErrorKey] as? Error
            self?.report(run, .error(DriverEventError(class: Self.kind(of: error))))
        })
        engine.replaceCurrentItem(with: item)
        dress()
        if given.startAtMs > 0 {
            engine.seek(to: CMTime(value: Int64(given.startAtMs), timescale: 1000), toleranceBefore: .zero, toleranceAfter: .zero)
        }
        engine.play()
    }

    private static func kind(of error: Error?) -> ErrorClass {
        guard let error = error as NSError? else { return .network }
        if error.domain == NSURLErrorDomain { return .network }
        switch AVError.Code(rawValue: error.code) {
        case .decodeFailed, .decoderNotFound, .decoderTemporarilyUnavailable: return .decode
        case .fileFormatNotRecognized, .failedToParse: return .format
        default: return .network
        }
    }

    /// Sound and subtitle tracks from the title's own choices. A track's id is its place in its
    /// group, so it holds only for this title.
    private func readTracks(_ item: AVPlayerItem, _ run: Int64) async {
        let asset = item.asset
        let audible = try? await asset.loadMediaSelectionGroup(for: .audible)
        let legible = try? await asset.loadMediaSelectionGroup(for: .legible)
        await MainActor.run {
            guard run == self.generation else { return }
            self.audioGroup = audible
            self.textGroup = legible
            self.tell(item)
        }
    }

    private func tell(_ item: AVPlayerItem) {
        func listed(_ group: AVMediaSelectionGroup?, _ name: String) -> [Track] {
            guard let group else { return [] }
            let chosen = item.currentMediaSelection.selectedMediaOption(in: group)
            return group.options.enumerated().map { index, option in
                Track(
                    id: "\(name):\(index)",
                    label: option.displayName,
                    lang: option.extendedLanguageTag ?? option.locale?.identifier,
                    selected: option == chosen
                )
            }
        }
        report(generation, .tracks(DriverEventTracks(audio: listed(audioGroup, "audio"), text: listed(textGroup, "text"))))
    }

    private func choose(_ group: AVMediaSelectionGroup?, _ id: String?) {
        guard let group, let item = engine.currentItem else { return }
        let option = id.flatMap { $0.split(separator: ":").last.flatMap { Int($0) } }.flatMap { group.options[safe: $0] }
        item.select(option, in: group)
        tell(item)
    }
}

/// The picture of what plays.
public final class PlayerScreen: UIView {
    public override class var layerClass: AnyClass { AVPlayerLayer.self }
    var screen: AVPlayerLayer { layer as! AVPlayerLayer }
    var ready: NSKeyValueObservation?
}

private extension Array {
    subscript(safe index: Int) -> Element? { indices.contains(index) ? self[index] : nil }
}

/// A colour as the text markup wants it: alpha, red, green and blue from 0 to 1. Nothing for none.
private func argb(_ text: String) -> [Double]? {
    let words: [String: [Double]] = ["white": [1, 1, 1, 1], "black": [1, 0, 0, 0], "yellow": [1, 1, 1, 0]]
    if let named = words[text.lowercased()] { return named }
    guard text.hasPrefix("#"), text.count == 7, let value = Int(text.dropFirst(), radix: 16) else { return nil }
    return [1, Double((value >> 16) & 0xFF) / 255, Double((value >> 8) & 0xFF) / 255, Double(value & 0xFF) / 255]
}

/// Brand films and sounds, each on its own engine so it can be handed back as soon as it is off screen.
public final class AvReel: Reel {
    private var playing: [ReelView] = []
    private var still = false

    public init() {}

    public func picture(_ context: Any, url: String, loop: Bool, ready: @escaping () -> Void, ended: @escaping () -> Void) -> Any {
        let view = ReelView()
        guard let address = URL(string: url) else {
            DispatchQueue.main.async(execute: ended)
            return view
        }
        let item = AVPlayerItem(url: address)
        let engine = AVQueuePlayer()
        if loop {
            view.looper = AVPlayerLooper(player: engine, templateItem: item)
        } else {
            engine.insert(item, after: nil)
            view.watch.append(NotificationCenter.default.addObserver(
                forName: .AVPlayerItemDidPlayToEndTime, object: item, queue: .main
            ) { _ in ended() })
        }
        view.watch.append(NotificationCenter.default.addObserver(
            forName: .AVPlayerItemFailedToPlayToEndTime, object: nil, queue: .main
        ) { note in
            if let failed = note.object as? AVPlayerItem, failed.asset === item.asset { ended() }
        })
        var told = false
        view.status = engine.observe(\.currentItem?.status, options: [.initial, .new]) { player, _ in
            let status = player.currentItem?.status
            DispatchQueue.main.async {
                if status == .readyToPlay && !told {
                    told = true
                    ready()
                } else if status == .failed {
                    ended()
                }
            }
        }
        view.screen.player = engine
        view.screen.videoGravity = .resizeAspectFill
        playing.append(view)
        return view
    }

    public func play(_ picture: Any, _ on: Bool) {
        guard let view = picture as? ReelView else { return }
        view.wanted = on
        if on && !still { view.screen.player?.play() } else { view.screen.player?.pause() }
    }

    public func release(_ picture: Any) {
        guard let view = picture as? ReelView else { return }
        playing.removeAll { $0 === view }
        view.screen.player?.pause()
        view.looper?.disableLooping()
        view.looper = nil
        view.watch.forEach(NotificationCenter.default.removeObserver)
        view.watch = []
        view.status = nil
        view.screen.player = nil
    }

    public func hold(_ still: Bool) {
        self.still = still
        for view in playing {
            if view.wanted && !still { view.screen.player?.play() } else { view.screen.player?.pause() }
        }
    }
}

/// A film's picture. Until the first frame, what is behind it shows through.
public final class ReelView: UIView {
    public override class var layerClass: AnyClass { AVPlayerLayer.self }
    var screen: AVPlayerLayer { layer as! AVPlayerLayer }
    var looper: AVPlayerLooper?
    var watch: [NSObjectProtocol] = []
    var status: NSKeyValueObservation?
    var wanted = false
}
