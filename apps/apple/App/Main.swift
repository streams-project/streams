// The one place that plugs the adapters into the ports.
import Adapters
import CoreHost
import SurfacesTV
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
            reel: AvReel(),
            pictures: UrlPictures(),
            deviceClass: ThisDevice.deviceClass,
            form: ThisDevice.form,
            version: ThisDevice.version,
            os: ThisDevice.os
        )
        RemoteUIView.heard = { host.touched() }
        _host = State(initialValue: host)
    }

    var body: some Scene {
        WindowGroup {
            Television(shell: host)
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
