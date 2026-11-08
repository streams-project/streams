import SwiftUI

/// The width a screen has to work with, by class, never by device name: compact phones, medium
/// (large phones on their side, small tablets), expanded tablets. These equal Sizes.kt.
public enum WidthClass { case compact, medium, expanded }

public func widthClass(_ width: CGFloat) -> WidthClass {
    width < 600 ? .compact : width < 840 ? .medium : .expanded
}

/// The side margin of a page: 12 points on a phone, wider on tablets.
public func pageEdge(_ width: CGFloat) -> CGFloat {
    switch widthClass(width) {
    case .compact: 12
    case .medium: 24
    case .expanded: 40
    }
}
