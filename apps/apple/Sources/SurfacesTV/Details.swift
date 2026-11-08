import Contract
import Ports
import SwiftUI

private let settleMs: UInt64 = 240

/// What has been read already, for this run of the screen.
@MainActor
public final class DetailStore {
    var kept: [TitleKey: TitleDetail] = [:]
    public init() {}
}

/// The path of a title's details.
func detailPath(_ title: TitleKey) -> String {
    "/v1/titles/" + title.rawValue.replacingOccurrences(of: ":", with: "/")
}

/// The details of whatever has focus, once the viewer settles on it.
@MainActor
func detailOf(_ shell: Shell, _ store: DetailStore, _ title: TitleKey) async -> TitleDetail? {
    if let kept = store.kept[title] { return kept }
    try? await Task.sleep(nanoseconds: settleMs * 1_000_000)
    if Task.isCancelled { return nil }
    guard let text = await shell.read(detailPath(title)),
          let detail = try? JSONDecoder().decode(TitleDetail.self, from: Data(text.utf8)) else { return nil }
    store.kept[title] = detail
    return detail
}

/// Text lines a set distance apart, as the references measure them.
extension View {
    func lines(_ size: CGFloat, _ height: CGFloat) -> some View {
        lineSpacing(max(0, height - size * 1.21))
    }
}
