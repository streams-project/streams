#if canImport(UIKit)
import SwiftUI
import UIKit

/// One thing a screen reader would stop on: the words it reads, what kind of thing it says it is, and
/// how deep in the screen it sits. A screen with no entries says nothing to a person who cannot see it.
public struct Heard: Sendable, Equatable {
    /// The words read out. Empty means the element is there and silent.
    public let label: String
    /// What the element says it is, by name: `button`, `image`, `header`, `selected`, `link`.
    public let kinds: [String]
    public let depth: Int

    public var isButton: Bool { kinds.contains("button") }
    public var isImage: Bool { kinds.contains("image") }
}

/// The names of the traits this project cares about, so a test reads as words and not as a bit field.
private let traitNames: [(UIAccessibilityTraits, String)] = [
    (.button, "button"),
    (.image, "image"),
    (.header, "header"),
    (.selected, "selected"),
    (.link, "link"),
    (.staticText, "text"),
    (.notEnabled, "disabled"),
    (.updatesFrequently, "updates"),
]

private func names(_ traits: UIAccessibilityTraits) -> [String] {
    traitNames.filter { traits.contains($0.0) }.map(\.1)
}

/// Draws a view off screen at a given size and reports everything a screen reader would find in it,
/// in the order it would find it. This is the Apple answer to `uiautomator dump`: the same question,
/// asked of a view instead of a running app, so it can be asked by a test.
///
/// The window is made key because SwiftUI only builds its accessibility elements for a view that is
/// in a window and laid out.
@MainActor
public func heard<V: View>(_ view: V, size: CGSize = CGSize(width: 1920, height: 1080)) -> [Heard] {
    let host = UIHostingController(rootView: view)
    let window = UIWindow(frame: CGRect(origin: .zero, size: size))
    window.rootViewController = host
    window.isHidden = false
    window.makeKeyAndVisible()
    host.view.frame = CGRect(origin: .zero, size: size)
    host.view.setNeedsLayout()
    host.view.layoutIfNeeded()
    // One turn of the run loop: SwiftUI commits its accessibility tree on the next turn, not in layout.
    RunLoop.current.run(until: Date().addingTimeInterval(0.1))
    var found: [Heard] = []
    walk(host.view, 0, &found)
    window.isHidden = true
    return found
}

/// Walks the tree the same way an assistive technology does: an element that says it is one stops the
/// walk there, a view that lists its own elements hands those over, and anything else is its subviews.
@MainActor
private func walk(_ node: AnyObject, _ depth: Int, _ found: inout [Heard]) {
    guard let node = node as? NSObject else { return }
    if node.accessibilityElementsHidden { return }
    if let view = node as? UIView, view.isHidden || view.alpha == 0 { return }
    if node.isAccessibilityElement {
        found.append(
            Heard(
                label: node.accessibilityLabel ?? "",
                kinds: names(node.accessibilityTraits),
                depth: depth
            )
        )
        return
    }
    if let elements = node.accessibilityElements {
        for element in elements { walk(element as AnyObject, depth + 1, &found) }
        return
    }
    if let view = node as? UIView {
        for sub in view.subviews { walk(sub, depth + 1, &found) }
    }
}

/// The words of every element, for a test that wants to read a screen as a list.
public func words(_ found: [Heard]) -> [String] { found.map(\.label) }

/// Anything an assistive technology would stop on and find nothing to read.
public func silent(_ found: [Heard]) -> [Heard] { found.filter { $0.label.isEmpty } }

/// Labels that are an asset file name rather than words: `icon_play`, `icon back`. UIKit falls back to
/// the name of the picture when no one gave the picture words, and the name is not in any language.
public func assetNames(_ found: [Heard]) -> [Heard] {
    found.filter { $0.label.lowercased().hasPrefix("icon_") || $0.label.lowercased().hasPrefix("icon ") }
}
#endif
