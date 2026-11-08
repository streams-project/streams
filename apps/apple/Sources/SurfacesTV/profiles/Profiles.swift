import Contract
import Ports
import SwiftUI

private let defaultProfilesMax = 5
private let defaultNameMax = 32
private let defaultPinDigits = 4

/// Who is watching: a column of profiles on the left, and a title chosen for the one in focus.
public struct Profiles: View {
    let shell: Shell
    let seen: Seen
    @State private var hello: Hello?
    @State private var at = 0
    @State private var pencil = false
    @State private var locked: Profile?
    @State private var editing: Profile?
    @State private var adding = false
    @State private var wanted: String?

    /// True once this run has been past the chooser, so Back returns rather than leaves the app.
    var fromInside = false

    public init(shell: Shell, seen: Seen, fromInside: Bool = false) {
        self.fromInside = fromInside
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        let list = seen.profiles
        let profilesMax = hello?.limits.profilesMax ?? defaultProfilesMax
        let nameMax = hello?.limits.profileNameMax ?? defaultNameMax
        let pinDigits = hello?.limits.pinDigits ?? defaultPinDigits
        let canAdd = list.count < profilesMax
        let maxIndex = canAdd ? list.count : list.count - 1

        Group {
            if adding {
                AddProfile(shell: shell, seen: seen, nameMax: nameMax) { created in
                    adding = false
                    pencil = false
                    wanted = created
                }
            } else if let person = editing {
                EditProfile(shell: shell, seen: seen, person: person, pinDigits: pinDigits) {
                    editing = nil
                    pencil = false
                }
            } else if let person = locked {
                PinPanel(shell: shell, seen: seen, person: person, digitsCount: pinDigits, unlocked: { pin in
                    let toEdit = pencil
                    locked = nil
                    if toEdit {
                        editing = person
                    } else {
                        shell.speak(person.language)
                        shell.act(.enterProfile(ActionEnterProfile(pin: pin, profile: person.id)))
                    }
                }, leave: { locked = nil })
            } else if let feature = list[safe: at] ?? list.last {
                chooser(list, feature, canAdd, maxIndex)
            }
        }
        .task { hello = await shell.read("/v1/hello").flatMap { try? JSONDecoder().decode(Hello.self, from: Data($0.utf8)) } }
        .onChange(of: list.map(\.id)) { _, ids in
            // A profile just made takes the focus once it is in the list.
            if let wanted, let found = ids.firstIndex(of: wanted) {
                at = found
                self.wanted = nil
            }
            if maxIndex >= 0 && at > maxIndex { at = maxIndex }
        }
    }

    private func chooser(_ list: [Profile], _ feature: Profile, _ canAdd: Bool, _ maxIndex: Int) -> some View {
        let person = list[safe: at]

        func press(_ word: Word) -> Bool {
            switch word {
            case .up:
                guard at > 0 else { return false }
                at -= 1
                pencil = false
            case .down:
                guard at < maxIndex else { return false }
                at += 1
                pencil = false
            case .left:
                guard person != nil else { return false }
                pencil = true
            case .right: pencil = false
            case .ok:
                if let person {
                    if person.hasPin { locked = person }
                    else if pencil { editing = person }
                    else {
                        shell.speak(person.language)
                        shell.act(.enterProfile(ActionEnterProfile(profile: person.id)))
                    }
                } else {
                    adding = true
                }
            // Opened with Switch Profiles, Back returns to the page; at the start of the app it leaves.
            case .back:
                if fromInside { shell.back() } else { return false }
            }
            return true
        }

        return ZStack(alignment: .topLeading) {
            ProfileFeature(shell: shell, id: feature.id)
            VStack(spacing: 8.ref) {
                Mark(68)
                Text(seen.say("profiles.choose")).brand(24, 500).foregroundStyle(.white).fixedSize()
            }
            .frame(width: 200.ref)
            .offset(x: 190.ref, y: 44.ref)
            VStack(alignment: .leading, spacing: 26.ref) {
                ForEach(Array(list.enumerated()), id: \.element.id) { index, profile in
                    ProfileRow(person: profile, on: index == at, pencil: pencil && index == at, server: seen.server)
                }
                if canAdd { AddProfileRow(word: seen.say("profile.add"), on: at == list.count) }
            }
            .frame(width: 720.ref, alignment: .topLeading)
            .offset(y: 168.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { press($0) }
    }
}
