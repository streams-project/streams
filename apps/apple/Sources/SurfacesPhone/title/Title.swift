import Contract
import Ports
import SurfacesShared
import SwiftUI
import UserNotifications

// A reminder, set for the morning a title arrives.
//
// Guarded by platform because `UNMutableNotificationContent`'s `title`, `body` and `sound` are not
// on tvOS, while `Package.swift:8` declares tvOS for the whole package - SwiftPM has no per-target
// platform list, so every target is offered to every platform the package names. Unguarded, this
// one file stopped the package compiling for tvOS at all, which is why no television test in this
// project had ever been run on a television (measured 2026-11-12: three errors, `'title' is
// unavailable in tvOS`, `'body'` and `'sound'`, all from the function below).
//
// The television keeps the two names and does nothing with them, rather than the call sites
// learning about platforms: a television does not set reminders, and no television screen reaches
// this file at all. Nothing about the phone changes.
#if os(tvOS)
private func scheduleNotification(titleKey: TitleKey, titleName: String, dateYmd: String) {}

private func cancelNotification(titleKey: TitleKey) {}
#else
private func scheduleNotification(titleKey: TitleKey, titleName: String, dateYmd: String) {
    let center = UNUserNotificationCenter.current()
    center.requestAuthorization(options: [.alert, .sound, .badge]) { granted, _ in
        guard granted else { return }
        let parts = dateYmd.split(separator: "-").compactMap { Int($0) }
        guard parts.count == 3 else { return }
        var dateComponents = DateComponents()
        dateComponents.year = parts[0]
        dateComponents.month = parts[1]
        dateComponents.day = parts[2]
        dateComponents.hour = 9
        dateComponents.minute = 0

        let trigger = UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: false)
        let content = UNMutableNotificationContent()
        content.title = titleName
        content.body = "Now available to watch"
        content.sound = .default

        let request = UNNotificationRequest(identifier: titleKey.rawValue, content: content, trigger: trigger)
        center.add(request)
    }
}

private func cancelNotification(titleKey: TitleKey) {
    UNUserNotificationCenter.current().removePendingNotificationRequests(withIdentifiers: [titleKey.rawValue])
}
#endif

private enum TitleTab {
    case episodes
    case moreLikeThis
}

extension TitleDetail {
    var isSeries: Bool { title.rawValue.hasPrefix("tv:") }
    var path: String { title.rawValue.replacingOccurrences(of: ":", with: "/") }
}

private func playTitle(_ shell: Shell, _ data: TitleDetail) {
    if !data.isSeries {
        shell.go(Address.watch + data.path)
        return
    }
    if let next = data.resume?.episode {
        shell.go("\(Address.watch)\(data.path)/\(next.season)/\(next.episode)")
        return
    }
    let number = data.seasons?.first?.number ?? 1
    Task {
        let season = await shell.read("/v1/titles/\(data.path)/seasons/\(number)")
            .flatMap { try? JSONDecoder().decode(Season.self, from: Data($0.utf8)) }
        let first = season?.episodes.first?.number ?? 1
        shell.go("\(Address.watch)\(data.path)/\(number)/\(first)")
    }
}

/// The phone title page: 16:9 art under a 52 pt bar, title or logo, facts, white play button,
/// expandable synopsis, cast/directors line, action buttons row, divider, and Episodes / More Like
/// This tabs.
public struct Title: View {
    let shell: Shell
    let seen: Seen

    @State private var selectedTab: TitleTab = .episodes
    @State private var saved: Bool = false
    @State private var reminded: Bool = false
    @State private var rating: Rating? = nil
    @State private var rateMenuOpen: Bool = false
    @State private var synopsisExpanded: Bool = false
    @State private var castExpanded: Bool = false

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        if let data = seen.detail, data.title == seen.route?.title {
            let isSeries = data.isSeries
            let words = seen.words

            ZStack(alignment: .top) {
                Back.ignoresSafeArea()

                VStack(spacing: 0) {
                    // Pinned 52 pt top bar with back arrow and cast/close buttons. All three draw a
                    // picture and no text, so all three need words of their own: read as they stood
                    // the first two were the names of their own asset files and the third, which is
                    // two lines on a canvas, was a stop with nothing to read at all.
                    HStack(spacing: 0) {
                        Button {
                            shell.back()
                        } label: {
                            Icon("icon_back", 24, tint: .white)
                                .frame(width: 48, height: 48)
                        }
                        .buttonStyle(.plain)
                        .padding(.leading, 4)
                        .says(words["common.back"] ?? "")

                        Spacer()

                        Button {
                            // cast action
                        } label: {
                            Icon("icon_cast", 24, tint: .white)
                                .frame(width: 44, height: 44)
                        }
                        .buttonStyle(.plain)
                        .says(words["say.cast"] ?? "")

                        Button {
                            shell.back()
                        } label: {
                            CloseCross()
                                .frame(width: 44, height: 44)
                        }
                        .buttonStyle(.plain)
                        .padding(.trailing, 4)
                        .says(words["say.close"] ?? "")
                    }
                    .frame(height: 52)
                    .background(Back)

                    // Scrollable content
                    ScrollView(.vertical, showsIndicators: false) {
                        VStack(alignment: .leading, spacing: 0) {
                            // Top: 16:9 backdrop art. Decoration: the title is named, described and
                            // listed below it, so a reader that announced the picture as well would
                            // say the same thing twice. `Art` marks itself.
                            ZStack {
                                Plate
                                Art(shell.pictures, wide(data.images))
                            }
                            .aspectRatio(16 / 9, contentMode: .fit)
                            .frame(maxWidth: .infinity)
                            .clipped()

                            // Body content with 10 pt margins
                            VStack(alignment: .leading, spacing: 10) {
                                // Title / logo. The treatment is a picture of the title's name, so where
                                // one is drawn it is the only thing naming the title and carries the
                                // words; where there is none the name is drawn as text instead.
                                // Either way it is the page's heading, so a reader can jump to it.
                                if let logoUrl = logo(data.images) {
                                    Art(shell.pictures, logoUrl, fit: true, says: data.name)
                                        .frame(maxWidth: 260, maxHeight: 80, alignment: .leading)
                                        .accessibilityAddTraits(.isHeader)
                                } else {
                                    Text(data.name)
                                        .brand(24, 700)
                                        .foregroundStyle(.white)
                                        .lineLimit(2)
                                        .accessibilityAddTraits(.isHeader)
                                }

                                // Meta line: year, age badge, seasons/length, HD badge
                                HStack(spacing: 8) {
                                    if let year = data.year {
                                        Text(number(year))
                                            .brand(14, 400)
                                            .foregroundStyle(Muted)
                                    }

                                    if let age = data.age {
                                        Text(age.label)
                                            .brand(12, 500)
                                            .foregroundStyle(.white.opacity(0.9))
                                            .padding(.horizontal, 6)
                                            .padding(.vertical, 2)
                                            .background(Color.white.opacity(0.15), in: RoundedRectangle(cornerRadius: Corner.field))
                                    }

                                    let seasonsCount = data.seasons?.count ?? 0
                                    let minutes = data.runtimeMin
                                    if seasonsCount > 0 {
                                        Text(words.counted("title.season", seasonsCount))
                                            .brand(14, 400)
                                            .foregroundStyle(Muted)
                                    } else if let minutes, minutes > 0 {
                                        Text(formatLength(words, minutes))
                                            .brand(14, 400)
                                            .foregroundStyle(Muted)
                                    }

                                    Text("HD")
                                        .brand(10, 700)
                                        .foregroundStyle(.white.opacity(0.9))
                                        .padding(.horizontal, 5)
                                        .padding(.vertical, 1)
                                        .overlay(RoundedRectangle(cornerRadius: 3).stroke(Color.white.opacity(0.4), lineWidth: 1))
                                }

                                if let availableFrom = data.availableFrom {
                                    let dateText = availableFrom.rawValue
                                    let comingLabel = words["gate.coming"]?.replacingOccurrences(of: "{date}", with: dateText) ?? "Coming on \(dateText)"
                                    Text(comingLabel)
                                        .brand(15, 600)
                                        .foregroundStyle(.white)
                                        .padding(.top, 10)

                                    Button {
                                        let next = !reminded
                                        reminded = next
                                        shell.act(.command(ActionCommand(command: .remind(CommandRemind(on: next, title: data.title)))))
                                        if next {
                                            scheduleNotification(titleKey: data.title, titleName: data.name, dateYmd: dateText)
                                        } else {
                                            cancelNotification(titleKey: data.title)
                                        }
                                    } label: {
                                        HStack(spacing: 8) {
                                            Icon(reminded ? "icon_check" : "icon_bell", 20, tint: .white)
                                            Text(words[reminded ? "remind.set" : "remind.me"] ?? (reminded ? "Reminder Set" : "Remind Me"))
                                                .brand(16, 700)
                                                .foregroundStyle(.white)
                                        }
                                        .frame(maxWidth: .infinity)
                                        .frame(height: 48)
                                        .background(reminded ? Raised : Accent, in: RoundedRectangle(cornerRadius: min(Corner.button, 24)))
                                    }
                                    .buttonStyle(.plain)
                                    .padding(.top, 8)
                                    // The button draws "Remind Me" or "Reminder Set", and which of
                                    // the two it says is the whole of what a person needs: the tick
                                    // that replaces the bell says the same thing in a picture.
                                    .saysItsWords()
                                } else {
                                    // Play button: white, full width (10 pt margins), 48 pt high, Corner.button radius
                                    Button {
                                        playTitle(shell, data)
                                    } label: {
                                        HStack(spacing: 8) {
                                            Icon("icon_play", 20, tint: .black)
                                            Text(seen.say("common.play").isEmpty ? "Play" : seen.say("common.play"))
                                                .brand(16, 700)
                                                .foregroundStyle(.black)
                                        }
                                        .frame(maxWidth: .infinity)
                                        .frame(height: 48)
                                        .background(Color.white, in: RoundedRectangle(cornerRadius: min(Corner.button, 24)))
                                    }
                                    .buttonStyle(.plain)
                                    // The word is drawn in the button and the triangle beside it
                                    // repeats it, so the button gathers what it draws.
                                    .saysItsWords()
                                }

                                // Synopsis: 16 sp, expandable (3 lines with "more")
                                if let overview = data.overview, !overview.isEmpty {
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(overview)
                                            .brand(16, 400)
                                            .foregroundStyle(.white)
                                            .lineLimit(synopsisExpanded ? nil : 3)

                                        if !synopsisExpanded && overview.count > 120 {
                                            Text("… more")
                                                .brand(14, 700)
                                                .foregroundStyle(Color(red: 231 / 255, green: 231 / 255, blue: 231 / 255))
                                                .padding(.top, 2)
                                        }
                                    }
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        synopsisExpanded.toggle()
                                    }
                                    // The synopsis is prose and is read as it stands, but the tap
                                    // that opens it out is drawn as "… more" and nothing says the
                                    // whole block answers a tap. So the block is one control whose
                                    // words are the synopsis and whose state is what a tap does -
                                    // and when there is nothing more to show, it is not a control.
                                    .modifier(TapsForMore(
                                        label: overview,
                                        state: overview.count > 120
                                            ? (words[synopsisExpanded ? "say.show_less" : "say.show_more"] ?? "")
                                            : ""
                                    ))
                                }

                                // Cast / director line: muted color with "more" expandable
                                let genres = data.genres ?? []
                                if !genres.isEmpty {
                                    let displayGenres = castExpanded ? genres : Array(genres.prefix(3))
                                    HStack(spacing: 4) {
                                        Text(displayGenres.joined(separator: ", "))
                                            .brand(13, 400)
                                            .foregroundStyle(Muted)

                                        if !castExpanded && genres.count > 3 {
                                            Text("… more")
                                                .brand(13, 500)
                                                .foregroundStyle(Color(red: 231 / 255, green: 231 / 255, blue: 231 / 255))
                                        }
                                    }
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        if genres.count > 3 { castExpanded.toggle() }
                                    }
                                    // The same: a line of genres that answers a tap only when there
                                    // are more than three of them to show.
                                    .modifier(TapsForMore(
                                        label: displayGenres.joined(separator: ", "),
                                        state: genres.count > 3
                                            ? (words[castExpanded ? "say.show_less" : "say.show_more"] ?? "")
                                            : ""
                                    ))
                                }

                                // Action buttons: My List, Rate, Share in a row (about 96 pt wide, 8 pt apart)
                                HStack(alignment: .top, spacing: 8) {
                                    // My List
                                    Button {
                                        saved.toggle()
                                        shell.act(.command(ActionCommand(command: .list(CommandList(inList: saved, title: data.title)))))
                                    } label: {
                                        VStack(spacing: 6) {
                                            Icon(saved ? "icon_check" : "icon_plus", 24, tint: .white)
                                            Text(seen.say(saved ? "list.added" : "list.add").isEmpty ? "My List" : seen.say(saved ? "list.added" : "list.add"))
                                                .brand(12, 400)
                                                .foregroundStyle(Muted)
                                                .lineLimit(1)
                                        }
                                        .frame(width: 96)
                                        .padding(.vertical, 4)
                                    }
                                    .buttonStyle(.plain)
                                    // "My List" or "In My List" is drawn under the icon, and which
                                    // of the two it says is what tells a person whether the title is
                                    // saved. The tick that replaces the plus says it in a picture.
                                    .saysItsWords()

                                    // Rate
                                    VStack(spacing: 6) {
                                        let rateIcon = rating == .down ? "icon_thumb_down_filled"
                                            : rating == .up ? "icon_thumb_up_filled"
                                            : rating == .love ? "icon_love_filled"
                                            : "icon_thumb_up"
                                        Icon(rateIcon, 24, tint: .white)
                                        Text("Rate")
                                            .brand(12, 400)
                                            .foregroundStyle(Muted)
                                            .lineLimit(1)
                                    }
                                    .frame(width: 96)
                                    .padding(.vertical, 4)
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        rateMenuOpen.toggle()
                                    }
                                    // The control draws its own word, and the thumb above it changes
                                    // to show the rating already given - which says nothing a person
                                    // listening can hear, so the rating is read as the state.
                                    .saysItsWords(state: rateWord(rating, words))
                                    .overlay(alignment: .top) {
                                        if rateMenuOpen {
                                            // Three thumbs with no words anywhere near them: the
                                            // clearest case on this page of a control a person
                                            // cannot hear. Each is given its own name, and the one
                                            // already chosen says so, because the only thing
                                            // marking it is that its picture is filled in.
                                            HStack(spacing: 4) {
                                                Button {
                                                    let next: Rating? = rating == .down ? nil : .down
                                                    rating = next
                                                    shell.act(.command(ActionCommand(command: .rate(CommandRate(rating: next, title: data.title)))))
                                                    rateMenuOpen = false
                                                } label: {
                                                    Icon(rating == .down ? "icon_thumb_down_filled" : "icon_thumb_down", 22, tint: .white)
                                                        .frame(width: 36, height: 36)
                                                }
                                                .buttonStyle(.plain)
                                                .says(
                                                    words["player.down"] ?? "",
                                                    state: rating == .down ? (words["say.chosen"] ?? "") : ""
                                                )

                                                Button {
                                                    let next: Rating? = rating == .up ? nil : .up
                                                    rating = next
                                                    shell.act(.command(ActionCommand(command: .rate(CommandRate(rating: next, title: data.title)))))
                                                    rateMenuOpen = false
                                                } label: {
                                                    Icon(rating == .up ? "icon_thumb_up_filled" : "icon_thumb_up", 22, tint: .white)
                                                        .frame(width: 36, height: 36)
                                                }
                                                .buttonStyle(.plain)
                                                .says(
                                                    words["player.up"] ?? "",
                                                    state: rating == .up ? (words["say.chosen"] ?? "") : ""
                                                )

                                                Button {
                                                    let next: Rating? = rating == .love ? nil : .love
                                                    rating = next
                                                    shell.act(.command(ActionCommand(command: .rate(CommandRate(rating: next, title: data.title)))))
                                                    rateMenuOpen = false
                                                } label: {
                                                    Icon(rating == .love ? "icon_love_filled" : "icon_love", 22, tint: .white)
                                                        .frame(width: 36, height: 36)
                                                }
                                                .buttonStyle(.plain)
                                                .says(
                                                    words["player.love"] ?? "",
                                                    state: rating == .love ? (words["say.chosen"] ?? "") : ""
                                                )
                                            }
                                            .padding(.horizontal, 8)
                                            .padding(.vertical, 4)
                                            .background(Raised, in: RoundedRectangle(cornerRadius: min(Corner.button, 20)))
                                            .offset(y: -44)
                                        }
                                    }

                                    // Share
                                    Button {
                                        // share
                                    } label: {
                                        VStack(spacing: 6) {
                                            Icon("icon_share", 24, tint: .white)
                                            Text("Share")
                                                .brand(12, 400)
                                                .foregroundStyle(Muted)
                                                .lineLimit(1)
                                        }
                                        .frame(width: 96)
                                        .padding(.vertical, 4)
                                    }
                                    .buttonStyle(.plain)
                                    // Draws its own word under the icon.
                                    .saysItsWords()
                                }
                                .padding(.top, 6)
                            }
                            .padding(.horizontal, 10)
                            .padding(.top, 10)

                            // Divider
                            Rectangle()
                                .fill(Color.white.opacity(0.15))
                                .frame(height: 1)
                                .padding(.top, 16)

                            // Tabs: "Episodes" (for series) and "More Like This" (48 pt boxes, 4 pt accent line on chosen tab)
                            HStack(spacing: 0) {
                                if isSeries {
                                    let epChosen = selectedTab == .episodes
                                    Button {
                                        selectedTab = .episodes
                                    } label: {
                                        ZStack(alignment: .top) {
                                            if epChosen {
                                                Accent.frame(height: 4)
                                            }
                                            Text(seen.say("title.episodes_more").isEmpty ? "Episodes" : seen.say("title.episodes_more"))
                                                .brand(14, 700)
                                                .foregroundStyle(epChosen ? .white : Muted)
                                                .frame(maxHeight: .infinity)
                                        }
                                        .frame(height: 48)
                                        .padding(.horizontal, 12)
                                    }
                                    .buttonStyle(.plain)
                                    // The tab draws its own word. The chosen one is drawn whiter
                                    // with a four point accent line over it, and neither of those
                                    // is something a person listening can see.
                                    .saysItsWords(on: epChosen)
                                }

                                let moreChosen = selectedTab == .moreLikeThis
                                Button {
                                    selectedTab = .moreLikeThis
                                } label: {
                                    ZStack(alignment: .top) {
                                        if moreChosen {
                                            Accent.frame(height: 4)
                                        }
                                        Text(seen.say("title.more_like").isEmpty ? "More Like This" : seen.say("title.more_like"))
                                            .brand(14, 700)
                                            .foregroundStyle(moreChosen ? .white : Muted)
                                            .frame(maxHeight: .infinity)
                                    }
                                    .frame(height: 48)
                                    .padding(.horizontal, 12)
                                }
                                .buttonStyle(.plain)
                                .saysItsWords(on: moreChosen)

                                Spacer()
                            }
                            .padding(.horizontal, 10)

                            // Tab content
                            if isSeries && selectedTab == .episodes {
                                PhoneEpisodes(shell: shell, seen: seen, data: data)
                            } else {
                                PhoneMoreLike(shell: shell, data: data)
                            }

                            Spacer(minLength: 96)
                        }
                    }
                }
            }
            .onAppear {
                saved = data.inList == true
                reminded = data.reminderSet == true
                rating = data.rating
                selectedTab = isSeries ? .episodes : .moreLikeThis
            }
        }
    }
}

/// What a rating already given is heard as, after the Rate control's own word. The three names are
/// the player's, so the phone and the television call a rating the same thing.
///
/// `Rating` is a `RawRepresentable` struct on the wire, not a closed set, so this is a lookup and
/// not a switch: a rating the contract gains later reads as nothing here rather than failing to
/// build, and the gap shows up as a silent state in the tests.
@MainActor
private func rateWord(_ rating: Rating?, _ words: [String: String]) -> String {
    guard let rating else { return "" }
    let keys: [Rating: String] = [.down: "player.down", .up: "player.up", .love: "player.love"]
    return keys[rating].flatMap { words[$0] } ?? ""
}

/// A block of prose that opens out when it is tapped, and is only prose when there is nothing more
/// to show. `state` is what a tap would do, from `strings/` (`say.show_more`, `say.show_less`); an
/// empty one leaves the block as text a reader walks rather than a button that does nothing.
private struct TapsForMore: ViewModifier {
    let label: String
    let state: String

    func body(content: Content) -> some View {
        if state.isEmpty {
            content
        } else {
            content.says(label, state: state)
        }
    }
}

private struct CloseCross: View {
    var body: some View {
        Canvas { context, size in
            let stroke = StrokeStyle(lineWidth: 2, lineCap: .round)
            context.stroke(Path { p in
                p.move(to: CGPoint(x: 4, y: 4))
                p.addLine(to: CGPoint(x: size.width - 4, y: size.height - 4))
                p.move(to: CGPoint(x: 4, y: size.height - 4))
                p.addLine(to: CGPoint(x: size.width - 4, y: 4))
            }, with: .color(.white), style: stroke)
        }
        .frame(width: 16, height: 16)
    }
}

/// Episodes tab: Season chip opening full-screen season list, and episode rows.
struct PhoneEpisodes: View {
    let shell: Shell
    let seen: Seen
    let data: TitleDetail

    @State private var chosenSeasonIndex: Int = 0
    @State private var showSeasonPicker: Bool = false
    @State private var episodes: [Episode] = []

    var body: some View {
        let seasons = data.seasons ?? []
        let currentSeason = seasons[safe: chosenSeasonIndex]
        let seasonNumber = currentSeason?.number ?? 1
        let words = seen.words

        VStack(alignment: .leading, spacing: 12) {
            if seasons.count > 1 {
                Button {
                    showSeasonPicker = true
                } label: {
                    HStack(spacing: 6) {
                        Text(seen.say("title.season.number").fill("n", seasonNumber))
                            .brand(14, 700)
                            .foregroundStyle(.white)
                        Icon("icon_caret_down", 16, tint: .white)
                    }
                    .padding(.horizontal, 14)
                    .frame(height: 36)
                    .background(Color(red: 33 / 255, green: 33 / 255, blue: 33 / 255), in: RoundedRectangle(cornerRadius: Corner.chip))
                }
                .buttonStyle(.plain)
                .padding(.top, 6)
                // The chip draws the season it is showing, and the caret beside it is the only
                // thing saying there are others to pick from - so the chip says what it opens.
                .says(
                    (words["say.season_pick"] ?? "").isEmpty
                        ? seen.say("title.season.number").fill("n", seasonNumber)
                        : (words["say.season_pick"] ?? ""),
                    state: seen.say("title.season.number").fill("n", seasonNumber)
                )
            } else if seasons.count == 1 {
                Text(seen.say("title.season.number").fill("n", seasonNumber))
                    .brand(14, 700)
                    .foregroundStyle(.white)
                    .padding(.top, 6)
            }

            VStack(spacing: 16) {
                ForEach(episodes, id: \.number) { episode in
                    PhoneEpisodeRow(episode: episode, seasonNumber: seasonNumber, dataPath: data.path, shell: shell)
                }
            }
        }
        .padding(.horizontal, 10)
        .task(id: "\(data.path)-\(seasonNumber)") {
            if let text = await shell.read("/v1/titles/\(data.path)/seasons/\(seasonNumber)"),
               let decoded = try? JSONDecoder().decode(Season.self, from: Data(text.utf8)) {
                episodes = decoded.episodes
            }
        }
        .onAppear {
            if let resumeSeason = data.resume?.episode?.season,
               let idx = seasons.firstIndex(where: { $0.number == resumeSeason }) {
                chosenSeasonIndex = idx
            }
        }
        .fullScreenCover(isPresented: $showSeasonPicker) {
            SeasonPickerSheet(
                seasons: seasons,
                chosenIndex: $chosenSeasonIndex,
                words: words,
                close: { showSeasonPicker = false }
            )
        }
    }
}

private struct PhoneEpisodeRow: View {
    let episode: Episode
    let seasonNumber: Int
    let dataPath: String
    let shell: Shell

    var body: some View {
        Button {
            shell.go("\(Address.watch)\(dataPath)/\(seasonNumber)/\(episode.number)")
        } label: {
            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .center, spacing: 12) {
                    // 130x80 pt still thumbnail
                    ZStack {
                        Plate
                        Art(shell.pictures, wide(episode.images))

                        // Play circle overlay
                        ZStack {
                            Circle()
                                .fill(Color.black.opacity(0.5))
                            Circle()
                                .stroke(Color.white.opacity(0.8), lineWidth: 1.5)
                            Icon("icon_play", 16, tint: .white)
                        }
                        .frame(width: 36, height: 36)

                        // Progress bar in accent blue on bottom edge
                        if let percent = episode.percent, percent > 0 {
                            VStack {
                                Spacer()
                                GeometryReader { geo in
                                    ZStack(alignment: .leading) {
                                        Color.black.opacity(0.6)
                                        Accent.frame(width: geo.size.width * CGFloat(min(max(percent, 0), 100)) / 100.0)
                                    }
                                }
                                .frame(height: 3)
                            }
                        }
                    }
                    .frame(width: 130, height: 80)
                    .clipShape(RoundedRectangle(cornerRadius: Corner.card))

                    // Episode number and name bold, runtime
                    VStack(alignment: .leading, spacing: 2) {
                        Text("\(episode.number). \(episode.name)")
                            .brand(14, 700)
                            .foregroundStyle(.white)
                            .lineLimit(2)
                            .multilineTextAlignment(.leading)

                        if let minutes = episode.runtimeMin, minutes > 0 {
                            Text(shell.seen.words.counted("title.length.minutes", minutes))
                                .brand(12, 400)
                                .foregroundStyle(Muted)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                // Synopsis below
                if let overview = episode.overview, !overview.isEmpty {
                    Text(overview)
                        .brand(13, 400)
                        .foregroundStyle(Muted)
                        .lineLimit(4)
                        .multilineTextAlignment(.leading)
                }
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        // The row draws the episode's number, its name, how long it runs and what it is about: all
        // of it words already, so the row gathers them. The still behind them and the play circle
        // over it are decoration (`Art` and `Icon` mark themselves). The bar across the bottom of
        // the still is how far through the episode the viewer is, and it is the one thing on the row
        // a person listening cannot see at all, so it is read as the row's state.
        .saysItsWords(state: watchedWord(episode.percent, shell.seen.words))
    }
}

/// How far through an episode the viewer is, in words, or nothing for one not started. Read after
/// the row's own words.
///
/// `say.watched_percent` - "{percent}% watched" - landed in all eighteen languages on 2026-11-12
/// and is what this reads. The key that already existed, `say.watched`, is "{title}, {percent}%
/// watched" and names the title as well: on a row that already draws the episode's number and its
/// name that would say the name twice. Emptying its `{title}` and trimming what was left was
/// rejected, and the Arabic is why: there `say.watched` ends in منه, "of it", masculine, while an
/// episode is الحلقة, feminine, so a trimmed pattern would have produced a sentence that does not
/// agree. The new key carries no pronoun at all. Cutting a translated pattern apart in code is
/// never the answer; a key of its own is.
@MainActor
private func watchedWord(_ percent: Int?, _ words: [String: String]) -> String {
    guard let percent, percent > 0 else { return "" }
    return (words["say.watched_percent"] ?? "").fill("percent", percent)
}

private struct SeasonPickerSheet: View {
    let seasons: [SeasonSummary]
    @Binding var chosenIndex: Int
    let words: [String: String]
    let close: () -> Void

    var body: some View {
        ZStack {
            Color.black.opacity(0.92).ignoresSafeArea()

            ScrollView(.vertical, showsIndicators: false) {
                VStack(spacing: 0) {
                    Spacer(minLength: 90)
                    ForEach(Array(seasons.enumerated()), id: \.element.number) { index, s in
                        let isChosen = index == chosenIndex
                        Button {
                            chosenIndex = index
                            close()
                        } label: {
                            Text((words["title.season.number"] ?? "").fill("n", s.number))
                                .brand(isChosen ? 22 : 18, isChosen ? 700 : 400)
                                .foregroundStyle(isChosen ? .white : Muted)
                                .frame(width: 248, height: 52)
                        }
                        .buttonStyle(.plain)
                        // Each line draws its own season. The one showing is drawn larger, bolder
                        // and whiter, which is three ways of saying the same thing and none of them
                        // audible, so the line says it is the chosen one.
                        .saysItsWords(on: isChosen)
                    }
                    Spacer(minLength: 110)
                }
                .frame(maxWidth: .infinity)
            }

            VStack {
                Spacer()
                Button(action: close) {
                    ZStack {
                        Circle()
                            .fill(Color.white)
                            .frame(width: 64, height: 64)
                        Canvas { context, size in
                            let stroke = StrokeStyle(lineWidth: 2.5, lineCap: .round)
                            context.stroke(Path { p in
                                p.move(to: CGPoint(x: 4, y: 4))
                                p.addLine(to: CGPoint(x: size.width - 4, y: size.height - 4))
                                p.move(to: CGPoint(x: 4, y: size.height - 4))
                                p.addLine(to: CGPoint(x: size.width - 4, y: 4))
                            }, with: .color(.black), style: stroke)
                        }
                        .frame(width: 20, height: 20)
                    }
                }
                .buttonStyle(.plain)
                .padding(.bottom, 20)
                // Two lines drawn on a canvas: nothing here for a reader to gather, and it is the
                // only way out of a sheet that fills the screen.
                .says(words["say.close"] ?? "")
            }
        }
    }
}

/// More Like This tab on phones: 3-column grid of tall tiles.
struct PhoneMoreLike: View {
    let shell: Shell
    let data: TitleDetail

    var body: some View {
        let tiles = data.moreLikeThis ?? []
        let columns = [
            GridItem(.flexible(), spacing: 8),
            GridItem(.flexible(), spacing: 8),
            GridItem(.flexible(), spacing: 8),
        ]

        LazyVGrid(columns: columns, spacing: 8) {
            ForEach(tiles, id: \.title.rawValue) { tile in
                Button {
                    shell.go(titleAddress(tile.title))
                } label: {
                    ZStack {
                        Plate
                        Art(shell.pictures, tall(tile.images))
                    }
                    .aspectRatio(119 / 166.6, contentMode: .fill)
                    .clipShape(RoundedRectangle(cornerRadius: Corner.card))
                }
                .buttonStyle(.plain)
                // A tile is a picture and nothing else. The title's name is the tile, and it comes
                // from the answer already in the viewer's language, so it needs no key.
                .says(tile.name)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 12)
    }
}

private extension Array {
    subscript(safe index: Int) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}
