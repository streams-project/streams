# apple

SwiftUI apps for iPhone, iPad and Apple TV, with AVPlayer. It also covers Apple Vision Pro.

Part of an open video platform. Status: compiles and tests, never run on a device. A phone surface and a
television surface in Swift behind this family's ports. `./check` holds the family to those ports with
`platform/tools/check_rings.py` and compiles the generated `Contract.swift` where swiftc is present, but it
does not build the app. The tests under `Tests/` run with
`xcodebuild test -scheme AppleFamily-Package`, on the iOS simulator and the television half on the tvOS one;
`./check` does not run xcodebuild, so nothing automated reaches them. The root README's status table carries
this line for every platform.

No size claims here on purpose: `platform/tools/check_claims.py` refuses them, because a count in prose is a
promise nothing enforces. To count the code, run `git ls-files -- 'apps/apple/*.swift' | xargs wc -l`.

Licensed under either of MIT or Apache 2.0, at your option.
