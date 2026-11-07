import Contract
import Ports
import SwiftUI

public func facts(detail: TitleDetail, words: [String: String]) -> [String] {
    var list: [String] = []
    if let genre = detail.genres?.first {
        list.append(genre)
    }
    if let year = detail.year {
        list.append("\(year)")
    }
    let seasons = detail.seasons?.count ?? 0
    let minutes = detail.runtimeMin
    if seasons > 0 {
        let key = seasons == 1 ? "title.season.one" : "title.season.many"
        let word = words[key] ?? ""
        list.append(word.replacingOccurrences(of: "{n}", with: "\(seasons)"))
    } else if let m = minutes {
        list.append("\(m / 60)h \(m % 60)m")
    }
    if let age = detail.age {
        list.append(age.label)
    }
    return list
}

public struct HeroCard: View {
    public let lead: Tile
    public let button: Int
    public let words: [String: String]
    public let detail: TitleDetail?

    public init(lead: Tile, button: Int, words: [String: String], detail: TitleDetail? = nil) {
        self.lead = lead
        self.button = button
        self.words = words
        self.detail = detail
    }

    public var body: some View {
        ZStack(alignment: .bottomLeading) {
            // Wide Backdrop
            Art(wideImage(lead.images), contentMode: .fill)
                .frame(width: 1789.ref, height: 767.ref)
                .clipped()

            // Gradients
            LinearGradient(
                colors: [Color.black.opacity(0.85), Color.black.opacity(0.3), Color.clear],
                startPoint: .leading,
                endPoint: .trailing
            )
            .frame(width: 1100.ref)

            LinearGradient(
                colors: [Color.clear, Color.black.opacity(0.7)],
                startPoint: .center,
                endPoint: .bottom
            )

            // Content
            VStack(alignment: .leading, spacing: 18.ref) {
                // Title / Logo
                let logoUrl = (lead.images ?? []).first(where: { $0.role == .logo })?.url
                    ?? (detail?.images ?? []).first(where: { $0.role == .logo })?.url

                if let logo = logoUrl {
                    Art(logo, contentMode: .fit)
                        .frame(maxWidth: 550.ref, maxHeight: 180.ref, alignment: .bottomLeading)
                } else {
                    Text(lead.name)
                        .font(.system(size: 72.refType, weight: .black))
                        .foregroundStyle(.white)
                        .lineLimit(2)
                }

                // Facts
                if let d = detail {
                    let items = facts(detail: d, words: words)
                    HStack(spacing: 14.ref) {
                        ForEach(Array(items.enumerated()), id: \.offset) { idx, item in
                            if idx > 0 {
                                Circle()
                                    .fill(BrandTheme.muted)
                                    .frame(width: 6.ref, height: 6.ref)
                            }
                            Text(item)
                                .font(.system(size: 24.refType, weight: .bold))
                                .foregroundStyle(.white)
                        }
                    }
                }

                // Overview
                let overview = detail?.overview ?? ""
                if !overview.isEmpty {
                    Text(overview)
                        .font(.system(size: 28.refType, weight: .regular))
                        .foregroundStyle(.white.opacity(0.85))
                        .lineSpacing(6.ref)
                        .lineLimit(3)
                        .frame(maxWidth: 800.ref, alignment: .leading)
                }

                // Action Buttons
                HStack(spacing: 20.ref) {
                    // Play Button (0)
                    let playOn = button == 0
                    HStack(spacing: 12.ref) {
                        Image(systemName: "play.fill")
                            .font(.system(size: 22.refType, weight: .bold))
                        Text(words["common.play"] ?? "Play")
                            .font(.system(size: 28.refType, weight: .bold))
                    }
                    .foregroundStyle(playOn ? .black : .white)
                    .padding(.horizontal, 36.ref)
                    .padding(.vertical, 18.ref)
                    .background(playOn ? Color.white : Color.white.opacity(0.2))
                    .clipShape(Capsule())

                    // More Info Button (1)
                    let infoOn = button == 1
                    HStack(spacing: 12.ref) {
                        Image(systemName: "info.circle")
                            .font(.system(size: 22.refType, weight: .bold))
                        Text(words["common.more_info"] ?? "More Info")
                            .font(.system(size: 28.refType, weight: .bold))
                    }
                    .foregroundStyle(infoOn ? .black : .white)
                    .padding(.horizontal, 36.ref)
                    .padding(.vertical, 18.ref)
                    .background(infoOn ? Color.white : Color.white.opacity(0.2))
                    .clipShape(Capsule())
                }
                .padding(.top, 12.ref)
            }
            .padding(.leading, 60.ref)
            .padding(.bottom, 60.ref)
        }
        .frame(width: 1789.ref, height: 767.ref)
        .background(BrandTheme.plate)
        .clipShape(RoundedRectangle(cornerRadius: 24.ref))
        .overlay(
            RoundedRectangle(cornerRadius: 24.ref)
                .stroke(Color.white.opacity(0.25), lineWidth: 2.ref)
        )
    }
}
