import Contract
import Foundation
import Ports
import SwiftUI

public enum SettingsSection: String, CaseIterable, Sendable {
    case playback = "settings.playback"
    case languages = "settings.languages"
    case subtitles = "settings.subtitles"
    case history = "settings.history"
    case devices = "settings.devices"
    case account = "settings.account"
    case about = "settings.about"

    public var key: String { rawValue }
}

public enum SettingsAction: Equatable, Sendable {
    case flipAutoplayNext
    case flipAutoplayPreviews
    case pickAudioLanguage
    case pickSubtitleLanguage
    case cycleSubtitleSize
    case cycleSubtitleColor
    case cycleSubtitleBackground
    case askClearHistory
    case askRemoveDevice(id: String, name: String)
    case askRemoveOtherDevices
    case askSignOutEverywhere
}

public struct SettingsRow: Equatable, Sendable {
    public let label: String
    public let value: String
    public let action: SettingsAction?
    /// Drawn in the warning colour, for a value the viewer should not pass over.
    public let warn: Bool

    public init(_ label: String, value: String = "", action: SettingsAction? = nil, warn: Bool = false) {
        self.label = label
        self.value = value
        self.action = action
        self.warn = warn
    }
}

private let dateFormatter: DateFormatter = {
    let formatter = DateFormatter()
    formatter.dateFormat = "yyyy-MM-dd HH:mm"
    return formatter
}()

public func formatSeenMs(_ ms: Int) -> String {
    guard ms > 0 else { return "" }
    let date = Date(timeIntervalSince1970: Double(ms) / 1000.0)
    return dateFormatter.string(from: date)
}

public func buildSettingsRows(
    section: SettingsSection,
    words: [String: String],
    autoplayNext: Bool,
    autoplayPreviews: Bool,
    audioLanguage: String?,
    subtitleLanguage: String?,
    languages: [Language],
    uiLanguage: String,
    subSize: String,
    subColor: String,
    subBg: String,
    historyCleared: Bool,
    devices: [DeviceSession],
    account: Account?,
    server: String,
    serverPlain: Bool
) -> [SettingsRow] {
    let onWord = words["settings.on"] ?? ""
    let offWord = words["settings.off"] ?? ""

    switch section {
    case .playback:
        return [
            SettingsRow(words["settings.autoplay_next"] ?? "", value: autoplayNext ? onWord : offWord, action: .flipAutoplayNext),
            SettingsRow(words["settings.autoplay_previews"] ?? "", value: autoplayPreviews ? onWord : offWord, action: .flipAutoplayPreviews),
        ]
    case .languages:
        let audioCode = (audioLanguage?.isEmpty == false ? audioLanguage : nil) ?? uiLanguage
        let audioName = languages.first(where: { $0.code == audioCode })?.name ?? audioCode
        let subName: String
        if let sub = subtitleLanguage, !sub.isEmpty {
            subName = languages.first(where: { $0.code == sub })?.name ?? sub
        } else {
            subName = offWord
        }
        return [
            SettingsRow(words["settings.audio_language"] ?? "", value: audioName, action: .pickAudioLanguage),
            SettingsRow(words["settings.subtitle_language"] ?? "", value: subName, action: .pickSubtitleLanguage),
        ]
    case .subtitles:
        let sizeWord: String = switch subSize {
        case "0": words["settings.size_small"] ?? ""
        case "2": words["settings.size_large"] ?? ""
        default: words["settings.size_medium"] ?? ""
        }
        let colorWord = subColor.lowercased() == "#ffff00" ? (words["settings.color_yellow"] ?? "") : (words["settings.color_white"] ?? "")
        let bgWord = subBg == "black" ? (words["settings.bg_black"] ?? "") : (words["settings.bg_none"] ?? "")
        return [
            SettingsRow(words["settings.subtitle_size"] ?? "", value: sizeWord, action: .cycleSubtitleSize),
            SettingsRow(words["settings.subtitle_color"] ?? "", value: colorWord, action: .cycleSubtitleColor),
            SettingsRow(words["settings.subtitle_background"] ?? "", value: bgWord, action: .cycleSubtitleBackground),
        ]
    case .history:
        return [
            SettingsRow(words["settings.clear_history"] ?? "", value: historyCleared ? (words["settings.cleared"] ?? "") : "", action: .askClearHistory),
        ]
    case .devices:
        var rows = devices.map { device in
            let tag = device.current ? " · \(words["settings.device_current"] ?? "")" : ""
            let label = "\(device.name) (\(device.`class`.rawValue))\(tag)"
            let seenText = formatSeenMs(device.lastSeenMs)
            let action: SettingsAction? = device.current ? nil : .askRemoveDevice(id: device.id, name: device.name)
            return SettingsRow(label, value: seenText, action: action)
        }
        rows.append(SettingsRow(words["settings.devices_sign_out_others"] ?? "", value: "", action: .askRemoveOtherDevices))
        return rows
    case .account:
        let fullName = [account?.firstName, account?.lastName]
            .compactMap { $0 }
            .joined(separator: " ")
            .trimmingCharacters(in: .whitespaces)
        return [
            SettingsRow(words["settings.account_email"] ?? "", value: account?.email ?? ""),
            SettingsRow(words["settings.account_name"] ?? "", value: fullName),
        ]
    case .about:
        return [
            // An unencrypted server is marked here, because this is the only screen that says
            // which server this device talks to.
            SettingsRow(words["settings.server"] ?? "", value: server, warn: serverPlain),
            SettingsRow(words["settings.sign_out_everywhere"] ?? "", value: "", action: .askSignOutEverywhere),
        ]
    }
}

public struct SettingsRowItem: View {
    let row: SettingsRow
    let on: Bool

    public init(row: SettingsRow, on: Bool) {
        self.row = row
        self.on = on
    }

    public var body: some View {
        let labelColor = on ? Back : Color.white
        let valueColor = row.warn ? Join : (on ? Back : Muted)

        HStack {
            Text(row.label)
                .brand(30)
                .foregroundStyle(labelColor)
                .lineLimit(1)
            Spacer(minLength: 24.ref)
            if !row.value.isEmpty {
                Text(row.value)
                    .brand(30, 500)
                    .foregroundStyle(valueColor)
                    .lineLimit(1)
            }
        }
        .frame(width: 1080.ref)
        .padding(.horizontal, 32.ref)
        .padding(.vertical, 14.ref)
        .background(on ? Color.white : Color.clear, in: Capsule())
        .padding(.bottom, 6.ref)
    }
}

public struct SubtitlePreview: View {
    let words: [String: String]
    let size: String
    let color: String
    let background: String

    public init(words: [String: String], size: String, color: String, background: String) {
        self.words = words
        self.size = size
        self.color = color
        self.background = background
    }

    public var body: some View {
        let fontSize: CGFloat = switch size {
        case "0": 26
        case "2": 42
        default: 34
        }
        let ink = color.lowercased() == "#ffff00" ? Color(red: 1, green: 1, blue: 0) : Color.white
        let boxBg = background == "black" ? Color.black.opacity(0.9) : Color.clear

        VStack(alignment: .leading, spacing: 0) {
            ZStack(alignment: .center) {
                RoundedRectangle(cornerRadius: Corner.field * 1.5)
                    .fill(Raised)
                    .frame(width: 1040.ref, height: 140.ref)
                Text(words["settings.subtitle_preview"] ?? "")
                    .brand(fontSize, 600)
                    .foregroundStyle(ink)
                    .lineLimit(1)
                    .padding(.horizontal, 20.ref)
                    .padding(.vertical, 8.ref)
                    .background(boxBg, in: shapeOf(Round.chip))
            }
            .frame(width: 1040.ref, height: 140.ref)
            Spacer().frame(height: 8.ref)
        }
        .padding(.top, 28.ref)
        .padding(.leading, 16.ref)
    }
}
