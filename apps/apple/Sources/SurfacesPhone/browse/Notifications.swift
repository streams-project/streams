import Contract
import Ports
import SurfacesShared
import SwiftUI

/// The phone notifications feed: unread count in TopBar, notifications feed with kind tags, title poster, title
/// name, localized date and unread indicator dot. Tapping marks read and navigates to title.
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
                    Button {
                        shell.back()
                    } label: {
                        Icon("icon_back", 24, tint: .white)
                            .frame(width: 48, height: 48)
                    }
                    .buttonStyle(.plain)
                    .padding(.leading, 4)

                    Text(words["notifications.title"] ?? "Notifications")
                        .brand(22, 700)
                        .foregroundStyle(.white)
                        .lineLimit(1)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.leading, 8)
                }
                .frame(height: 52)
                .background(Back)

                if loading && items.isEmpty {
                    Spacer()
                    ProgressView()
                        .tint(Accent)
                    Spacer()
                } else if items.isEmpty {
                    Spacer()
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
                if let urlString = poster?.url, let url = URL(string: urlString) {
                    AsyncImage(url: url) { phase in
                        if let image = phase.image {
                            image.resizable().scaledToFill()
                        } else {
                            Plate
                        }
                    }
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
