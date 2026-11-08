import Contract
import Ports
import SwiftUI

/// The main browse pages under the bottom bar.
struct Pages: View {
    let shell: Shell
    let seen: Seen

    var body: some View {
        Home(shell: shell, seen: seen)
    }
}
