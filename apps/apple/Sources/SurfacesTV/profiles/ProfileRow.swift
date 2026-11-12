import Contract
import SwiftUI

let faceLarge: CGFloat = 194
private let faceSmall: CGFloat = 130
private let grow = 0.2

/// One profile in the column. The one in focus grows in place, with its name and the pencil.
///
/// Only the row in focus draws the name, so an unfocused row has nothing drawn on it a reader could
/// gather: the row carries the name itself. `lockedWord` is the padlock drawn under a locked
/// profile, said as the whole label because `say.profile_locked` already holds the name;
/// `pencilWord` is the pencil, which means this press edits rather than enters. Both come from
/// `strings/` and are given by the screen, which is the part that holds the words.
struct ProfileRow: View {
    let person: Profile
    let on: Bool
    let pencil: Bool
    let server: String
    var pencilWord: String = ""
    var lockedWord: String = ""

    var body: some View {
        let size = on ? faceLarge : faceSmall
        ZStack(alignment: .topLeading) {
            if on {
                Circle()
                    .fill(pencil ? Color.white : .clear)
                    .frame(width: 56.ref, height: 56.ref)
                    .overlay(Icon("icon_pencil", 30, tint: pencil ? Back : Muted))
                    .offset(x: 111.ref, y: 67.ref)
            }
            Face(person, size, server)
                .overlay(shapeOf(Round.face(size)).stroke(.white, lineWidth: on && !pencil ? 4.ref : 0))
                .offset(x: (on ? 193 : 225).ref)
            if on {
                HStack(spacing: 16.ref) {
                    Text(person.name).brand(40, 700).foregroundStyle(.white)
                    if person.hasPin { Icon("icon_locked", 32, tint: Muted) }
                }
                .offset(x: 420.ref, y: 74.ref)
            }
        }
        .frame(width: 720.ref, height: size.ref, alignment: .topLeading)
        .animation(glide(grow), value: on)
        // A padlock says a PIN is wanted; the pencil says this press edits rather than enters. Both
        // are pictures, and the second only appears on the row the remote is on, so which of the
        // two a press will do is the thing a person listening most needs told.
        .says(person.hasPin && !lockedWord.isEmpty ? lockedWord : person.name, on: on)
        .accessibilityValue(on && pencil ? pencilWord : "")
    }
}
