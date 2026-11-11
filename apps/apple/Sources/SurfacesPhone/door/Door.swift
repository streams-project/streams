import Contract
import Ports
import SwiftUI

/// Where a phone points at its server, for self hosters.
struct ServerAddress: View {
    let shell: Shell
    @State private var address = "https://"

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            TextField("", text: $address)
                .keyboardType(.URL).textInputAutocapitalization(.never).autocorrectionDisabled()
                .padding(14).background(Raised, in: RoundedRectangle(cornerRadius: Corner.field))
            Button { shell.connect(address) } label: {
                Image(systemName: "arrow.right").frame(maxWidth: .infinity, minHeight: 48)
            }
            .buttonStyle(.borderedProminent).tint(.white).foregroundStyle(Back)
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
            Mark(120)
            Spacer()
            Button { shell.go(Address.sign_in) } label: {
                Text(seen.say("door.sign_in")).brand(17, 700).frame(maxWidth: .infinity, minHeight: 52)
            }
            .background(Color.white, in: Capsule()).foregroundStyle(Back)
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
            TextField("", text: $email, prompt: Text(verbatim: "Email"))
                .keyboardType(.emailAddress).textContentType(.username)
                .textInputAutocapitalization(.never).autocorrectionDisabled()
                .padding(14).background(Raised, in: RoundedRectangle(cornerRadius: Corner.field))
            SecureField("", text: $password)
                .textContentType(.password)
                .padding(14).background(Raised, in: RoundedRectangle(cornerRadius: Corner.field))
            Button { shell.act(.signIn(ActionSignIn(email: email, password: password))) } label: {
                Text(seen.say("door.sign_in")).brand(17, 700).frame(maxWidth: .infinity, minHeight: 52)
            }
            .background(Color.white, in: Capsule()).foregroundStyle(Back)
            if let notice = seen.notice { Text(notice).brand(14).foregroundStyle(Muted) }
        }
        .padding(24)
    }
}
