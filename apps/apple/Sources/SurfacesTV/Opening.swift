// The opening, drawn live. Its motion is direction C as the owner approved it, read from the
// renderer that made the old video (streams-studio/showreel/opening.py). Opening.kt is the same
// file for Android.
import Observation
import SwiftUI
import UIKit

private let wide: CGFloat = 1920
private let high: CGFloat = 1080
private let middle = high / 2
private let lockWide: CGFloat = 760
private let markHigh = lockWide / 3.922
private let big: CGFloat = 1.4

/// When the first stage ends and the mark and text rest.
private let rest = 3.2
/// The shortest rest when everything is ready before the first stage ends.
private let shortRest = 0.6
/// Longest the opening waits for the landing before it goes on anyway.
private let waitAtMost = 8.0
private let ringFade = 0.35
private let secondLong = 2.0
private let notesAt = 2.05
/// How long before the window opens the showreel starts, under the cover, past its fade from black.
private let filmLead = 1.0

private let night = Color(red: 0x06 / 255, green: 0x07 / 255, blue: 0x0F / 255)
private let lineFrom = Color(red: 0x5A / 255, green: 0x78 / 255, blue: 0xFF / 255)
private let lineTo = Color(red: 0x96 / 255, green: 0x96 / 255, blue: 0xFF / 255)
private let ringColor = Color(red: 0x2F / 255, green: 0x6B / 255, blue: 0xFF / 255)

/// Picture pixels for one unit of the art. The mark is never shown above 0.86 pixels a unit.
private let bake: CGFloat = 0.9
private let textBake: CGFloat = 0.5

private func smooth(_ x: Double) -> Double {
    let c = min(max(x, 0), 1)
    return c * c * c * (c * (6 * c - 15) + 10)
}

private func span(_ t: Double, _ a: Double, _ b: Double) -> Double {
    min(max((t - a) / (b - a), 0), 1)
}

/// The moments of the opening that the page around it acts on.
public enum Moment {
    /// The two soft notes.
    case notes
    /// The first stage is over; the landing may draw under the cover, so it is ready to be seen.
    case warm
    /// The second stage begins, with its swell.
    case second
    /// The showreel should start.
    case roll
    /// The opening is over.
    case done
}

/// A path made of M, L, C and Z only, as make_opening.py writes them.
private func path(_ d: String) -> CGPath {
    let out = CGMutablePath()
    var numbers: [CGFloat] = []
    var command: Character = "M"
    func flush() {
        switch command {
        case "M" where numbers.count >= 2: out.move(to: CGPoint(x: numbers[0], y: numbers[1]))
        case "L" where numbers.count >= 2: out.addLine(to: CGPoint(x: numbers[0], y: numbers[1]))
        case "C" where numbers.count >= 6:
            out.addCurve(
                to: CGPoint(x: numbers[4], y: numbers[5]),
                control1: CGPoint(x: numbers[0], y: numbers[1]),
                control2: CGPoint(x: numbers[2], y: numbers[3])
            )
        case "Z": out.closeSubpath()
        default: break
        }
        numbers = []
    }
    var word = ""
    for character in d + " " {
        if "MLCZ".contains(character) {
            if !word.isEmpty { numbers.append(CGFloat(Double(word) ?? 0)); word = "" }
            flush()
            command = character
        } else if character == " " {
            if !word.isEmpty { numbers.append(CGFloat(Double(word) ?? 0)); word = "" }
        } else {
            word.append(character)
        }
    }
    flush()
    return out
}

private func tone(_ hex: String) -> CGColor {
    let digits = Array(hex.dropFirst())
    func byte(_ i: Int) -> CGFloat { CGFloat(Int(String(digits[i ..< i + 2]), radix: 16) ?? 0) / 255 }
    return CGColor(srgbRed: byte(0), green: byte(2), blue: byte(4), alpha: digits.count > 6 ? byte(6) : 1)
}

private struct Fill {
    let from: CGPoint
    let to: CGPoint
    let gradient: CGGradient

    init(_ json: [String: Any]) {
        let a = json["from"] as? [Double] ?? [0, 0]
        let b = json["to"] as? [Double] ?? [0, 0]
        from = CGPoint(x: a[0], y: a[1])
        to = CGPoint(x: b[0], y: b[1])
        let stops = json["stops"] as? [[Any]] ?? []
        gradient = CGGradient(
            colorsSpace: CGColorSpace(name: CGColorSpace.sRGB),
            colors: stops.map { tone($0[1] as? String ?? "#000000") } as CFArray,
            locations: stops.map { CGFloat($0[0] as? Double ?? 0) }
        )!
    }

    func draw(_ c: CGContext, inside shape: CGPath, alpha: CGFloat = 1) {
        c.saveGState()
        c.addPath(shape)
        c.clip()
        c.setAlpha(alpha)
        c.drawLinearGradient(gradient, start: from, end: to, options: [.drawsBeforeStartLocation, .drawsAfterEndLocation])
        c.restoreGState()
    }
}

/// The brand's mark and text, from opening.json, made by platform/tools/make_opening.py.
@MainActor
private final class BrandArt {
    static let shared = BrandArt()

    let arrowPath: Path
    let centre: CGPoint
    /// Screen pixels for one unit of the art, at lockup size.
    let unit: CGFloat
    let markX: CGFloat
    let textX: CGFloat
    let textUnit: CGFloat
    let textWide: CGFloat
    let textHigh: CGFloat
    let corner: CGPoint
    let ribbon: Image
    let arrow: Image
    let text: Image
    let pictureSize: CGSize
    let textSize: CGSize

    init() {
        let url = Bundle.module.url(forResource: "opening", withExtension: "json")!
        let json = (try? JSONSerialization.jsonObject(with: Data(contentsOf: url))) as? [String: Any] ?? [:]
        let mark = json["mark"] as? [String: Any] ?? [:]
        let box = (mark["box"] as? [Double] ?? [0, 0, 1, 1]).map { CGFloat($0) }
        let word = json["word"] as? [String: Any] ?? [:]
        let view = (word["view"] as? [Double] ?? [0, 0, 1, 1]).map { CGFloat($0) }
        let part = { (name: String) in mark[name] as? [String: Any] ?? [:] }
        let arrowShape = path(part("arrow")["path"] as? String ?? "")
        arrowPath = Path(arrowShape)
        centre = CGPoint(x: (box[0] + box[2]) / 2, y: (box[1] + box[3]) / 2)
        unit = markHigh / (box[3] - box[1])
        let markWide = markHigh * (box[2] - box[0]) / (box[3] - box[1])
        let cap = markHigh / 1.42
        markX = (wide - lockWide) / 2 + markWide / 2
        textX = (wide - lockWide) / 2 + markWide + cap * 0.2
        textUnit = cap * 1.08 / view[3]
        textWide = view[2] * textUnit
        textHigh = view[3] * textUnit
        corner = CGPoint(x: box[0] - 8, y: box[1] - 8)
        pictureSize = CGSize(width: ((box[2] - box[0] + 16) * bake).rounded(), height: ((box[3] - box[1] + 16) * bake).rounded())
        textSize = CGSize(width: (view[2] * textBake).rounded(), height: (view[3] * textBake).rounded())

        let format = UIGraphicsImageRendererFormat()
        format.scale = 1
        func picture(_ size: CGSize, _ draw: (CGContext) -> Void) -> Image {
            Image(uiImage: UIGraphicsImageRenderer(size: size, format: format).image { draw($0.cgContext) })
        }
        let corner = corner
        func onArt(_ c: CGContext) {
            c.scaleBy(x: bake, y: bake)
            c.translateBy(x: -corner.x, y: -corner.y)
        }
        let ribbonShape = path(part("ribbon")["path"] as? String ?? "")
        let ribbonFill = Fill(part("ribbon")["fill"] as? [String: Any] ?? [:])
        ribbon = picture(pictureSize) { c in
            onArt(c)
            ribbonFill.draw(c, inside: ribbonShape)
        }
        let arrowFill = Fill(part("arrow")["fill"] as? [String: Any] ?? [:])
        let shine = part("shine")
        let shineFill = Fill(shine["fill"] as? [String: Any] ?? [:])
        let glow = part("glow")
        let glowBox = path(glow["path"] as? String ?? "").boundingBoxOfPath
        let blur = CGFloat(glow["blur"] as? Double ?? 0)
        let glowTone = tone(glow["color"] as? String ?? "#FFFFFF")
        arrow = picture(pictureSize) { c in
            onArt(c)
            arrowFill.draw(c, inside: arrowShape)
            shineFill.draw(c, inside: arrowShape, alpha: CGFloat(shine["opacity"] as? Double ?? 0))
            // The glow is a blurred ellipse: a soft radial fall off on a circle stretched to its shape.
            let area = glowBox.insetBy(dx: -2 * blur, dy: -2 * blur)
            let soft = CGGradient(
                colorsSpace: CGColorSpace(name: CGColorSpace.sRGB),
                colors: [glowTone, glowTone.copy(alpha: 0.55)!, glowTone.copy(alpha: 0)!] as CFArray,
                locations: [0, glowBox.width / area.width, 1]
            )!
            c.saveGState()
            c.addPath(arrowShape)
            c.clip()
            c.setAlpha(CGFloat(glow["opacity"] as? Double ?? 0))
            c.translateBy(x: area.midX, y: area.midY)
            c.scaleBy(x: 1, y: area.height / area.width)
            c.drawRadialGradient(soft, startCenter: .zero, startRadius: 0, endCenter: .zero, endRadius: area.width / 2, options: [])
            c.restoreGState()
        }
        let textShape = path(word["path"] as? String ?? "")
        text = picture(textSize) { c in
            c.scaleBy(x: textBake, y: textBake)
            c.translateBy(x: -view[0], y: -view[1])
            c.addPath(textShape)
            c.setFillColor(UIColor.white.cgColor)
            c.fillPath()
        }
    }
}

/// When things happen, decided as the frames go by. Not observed: the timeline redraws every frame.
@MainActor
private final class Clock {
    var start: Date?
    var ready = false
    var readyAt = Double.nan
    var second = Double.nan
    var ringFrom = Double.nan
    var told: Set<String> = []
    var over = false
}

private func part(_ c: inout GraphicsContext, _ art: BrandArt, _ image: Image, _ x: CGFloat, _ scale: CGFloat, _ alpha: Double) {
    guard alpha > 0 else { return }
    var d = c
    d.opacity = alpha
    d.translateBy(x: x, y: middle)
    d.scaleBy(x: scale * art.unit / bake, y: scale * art.unit / bake)
    d.translateBy(x: (art.corner.x - art.centre.x) * bake, y: (art.corner.y - art.centre.y) * bake)
    d.draw(image, in: CGRect(origin: .zero, size: art.pictureSize))
}

/// The text, coming out from behind the mark: nothing left of `edge` shows.
private func text(_ c: inout GraphicsContext, _ art: BrandArt, _ x: CGFloat, _ edge: CGFloat, _ alpha: Double) {
    guard alpha > 0 else { return }
    var d = c
    d.opacity = alpha
    d.clip(to: Path(CGRect(x: edge, y: 0, width: wide - edge, height: high)))
    d.translateBy(x: x, y: middle - art.textHigh / 2)
    d.scaleBy(x: art.textUnit / textBake, y: art.textUnit / textBake)
    d.draw(art.text, in: CGRect(origin: .zero, size: art.textSize))
}

/// Mark and text: `q` 0 is the mark alone in the middle, 1 is both at rest.
private func lockup(_ c: inout GraphicsContext, _ art: BrandArt, _ q: Double) {
    let x = wide / 2 + (art.markX - wide / 2) * q
    if q > 0.002 { text(&c, art, art.textX + (x - art.markX) - (art.textWide + 60) * (1 - q), x, 1) }
    let size = big + (1 - big) * q
    part(&c, art, art.ribbon, x, size, 1)
    part(&c, art, art.arrow, x, size, 1)
}

/// The first stage: a line pulls back into the arrow, the ribbon slides in, the text comes out.
private func first(_ c: inout GraphicsContext, _ art: BrandArt, _ t: Double) {
    if t >= 2.25 {
        lockup(&c, art, smooth(span(t, 2.25, 3.15)))
        return
    }
    let grow = smooth(span(t, 0.2, 0.95))
    let back = smooth(span(t, 0.95, 1.7))
    let from = wide / 2 - 40
    let reach = 560 * grow * (1 - back)
    if back < 1 && reach > 0 {
        c.fill(
            Path(CGRect(x: from - 20, y: middle - 40, width: reach + 40, height: 80)),
            with: .linearGradient(
                Gradient(colors: [.clear, lineFrom.opacity(0.35), .clear]),
                startPoint: CGPoint(x: 0, y: middle - 40), endPoint: CGPoint(x: 0, y: middle + 40)
            )
        )
        c.fill(
            Path(CGRect(x: from, y: middle - 4, width: reach, height: 8)),
            with: .linearGradient(
                Gradient(colors: [lineFrom, lineTo]),
                startPoint: CGPoint(x: from, y: 0), endPoint: CGPoint(x: from + reach, y: 0)
            )
        )
    }
    let tri = smooth(span(t, 0.25, 0.75))
    let open = smooth(span(t, 1.05, 2.0))
    let settle = 0.025 * sin(Double.pi * span(t, 1.8, 2.25))
    let size = (0.16 + 0.02 * tri + (big - 0.18) * open) * (1 + settle)
    let rib = smooth(span(t, 1.5, 2.1))
    part(&c, art, art.ribbon, wide / 2 + 54 * (1 - rib), big * (0.92 + 0.08 * rib), rib)
    part(&c, art, art.arrow, from + 40 * open, size, tri)
}

private func ring(_ c: inout GraphicsContext, _ t: Double, _ alpha: Double) {
    guard alpha > 0 else { return }
    let start = (t * 347).truncatingRemainder(dividingBy: 360)
    var arc = Path()
    arc.addArc(
        center: CGPoint(x: 0.498 * wide, y: 0.804 * high), radius: 33,
        startAngle: .degrees(start), endAngle: .degrees(start + 280), clockwise: false
    )
    c.stroke(arc, with: .color(ringColor.opacity(alpha)), style: StrokeStyle(lineWidth: 8, lineCap: .round))
}

/// The second stage, `u` seconds in: the text and ribbon go, and the arrow grows into a window onto
/// the landing, which is drawn under this cover.
private func second(_ c: inout GraphicsContext, _ art: BrandArt, _ u: Double) {
    let p = span(u, 0.3, secondLong)
    let grown = 1 + 90 * pow(smooth(p), 2.7)
    let x = art.markX + (wide / 2 - art.markX) * smooth(span(p, 0, 0.55))
    let see = smooth(span(p, 0.02, 0.36))
    let k = grown * art.unit
    let window = art.arrowPath.applying(
        CGAffineTransform(translationX: -art.centre.x, y: -art.centre.y)
            .concatenating(CGAffineTransform(scaleX: k, y: k))
            .concatenating(CGAffineTransform(translationX: x, y: middle))
    )
    // The cover has the arrow cut out of it, and the landing under this layer shows through.
    var cover = Path(CGRect(x: 0, y: 0, width: wide, height: high))
    cover.addPath(window)
    c.fill(cover, with: .color(night), style: FillStyle(eoFill: true))
    let words = 1 - smooth(span(u, 0.15, 0.65))
    let band = 1 - smooth(span(u, 0.1, 0.5))
    if words > 0 || band > 0 {
        var d = c
        d.clip(to: window, options: .inverse)
        text(&d, art, art.textX, art.markX, words)
        part(&d, art, art.ribbon, art.markX, 1, band)
    }
    part(&c, art, art.arrow, x, grown, 1 - see)
}

/// The opening over the landing. `ready` turns true when the landing can show; the rest waits for it.
public struct Opening: View {
    let ready: Bool
    let moment: (Moment) -> Void
    @State private var clock = Clock()

    public init(ready: Bool, moment: @escaping (Moment) -> Void) {
        self.ready = ready
        self.moment = moment
    }

    /// Decides what happens at this time, once, and says so.
    private func step(_ t: Double) {
        let c = clock
        guard !c.over else { return }
        func once(_ at: Bool, _ which: Moment) {
            if at && c.told.insert("\(which)").inserted { moment(which) }
        }
        if c.readyAt.isNaN && (c.ready || t >= waitAtMost) { c.readyAt = t }
        once(t >= notesAt, .notes)
        once(t >= rest, .warm)
        if c.second.isNaN && t >= rest {
            if !c.readyAt.isNaN && c.ringFrom.isNaN {
                // The showreel starts a little before the window, so the arrow opens onto the film.
                c.second = max(rest + shortRest, c.readyAt + filmLead)
            } else if !c.readyAt.isNaN {
                c.second = max(c.ringFrom + 2 * ringFade, c.readyAt + filmLead)
            } else if c.ringFrom.isNaN && t >= rest + shortRest {
                c.ringFrom = t
            }
        }
        let s = c.second
        once(!s.isNaN && t >= s - filmLead, .roll)
        once(!s.isNaN && t >= s, .second)
        if !s.isNaN && t >= s + secondLong {
            c.over = true
            moment(.done)
        }
    }

    public var body: some View {
        TimelineView(.animation) { timeline in
            let start = clock.start ?? timeline.date
            let t = timeline.date.timeIntervalSince(start)
            Canvas { context, size in
                var c = context
                c.scaleBy(x: size.width / wide, y: size.height / high)
                let art = BrandArt.shared
                let s = clock.second
                if !s.isNaN && t >= s {
                    second(&c, art, t - s)
                } else {
                    c.fill(Path(CGRect(x: 0, y: 0, width: wide, height: high)), with: .color(night))
                    first(&c, art, min(t, rest))
                    let from = clock.ringFrom
                    if !from.isNaN {
                        let fadeOut = s.isNaN ? 1 : 1 - smooth(span(t, s - ringFade, s))
                        ring(&c, t, smooth(span(t, from, from + ringFade)) * fadeOut)
                    }
                }
            }
            .onChange(of: timeline.date) { _, now in
                if clock.start == nil { clock.start = now }
                step(now.timeIntervalSince(clock.start ?? now))
            }
        }
        .ignoresSafeArea()
        .remote { _ in
            if !clock.over {
                clock.over = true
                moment(.done)
            }
            return true
        }
        .onChange(of: ready, initial: true) { _, now in clock.ready = now }
        .onAppear { _ = BrandArt.shared }
    }
}
