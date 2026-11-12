import Contract
import Ports
import SwiftUI

/// Who is watching. P2 builds the sheet over a title's art and the entering animation.
struct People: View {
    let shell: Shell
    let seen: Seen

    var body: some View {
        VStack(spacing: 28) {
            Text(seen.say("profiles.choose.phone")).brand(22, 700)
                .accessibilityAddTraits(.isHeader)
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 104), spacing: 20)], spacing: 20) {
                ForEach(seen.profiles, id: \.id) { person in
                    Button {
                        shell.act(.enterProfile(ActionEnterProfile(profile: person.id)))
                    } label: {
                        VStack(spacing: 8) {
                            // The plate carries the profile's first letter, which is a letter and
                            // not a word: the name is drawn under it and says the whole of it.
                            // Unmarked the cell read as "I, Ibrahim".
                            RoundedRectangle(cornerRadius: Corner.card).fill(person.kids ? Join : Accent)
                                .frame(width: 96, height: 96)
                                .overlay(Text(String(person.name.prefix(1)).uppercased()).brand(40, 800))
                                .decoration()
                            Text(person.name).brand(14).foregroundStyle(.white).lineLimit(1)
                        }
                    }
                    .buttonStyle(.plain)
                    // The name is drawn in the cell, so the cell gathers it.
                    .saysItsWords()
                }
            }
        }
        .padding(24)
    }
}
