import Contract
import Ports
import SurfacesShared
import SwiftUI
import UserNotifications

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
                    // Pinned 52 pt top bar with back arrow and cast/close buttons
                    HStack(spacing: 0) {
                        Button {
                            shell.back()
                        } label: {
                            Icon("icon_back", 24, tint: .white)
                                .frame(width: 48, height: 48)
                        }
                        .buttonStyle(.plain)
                        .padding(.leading, 4)

                        Spacer()

                        Button {
                            // cast action
                        } label: {
                            Icon("icon_cast", 24, tint: .white)
                                .frame(width: 44, height: 44)
                        }
                        .buttonStyle(.plain)

                        Button {
                            shell.back()
                        } label: {
                            CloseCross()
                                .frame(width: 44, height: 44)
                        }
                        .buttonStyle(.plain)
                        .padding(.trailing, 4)
                    }
                    .frame(height: 52)
                    .background(Back)

                    // Scrollable content
                    ScrollView(.vertical, showsIndicators: false) {
                        VStack(alignment: .leading, spacing: 0) {
                            // Top: 16:9 backdrop art
                            ZStack {
                                Plate
                                Art(wide(data.images))
                            }
                            .aspectRatio(16 / 9, contentMode: .fit)
                            .frame(maxWidth: .infinity)
                            .clipped()

                            // Body content with 10 pt margins
                            VStack(alignment: .leading, spacing: 10) {
                                // Title / logo
                                if let logoUrl = logo(data.images) {
                                    Art(logoUrl, fit: true)
                                        .frame(maxWidth: 260, maxHeight: 80, alignment: .leading)
                                } else {
                                    Text(data.name)
                                        .brand(24, 700)
                                        .foregroundStyle(.white)
                                        .lineLimit(2)
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
                                    .overlay(alignment: .top) {
                                        if rateMenuOpen {
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
                        Art(wide(episode.images))

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
    }
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
                        Art(tall(tile.images))
                    }
                    .aspectRatio(119 / 166.6, contentMode: .fill)
                    .clipShape(RoundedRectangle(cornerRadius: Corner.card))
                }
                .buttonStyle(.plain)
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
