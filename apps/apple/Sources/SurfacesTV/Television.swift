import CoreHost
import SwiftUI

public struct Television: View {
    public init() {}

    public var body: some View {
        ZStack {
            Color(red: 11.0 / 255.0, green: 11.0 / 255.0, blue: 16.0 / 255.0)
                .ignoresSafeArea()
            VStack(spacing: 24) {
                Text("core_abi()")
                    .font(.system(size: 44, weight: .medium))
                    .foregroundStyle(Color(red: 166.0 / 255.0, green: 166.0 / 255.0, blue: 179.0 / 255.0))
                Text("\(NativeCore.abi())")
                    .font(.system(size: 120, weight: .bold))
                    .foregroundStyle(.white)
            }
        }
    }
}
