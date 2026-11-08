import SwiftUI
import UIKit

/// What a remote can say. A screen answers true when it used the word.
public enum Word: Sendable {
    case up, down, left, right, ok, back
}

/// Gives a whole screen the remote. The screen keeps its own focus as plain state.
public struct RemoteReceiver: UIViewRepresentable {
    public let on: Bool
    public let press: (Word) -> Bool

    public func makeUIView(context: Context) -> RemoteUIView {
        let view = RemoteUIView()
        view.handler = press
        view.on = on
        return view
    }

    public func updateUIView(_ view: RemoteUIView, context: Context) {
        // A key always reaches the newest answer, with what the screen holds now.
        view.handler = press
        view.on = on
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

    public override var canBecomeFocused: Bool { on }

    public override func didMoveToWindow() {
        super.didMoveToWindow()
        claim()
    }

    /// Asks the focus engine to look again, so the screen that is on now gets the presses.
    private func claim() {
        guard window != nil else { return }
        DispatchQueue.main.async { [weak self] in
            guard let self, let window = self.window else { return }
            let root: UIFocusEnvironment = window.rootViewController ?? window
            root.setNeedsFocusUpdate()
            root.updateFocusIfNeeded()
            if self.on && !self.isFocused {
                self.setNeedsFocusUpdate()
                self.updateFocusIfNeeded()
            }
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

    public override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        Self.heard?()
        guard on, let press = presses.first, let word = Self.word(press) else {
            super.pressesBegan(presses, with: event)
            return
        }
        // Back that no screen used goes to the system, which leaves the app.
        if !(handler?(word) ?? false) && word == .back {
            super.pressesBegan(presses, with: event)
        }
    }
}

extension View {
    /// Gives this screen the remote.
    public func remote(_ press: @escaping (Word) -> Bool) -> some View {
        remote(on: true, press)
    }

    /// `on` goes false while something drawn over the screen holds the remote, and takes it back after.
    public func remote(on: Bool, _ press: @escaping (Word) -> Bool) -> some View {
        background(RemoteReceiver(on: on, press: press))
    }
}
