// The brand's colours and text. brand.json is the source; these equal Theme.kt.
import CoreText
import Ports
import SwiftUI
import UIKit

public let Back = Color(red: 0x0B / 255.0, green: 0x0B / 255.0, blue: 0x10 / 255.0)
public let Plate = Color(red: 0x16 / 255.0, green: 0x16 / 255.0, blue: 0x1D / 255.0)
public let Raised = Color(red: 0x24 / 255.0, green: 0x24 / 255.0, blue: 0x2E / 255.0)
public let Muted = Color(red: 0xA6 / 255.0, green: 0xA6 / 255.0, blue: 0xB3 / 255.0)
public let Accent = Color(red: 0x2F / 255.0, green: 0x6B / 255.0, blue: 0xFF / 255.0)
public let Join = Color(red: 0xE5 / 255.0, green: 0x35 / 255.0, blue: 0x2B / 255.0)

/// The brand's text. A variable font, so every weight comes from one file.
public enum BrandFont {
    private static let wght = 0x7767_6874
    private static let face: CTFontDescriptor? = {
        guard let url = Bundle.module.url(forResource: "inter", withExtension: "ttf") else { return nil }
        CTFontManagerRegisterFontsForURL(url as CFURL, .process, nil)
        let found = CTFontManagerCreateFontDescriptorsFromURL(url as CFURL) as? [CTFontDescriptor]
        return found?.first
    }()
    nonisolated(unsafe) private static var made: [Int: UIFont] = [:]

    /// How much larger this language's text is drawn (`language.text_scale`; Arabic reads small
    /// beside Latin at one size). Set from the words as they arrive.
    nonisolated(unsafe) public static var scale: CGFloat = 1

    /// Inter at a reference size and a weight from 400 to 800.
    public static func of(_ size: CGFloat, _ weight: Int = 400) -> Font {
        let size = size * scale
        let key = Int(size * 10) * 1000 + weight
        if let kept = made[key] { return Font(kept) }
        guard let face else { return .system(size: size) }
        let set = CTFontDescriptorCreateCopyWithAttributes(
            face,
            [kCTFontVariationAttribute: [wght: weight]] as CFDictionary
        )
        let font = CTFontCreateWithFontDescriptor(set, size, nil) as UIFont
        made[key] = font
        return Font(font)
    }
}

extension Text {
    /// Brand text at a reference size and weight.
    public func brand(_ size: CGFloat, _ weight: Int = 400) -> Text {
        font(BrandFont.of(size, weight))
    }
}

extension Seen {
    /// A word of the interface, or nothing while the words are still on their way.
    @MainActor public func say(_ key: String) -> String {
        words[key] ?? ""
    }
}

extension String {
    /// A word's `{name}` place filled with a value, isolated (U+2068, U+2069) so a number, a title
    /// or an address keeps its own order inside a line of either direction.
    public func fill(_ name: String, _ value: Any) -> String {
        let text = value is any BinaryInteger ? number(value) : "\(value)"
        return replacingOccurrences(of: "{\(name)}", with: "\u{2068}\(text)\u{2069}")
    }
}

/// How the interface's language writes numbers and counts: its code (for plural forms) and its
/// digits (`language.digits` in its strings, `arab` for Arabic-Indic). The app sets both as the
/// words arrive.
public enum Speaking {
    nonisolated(unsafe) public static var code = "en"
    nonisolated(unsafe) public static var arabicDigits = false
}

/// A number as the interface's language writes it.
public func number(_ value: Any) -> String {
    let plain = "\(value)"
    guard Speaking.arabicDigits else { return plain }
    return String(plain.map { c -> Character in
        guard let digit = c.wholeNumberValue, c.isASCII else { return c }
        return Character(UnicodeScalar(0x0660 + digit)!)
    })
}

/// The plural form a count takes in a language, as CLDR names them: zero, one, two, few, many, other.
public func pluralOf(_ n: Int, _ lang: String) -> String {
    switch lang.split(separator: "-").first.map(String.init) ?? lang {
    case "ar":
        if n == 0 { return "zero" }
        if n == 1 { return "one" }
        if n == 2 { return "two" }
        if (3...10).contains(n % 100) { return "few" }
        if (11...99).contains(n % 100) { return "many" }
        return "other"
    case "ru", "uk":
        if n % 10 == 1 && n % 100 != 11 { return "one" }
        if (2...4).contains(n % 10) && !(12...14).contains(n % 100) { return "few" }
        return "many"
    case "pl":
        if n == 1 { return "one" }
        if (2...4).contains(n % 10) && !(12...14).contains(n % 100) { return "few" }
        return "many"
    case "he", "iw":
        return n == 1 ? "one" : n == 2 ? "two" : "other"
    case "ja", "ko", "zh":
        return "other"
    case "fr", "pt", "hi":
        return n <= 1 ? "one" : "other"
    default:
        return n == 1 ? "one" : "other"
    }
}

extension Dictionary where Key == String, Value == String {
    /// A counted word: `{base}.{form}` for the count's plural form, else `other`, `many`, then `one`.
    public func counted(_ base: String, _ n: Int) -> String {
        let form = pluralOf(n, Speaking.code)
        let word = [form, "other", "many", "one"].lazy.compactMap { self["\(base).\($0)"] }.first ?? ""
        return word.fill("n", n)
    }
}
