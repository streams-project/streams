import SwiftUI
import UIKit

public enum Word: Sendable {
    case up, down, left, right, ok, back
}

public struct RemoteReceiver: UIViewRepresentable {
    public let press: (Word) -> Bool

    public init(press: @escaping (Word) -> Bool) {
        self.press = press
    }

    public func makeUIView(context: Context) -> RemoteUIView {
        let view = RemoteUIView()
        view.handler = press
        DispatchQueue.main.async {
            view.setNeedsFocusUpdate()
            view.updateFocusIfNeeded()
            view.becomeFirstResponder()
        }
        return view
    }

    public func updateUIView(_ uiView: RemoteUIView, context: Context) {
        uiView.handler = press
    }
}

public final class RemoteUIView: UIView {
    var handler: ((Word) -> Bool)?

    public override var canBecomeFocused: Bool { true }
    public override var canBecomeFirstResponder: Bool { true }

    public override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        for press in presses {
            var word: Word? = nil
            switch press.type {
            case .upArrow: word = .up
            case .downArrow: word = .down
            case .leftArrow: word = .left
            case .rightArrow: word = .right
            case .select: word = .ok
            case .menu: word = .back
            case .playPause: word = .ok
            @unknown default:
                if let key = press.key {
                    switch key.keyCode {
                    case .keyboardUpArrow: word = .up
                    case .keyboardDownArrow: word = .down
                    case .keyboardLeftArrow: word = .left
                    case .keyboardRightArrow: word = .right
                    case .keyboardReturnOrEnter: word = .ok
                    case .keyboardEscape: word = .back
                    default: word = nil
                    }
                }
            }

            if let w = word {
                let handled = handler?(w) ?? false
                if !handled && press.type == .menu {
                    super.pressesBegan(presses, with: event)
                }
                return
            }
        }
        super.pressesBegan(presses, with: event)
    }
}

extension View {
    public func remote(_ handler: @escaping (Word) -> Bool) -> some View {
        self.background(RemoteReceiver(press: handler))
    }
}
