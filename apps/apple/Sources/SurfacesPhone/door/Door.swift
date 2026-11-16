import Contract
import Ports
import SwiftUI

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
            VStack(spacing: 12) {
                Button { shell.go(Address.sign_in) } label: {
                    Text(seen.say("door.sign_in")).brand(17, 700).frame(maxWidth: .infinity, minHeight: 52)
                }
                .background(Color.white, in: Capsule()).foregroundStyle(Back)
                .saysItsWords()
                if !seen.accountsRequired {
                    Button { shell.go(Address.home) } label: {
                        Text(seen.say("nav.home")).brand(17, 700).frame(maxWidth: .infinity, minHeight: 52)
                    }
                    .background(Color.white.opacity(0.16), in: Capsule()).foregroundStyle(.white)
                    .saysItsWords()
                }
            }
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
