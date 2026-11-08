import Contract
import Foundation
import Ports

/// One choice under the bar: a sound, with or without subtitles, or the way to the full lists.
struct Choice {
    let word: String
    let audio: Track?
    let text: Track?
    let ticked: Bool
    var other = false
}

extension Track {
    /// True for the track in play.
    var playing: Bool { selected == true }
}

/// What a viewer reads for a track: its language, as sources often label tracks for machines.
func nameOf(_ track: Track, _ language: String) -> String {
    if let lang = track.lang, let word = Locale(identifier: language).localizedString(forLanguageCode: lang), !word.isEmpty {
        return word.prefix(1).uppercased() + word.dropFirst()
    }
    if let label = track.label, !label.isEmpty { return label }
    return track.id
}

/// One line per language, keeping the track in play when several share a language.
func oneOfEach(_ tracks: [Track]) -> [Track] {
    var order: [String] = []
    var groups: [String: [Track]] = [:]
    for track in tracks {
        let key = track.lang ?? track.id
        if groups[key] == nil { order.append(key) }
        groups[key, default: []].append(track)
    }
    return order.compactMap { key in groups[key].flatMap { same in same.first { $0.playing } ?? same.first } }
}

/// The stream's own subtitles, then the server's files for languages the stream lacks.
@MainActor
func linesOf(_ seen: Seen, _ files: [Track]) -> [Track] {
    let own = oneOfEach(seen.text)
    let chosen = files.contains { $0.playing }
    let kept = chosen ? own.map { var off = $0; off.selected = false; return off } : own
    return kept + files.filter { file in !own.contains { $0.lang == file.lang } }
}

/// The short list: this sound alone, this sound with its subtitles, and everything else.
@MainActor
func choicesOf(_ seen: Seen, _ files: [Track]) -> [Choice] {
    let sounds = oneOfEach(seen.audio)
    let lines = linesOf(seen, files)
    if sounds.count < 2 && lines.isEmpty { return [] }
    guard let sound = sounds.first(where: { $0.playing }) ?? sounds.first else { return [] }
    let reading = lines.first { $0.playing }
    let name = nameOf(sound, seen.language)
    let same = lines.first { $0.lang == sound.lang } ?? lines.first { $0.lang == seen.language }
    var out = [Choice(word: name, audio: sound, text: nil, ticked: reading == nil)]
    if let same {
        let word = seen.say("player.with_subtitles").replacingOccurrences(of: "{audio}", with: name)
        out.append(Choice(word: word, audio: sound, text: same, ticked: reading?.id == same.id))
    }
    let elsewhere = reading != nil && reading?.id != same?.id
    if sounds.count > 1 || lines.count > 1 {
        out.append(Choice(word: seen.say("player.other"), audio: nil, text: nil, ticked: elsewhere, other: true))
    }
    return out
}
