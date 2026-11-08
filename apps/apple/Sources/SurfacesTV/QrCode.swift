import CoreImage
import CoreImage.CIFilterBuiltins
import SwiftUI

/// A code a phone can scan, on white.
struct QrCode: View {
    let text: String

    private var grid: CGImage? {
        let filter = CIFilter.qrCodeGenerator()
        filter.message = Data(text.utf8)
        filter.correctionLevel = "L"
        guard let image = filter.outputImage else { return nil }
        return CIContext().createCGImage(image, from: image.extent)
    }

    var body: some View {
        ZStack {
            if let grid {
                SwiftUI.Image(decorative: grid, scale: 1)
                    .interpolation(.none)
                    .resizable()
                    .aspectRatio(1, contentMode: .fit)
            }
        }
        .padding(24.ref)
        .background(Color.white, in: RoundedRectangle(cornerRadius: 12.ref))
    }
}
