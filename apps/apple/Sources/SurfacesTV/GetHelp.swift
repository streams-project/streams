import Contract
import Ports
import SwiftUI
import UIKit

/// The lines of Get Help. Apple TV has no Exit line: an app cannot quit itself there, and Back leaves.
private enum Line: String, CaseIterable {
    case back = "help.back"
    case member = "help.member"
    case device = "help.device"
    case network = "help.network"
    case centre = "help.centre"
    case settings = "help.settings"
    case contact = "help.contact"
    case check = "help.check"
    case reload = "help.reload"
    case signOut = "menu.sign_out"
}

/// Lines that need an account.
private let memberOnly: Set<Line> = [.member, .signOut]

private func decode<T: Decodable>(_ type: T.Type, _ text: String?) -> T? {
    text.flatMap { try? JSONDecoder().decode(type, from: Data($0.utf8)) }
}

private struct Fact: View {
    let head: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(head).brand(30, 700).foregroundStyle(.white)
            Text(value).brand(28).foregroundStyle(Muted)
        }
        .padding(.bottom, 22.ref)
    }
}

private struct Hint: View {
    let text: String

    var body: some View {
        Text(text).brand(30).foregroundStyle(.white.opacity(0.85)).frame(width: 900.ref, alignment: .leading)
    }
}

/// Who and what this is, and the ways to get unstuck. The panel follows the focus.
public struct GetHelp: View {
    let shell: Shell
    let seen: Seen
    @State private var at = 1
    @State private var centre = false
    @State private var asking = false
    @State private var answer = 1
    @State private var checked: String?
    @State private var account: Account?
    @State private var hello: Hello?

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        let signedIn = !seen.profiles.isEmpty
        Group {
            if centre {
                HelpCentre(shell: shell, seen: seen) { centre = false }
            } else {
                page(signedIn)
            }
        }
        .task(id: signedIn) {
            if signedIn { account = decode(Account.self, await shell.read("/v1/account")) }
        }
        .task { hello = decode(Hello.self, await shell.read("/v1/hello")) }
    }

    private func page(_ signedIn: Bool) -> some View {
        let person = seen.profiles.first { $0.id == seen.active } ?? seen.profiles.first
        let lines = Line.allCases.filter { line in
            if memberOnly.contains(line) { return signedIn }
            if line == .settings { return signedIn && person?.kids != true }
            return true
        }
        let line = lines[safe: at] ?? .back

        func check() {
            let start = Date()
            Task {
                let answered = await shell.read("/v1/ready") != nil
                let took = Int(Date().timeIntervalSince(start) * 1000)
                checked = answered
                    ? seen.say("help.check.ok").fill("ms", String(took))
                    : seen.say("help.check.fail")
            }
        }

        func choose() {
            switch line {
            case .back: shell.back()
            case .centre: centre = true
            case .settings: shell.go(Address.settings)
            case .check: check()
            case .reload: shell.go(Address.start)
            case .signOut:
                asking = true
                answer = 1
            default: break
            }
        }

        func press(_ word: Word) -> Bool {
            if asking {
                switch word {
                case .left: answer = 0
                case .right: answer = 1
                case .back: asking = false
                case .ok: if answer == 0 { shell.act(.signOut(ActionSignOut(everywhere: false))) } else { asking = false }
                default: break
                }
                return true
            }
            switch word {
            case .up: if at > 0 { at -= 1 }
            case .down: if at < lines.count - 1 { at += 1 }
            case .ok: choose()
            // The menu is the left edge of the page: Left goes back, as Back does.
            case .back, .left: shell.back()
            default: break
            }
            return true
        }

        let language = Locale(identifier: seen.language).localizedString(forLanguageCode: seen.language) ?? seen.language
        let version = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""

        return ZStack(alignment: .topLeading) {
            Color.black
            Text(seen.say("menu.help")).brand(44, 700).foregroundStyle(.white).offset(x: subEdge.ref, y: 60.ref)
            Menu(lines: lines.map { MenuLine(word: seen.say($0.rawValue)) }, focus: asking ? -1 : at, width: 560, kept: asking ? at : -1)
                .offset(x: (subEdge - menuWords).ref, y: 136.ref)
            VStack(alignment: .leading, spacing: 0) {
                switch line {
                case .member:
                    Fact(head: seen.say("help.profile_name"), value: person?.name ?? "")
                    Fact(head: seen.say("help.account_email"), value: account?.email ?? "")
                    Fact(head: seen.say("help.language"), value: language)
                case .device:
                    Fact(head: seen.say("help.app_version"), value: version)
                    Fact(head: seen.say("help.software"), value: "\(UIDevice.current.systemName) \(UIDevice.current.systemVersion)")
                    Fact(head: seen.say("help.model"), value: UIDevice.current.model)
                case .network: Fact(head: seen.say("help.server"), value: seen.server)
                case .centre: Hint(text: seen.say("help.centre.hint"))
                case .settings: Hint(text: seen.say("help.settings.hint"))
                case .contact:
                    let address = hello?.brand.supportEmail ?? ""
                    if address.isEmpty { Hint(text: seen.say("help.contact.none")) }
                    else { Fact(head: seen.say("help.contact.hint"), value: address) }
                case .check: Hint(text: checked ?? seen.say("help.check.hint"))
                case .reload: Hint(text: seen.say("help.reload.hint"))
                case .signOut:
                    Hint(text: seen.say("help.sign_out.hint"))
                    if asking {
                        HStack(spacing: 20.ref) {
                            Pill(seen.say("menu.sign_out"), on: answer == 0)
                            Pill(seen.say("common.cancel"), on: answer == 1)
                        }
                        .padding(.top, 36.ref)
                    }
                case .back: EmptyView()
                }
            }
            .frame(width: 1040.ref, alignment: .leading)
            .offset(x: 720.ref, y: 136.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { press($0) }
    }
}
