import Contract
import Ports
import SwiftUI

/// Who is watching. P2 builds the sheet over a title's art and the entering animation.
struct People: View {
    let shell: Shell
    let seen: Seen

    var body: some View {
        VStack(spacing: 28) {
            Text(seen.say("profiles.title")).brand(22, 700)
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 104), spacing: 20)], spacing: 20) {
                ForEach(seen.profiles, id: \.id) { person in
                    Button {
                        shell.act(.enterProfile(ActionEnterProfile(profile: person.id)))
                    } label: {
                        VStack(spacing: 8) {
                            RoundedRectangle(cornerRadius: Corner.card).fill(person.kids ? Join : Accent)
                                .frame(width: 96, height: 96)
                                .overlay(Text(String(person.name.prefix(1)).uppercased()).brand(40, 800))
                            Text(person.name).brand(14).foregroundStyle(.white).lineLimit(1)
                        }
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .padding(24)
    }
}
