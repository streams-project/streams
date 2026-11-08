import Contract
import Ports
import SwiftUI

/// The pages of Edit Profile.
enum EditStep { case menu, name, icon, language, lock, delete }

private func rowsOf(_ sets: AvatarSets?, _ history: String, _ person: Profile) -> [PictureRow] {
    let rows = (sets?.sets ?? []).map { PictureRow(label: $0.name, pictures: $0.pictures.map(\.path)) }
    guard let worn = person.avatar, worn.hasPrefix("/") else { return rows }
    return Array(rows.prefix(1)) + [PictureRow(label: history, pictures: [worn])] + Array(rows.dropFirst())
}

private func decode<T: Decodable>(_ type: T.Type, _ text: String?) -> T? {
    text.flatMap { try? JSONDecoder().decode(type, from: Data($0.utf8)) }
}

/// Edit Profile. It opens from the picker and from Settings, and takes the same profile either way.
struct EditProfile: View {
    let shell: Shell
    let seen: Seen
    let start: Profile
    var pinDigits = 4
    let leave: () -> Void
    @State private var person: Profile
    @State private var step = EditStep.menu
    @State private var refused = false
    @State private var menuAt = 0
    @State private var sets: AvatarSets?
    @State private var languages: [Language] = []
    @State private var limit = ""

    init(shell: Shell, seen: Seen, person: Profile, pinDigits: Int = 4, leave: @escaping () -> Void) {
        self.shell = shell
        self.seen = seen
        self.start = person
        self.pinDigits = pinDigits
        self.leave = leave
        _person = State(initialValue: person)
    }

    private func save(_ change: ProfileUpdate) {
        Task {
            let body = String(data: (try? JSONEncoder().encode(change)) ?? Data(), encoding: .utf8) ?? "{}"
            let answer = await shell.write("PATCH", "/v1/profiles/\(person.id)", body)
            refused = answer == nil
            if let updated = decode(Profile.self, answer) {
                person = updated
                shell.keep(updated)
                step = .menu
            }
        }
    }

    private func remove() {
        Task {
            guard await shell.write("DELETE", "/v1/profiles/\(person.id)", "") != nil else { return }
            let updated = decode(ProfileList.self, await shell.read("/v1/profiles"))?.profiles
                ?? seen.profiles.filter { $0.id != person.id }
            shell.keepAll(updated)
            leave()
        }
    }

    var body: some View {
        Group {
            switch step {
            case .menu, .delete:
                EditMenu(
                    seen: seen, person: person, limit: limit,
                    language: languages.first { $0.code == (person.language ?? seen.language) }?.name,
                    pictures: !(sets?.sets ?? []).isEmpty, at: menuAt,
                    move: { menuAt = $0 },
                    open: { chosen in if chosen == .delete { remove() } else { step = chosen } },
                    leave: leave
                )
            case .name:
                EditName(seen: seen, person: person, refused: refused) { name in
                    if let name { save(ProfileUpdate(name: name)) } else { step = .menu }
                }
            case .icon:
                EditIcon(seen: seen, person: person, rows: rowsOf(sets, seen.say("profile.icon.history"), person)) { chosen in
                    if let chosen { save(ProfileUpdate(avatar: chosen)) } else { step = .menu }
                }
            case .language:
                EditLanguage(seen: seen, list: languages, current: person.language ?? seen.language) { code in
                    if let code { save(ProfileUpdate(language: code)) } else { step = .menu }
                }
            case .lock:
                EditLock(shell: shell, seen: seen, person: person, pinDigits: pinDigits) { updated in
                    if let updated { person = updated }
                    step = .menu
                }
            }
        }
        .task(id: person.id) {
            sets = decode(AvatarSets.self, await shell.read("/v1/profiles/\(person.id)/avatars"))
            limit = decode(ProfileLimits.self, await shell.read("/v1/profiles/\(person.id)/limits"))?.ageRating ?? ""
        }
        .task { languages = decode(LanguageList.self, await shell.read("/v1/languages"))?.languages ?? [] }
        .onChange(of: person.language, initial: true) { _, language in shell.speak(language) }
    }
}
