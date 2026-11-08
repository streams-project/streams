import CoreGraphics
import Foundation
import SwiftUI


extension Int {
    public var ref: CGFloat { CGFloat(self) }
    public var refType: CGFloat { CGFloat(self) }
}

extension Double {
    public var ref: CGFloat { CGFloat(self) }
    public var refType: CGFloat { CGFloat(self) }
}

extension CGFloat {
    public var ref: CGFloat { self }
    public var refType: CGFloat { self }
}
