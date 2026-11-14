import Contract
import Ports
import SwiftUI

private let firstLine: CGFloat = 262
private let rowStep: CGFloat = 78
private let screenHeight: CGFloat = 1080

private enum LanguageTarget {
    case audio
    case subtitle
}

/// Settings: seven sections on the left and their rows on the right.
public struct Settings: View {
    public let shell: Shell
    public let seen: Seen

    @State private var autoplayNext = true
    @State private var autoplayPreviews = true
    @State private var audioLanguage: String?
    @State private var subtitleLanguage: String?
    @State private var subSize = "1"
    @State private var subColor = "#ffffff"
    @State private var subBg = "none"
    @State private var historyCleared = false
    @State private var devices: [DeviceSession] = []
    @State private var account: Account?
    @State private var languages: [Language] = []

    @State private var sectionIndex = 0
    @State private var right = false
    @State private var line = 0
    @State private var langPicker: LanguageTarget?
    @State private var pendingConfirm: SettingsAction?
    // The two line question a row can ask, and where the remote is on it. Both are `parts/Question`,
    // which the reminder screen asks with as well, so the panel cannot answer one word here and a
    // different one there.
    @State private var question = Question()

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    private func sendSetting(key: SettingKey, value: String) {
        shell.act(.command(ActionCommand(command: .setting(CommandSetting(key: key, value: value)))))
    }

    private func sendSubtitleStyle(size: String, color: String, bg: String) {
        let json = "{\"size\":\"\(size)\",\"color\":\"\(color)\",\"background\":\"\(bg)\"}"
        sendSetting(key: .subtitleStyle, value: json)
    }

    private func refreshDevices() async {
        if let text = await shell.read("/v1/devices"),
           let list = try? JSONDecoder().decode(DeviceList.self, from: Data(text.utf8)) {
            devices = list.devices
        }
    }

    private func executeConfirmed(_ action: SettingsAction) {
        pendingConfirm = nil
        switch action {
        case .askClearHistory:
            historyCleared = true
            shell.act(.command(ActionCommand(command: .clearHistory(CommandClearHistory()))))
        case let .askRemoveDevice(id, _):
            Task {
                _ = await shell.write("DELETE", "/v1/devices?id=\(id)", "")
                await refreshDevices()
            }
        case .askRemoveOtherDevices:
            Task {
                _ = await shell.write("DELETE", "/v1/devices", "")
                await refreshDevices()
            }
        case .askSignOutEverywhere:
            shell.act(.signOut(ActionSignOut(everywhere: true)))
        default:
            break
        }
    }

    private func activateRow(_ action: SettingsAction?) {
        guard let action else { return }
        switch action {
        case .flipAutoplayNext:
            autoplayNext.toggle()
            sendSetting(key: .autoplayNext, value: String(autoplayNext))
        case .flipAutoplayPreviews:
            autoplayPreviews.toggle()
            sendSetting(key: .autoplayPreviews, value: String(autoplayPreviews))
        case .pickAudioLanguage:
            langPicker = .audio
        case .pickSubtitleLanguage:
            langPicker = .subtitle
        case .cycleSubtitleSize:
            subSize = switch subSize {
            case "0": "1"
            case "1": "2"
            default: "0"
            }
            sendSubtitleStyle(size: subSize, color: subColor, bg: subBg)
        case .cycleSubtitleColor:
            subColor = subColor.lowercased() == "#ffff00" ? "#ffffff" : "#ffff00"
            sendSubtitleStyle(size: subSize, color: subColor, bg: subBg)
        case .cycleSubtitleBackground:
            subBg = subBg == "black" ? "none" : "black"
            sendSubtitleStyle(size: subSize, color: subColor, bg: subBg)
        case .askClearHistory, .askRemoveDevice, .askRemoveOtherDevices, .askSignOutEverywhere:
            pendingConfirm = action
            question = Question()
        }
    }

    private func press(_ word: Word, rowsCount: Int) -> Bool {
        if let asking = pendingConfirm {
            switch question.press(word) {
            case .yes: executeConfirmed(asking)
            case .close: pendingConfirm = nil
            case .moved, .nothing: break
            }
            return true
        }

        switch word {
        case .back where right:
            right = false
        case .back:
            shell.back()
        case .left where right:
            right = false
        case .left:
            // On sections (left edge), Left goes back.
            shell.back()
        case .up where right:
            if line > 0 { line -= 1 }
        case .down where right:
            if line < rowsCount - 1 { line += 1 }
        case .ok where right:
            let rows = currentRows()
            activateRow(rows[safe: line]?.action)
        case .up where !right:
            if sectionIndex > 0 {
                sectionIndex -= 1
                line = 0
                historyCleared = false
            }
        case .down where !right:
            if sectionIndex < SettingsSection.allCases.count - 1 {
                sectionIndex += 1
                line = 0
                historyCleared = false
            }
        case .right, .ok:
            if !right && rowsCount > 0 {
                right = true
            }
        default:
            break
        }
        return true
    }

    /// What the remote is on, for the screen box that says it aloud (`app/Remote.swift`). Every
    /// word here is one the screen already draws, out of `strings/`.
    private func hereIs(sections: [SettingsSection], rows: [SettingsRow]) -> String {
        if pendingConfirm != nil {
            // The question's two lines are its own word and Cancel; the question itself is read
            // from the tree above them.
            return question.word(yes: askedWord(), close: seen.words["common.cancel"] ?? "")
        }
        if right {
            guard let row = rows[safe: line] else { return "" }
            return sayBoth(row.label, row.value)
        }
        return seen.words[sections[safe: sectionIndex]?.key ?? ""] ?? ""
    }

    /// The word on the line that answers a question with yes.
    private func askedWord() -> String {
        switch pendingConfirm {
        case .askClearHistory: return seen.words["settings.clear_history"] ?? ""
        case .askRemoveDevice: return seen.words["settings.device_remove"] ?? ""
        case .askRemoveOtherDevices: return seen.words["settings.devices_sign_out_others"] ?? ""
        case .askSignOutEverywhere: return seen.words["settings.sign_out_everywhere"] ?? ""
        default: return ""
        }
    }

    private func currentRows() -> [SettingsRow] {
        let section = SettingsSection.allCases[sectionIndex]
        return buildSettingsRows(
            section: section,
            words: seen.words,
            autoplayNext: autoplayNext,
            autoplayPreviews: autoplayPreviews,
            audioLanguage: audioLanguage,
            subtitleLanguage: subtitleLanguage,
            languages: languages,
            uiLanguage: seen.language.isEmpty ? "en" : seen.language,
            subSize: subSize,
            subColor: subColor,
            subBg: subBg,
            historyCleared: historyCleared,
            devices: devices,
            account: account,
            server: seen.server,
            serverPlain: seen.serverPlain
        )
    }

    public var body: some View {
        if let target = langPicker {
            let list = target == .subtitle
                ? [Language(code: "", dir: .ltr, name: seen.words["settings.off"] ?? "")] + languages
                : languages
            let current = target == .subtitle
                ? (subtitleLanguage ?? "")
                : ((audioLanguage?.isEmpty == false ? audioLanguage : nil) ?? seen.language)

            EditLanguage(seen: seen, list: list, current: current) { chosen in
                langPicker = nil
                guard let chosen else { return }
                if target == .audio {
                    audioLanguage = chosen
                    sendSetting(key: .audioLanguage, value: chosen)
                } else {
                    subtitleLanguage = chosen.isEmpty ? nil : chosen
                    sendSetting(key: .subtitleLanguage, value: chosen)
                }
            }
        } else {
            let sections = SettingsSection.allCases
            let section = sections[sectionIndex]
            let rows = currentRows()
            let scrollOffset = CGFloat(max(0, line - 7)) * rowStep

            ZStack(alignment: .topLeading) {
                Color.black
                PageHead(title: seen.words["settings.title"] ?? "", help: seen.words["settings.intro"] ?? "")

                let menuLines = sections.map { MenuLine(word: seen.words[$0.key] ?? "") }
                Menu(
                    lines: menuLines,
                    focus: right ? -1 : sectionIndex,
                    width: 480,
                    kept: sectionIndex
                )
                .offset(x: (subEdge - menuWords).ref, y: firstLine.ref)

                ZStack(alignment: .topLeading) {
                    if let asking = pendingConfirm {
                        let (askPrompt, yesWord): (String, String) = switch asking {
                        case .askClearHistory:
                            (seen.words["settings.clear_history.ask"] ?? "", seen.words["settings.clear_history"] ?? "")
                        case .askRemoveDevice:
                            (seen.words["settings.device_remove.ask"] ?? "", seen.words["settings.device_remove"] ?? "")
                        case .askRemoveOtherDevices:
                            (seen.words["settings.devices_sign_out_others.ask"] ?? "", seen.words["settings.devices_sign_out_others"] ?? "")
                        case .askSignOutEverywhere:
                            (seen.words["settings.sign_out_everywhere.ask"] ?? "", seen.words["settings.sign_out_everywhere"] ?? "")
                        default:
                            ("", "")
                        }

                        // The question itself is a heading, because it is what the two lines
                        // under it answer. The panel is `parts/QuestionPanel`, which the
                        // reminder screen draws as well.
                        QuestionPanel(
                            prompt: askPrompt,
                            yes: yesWord,
                            close: seen.words["common.cancel"] ?? "",
                            at: question.at
                        )
                    } else {
                        VStack(alignment: .leading, spacing: 0) {
                            ForEach(Array(rows.enumerated()), id: \.offset) { index, row in
                                SettingsRowItem(row: row, on: right && index == line)
                            }
                            if section == .subtitles {
                                SubtitlePreview(words: seen.words, size: subSize, color: subColor, background: subBg)
                            }
                        }
                        .offset(y: -scrollOffset.ref)
                        .animation(.easeOut(duration: 0.24), value: line)
                    }
                }
                .frame(width: 1100.ref, height: (screenHeight - firstLine - 60).ref, alignment: .topLeading)
                .clipped()
                .offset(x: 700.ref, y: firstLine.ref)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            // What the remote is on: a section in the column on the left, a row of that section on
            // the right, or a line of the question a row asked. A row is said with how it stands,
            // because on this screen the value is the point of stopping on the row at all.
            .remote(says: hereIs(sections: sections, rows: rows)) { press($0, rowsCount: rows.count) }
            .task {
                if let text = await shell.read("/v1/languages"),
                   let list = try? JSONDecoder().decode(LanguageList.self, from: Data(text.utf8)) {
                    languages = list.languages
                }
                if let text = await shell.read("/v1/profiles/me/settings"),
                   let loaded = try? JSONDecoder().decode(ProfileSettings.self, from: Data(text.utf8)) {
                    autoplayNext = loaded.autoplayNext
                    autoplayPreviews = loaded.autoplayPreviews
                    audioLanguage = loaded.audioLanguage
                    subtitleLanguage = loaded.subtitleLanguage
                    if let style = loaded.subtitleStyle {
                        subSize = style["size"] ?? "1"
                        subColor = style["color"] ?? "#ffffff"
                        subBg = style["background"] ?? "none"
                    }
                }
                await refreshDevices()
                if let text = await shell.read("/v1/account"),
                   let acc = try? JSONDecoder().decode(Account.self, from: Data(text.utf8)) {
                    account = acc
                }
            }
        }
    }
}
