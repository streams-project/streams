import Contract
import Ports
import SwiftUI
import UIKit

private enum Way { case phone, remote }
private enum Ask { case email, password }

private let wash = LinearGradient(
    stops: [.init(color: Accent.opacity(0.45), location: 0), .init(color: .black, location: 0.6)],
    startPoint: .top, endPoint: .bottom
)

private struct Pills: View {
    let seen: Seen
    let way: Way
    let on: Bool

    var body: some View {
        HStack(spacing: 0) {
            ForEach([(Way.phone, "signin.phone"), (Way.remote, "signin.remote")], id: \.1) { each, key in
                let here = each == way
                Text(seen.say(key))
                    .brand(26, 500)
                    .foregroundStyle(here && on ? Back : .white)
                    .frame(width: 240.ref, height: 54.ref)
                    .background(here && on ? Color.white : here ? Color.white.opacity(0.3) : .clear, in: Capsule())
            }
        }
        .padding(4.ref)
        .background(Color.white.opacity(0.16), in: Capsule())
        .frame(maxWidth: .infinity)
        .padding(.top, 118.ref)
    }
}

private struct Step: View {
    let number: Int
    let left: CGFloat

    var body: some View {
        Text(String(number))
            .brand(28, 700)
            .foregroundStyle(.white)
            .frame(width: 52.ref, height: 52.ref)
            .background(Color.white.opacity(0.2), in: Circle())
            .offset(x: left.ref, y: 214.ref)
    }
}

/// The code a phone approves. The core keeps it alive, so this only draws it.
private struct UsePhone: View {
    let seen: Seen

    var body: some View {
        let device = seen.device
        let code = device?.userCode
        ZStack(alignment: .topLeading) {
            Step(number: 1, left: 170)
            // The address never breaks across lines: a word joiner between its letters keeps it whole.
            let address = (device?.verificationUri ?? "").map(String.init).joined(separator: "\u{2060}")
            Text(seen.say("signin.step1").replacingOccurrences(of: "{address}", with: address))
                .brand(32).lines(32, 42).foregroundStyle(.white)
                .frame(width: 680.ref, alignment: .leading)
                .offset(x: 240.ref, y: 214.ref)
            if let link = device?.verificationUriComplete {
                QrCode(text: link).frame(width: 440.ref, height: 440.ref).offset(x: 240.ref, y: 410.ref)
            }
            Step(number: 2, left: 1030)
            Text(seen.say("signin.step2"))
                .brand(32).lines(32, 42).foregroundStyle(.white)
                .frame(width: 680.ref, alignment: .leading)
                .offset(x: 1100.ref, y: 214.ref)
            Text(device?.state == .off ? seen.say("signin.off") : code ?? seen.say("signin.getting"))
                .brand(code != nil ? 88 : 36, 800)
                .tracking(code != nil ? 6 : 0)
                .foregroundStyle(.white)
                .frame(width: 760.ref, alignment: .leading)
                .offset(x: 1100.ref, y: 410.ref)
        }
    }
}

private struct Field: View {
    let label: String
    let value: String
    let hidden: Bool

    var body: some View {
        let shape = shapeOf(Round.field)
        Text(value.isEmpty ? label : hidden ? String(repeating: "•", count: value.count) : value)
            .brand(32)
            .foregroundStyle(value.isEmpty ? Muted : .white)
            .lineLimit(1)
            .padding(.horizontal, 26.ref)
            .frame(width: 800.ref, height: 76.ref, alignment: .leading)
            .overlay(shape.stroke(Color.white.opacity(0.5), lineWidth: 2.ref))
            .offset(x: 960.ref, y: 280.ref)
    }
}

/// Choose how to sign in: a phone and a code, or the remote and a keyboard.
public struct SignIn: View {
    let shell: Shell
    let seen: Seen
    @State private var way = Way.phone
    @State private var onPills = true
    @State private var ask = Ask.email
    @State private var email = ""
    @State private var keys = KeyboardState("")

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        let back = seen.say("common.back")
        let next = seen.say("common.next")

        func finish(_ text: String?) {
            if text == nil && ask == .password {
                ask = .email
                keys.text = email
            } else if text == nil {
                onPills = true
            } else if ask == .email {
                email = text!.trimmingCharacters(in: .whitespaces)
                ask = .password
                keys.text = ""
            } else {
                shell.act(.signIn(ActionSignIn(email: email, password: text!)))
                keys.text = ""
            }
        }

        func press(_ word: Word) -> Bool {
            if !onPills {
                if !keys.press(word, back, next, finish) { onPills = true }
                return true
            }
            switch word {
            // Left past Use Phone, and Back, return to the page the viewer came from.
            case .left: if way == .phone { shell.back() } else { way = .phone }
            case .right: way = .remote
            case .down, .ok: if way == .remote { onPills = false }
            case .back: shell.back()
            case .up: break
            }
            return true
        }

        return ZStack(alignment: .topLeading) {
            wash
            Mark(64).offset(x: 1796.ref, y: 40.ref)
            Text(seen.say("signin.choose"))
                .brand(44, 700)
                .foregroundStyle(.white)
                .frame(maxWidth: .infinity)
                .padding(.top, 48.ref)
            Pills(seen: seen, way: way, on: onPills)
            if way == .phone {
                UsePhone(seen: seen)
            } else {
                Keyboard(state: keys, cancel: back, done: next).offset(x: 160.ref, y: 250.ref)
                Field(label: seen.say(ask == .email ? "signin.email" : "signin.password"), value: keys.text, hidden: ask == .password)
                if let notice = seen.notice {
                    Text(notice).brand(28).foregroundStyle(Muted)
                        .frame(width: 800.ref, alignment: .leading)
                        .offset(x: 960.ref, y: 380.ref)
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { press($0) }
        .onAppear { shell.act(.deviceSignIn(ActionDeviceSignIn(deviceName: UIDevice.current.model))) }
    }
}
