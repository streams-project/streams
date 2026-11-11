import SwiftUI
import UIKit

/// What a remote can say. A screen answers true when it used the word.
public enum Word: Sendable {
    case up, down, left, right, ok, back
}

private struct CoveredKey: EnvironmentKey {
    static let defaultValue = false
}

extension EnvironmentValues {
    /// True while the opening covers the screens: they keep their state but take no keys.
    public var covered: Bool {
        get { self[CoveredKey.self] }
        set { self[CoveredKey.self] = newValue }
    }
}

/// Gives a whole screen the remote. The screen keeps its own focus as plain state.
public struct RemoteReceiver: UIViewRepresentable {
    public let on: Bool
    public let press: (Word) -> Bool
    @Environment(\.covered) private var covered

    public func makeUIView(context: Context) -> RemoteUIView {
        let view = RemoteUIView()
        view.handler = press
        view.on = on && !covered
        return view
    }

    public func updateUIView(_ view: RemoteUIView, context: Context) {
        // A key always reaches the newest answer, with what the screen holds now.
        view.handler = press
        // Under the opening a screen keeps its place but takes no keys, and takes them back after.
        view.on = on && !covered
    }
}

/// One focusable view per screen. It takes the system focus while `on`, and gives it up while
/// something drawn over the screen holds the remote.
public final class RemoteUIView: UIView {
    /// Hears every press, before any screen does. The app plugs the host's `touched` in here.
    nonisolated(unsafe) public static var heard: (() -> Void)?

    var handler: ((Word) -> Bool)?
    var on = true {
        didSet { if on != oldValue { claim() } }
    }

    /// The presses a screen has already answered. A press arrives in three parts, and only the first
    /// is offered to the screen; the parts after it must be held back too. A Back the screen used and
    /// whose end still reached the system would leave the app on the way up. The set belongs to the
    /// app, not to one receiver, because the press that changes the screen is answered by the
    /// receiver going away and may end on the one that takes its place.
    nonisolated(unsafe) private static var used: Set<String> = []

    public override var canBecomeFocused: Bool { on }

    public override func didMoveToWindow() {
        super.didMoveToWindow()
        claim()
    }

    /// Asks the focus engine to look again, so the screen that is on now gets the presses. A new
    /// screen can appear while the old one still holds the focus and leave a moment later, so the
    /// view keeps asking for a short while until it has the focus.
    private func claim(_ tries: Int = 8) {
        guard window != nil else { return }
        DispatchQueue.main.asyncAfter(deadline: .now() + (tries == 8 ? 0 : 0.12)) { [weak self] in
            guard let self, let window = self.window else { return }
            let root: UIFocusEnvironment = window.rootViewController ?? window
            root.setNeedsFocusUpdate()
            root.updateFocusIfNeeded()
            if self.on && !self.isFocused {
                self.setNeedsFocusUpdate()
                self.updateFocusIfNeeded()
            }
            if self.on && !self.isFocused && tries > 1 { self.claim(tries - 1) }
        }
    }

    private static func word(_ press: UIPress) -> Word? {
        switch press.type {
        case .upArrow: return .up
        case .downArrow: return .down
        case .leftArrow: return .left
        case .rightArrow: return .right
        case .select, .playPause: return .ok
        case .menu: return .back
        default: break
        }
        switch press.key?.keyCode {
        case .keyboardUpArrow: return .up
        case .keyboardDownArrow: return .down
        case .keyboardLeftArrow: return .left
        case .keyboardRightArrow: return .right
        case .keyboardReturnOrEnter: return .ok
        case .keyboardEscape: return .back
        default: return nil
        }
    }

    /// The same name for every part of one press: its button, or the key that stood in for it.
    private static func tag(_ press: UIPress) -> String {
        "\(press.type.rawValue):\(press.key?.keyCode.rawValue ?? -1)"
    }

    public override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        Self.heard?()
        guard on, let press = presses.first, let word = Self.word(press) else {
            super.pressesBegan(presses, with: event)
            return
        }
        if handler?(word) ?? false {
            Self.used.insert(Self.tag(press))
            return
        }
        // Back that no screen used goes to the system, which leaves the app.
        if word == .back {
            super.pressesBegan(presses, with: event)
        }
    }

    public override func pressesChanged(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        if !presses.isEmpty && presses.allSatisfy({ Self.used.contains(Self.tag($0)) }) { return }
        super.pressesChanged(presses, with: event)
    }

    public override func pressesEnded(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        if finish(presses) { return }
        super.pressesEnded(presses, with: event)
    }

    public override func pressesCancelled(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        if finish(presses) { return }
        super.pressesCancelled(presses, with: event)
    }

    /// True when every press ending here was one a screen answered, so its end stops here as well.
    private func finish(_ presses: Set<UIPress>) -> Bool {
        let ours = !presses.isEmpty && presses.allSatisfy { Self.used.contains(Self.tag($0)) }
        for press in presses { Self.used.remove(Self.tag(press)) }
        return ours
    }
}

extension View {
    /// Gives this screen the remote.
    public func remote(_ press: @escaping (Word) -> Bool) -> some View {
        remote(on: true, press)
    }

    /// `on` goes false while something drawn over the screen holds the remote, and takes it back after.
    /// The receiver lies over the screen, where it draws nothing: a picture of a film is a view of its
    /// own, and a receiver behind one that fills the screen can never take the focus.
    public func remote(on: Bool, _ press: @escaping (Word) -> Bool) -> some View {
        modifier(RemoteHold(on: on, press: press))
    }
}

/// Pages say Left for towards the start of a line. In a right to left language the start is on the
/// right, so the remote's arrows swap here, once, for every page. The player keeps its own left to
/// right way, so its arrows stay as they are.
private struct RemoteHold: ViewModifier {
    let on: Bool
    let press: (Word) -> Bool
    @Environment(\.layoutDirection) private var way

    func body(content: Content) -> some View {
        let turned = way == .rightToLeft
        content.overlay(RemoteReceiver(on: on, press: { word in
            press(turned ? word.turned : word)
        }))
    }
}

extension Word {
    /// Left and right swapped, for a right to left page.
    var turned: Word {
        switch self {
        case .left: .right
        case .right: .left
        default: self
        }
    }
}
