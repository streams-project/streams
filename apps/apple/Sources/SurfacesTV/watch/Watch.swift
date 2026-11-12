import Contract
import Ports
import SwiftUI
import UIKit

private let shownFor = 3.0
private let seekStep: Int64 = 10_000
/// How long the bar waits for the next press before it seeks, and for the picture to catch up.
private let seekSettle = 0.35
private let seekCatchUp = 1.5
/// How long the end waits, so the next episode can start before the page goes back.
private let endedWait = 0.8
/// How far subtitles rise, as a share of the picture's height.
private let wordsLow: Float = 0
private let wordsHigh: Float = 0.2

private enum Reach { case top, rate, bar, tracks }
private enum Control { case back, restart, next }

private let shade = LinearGradient(
    stops: [
        .init(color: .black.opacity(0.55), location: 0),
        .init(color: .clear, location: 0.3),
        .init(color: .clear, location: 0.6),
        .init(color: .black.opacity(0.7), location: 1),
    ],
    startPoint: .top, endPoint: .bottom
)

private struct RateChoice {
    let rating: Rating
    let word: String
    let icon: String
}

private let rateChoices = [
    RateChoice(rating: .down, word: "player.down", icon: "icon_thumb_down"),
    RateChoice(rating: .up, word: "player.up", icon: "icon_thumb_up"),
    RateChoice(rating: .love, word: "player.love", icon: "icon_love"),
]

private func clock(_ ms: Int64) -> String {
    let seconds = max(0, ms / 1000)
    if seconds >= 3600 { return number(String(format: "%d:%02d:%02d", seconds / 3600, seconds / 60 % 60, seconds % 60)) }
    return number(String(format: "%d:%02d", seconds / 60 % 60, seconds % 60))
}

/// The picture of what plays, from the player port.
private struct Picture: UIViewRepresentable {
    let shell: Shell

    func makeUIView(context: Context) -> UIView {
        shell.player.picture(context) as? UIView ?? UIView()
    }

    func updateUIView(_ view: UIView, context: Context) {}
}

/// What a control at the top of the picture is called, in the viewer's language.
@MainActor
private func controlWord(_ control: Control, _ seen: Seen) -> String {
    switch control {
    case .back: return seen.say("player.back")
    case .restart: return seen.say("player.restart")
    case .next: return seen.say("player.next")
    }
}

private struct TopIcons: View {
    let controls: [Control]
    let focus: Int
    let seen: Seen

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(Array(controls.enumerated()), id: \.offset) { index, control in
                let on = index == focus
                let x = CGFloat(120 + index * 124)
                Circle()
                    .fill(on ? Color.white : .clear)
                    .frame(width: 72.ref, height: 72.ref)
                    .overlay(Icon(control == .back ? "icon_back" : control == .restart ? "icon_restart" : "icon_next", 46, tint: on ? Back : .white))
                    .offset(x: x.ref, y: 112.ref)
                    // The word under the icon is drawn only for the one in focus, so two of the
                    // three are pictures with nothing beside them at any moment: each is given its
                    // own word instead of gathering one that may not be there.
                    .says(controlWord(control, seen), on: on)
                if on {
                    // The word for the control above, drawn once it is in focus. The control says
                    // it already, so a reader that stopped here would say it twice.
                    Text(seen.say(control == .back ? "player.back" : control == .restart ? "player.restart" : "player.next"))
                        .brand(28).foregroundStyle(.white).multilineTextAlignment(.center)
                        .frame(width: 252.ref)
                        .offset(x: (x - 90).ref, y: 202.ref)
                        .decoration()
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

private struct RatePills: View {
    let focus: Int
    let chosen: Rating?
    let seen: Seen

    var body: some View {
        ZStack(alignment: .topLeading) {
            HStack(spacing: 22.ref) {
                ForEach(Array(rateChoices.enumerated()), id: \.offset) { index, rate in
                    let on = index == focus
                    let ink = on ? Back : Color.white
                    HStack(spacing: 12.ref) {
                        Icon(rate.icon, 34, tint: ink)
                        Text(seen.say(rate.word)).brand(28, 700).foregroundStyle(ink)
                    }
                    .padding(.horizontal, 22.ref)
                    .frame(height: 62.ref)
                    .background(on ? Color.white : Color.white.opacity(chosen == rate.rating ? 0.4 : 0.2), in: Capsule())
                    // Each pill draws its own word, so it gathers it and the thumb beside it is
                    // decoration. Three plates are drawn and they mean two different things: white
                    // is where the remote is, half lit is the rating already given. A person
                    // listening sees neither.
                    .saysItsWords(state: chosen == rate.rating ? seen.say("say.chosen") : "")
                    .accessibilityAddTraits(on ? [.isSelected] : [])
                }
            }
            .offset(x: 122.ref, y: 596.ref)
            Text(seen.say("player.hint")).brand(28).foregroundStyle(.white).offset(x: 122.ref, y: 694.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

/// The bar: one control, named by what Ok does to it and valued by where the film is.
///
/// This is the one part of the player a person listening cannot read any other way. The two clocks
/// are drawn as text and a reader could stop on each, but it would then read "12:30" and "1:45:00"
/// as two unrelated numbers with the bar between them saying nothing at all, and the bar is where
/// the whole of the information is. So the four pieces are one node: its name is play or pause,
/// because that is what pressing it does, and its value is how far through the film it is, from
/// `say.position`.
/// Internal rather than private so a test can host one: the player draws itself from a live `Play`
/// state no test process has, so the bar is read as the part it is.
struct Bar: View {
    let seen: Seen
    let at: Int64
    let playing: Bool
    let on: Bool

    var body: some View {
        let done = seen.lengthMs > 0 ? min(max(Double(at) / Double(seen.lengthMs), 0), 1) : 0
        let where_ = seen.lengthMs > 0
            ? seen.say("say.position").fill("at", clock(at)).fill("length", clock(seen.lengthMs))
            : clock(at)
        ZStack(alignment: .topLeading) {
            Circle()
                .fill(on ? Color.white : .clear)
                .frame(width: 78.ref, height: 78.ref)
                .overlay(Icon(playing ? "icon_pause" : "icon_play", 48, tint: on ? Back : .white))
                .offset(x: 262.ref, y: 874.ref)
            Text(clock(at)).brand(30).foregroundStyle(.white)
                .frame(width: 100.ref, alignment: .trailing)
                .offset(x: 360.ref, y: 894.ref)
            ZStack(alignment: .leading) {
                Capsule().fill(Color.white.opacity(0.4))
                Capsule().fill(Accent).frame(width: (1052 * done).ref)
            }
            .frame(width: 1052.ref, height: 6.ref)
            .offset(x: 476.ref, y: 910.ref)
            Text(clock(seen.lengthMs)).brand(30).foregroundStyle(.white).offset(x: 1548.ref, y: 894.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .says(seen.say(playing ? "say.pause" : "common.play"), on: on)
        .accessibilityValue(where_)
    }
}

/// The picture, and the controls that rise over it when the viewer reaches for the remote.
public struct Watch: View {
    let shell: Shell
    let seen: Seen
    @State private var awake = 1
    @State private var shown = true
    @State private var zone = Reach.bar
    @State private var top = 0
    @State private var rate = 1
    @State private var detail: TitleDetail?
    @State private var rated: Rating?
    @State private var stay = 0
    @State private var track = 0
    @State private var aim: Int64?
    @State private var panel = TrackPanelState()
    @State private var side = SideWords()
    @State private var kept: ProfileSettings?

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        let play = seen.play
        let paused = play?.state == .paused
        let failed = play?.state == .failed
        let asking = play?.stillAsking == true
        let choices = choicesOf(seen, side.tracks)
        let part = seen.windowNow([.intro, .recap])
        let closing = play?.nextEpisode != nil ? seen.windowNow([.credits]) : nil
        let age = seen.windowNow([.ageCard])?.text
        let next = play?.nextEpisode
        let controls: [Control] = next != nil ? [.back, .restart, .next] : [.back, .restart]

        func toggle() { shell.act(paused ? .unpause(ActionUnpause()) : .pause(ActionPause())) }

        /// Presses add up from where the last one aimed, as the position only moves once a second.
        func seek(_ by: Int64) {
            let end = seen.lengthMs > 0 ? seen.lengthMs : Int64.max
            aim = min(max((aim ?? seen.positionMs) + by, 0), end)
        }

        func goNext() {
            guard let episode = next, let play else { return }
            shell.swap(Address.watch + play.title.rawValue.replacingOccurrences(of: ":", with: "/") + "/\(episode.season)/\(episode.episode)")
        }

        func leave() {
            shell.act(.stop(ActionStop()))
            shell.back()
        }

        func chooseLine(_ line: Track?) {
            guard let line, line.isFile else {
                side.off()
                shell.act(.selectText(ActionSelectText(track: line?.id)))
                return
            }
            shell.act(.selectText(ActionSelectText(track: nil)))
            guard let language = line.lang else { return }
            shell.act(.command(ActionCommand(command: .setting(CommandSetting(key: .subtitleLanguage, value: language)))))
            Task { await side.pick(shell, language, seen.lengthMs) }
        }

        func choose(_ choice: Choice) {
            if choice.other {
                panel.show(seen)
                return
            }
            if let sound = choice.audio, !sound.playing { shell.act(.selectAudio(ActionSelectAudio(track: sound.id))) }
            chooseLine(choice.text)
        }

        func press(_ word: Word) -> Bool {
            if panel.open {
                panel.press(word, shell, seen, side.tracks, chooseLine)
                return true
            }
            // Back leaves the film. On Apple TV an unused Back leaves the app, so the page acts itself.
            if word == .back {
                leave()
                return true
            }
            if asking {
                switch word {
                case .left: stay = 0
                case .right: stay = 1
                case .ok: if stay == 0 { goNext() } else { leave() }
                default: break
                }
                return true
            }
            let wasHidden = !shown
            awake += 1
            if failed { return true }
            if word == .ok && zone == .bar && wasHidden {
                if let part { shell.act(.seek(ActionSeek(toMs: part.seekToMs ?? part.endMs))) } else if closing != nil { goNext() } else { toggle() }
                return true
            }
            switch zone {
            case .bar:
                switch word {
                case .ok: toggle()
                case .left: seek(-seekStep)
                case .right: seek(seekStep)
                case .up: if !wasHidden { zone = paused ? .rate : .top }
                case .down:
                    if !wasHidden && !choices.isEmpty {
                        zone = .tracks
                        track = max(0, choices.firstIndex { $0.ticked } ?? 0)
                    }
                default: break
                }
            case .tracks:
                switch word {
                case .left: if track > 0 { track -= 1 }
                case .right: if track < choices.count - 1 { track += 1 }
                case .up: zone = .bar
                case .ok: if let choice = choices[safe: track] { choose(choice) }
                default: break
                }
            case .rate:
                switch word {
                case .left: if rate > 0 { rate -= 1 }
                case .right: if rate < rateChoices.count - 1 { rate += 1 }
                case .up: zone = .top
                case .down: zone = .bar
                case .ok:
                    if let title = play?.title {
                        let asked = rateChoices[rate].rating
                        rated = rated == asked ? nil : asked
                        shell.act(.command(ActionCommand(command: .rate(CommandRate(rating: rated, title: title)))))
                    }
                default: break
                }
            case .top:
                switch word {
                case .left: if top > 0 { top -= 1 }
                case .right: if top < controls.count - 1 { top += 1 }
                case .down: zone = paused ? .rate : .bar
                case .ok:
                    switch controls[top] {
                    case .back: leave()
                    case .restart: shell.act(.seek(ActionSeek(toMs: 0)))
                    case .next: goNext()
                    }
                default: break
                }
            }
            return true
        }

        // What the remote is on. The controls rise over the picture and fall away again after three
        // seconds, and the picture itself takes no focus, so with the controls down the bar is
        // where Ok lands: that is what is said. This screen has no system control on it at all, so
        // without this a person using a screen reader hears nothing from the moment play starts.
        func hereIs() -> String {
            if asking { return seen.say(stay == 0 ? "player.still.yes" : "player.still.no") }
            if failed { return seen.notice ?? seen.say("error.no_source") }
            if panel.open { return "" }
            switch zone {
            case .top: return controls[safe: top].map { controlWord($0, seen) } ?? ""
            case .rate: return seen.say(rateChoices[safe: rate]?.word ?? "")
            case .tracks: return choices[safe: track]?.word ?? ""
            case .bar: return sayBoth(
                seen.say(paused ? "common.play" : "say.pause"),
                seen.lengthMs > 0
                    ? seen.say("say.position")
                        .fill("at", clock(aim ?? seen.positionMs))
                        .fill("length", clock(seen.lengthMs))
                    : nil
            )
            }
        }

        return ZStack(alignment: .topLeading) {
            Color.black
            // The film. Decoration: it is the thing being watched, not a thing to be read, and it
            // carries no words of its own at any moment. The subtitles are drawn by the player
            // itself and reach no accessibility tree on this surface - see the report's open items.
            Picture(shell: shell).decoration()
            switch play?.state {
            case nil, .resolving?, .starting?, .stalled?, .switching?:
                ProgressView().progressViewStyle(.circular).tint(.white).scaleEffect(2)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            default:
                EmptyView()
            }
            if failed {
                Text(seen.notice ?? seen.say("error.no_source"))
                    .brand(40, 700).foregroundStyle(.white).multilineTextAlignment(.center)
                    .frame(width: 1100.ref)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if shown {
                shade
                TopIcons(controls: controls, focus: zone == .top ? top : -1, seen: seen)
                // What is playing: the title's name, and which episode under it. Read as one thing,
                // because the two lines are one answer to one question and the second is a
                // shorthand - "S2: E4" - that means nothing read on its own.
                VStack(alignment: .trailing, spacing: 0) {
                    Text(detail?.name ?? "").brand(34, 700).foregroundStyle(.white)
                    if let episode = play?.episode {
                        Text("S\(episode.season): E\(episode.episode)").brand(30).foregroundStyle(.white.opacity(0.75))
                    }
                }
                .accessibilityElement(children: .combine)
                .accessibilityAddTraits(.isHeader)
                .padding(.top, 96.ref)
                .padding(.trailing, 124.ref)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
                if paused { RatePills(focus: zone == .rate ? rate : -1, chosen: rated, seen: seen) }
                Bar(seen: seen, at: aim ?? seen.positionMs, playing: !paused, on: zone == .bar)
                TrackStrip(
                    choices: choices, focus: zone == .tracks ? track : -1,
                    chosenWord: seen.say("say.chosen")
                )
            }
            if !failed {
                if let age { AgeCard(label: age) }
                let moment: String? = part?.kind == .intro ? seen.say("player.skip_intro")
                    : part != nil ? seen.say("player.skip_recap")
                    : closing != nil ? seen.say("player.next") : nil
                if let moment { MomentPill(word: moment, on: !shown, raised: shown) }
            }
            SideLine(words: side, shell: shell, raised: shown)
            if panel.open { TrackPanel(state: panel, seen: seen, files: side.tracks) }
            if asking { StillAsking(seen: seen, focus: stay) }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote(says: hereIs()) { press($0) }
        .task(id: "\(awake) \(paused)") {
            shown = true
            guard !paused else { return }
            try? await Task.sleep(nanoseconds: UInt64(shownFor * 1e9))
            if Task.isCancelled { return }
            shown = false
            zone = .bar
        }
        .onChange(of: shown, initial: true) { _, now in shell.player.lift(now ? wordsHigh : wordsLow) }
        .task(id: play?.title) {
            guard let key = play?.title else { return }
            if let have = seen.detail, have.title == key { detail = have } else {
                detail = await shell.read("/v1/titles/" + key.rawValue.replacingOccurrences(of: ":", with: "/"))
                    .flatMap { try? JSONDecoder().decode(TitleDetail.self, from: Data($0.utf8)) }
            }
            rated = detail?.rating
        }
        .task {
            kept = await shell.read("/v1/profiles/me/settings")
                .flatMap { try? JSONDecoder().decode(ProfileSettings.self, from: Data($0.utf8)) }
        }
        .onChange(of: kept) { _, now in
            let look = now?.subtitleStyle ?? [:]
            shell.player.wordsLook(size: Int(look["size"] ?? "") ?? 1, color: look["color"] ?? "#ffffff", background: look["background"] ?? "none")
        }
        // A language the stream lacks may be a file on the server.
        .task(id: "\(play?.title.rawValue ?? "") \(play?.episode?.episode ?? 0) \(seen.lengthMs > 0) \(kept != nil)") {
            guard let key = play?.title else { return }
            let episode = play?.episode.map { "\($0.season):\($0.episode)" }
            if side.tracks.isEmpty { await side.load(shell, key.rawValue, episode) }
            guard let wanted = kept?.subtitleLanguage else { return }
            let inStream = seen.text.contains { $0.lang == wanted }
            if !inStream && side.chosen == nil && side.has(wanted) && seen.lengthMs > 0 {
                await side.pick(shell, wanted, seen.lengthMs)
            }
        }
        // When the next episode is starting, the state moves on and this never reaches the back.
        .task(id: "\(play?.state.rawValue ?? "") \(asking)") {
            guard play?.state == .ended && !asking else { return }
            try? await Task.sleep(nanoseconds: UInt64(endedWait * 1e9))
            if !Task.isCancelled { shell.back() }
        }
        .task(id: aim) {
            guard let target = aim else { return }
            try? await Task.sleep(nanoseconds: UInt64(seekSettle * 1e9))
            if Task.isCancelled { return }
            shell.act(.seek(ActionSeek(toMs: Int(target))))
            try? await Task.sleep(nanoseconds: UInt64(seekCatchUp * 1e9))
            if !Task.isCancelled { aim = nil }
        }
    }
}
