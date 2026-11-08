// The one place that plugs the adapters into the ports, for iPhone and iPad.
import Adapters
import CoreHost
import SurfacesPhone
import SwiftUI

@main
struct MainApp: App {
    @State private var host: Host
    @Environment(\.scenePhase) private var phase

    init() {
        let host = Host(
            network: UrlNetwork(),
            secrets: KeychainSecrets(),
            store: FileStore(),
            player: AvPlayer(),
            reel: AvReel()
        )
        // Pictures come through the system's shared cache; rows of posters need more than its default.
        URLCache.shared = URLCache(memoryCapacity: 64 << 20, diskCapacity: 512 << 20)
        _host = State(initialValue: host)
    }

    var body: some Scene {
        WindowGroup {
            Phone(shell: host)
        }
        .onChange(of: phase) { _, now in
            switch now {
            case .active: host.shown(true)
            case .background: host.shown(false)
            default: break
            }
        }
    }
}
