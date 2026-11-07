import Contract
import CoreBridge
import Foundation
import Ports

public enum NativeCore {
    public static func abi() -> Int {
        Int(core_abi())
    }
}
