import Adapters
import CoreHost
import SurfacesTV
import SwiftUI

@main
struct MainApp: App {
    @State private var host: Host

    init() {
        let network = UrlNetwork()
        let secrets = KeychainSecrets()
        let store = FileStore()
        let player = AvPlayer()
        let reel = AvReel()
        _host = State(initialValue: Host(
            network: network,
            secrets: secrets,
            store: store,
            player: player,
            reel: reel
        ))
    }

    var body: some Scene {
        WindowGroup {
            Television(shell: host)
                .onAppear { host.shown(true) }
                .onDisappear { host.shown(false) }
        }
    }
}
