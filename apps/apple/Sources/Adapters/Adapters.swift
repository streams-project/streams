import AVFoundation
import Contract
import CryptoKit
import Foundation
import Network
import Ports
import Security

/// The network port, using URLSession and NWPathMonitor.
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

    public func put(_ name: String, _ value: String) {
        let data = Data(value.utf8)
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: name,
        ]
        let update: [String: Any] = [
            kSecValueData as String: data,
        ]
        let status = SecItemUpdate(query as CFDictionary, update as CFDictionary)
        if status == errSecItemNotFound {
            var attributes = query
            attributes[kSecValueData as String] = data
            SecItemAdd(attributes as CFDictionary, nil)
        }
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

/// The player port on AVPlayer.
public final class AvPlayer: Player {
    private var player: AVPlayer?
    private var listener: ((Int64, DriverEvent) -> Void)?
    private var generation: Int64 = 0
    private var timeObserver: Any?

    public init() {}

    public func command(_ given: DriverCommand) {
        switch given {
        case .open(let open):
            generation = Int64(open.generation)
            guard let url = URL(string: open.url) else { return }
            let item = AVPlayerItem(url: url)
            if player == nil {
                player = AVPlayer(playerItem: item)
            } else {
                player?.replaceCurrentItem(with: item)
            }
            if open.startAtMs > 0 {
                let time = CMTime(value: Int64(open.startAtMs), timescale: 1000)
                player?.seek(to: time)
            }
            player?.play()
            listener?(generation, .firstFrame(DriverEventFirstFrame()))
        case .play:
            player?.play()
        case .pause:
            player?.pause()
        case .seek(let seek):
            let time = CMTime(value: Int64(seek.toMs), timescale: 1000)
            player?.seek(to: time)
        case .close:
            player?.pause()
            player?.replaceCurrentItem(with: nil)
            listener?(generation, .ended(DriverEventEnded()))
        default:
            break
        }
    }

    public func listen(_ report: @escaping (Int64, DriverEvent) -> Void) {
        self.listener = report
    }

    public func picture(_ context: Any) -> Any {
        return player as Any
    }

    public func now() -> Int64 {
        guard let p = player else { return 0 }
        let s = p.currentTime().seconds
        return s.isNaN || s.isInfinite ? 0 : Int64(s * 1000)
    }

    public func lift(_ share: Float) {}

    public func wordsLook(size: Int, color: String, background: String) {}
}

/// Brand films and trailer playback on AVPlayer.
public final class AvReel: Reel {
    public init() {}

    public func picture(_ context: Any, url: String, loop: Bool, ended: @escaping () -> Void) -> Any {
        guard let u = URL(string: url) else { return () }
        let player = AVPlayer(url: u)
        if !loop {
            NotificationCenter.default.addObserver(
                forName: .AVPlayerItemDidPlayToEndTime,
                object: player.currentItem,
                queue: .main
            ) { _ in ended() }
        }
        player.play()
        return player
    }

    public func release(_ picture: Any) {
        (picture as? AVPlayer)?.pause()
    }

    public func hold(_ still: Bool) {}
}
