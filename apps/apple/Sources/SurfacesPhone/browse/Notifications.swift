import Contract
import Ports
import SurfacesShared
import SwiftUI

/// The phone notifications feed: unread count in TopBar, notifications feed with kind tags, title poster, title
/// name, localized date and unread indicator dot. Tapping marks read and navigates to title.
/// A spinner's words, or the spinner gone from the tree. The same answer `Art`, `Icon` and `Face`
/// give: words a person hears, or nothing at all, never an element that is reached and silent. Of
/// the two silences, a reader that skips the ring is better than one that stops on it and says
/// nothing - and `words[...]` is empty for as long as a run has not read its language yet, which is
/// the state every run starts in, whatever `strings/` holds.
private struct Turning: ViewModifier {
    let words: String

    func body(content: Content) -> some View {
        if words.isEmpty {
            content.accessibilityHidden(true)
        } else {
            content.accessibilityLabel(words)
        }
    }
}

public struct NotificationsView: View {
    let shell: Shell
    let seen: Seen

    @State private var items: [NoticeItem] = []
    @State private var cursor: String? = nil
    @State private var hasMore: Bool = false
    @State private var loading: Bool = true
    @State private var loadingMore: Bool = false

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        let words = seen.words

        ZStack(alignment: .top) {
            Back.ignoresSafeArea()

            VStack(spacing: 0) {
                // TopBar with back button and screen title
                HStack(spacing: 0) {
                    // An arrow and no text: the one control in the bar that needs words.
                    Button {
                        shell.back()
                    } label: {
                        Icon("icon_back", 24, tint: .white)
                            .frame(width: 48, height: 48)
                    }
                    .buttonStyle(.plain)
                    .padding(.leading, 4)
                    .says(words["common.back"] ?? "")

                    Text(words["notifications.title"] ?? "Notifications")
                        .brand(22, 700)
                        .foregroundStyle(.white)
                        .lineLimit(1)
                        .accessibilityAddTraits(.isHeader)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.leading, 8)
                }
                .frame(height: 52)
                .background(Back)

                if loading && items.isEmpty {
                    Spacer()
                    // A turning ring, and the only thing on the page while it turns. Measured on
                    // iPhone 17 on 2026-11-12: a bare `ProgressView` reaches the tree as an element
                    // with the `updatesFrequently` trait and no label at all, so a reader stops on
                    // it and says nothing - and a page that is loading becomes indistinguishable
                    // from a page that is empty, which is the one question the ring exists to
                    // answer. `say.loading` landed in all eighteen languages on 2026-11-12, so the
                    // ring reads "Loading" in English and is silent only in the moment before a run
                    // has read its words.
                    ProgressView()
                        .tint(Accent)
                        .modifier(Turning(words: words["say.loading"] ?? ""))
                    Spacer()
                } else if items.isEmpty {
                    Spacer()
                    // An empty page: the bell is decoration (`Icon` marks itself) and the line under
                    // it says the whole of it.
                    VStack(spacing: 16) {
                        Icon("icon_bell", 48, tint: Muted)
                        Text(words["notifications.empty"] ?? "No notifications yet.")
                            .brand(16, 400)
                            .foregroundStyle(Muted)
                    }
                    Spacer()
                } else {
                    ScrollView(.vertical, showsIndicators: false) {
                        LazyVStack(spacing: 0) {
                            ForEach(items, id: \.id) { item in
                                notificationRow(item, words: words)
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        if item.unread {
                                            shell.send("POST", "/v1/notifications/read", "{\"up_to\":\"\(item.id)\"}") { _, _ in
                                                shell.refreshNotifications()
                                            }
                                            if let index = items.firstIndex(where: { $0.id == item.id }) {
                                                items[index].unread = false
                                            }
                                        }
                                        shell.go(titleAddress(item.tile.title))
                                    }
                                    .onAppear {
                                        if item.id == items.last?.id && hasMore && !loadingMore, let c = cursor {
                                            loadMore(c)
                                        }
                                    }
                            }
                            if loadingMore {
                                ProgressView()
                                    .tint(Accent)
                                    .padding(16)
                            }
                        }
                        .padding(.vertical, 8)
                    }
                }
            }
        }
        .task {
            loadInitial()
        }
    }

    private func loadInitial() {
        shell.ask("/v1/notifications") { status, text in
            Task { @MainActor in
                loading = false
                guard (200...299).contains(status),
                      let parsed = try? JSONDecoder().decode(Notifications.self, from: Data(text.utf8)) else { return }
                self.cursor = parsed.cursor
                self.hasMore = parsed.hasMore
                self.items = parsed.items
            }
        }
    }

    private func loadMore(_ c: String) {
        loadingMore = true
        shell.ask("/v1/notifications?cursor=\(c)") { status, text in
            Task { @MainActor in
                loadingMore = false
                guard (200...299).contains(status),
                      let parsed = try? JSONDecoder().decode(Notifications.self, from: Data(text.utf8)) else { return }
                self.cursor = parsed.cursor
                self.hasMore = parsed.hasMore
                self.items.append(contentsOf: parsed.items)
            }
        }
    }

    @ViewBuilder
    private func notificationRow(_ item: NoticeItem, words: [String: String]) -> some View {
        let poster = item.tile.images?.first(where: { $0.role == .poster })
        let kindLabel = labelFor(item.kind, words: words)

        HStack(spacing: 16) {
            // Poster thumbnail
            ZStack {
                Plate
                // The one shared view, so this row's poster takes the same path as every other
                // picture in the app: the port, the port's cache, and the plate underneath for as
                // long as there is nothing to draw over it.
                if let urlString = poster?.url {
                    Art(shell.pictures, urlString)
                }
            }
            .frame(width: 72, height: 104)
            .clipShape(RoundedRectangle(cornerRadius: Corner.card))

            // Details
            VStack(alignment: .leading, spacing: 4) {
                if !kindLabel.isEmpty {
                    let isAccent = item.kind == .reminderLive || item.kind == .newArrival
                    Text(kindLabel)
                        .brand(11, 600)
                        .foregroundStyle(.white)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(isAccent ? Accent : Plate, in: RoundedRectangle(cornerRadius: Corner.chip))
                        .padding(.bottom, 2)
                }

                Text(item.tile.name)
                    .brand(16, 700)
                    .foregroundStyle(.white)
                    .lineLimit(2)

                Text(item.date.rawValue)
                    .brand(13, 400)
                    .foregroundStyle(Muted)
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            // Unread dot
            if item.unread {
                Circle()
                    .fill(Accent)
                    .frame(width: 8, height: 8)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        // The row answers a tap and draws four things, three of them words: what kind of notice it
        // is, the title's name and the date. Those are gathered. The fourth is an eight point dot
        // that means the notice has not been read, and it is the one thing here a person listening
        // cannot see at all, so it is read as the row's state.
        .saysItsWords(state: item.unread ? (words["say.unread"] ?? "") : "")
    }

    private func labelFor(_ kind: NoticeKind, words: [String: String]) -> String {
        switch kind {
        case .coming: return words["notification.coming"] ?? "Coming Soon"
        case .reminderLive: return words["notification.reminder_live"] ?? "Now Available"
        case .newArrival: return words["notification.new_arrival"] ?? "New Arrival"
        case .newSeason: return words["notification.new_season"] ?? "New Season"
        case .newEpisode: return words["notification.new_episode"] ?? "New Episode"
        default: return ""
        }
    }
}
