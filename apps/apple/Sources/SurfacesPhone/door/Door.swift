import Contract
import Ports
import SwiftUI

/// Where a phone points at its server, for self hosters.
struct ServerAddress: View {
    let shell: Shell
    @State private var address = "https://"

    var body: some View {
        let words = shell.seen.words
        return VStack(alignment: .leading, spacing: 16) {
            // The field is given an empty label so nothing is drawn over the address, which leaves
            // it with nothing for a reader either: a person hears an empty text field and has no
            // way to learn it wants a server. `settings.server` is the word this project already
            // uses for the thing being typed.
            TextField("", text: $address)
                .keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled()
                .padding(14).background(Raised, in: RoundedRectangle(cornerRadius: Corner.field))
                .accessibilityLabel(words["settings.server"] ?? "")
            // An arrow and nothing else, so it carries the word for going on.
            Button { shell.connect(address) } label: {
                Image(systemName: "arrow.right").frame(maxWidth: .infinity, minHeight: 48)
            }
            .buttonStyle(.borderedProminent).tint(.white).foregroundStyle(Back)
            .says(words["common.next"] ?? "")
            // A refused address is said here, where it was typed, or the screen would simply do
            // nothing when the viewer presses on.
            if let notice = shell.seen.notice {
                Text(notice).brand(14).foregroundStyle(Join)
            }
        }
        .padding(24)
    }
}

/// The landing for a phone that is not signed in: the brand and the ways in. P1 builds the full
/// page with today's trending titles.
struct Door: View {
    let shell: Shell
    let seen: Seen

    var body: some View {
        VStack(spacing: 24) {
            Spacer()
            // The brand's mark is the whole of this page's picture and it is decoration (`Mark`
            // marks itself): unmarked it was read out as "mark", the name of its own asset file.
            Mark(120)
            Spacer()
            Button { shell.go(Address.sign_in) } label: {
                Text(seen.say("door.sign_in")).brand(17, 700).frame(maxWidth: .infinity, minHeight: 52)
            }
            .background(Color.white, in: Capsule()).foregroundStyle(Back)
            // The button draws its own word, the only one on the page.
            .saysItsWords()
        }
        .padding(24)
    }
}

/// Email and password. P1 adds the email code, sign up and recovery.
struct SignIn: View {
    let shell: Shell
    let seen: Seen
    @State private var email = ""
    @State private var password = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text(seen.say("door.sign_in")).brand(28, 700)
                .accessibilityAddTraits(.isHeader)
            // Two fields, one of them with a placeholder drawn in English from the code and the
            // other with nothing drawn at all. The placeholder is visible text and is left alone;
            // the words a person hears come from `strings/`, which has a key for each.
            TextField("", text: $email, prompt: Text(verbatim: "Email"))
                .keyboardType(.emailAddress).textContentType(.username)
                .textInputAutocapitalization(.never).autocorrectionDisabled()
                .padding(14).background(Raised, in: RoundedRectangle(cornerRadius: Corner.field))
                .accessibilityLabel(seen.say("signin.email"))
            SecureField("", text: $password)
                .textContentType(.password)
                .padding(14).background(Raised, in: RoundedRectangle(cornerRadius: Corner.field))
                .accessibilityLabel(seen.say("signin.password"))
            Button { shell.act(.signIn(ActionSignIn(email: email, password: password))) } label: {
                Text(seen.say("door.sign_in")).brand(17, 700).frame(maxWidth: .infinity, minHeight: 52)
            }
            .background(Color.white, in: Capsule()).foregroundStyle(Back)
            .saysItsWords()
            if let notice = seen.notice { Text(notice).brand(14).foregroundStyle(Muted) }
        }
        .padding(24)
    }
}
