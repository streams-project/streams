import Contract
import CoreImage
import Ports
import SurfacesShared
import SwiftUI
import UIKit

// MARK: - Safe Collection Access

private extension Collection {
    subscript(safe index: Index) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}

// MARK: - Arc Sheet Shape

/// The circular arc top edge of the chooser sheet: apex at 685.6 pt, sag of 15.2 pt at both edges,
/// and radius ~1465 pt, measured from the Pixel 8 and iPhone reference frames.
public struct ArcSheetShape: Shape {
    public var apexY: CGFloat
    public var radius: CGFloat
    public var sag: CGFloat

    public init(apexY: CGFloat = 685.6, radius: CGFloat = 1465.0, sag: CGFloat = 15.2) {
        self.apexY = apexY
        self.radius = radius
        self.sag = sag
    }

    public func path(in rect: CGRect) -> Path {
        var path = Path()
        let w = rect.width
        let h = rect.height
        let halfW = w / 2.0

        let r = radius > 0 ? radius : (halfW * halfW + sag * sag) / (2.0 * sag)
        let clampedHalfW = min(halfW, r)
        let theta = asin(Double(clampedHalfW / r))
        let centerY = apexY + r

        path.addArc(
            center: CGPoint(x: halfW, y: centerY),
            radius: r,
            startAngle: .radians(.pi * 1.5 - theta),
            endAngle: .radians(.pi * 1.5 + theta),
            clockwise: false
        )
        path.addLine(to: CGPoint(x: w, y: h))
        path.addLine(to: CGPoint(x: 0, y: h))
        path.closeSubpath()
        return path
    }
}

// MARK: - Tint Model

/// The extracted hue and saturation from a title's art, applying the measured brightness model.
struct GateTint: Equatable {
    var h: CGFloat
    var s: CGFloat

    static let neutral = GateTint(h: 0.0, s: 0.0)

    /// Bottom fade target and strip between art and arc: HSV(h, s, 0.13).
    var bandColor: Color {
        Color(hue: Double(h), saturation: Double(s), brightness: 0.13)
    }

    /// Sheet gradient: V starts at 0.13 from apex down to y 702.4 pt, then rises linearly to 0.40 at bottom.
    func sheetGradient(apexY: CGFloat, sheetHeight: CGFloat) -> some View {
        let stopLocation: CGFloat = sheetHeight > 0 ? min(max((702.4 - apexY) / sheetHeight, 0.04), 0.25) : 0.08
        return LinearGradient(
            stops: [
                .init(color: Color(hue: Double(h), saturation: Double(s), brightness: 0.13), location: 0),
                .init(color: Color(hue: Double(h), saturation: Double(s), brightness: 0.13), location: stopLocation),
                .init(color: Color(hue: Double(h), saturation: Double(s), brightness: 0.40), location: 1.0)
            ],
            startPoint: .top,
            endPoint: .bottom
        )
    }

    /// Edit tile: HSV(h, s, 0.44) with 20% white overlay.
    var editTileColor: Color {
        Color(hue: Double(h), saturation: Double(s), brightness: 0.44)
    }
}

/// CoreImage average color extraction in HSV color space.
private func extractTintFromImage(_ image: UIImage) -> GateTint? {
    guard let ciImage = CIImage(image: image) else { return nil }
    let extent = ciImage.extent
    let lowerHalf = CGRect(
        x: extent.origin.x,
        y: extent.origin.y,
        width: extent.width,
        height: extent.height / 2.0
    )
    guard let filter = CIFilter(name: "CIAreaAverage", parameters: [
        kCIInputImageKey: ciImage,
        kCIInputExtentKey: CIVector(cgRect: lowerHalf)
    ]), let output = filter.outputImage else { return nil }

    var bitmap = [UInt8](repeating: 0, count: 4)
    let context = CIContext(options: [.workingColorSpace: NSNull()])
    context.render(
        output,
        toBitmap: &bitmap,
        rowBytes: 4,
        bounds: CGRect(x: 0, y: 0, width: 1, height: 1),
        format: .RGBA8,
        colorSpace: nil
    )

    let r = CGFloat(bitmap[0]) / 255.0
    let g = CGFloat(bitmap[1]) / 255.0
    let b = CGFloat(bitmap[2]) / 255.0

    var h: CGFloat = 0
    var s: CGFloat = 0
    var v: CGFloat = 0
    var a: CGFloat = 0
    UIColor(red: r, green: g, blue: b, alpha: 1.0).getHue(&h, saturation: &s, brightness: &v, alpha: &a)
    return GateTint(h: h, s: max(s, 0.20))
}

// MARK: - Date Formatting Helper

private func formatGateDate(_ dateStr: String?, _ lang: String) -> String {
    guard let dateStr, !dateStr.isEmpty else { return "" }
    let parser = DateFormatter()
    parser.dateFormat = "yyyy-MM-dd"
    parser.timeZone = TimeZone(secondsFromGMT: 0)
    guard let date = parser.date(from: dateStr) else { return dateStr }

    let formatter = DateFormatter()
    formatter.locale = Locale(identifier: lang)
    formatter.setLocalizedDateFormatFromTemplate("dMMMM")
    let formatted = formatter.string(from: date)
    return number(formatted)
}

// MARK: - Gate View

/// The Apple twin of the Phone Gate Chooser. Displays background art carousel with Ken
/// Burns zoom, title logo, localized gate lines, arc sheet with dynamic tint model,
/// 3-column profile grid with Kids badge and lock icons, manage mode, and PIN entry sheet.
public struct GateView: View {
    public let shell: Shell
    public let seen: Seen

    @State private var gate: ProfileGate?
    @State private var hello: Hello?
    @State private var currentIndex: Int = 0
    @State private var isManaging: Bool = false
    @State private var selectedProfileForPin: Profile?
    @State private var showPinSheet: Bool = false
    @State private var zoomScale: CGFloat = 1.0
    @State private var tintState: GateTint = .neutral
    @State private var carouselTask: Task<Void, Never>?

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        GeometryReader { geo in
            let totalW = geo.size.width
            let totalH = geo.size.height
            let bottomInset = geo.safeAreaInsets.bottom

            // Layout computations matching Pixel 8 / iPhone measurements
            let defaultRow1Top: CGFloat = 669.7
            let maxRow1Top = max(340.0, totalH - bottomInset - 48.0 - 72.0 - 121.4)
            let row1Top = min(defaultRow1Top, maxRow1Top)
            let sheetApex = row1Top + 15.9
            let blockBottom = row1Top - 65.2
            let artHeight = row1Top

            ZStack(alignment: .top) {
                // Background base color
                Back.ignoresSafeArea()

                // Background Art & Fade
                artArea(width: totalW, height: artHeight)

                // Title Logo and Line Block
                titleBlock(width: totalW, blockBottom: blockBottom)

                // Profiles Choose / Manage Label
                labelBlock(width: totalW, row1Top: row1Top)

                // Arc Sheet Background
                arcSheet(width: totalW, height: totalH, apexY: sheetApex)

                // Profile Grid
                profileGrid(width: totalW, row1Top: row1Top)
            }
            .frame(width: totalW, height: totalH, alignment: .top)
        }
        .sheet(isPresented: $showPinSheet) {
            if let profile = selectedProfileForPin {
                PinSheet(
                    profile: profile,
                    shell: shell,
                    seen: seen,
                    onDismiss: {
                        showPinSheet = false
                        selectedProfileForPin = nil
                    }
                )
                .presentationDetents([.height(340)])
                .presentationDragIndicator(.visible)
            }
        }
        .task {
            await loadGateData()
            startCarousel()
        }
        .onDisappear {
            carouselTask?.cancel()
        }
    }

    // MARK: - Art Area

    @ViewBuilder
    private func artArea(width: CGFloat, height: CGFloat) -> some View {
        let activeTitle = gate?.titles[safe: currentIndex]
        let artUrl = activeTitle.flatMap { gateArtUrl($0.tile) }

        ZStack {
            // Band color background
            tintState.bandColor
                .frame(width: width, height: height)

            if let artUrl {
                Art(artUrl)
                    .scaleEffect(zoomScale, anchor: UnitPoint(x: 0.5, y: 0.51))
                    .frame(width: width, height: height)
                    .clipped()
                    .transition(.opacity)
                    .id(artUrl)

                // Vertical gradient fading to band colour: full strength to ~0.67, fading to 0.96
                LinearGradient(
                    stops: [
                        .init(color: .clear, location: 0.0),
                        .init(color: .clear, location: 0.67),
                        .init(color: tintState.bandColor.opacity(0.85), location: 0.88),
                        .init(color: tintState.bandColor, location: 0.98),
                        .init(color: tintState.bandColor, location: 1.0)
                    ],
                    startPoint: .top,
                    endPoint: .bottom
                )
                .frame(width: width, height: height)
            }
        }
        .frame(width: width, height: height)
        .clipped()
    }

    // MARK: - Title Block (Logo & Line)

    @ViewBuilder
    private func titleBlock(width: CGFloat, blockBottom: CGFloat) -> some View {
        let activeTitle = gate?.titles[safe: currentIndex]
        let logoUrl = activeTitle.flatMap { gateLogoUrl($0.tile) }
        let lineText = activeTitle.flatMap { makeLineText($0.line, seen) }

        VStack(spacing: 8.2) {
            if let logoUrl {
                Art(logoUrl, fit: true, says: activeTitle?.tile.name ?? "")
                    .frame(maxWidth: 167.4, maxHeight: 72.2)
            } else if let titleName = activeTitle?.tile.name {
                // The gate's art carousel behind this is decoration: the title it shows is named
                // here, by its treatment or by these words.
                Text(titleName)
                    .brand(20, 700)
                    .foregroundStyle(.white)
                    .lineLimit(2)
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: 220)
            }

            if let lineText {
                Text(lineText)
                    .brand(14, 700)
                    .foregroundStyle(.white)
                    .lineLimit(1)
            }
        }
        .frame(width: width)
        .position(x: width / 2.0, y: blockBottom - 45.0)
        .transition(.opacity)
        .id(currentIndex)
    }

    // MARK: - Label Block

    @ViewBuilder
    private func labelBlock(width: CGFloat, row1Top: CGFloat) -> some View {
        let text = isManaging ? seen.say("profiles.manage") : seen.say("profiles.choose.phone")
        let displayText = text.isEmpty ? (isManaging ? "Manage Profiles" : "Choose your profile") : text

        Text(displayText)
            .brand(14, 400)
            .foregroundStyle(Color.white.opacity(0.70))
            .lineLimit(1)
            .position(x: width / 2.0, y: row1Top - 25.0)
            // This line is what the whole page is for, so a reader can jump to it.
            .accessibilityAddTraits(.isHeader)
    }

    // MARK: - Arc Sheet

    @ViewBuilder
    private func arcSheet(width: CGFloat, height: CGFloat, apexY: CGFloat) -> some View {
        let sheetH = height - apexY
        ZStack {
            if tintState != .neutral {
                tintState.sheetGradient(apexY: apexY, sheetHeight: sheetH)
                    .overlay(Color.white.opacity(0.10))
            } else {
                Color(red: 0x1A / 255.0, green: 0x1A / 255.0, blue: 0x1A / 255.0)
                    .overlay(Color.white.opacity(0.10))
            }
        }
        .clipShape(ArcSheetShape(apexY: apexY, radius: 1465.0, sag: 15.2))
        .frame(width: width, height: height)
    }

    // MARK: - Profile Grid

    @ViewBuilder
    private func profileGrid(width: CGFloat, row1Top: CGFloat) -> some View {
        let columns = [
            GridItem(.fixed(72), spacing: 40),
            GridItem(.fixed(72), spacing: 40),
            GridItem(.fixed(72), spacing: 40)
        ]

        LazyVGrid(columns: columns, spacing: 20) {
            // Normal profiles
            ForEach(seen.profiles, id: \.id) { profile in
                profileCell(profile)
            }

            // Manage mode: Add Profile tile
            let profilesMax = hello?.limits.profilesMax ?? 5
            if isManaging && seen.profiles.count < profilesMax {
                addProfileCell()
            }

            // Edit tile / Done button
            actionCell()
        }
        .frame(width: 296)
        .position(x: width / 2.0, y: row1Top + 80.0)
    }

    // MARK: - Grid Cells

    @ViewBuilder
    private func profileCell(_ profile: Profile) -> some View {
        Button {
            if isManaging {
                // Tapping in manage mode
            } else if profile.hasPin {
                selectedProfileForPin = profile
                showPinSheet = true
            } else {
                shell.act(.enterProfile(ActionEnterProfile(profile: profile.id)))
            }
        } label: {
            VStack(spacing: 6) {
                ZStack(alignment: .bottomTrailing) {
                    // Avatar box
                    ZStack {
                        if let avatarUrl = profile.avatar, let url = URL(string: avatarUrl) {
                            AsyncImage(url: url) { phase in
                                switch phase {
                                case .success(let image):
                                    image.resizable().aspectRatio(contentMode: .fill).decoration()
                                default:
                                    fallbackAvatarLetter(profile)
                                }
                            }
                        } else {
                            fallbackAvatarLetter(profile)
                        }

                        // Dimming + pencil overlay in manage mode
                        if isManaging {
                            Color.black.opacity(0.50)
                            Icon("icon_pencil", 24, tint: .white)
                        }
                    }
                    .frame(width: 72, height: 72)
                    .clipShape(RoundedRectangle(cornerRadius: Corner.card * 100.0 / 130.0))
                    .shadow(color: Color.black.opacity(0.24), radius: 8, x: 0, y: 7)

                    // Kids badge
                    if profile.kids {
                        let kidsText = seen.say("profile.badge.kids")
                        Text(kidsText.isEmpty ? "kids" : kidsText)
                            .brand(8, 700)
                            .foregroundStyle(.white)
                            .padding(.horizontal, 4)
                            .padding(.vertical, 1.5)
                            .background(Join, in: Capsule())
                            .overlay(Capsule().stroke(Color.white, lineWidth: 0.75))
                            .padding(4.7)
                    }
                }

                // Name
                Text(profile.name)
                    .brand(14, 700)
                    .foregroundStyle(.white)
                    .lineLimit(1)
                    .frame(maxWidth: 88)

                // Lock icon
                if profile.hasPin {
                    Icon("icon_locked", 12, tint: .white.opacity(0.80))
                        .offset(y: -2)
                }
            }
        }
        .buttonStyle(.plain)
        // The padlock drawn under a locked profile's name, and the pencil drawn over its picture in
        // manage mode, are pictures: a person listening gets the name and nothing else unless the
        // cell says which it is. The locked form is the one `strings/` has words for.
        .says(
            profile.hasPin
                ? (seen.say("say.profile_locked")).fill("name", profile.name)
                : profile.name
        )
    }

    @ViewBuilder
    private func fallbackAvatarLetter(_ profile: Profile) -> some View {
        RoundedRectangle(cornerRadius: Corner.card * 100.0 / 130.0)
            .fill(profile.kids ? Join : Accent)
            .overlay(
                Text(String(profile.name.prefix(1)).uppercased())
                    .brand(32, 800)
                    .foregroundStyle(.white)
            )
    }

    @ViewBuilder
    private func addProfileCell() -> some View {
        Button {
            // Action to add profile
        } label: {
            VStack(spacing: 6) {
                ZStack {
                    Color(red: 0x33 / 255.0, green: 0x33 / 255.0, blue: 0x33 / 255.0)
                    Icon("icon_plus", 24, tint: .white)
                }
                .frame(width: 72, height: 72)
                .clipShape(RoundedRectangle(cornerRadius: Corner.card * 100.0 / 130.0))

                let addText = seen.say("profile.add")
                Text(addText.isEmpty ? "Add Profile" : addText)
                    .brand(14, 400)
                    .foregroundStyle(.white)
                    .lineLimit(1)
                    .frame(maxWidth: 88)
            }
        }
        .buttonStyle(.plain)
        // "Add Profile" is drawn under the plus, so the cell gathers it.
        .saysItsWords()
    }

    @ViewBuilder
    private func actionCell() -> some View {
        Button {
            withAnimation(.easeInOut(duration: 0.25)) {
                isManaging.toggle()
            }
        } label: {
            VStack(spacing: 6) {
                ZStack {
                    if isManaging {
                        Color(red: 0x33 / 255.0, green: 0x33 / 255.0, blue: 0x33 / 255.0)
                        Icon("icon_check", 22, tint: .white)
                    } else {
                        if tintState != .neutral {
                            tintState.editTileColor
                                .overlay(Color.white.opacity(0.20))
                        } else {
                            Color(red: 0x33 / 255.0, green: 0x33 / 255.0, blue: 0x33 / 255.0)
                        }
                        Icon("icon_pencil", 22.6, tint: .white)
                    }
                }
                .frame(width: 72, height: 72)
                .clipShape(RoundedRectangle(cornerRadius: Corner.card * 100.0 / 130.0))

                let editLabel = isManaging ? seen.say("common.done") : seen.say("profiles.edit")
                let fallback = isManaging ? "Done" : "Edit"
                Text(editLabel.isEmpty ? fallback : editLabel)
                    .brand(14, isManaging ? 700 : 400)
                    .foregroundStyle(.white)
                    .lineLimit(1)
                    .frame(maxWidth: 88)
            }
        }
        .buttonStyle(.plain)
        // "Edit" or "Done" is drawn under the pencil or the tick, so the cell gathers it, and which
        // of the two it says is what tells a person listening which way the gate stands.
        .saysItsWords()
    }

    // MARK: - Carousel & Data Loading

    private func loadGateData() async {
        if let helloText = await shell.read("/v1/hello") {
            hello = try? JSONDecoder().decode(Hello.self, from: Data(helloText.utf8))
        }
        if let gateText = await shell.read("/v1/profiles/gate") {
            gate = try? JSONDecoder().decode(ProfileGate.self, from: Data(gateText.utf8))
            if let firstTitle = gate?.titles.first {
                await updateTint(for: firstTitle)
            }
        }
    }

    private func startCarousel() {
        carouselTask?.cancel()
        carouselTask = Task { @MainActor in
            // Initial Ken Burns zoom
            withAnimation(.linear(duration: 3.85)) {
                zoomScale = 1.08
            }

            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 3_850_000_000)
                guard !Task.isCancelled else { break }
                guard let count = gate?.titles.count, count > 1 else { continue }
                let nextIndex = (currentIndex + 1) % min(count, 3)

                withAnimation(.easeOut(duration: 0.35)) {
                    currentIndex = nextIndex
                }

                if let nextTitle = gate?.titles[safe: nextIndex] {
                    Task {
                        await updateTint(for: nextTitle)
                    }
                }

                zoomScale = 1.00
                withAnimation(.linear(duration: 3.85)) {
                    zoomScale = 1.08
                }
            }
        }
    }

    private func updateTint(for title: GateTitle) async {
        guard let urlString = gateArtUrl(title.tile),
              let data = await shell.bytes(urlString),
              let image = UIImage(data: data),
              let extracted = extractTintFromImage(image) else {
            return
        }

        await MainActor.run {
            withAnimation(.easeInOut(duration: 0.55)) {
                tintState = extracted
            }
        }
    }
}

// MARK: - Image & Line Helpers

private func gateArtUrl(_ tile: Tile) -> String? {
    if let artUrl = tile.images?.first(where: { $0.role == .art })?.url {
        return artUrl
    }
    return tall(tile.images) ?? wide(tile.images)
}

private func gateLogoUrl(_ tile: Tile) -> String? {
    logo(tile.images)
}

@MainActor
private func makeLineText(_ line: GateLine, _ seen: Seen) -> String? {
    switch line.kind {
    case .coming:
        guard let day = line.date?.rawValue else { return nil }
        let dateText = formatGateDate(day, Speaking.code)
        let pattern = seen.say("gate.coming")
        return pattern.isEmpty ? "Coming on \(dateText)" : pattern.fill("date", dateText)
    case .seasonsComing:
        guard let day = line.date?.rawValue else { return nil }
        let dateText = formatGateDate(day, Speaking.code)
        let pattern = seen.say("gate.seasons_coming")
        return pattern.isEmpty ? "Seasons coming on \(dateText)" : pattern.fill("date", dateText)
    case .ranked:
        guard let rank = line.rank else { return nil }
        let rankStr = number(rank)
        let isTV = line.media == .tv
        let key = isTV ? "gate.ranked.series" : "gate.ranked.films"
        let pattern = seen.say(key)
        return pattern.isEmpty ? (isTV ? "No. \(rankStr) in Series Today" : "No. \(rankStr) in Films Today") : pattern.fill("rank", rankStr)
    case .none:
        return nil
    default:
        return nil
    }
}

// MARK: - PIN Sheet

/// Numeric 4-digit PIN sheet with auto-focused keypad, masked dots, auto-submit on 4th digit,
/// and error handling.
public struct PinSheet: View {
    public let profile: Profile
    public let shell: Shell
    public let seen: Seen
    public let onDismiss: () -> Void

    @State private var pin: String = ""
    @State private var errorMessage: String? = nil
    /// How many notices had been said when this sheet last submitted a PIN. The notice slot is app
    /// wide, so a notice is this sheet's answer only when the count has moved past this.
    @State private var noticesAtSubmit: Int? = nil
    @FocusState private var isFieldFocused: Bool

    public init(profile: Profile, shell: Shell, seen: Seen, onDismiss: @escaping () -> Void) {
        self.profile = profile
        self.shell = shell
        self.seen = seen
        self.onDismiss = onDismiss
    }

    public var body: some View {
        VStack(spacing: 20) {
            // Dismiss bar
            HStack {
                Spacer()
                Button {
                    onDismiss()
                } label: {
                    Icon("icon_back", 18, tint: .white.opacity(0.70))
                        .padding(8)
                }
                .says(seen.say("say.close"))
            }
            .padding(.horizontal, 16)
            .padding(.top, 12)

            // Header text
            let enterPrompt = seen.say("profile.pin.enter")
            Text(enterPrompt.isEmpty ? "Enter your PIN to access this profile." : enterPrompt)
                .brand(17, 700)
                .foregroundStyle(.white)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 24)
                .accessibilityAddTraits(.isHeader)

            // Profile preview. The plate carries the profile's first letter when it has no picture,
            // and a letter is not a word: the name is drawn beside it and says the whole of it.
            HStack(spacing: 10) {
                RoundedRectangle(cornerRadius: Corner.card * 100.0 / 130.0)
                    .fill(profile.kids ? Join : Accent)
                    .frame(width: 36, height: 36)
                    .overlay(
                        Text(String(profile.name.prefix(1)).uppercased())
                            .brand(16, 700)
                            .foregroundStyle(.white)
                    )
                    .decoration()
                Text(profile.name)
                    .brand(15, 700)
                    .foregroundStyle(.white)
            }

            // 4 masked boxes over a field one point across. The field is what takes the digits and
            // the boxes are a drawing of what it holds, so there is one control here and it has to
            // be named: nothing about it can be seen, and without words a reader has no way into
            // the sheet at all.
            //
            // Android names the field (`PinSheet.kt`). Here the words go on the boxes instead,
            // because on this toolkit the field is not in the tree to name: measured on iPhone 17
            // on 2026-11-12, a `TextField` of one point at 1 % opacity reaches no accessibility
            // element whatever label it is given. The boxes are what a person sees, so they are
            // what a person hears, and the field inside is read as part of them.
            HStack(spacing: 16) {
                ForEach(0..<4, id: \.self) { index in
                    let isFilled = index < pin.count
                    ZStack {
                        RoundedRectangle(cornerRadius: Corner.field)
                            .fill(Raised)
                            .frame(width: 52, height: 56)
                            .overlay(
                                RoundedRectangle(cornerRadius: Corner.field)
                                    .stroke(index == pin.count ? Accent : Color.white.opacity(0.12), lineWidth: 1.5)
                            )
                        if isFilled {
                            Circle()
                                .fill(Color.white)
                                .frame(width: 12, height: 12)
                        }
                    }
                }
            }
            .overlay {
                TextField("", text: $pin)
                    .keyboardType(.numberPad)
                    .focused($isFieldFocused)
                    .accentColor(.clear)
                    .foregroundColor(.clear)
                    .frame(width: 1, height: 1)
                    .opacity(0.01)
                    .onChange(of: pin) { newValue in
                        let filtered = newValue.filter { $0.isNumber }
                        if filtered.count > 4 {
                            pin = String(filtered.prefix(4))
                        } else {
                            pin = filtered
                        }
                        if pin.count == 4 {
                            submit(pin)
                        }
                    }
            }
            .says(seen.say("say.pin"))

            // Error notice
            let wrongPinStr = seen.say("error.pin_wrong")
            let fallbackError = wrongPinStr.isEmpty ? "That PIN is not right." : wrongPinStr
            if let error = errorMessage {
                Text(error)
                    .brand(13, 400)
                    .foregroundStyle(Join)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 20)
            } else if let before = noticesAtSubmit, seen.notices > before {
                Text((seen.notice ?? "").isEmpty ? fallbackError : (seen.notice ?? ""))
                    .brand(13, 400)
                    .foregroundStyle(Join)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 20)
            }

            Spacer()
        }
        .frame(maxWidth: .infinity)
        .background(Back.ignoresSafeArea())
        .onAppear {
            isFieldFocused = true
        }
    }

    private func submit(_ enteredPin: String) {
        let before = seen.notices
        noticesAtSubmit = before
        shell.act(.enterProfile(ActionEnterProfile(pin: enteredPin, profile: profile.id)))
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 350_000_000)
            // A notice that was already there is somebody else's. Only one said after this submit
            // is an answer to this PIN.
            if seen.notices > before {
                let wrongPinStr = seen.say("error.pin_wrong")
                errorMessage = wrongPinStr.isEmpty ? seen.notice : wrongPinStr
                pin = ""
            }
        }
    }
}

public typealias Gate = GateView
