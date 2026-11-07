import Contract
import SwiftUI

private func find(_ images: [Contract.Image]?, _ role: ImageRole) -> String? {
    images?.first(where: { $0.role == role })?.url
}

public func wideImage(_ images: [Contract.Image]?) -> String? {
    find(images, .backdrop) ?? find(images, .still) ?? find(images, .poster)
}

public func posterImage(_ images: [Contract.Image]?) -> String? {
    find(images, .poster) ?? wideImage(images)
}

public struct Art: View {
    public let url: String?
    public var contentMode: ContentMode

    public init(_ url: String?, contentMode: ContentMode = .fill) {
        self.url = url
        self.contentMode = contentMode
    }

    public var body: some View {
        Group {
            if let str = url, let u = URL(string: str) {
                AsyncImage(url: u) { phase in
                    switch phase {
                    case .success(let image):
                        image
                            .resizable()
                            .aspectRatio(contentMode: contentMode)
                    case .failure:
                        BrandTheme.plate
                    case .empty:
                        BrandTheme.plate.opacity(0.8)
                    @unknown default:
                        BrandTheme.plate
                    }
                }
            } else {
                BrandTheme.plate
            }
        }
    }
}
