import Contract
import Ports
import SwiftUI

/// The root of the window: whichever screen the address names.
///
/// The same job `SurfacesTV/app/Television.swift` does for the television, with everything about
/// the remote taken out. There is no opening sequence here and no cover over the screens, because
/// both exist on the television to hold the keys while something plays over the top, and there are
/// no keys to hold.
///
/// Background: on this platform the window is glass the viewer's room shows through, and the system
/// draws it. So nothing here fills the window with the brand's dark plate -- a window in space
/// painted opaque black blocks the passthrough for no gain, and the system's own material is what
/// keeps text readable against whatever room it is in. `app/Surface.swift` carries the whole of
/// that argument and the ink that goes with it. On every other platform this module compiles for,
/// the brand's background is drawn instead, because there is nothing behind the window there.
///
/// Where the window goes is the viewer's: the system remembers where they put it and this file does
/// not ask for a position. That is the ordinary sense of reach -- a session here lasts a film, not a
/// minute, and the one thing certain about a comfortable place to watch from is that this code does
/// not know it. What the app owes that session is a floor under the window's size, controls at the
/// end of the window nearest a resting hand (`NavBar`), and nothing that has to be held.
public struct Window: SwiftUI.View {
    let shell: Shell
    let seen: Seen

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some SwiftUI.View {
        Group {
            switch seen.route?.target {
            case .watch: Watch(shell: shell, seen: seen)
            case .title: TitlePage(shell: shell, seen: seen)
            default: Browse(shell: shell, seen: seen)
            }
        }
        // A floor and no ceiling. The floor is where `across(_:)` still answers more than one tile
        // (`app/Sizes.swift`); above it the viewer decides, and a film is the reason there is no
        // ceiling -- the whole point of the hardware is that the screen can be as large as the room
        // allows, so a maximum here would be this file overruling that.
        .frame(minWidth: 520, minHeight: 420)
        .modifier(Plate())
    }
}

/// What the window stands on.
private struct Plate: ViewModifier {
    func body(content: Content) -> some SwiftUI.View {
        #if os(visionOS)
        content
        #else
        content.background(Back)
        #endif
    }
}
