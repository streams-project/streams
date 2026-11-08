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
        replacingOccurrences(of: "{\(name)}", with: "\u{2068}\(value)\u{2069}")
    }
}
