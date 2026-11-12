#if canImport(UIKit)
import Darwin
import SwiftUI
import UIKit

/// One thing a screen reader would stop on: the words it reads, how it says the thing stands, what
/// kind of thing it says it is, and how deep in the screen it sits. A screen with no entries says
/// nothing to a person who cannot see it.
public struct Heard: Sendable, Equatable {
    /// The words read out. Empty means the element is there and silent.
    public let label: String
    /// How the thing stands, read after the label: "In My List", "Selected", "Off", "12:30 of
    /// 1:45:00". Empty for a control that only has one way to be.
    ///
    /// This was missing until 2026-11-12, and its absence meant something: `saysItsWords(state:)`,
    /// `says(_:state:)` and every `accessibilityValue` on both surfaces set a value no test here
    /// could read, so none of them was tested. `TelevisionTests.testTickedLineSaysItIsChosen` was
    /// written over that hole - it counted the `selected` trait, which on `TickGrid` marks where the
    /// remote is and not what is ticked, and so passed while asserting the opposite of its name.
    public let value: String
    /// What the element says it is, by name: `button`, `image`, `header`, `selected`, `link`,
    /// `switch`.
    public let kinds: [String]
    public let depth: Int

    public var isButton: Bool { kinds.contains("button") }
    public var isImage: Bool { kinds.contains("image") }
    public var isSelected: Bool { kinds.contains("selected") }
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
    (.toggleButton, "switch"),
]

private func names(_ traits: UIAccessibilityTraits) -> [String] {
    traitNames.filter { traits.contains($0.0) }.map(\.1)
}

/// SwiftUI builds no accessibility tree at all until something is listening for one.
///
/// Measured on iPhone 17 (iOS 26.5) on 2026-11-12: a `UIHostingController` holding
/// `Button("Hello")`, in a key window and laid out, reports `accessibilityElements == []` on its
/// `_UIHostingView` and no subviews, after 0.1 s and again after 1.1 s. So every reading this file
/// made before this switch existed was the empty list, for every view, labelled or not, and the
/// five tests that read it could not have passed whatever the surfaces did. With the switch on, the
/// same view reports two `AccessibilityNode` children, "Hello" with the button trait and "Plain"
/// with the text trait.
///
/// The switch is `_AXSSetAutomationEnabled` in `/usr/lib/libAccessibility.dylib`, which is what
/// UIKit's own test tooling turns on, and it is private. It is reached by name at run time rather
/// than linked, so a system that has renamed it leaves the tests red instead of failing to build.
/// This target is test support and no product links it (`Package.swift`), so nothing shipped
/// carries the call. The alternative was rejected on cost: the only public way to read a real
/// accessibility tree is an `XCUIApplication`, which needs an app target, a simulator boot per run
/// and a UI test host, for the same answer this gives in a fifth of a second.
@MainActor private var listening = false

@MainActor
private func listenForAccessibility() {
    if listening { return }
    listening = true
    guard let library = dlopen("/usr/lib/libAccessibility.dylib", RTLD_NOW),
          let symbol = dlsym(library, "_AXSSetAutomationEnabled") else { return }
    typealias Enable = @convention(c) (Bool) -> Void
    unsafeBitCast(symbol, to: Enable.self)(true)
}

/// Draws a view off screen at a given size and reports everything a screen reader would find in it,
/// in the order it would find it. This is the Apple answer to `uiautomator dump`: the same question,
/// asked of a view instead of a running app, so it can be asked by a test.
///
/// The window is made key because SwiftUI only builds its accessibility elements for a view that is
/// in a window and laid out.
/// `settle` is how long the run loop is given. 0.1 s is enough for a view that draws itself at
/// once. A screen whose parts sit inside a `ScrollView` and a lazy stack needs longer: measured on
/// iPhone 17 on 2026-11-12, the phone's Home reaches 6 elements at 0.1 s and 25 at 0.6 s, because a
/// lazy stack builds its children over later turns of the loop. A test that reads a whole screen
/// says how long it waited.
@MainActor
public func heard<V: View>(
    _ view: V,
    size: CGSize = CGSize(width: 1920, height: 1080),
    settle: TimeInterval = 0.1
) -> [Heard] {
    listenForAccessibility()
    let host = UIHostingController(rootView: view)
    let window = UIWindow(frame: CGRect(origin: .zero, size: size))
    window.rootViewController = host
    window.isHidden = false
    window.makeKeyAndVisible()
    host.view.frame = CGRect(origin: .zero, size: size)
    host.view.setNeedsLayout()
    host.view.layoutIfNeeded()
    // Turns of the run loop: SwiftUI commits its accessibility tree on a later turn, not in layout.
    RunLoop.current.run(until: Date().addingTimeInterval(settle))
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
                value: node.accessibilityValue ?? "",
                kinds: names(node.accessibilityTraits),
                depth: depth
            )
        )
        return
    }
    // A container hands over the elements it lists. An EMPTY list is not a container saying it holds
    // nothing: `_UIHostingView` reports `[]` whenever nothing is listening for an accessibility tree,
    // and reading that as "no children" is what made every reading here the empty list. So an empty
    // list falls through to the subviews, which can only add to what is found.
    if let elements = node.accessibilityElements, !elements.isEmpty {
        for element in elements { walk(element as AnyObject, depth + 1, &found) }
        return
    }
    if let view = node as? UIView {
        for sub in view.subviews { walk(sub, depth + 1, &found) }
    }
}

/// The words of every element, for a test that wants to read a screen as a list.
public func words(_ found: [Heard]) -> [String] { found.map(\.label) }

/// How every element says it stands, in the same order as ``words(_:)``. Named `states` and not
/// `values` because `XCTestCase` has a `values` of its own and would win inside a test.
public func states(_ found: [Heard]) -> [String] { found.map(\.value) }

/// The one element read as these words, or nil. For a test that cares about a single control on a
/// screen full of them and should not be written against a position in the list.
public func saying(_ found: [Heard], _ label: String) -> Heard? {
    found.first { $0.label == label }
}

/// Anything an assistive technology would stop on and find nothing to read.
public func silent(_ found: [Heard]) -> [Heard] { found.filter { $0.label.isEmpty } }

/// Labels that are an asset file name rather than words: `icon_play`, `icon back`. UIKit falls back to
/// the name of the picture when no one gave the picture words, and the name is not in any language.
public func assetNames(_ found: [Heard]) -> [Heard] {
    found.filter { $0.label.lowercased().hasPrefix("icon_") || $0.label.lowercased().hasPrefix("icon ") }
}
#endif
