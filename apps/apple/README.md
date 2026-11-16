# apple

SwiftUI apps for iPhone, iPad and Apple TV, with AVPlayer, and a window surface for Apple Vision Pro.

The Vision Pro half of that sentence was a claim with nothing behind it until 2026-11-14:
`Package.swift` declared tvOS and iOS and there was no third surface in the tree. It now declares
`.visionOS` and carries `SurfacesVision`, a window surface of its own.

It is **not a ported television**. It shares the ports, the contract, the brand and the words, and
shares neither surface's layout, chrome, colours, input model or player: the system moves focus by
where the viewer looks and never tells the app, so there is no key map and no focus index anywhere
in it; depth does the job a television's focused-tile expansion does, and costs no screen area;
navigation lives in an ornament outside the window rather than a band across the page; it draws on
the system's glass over a real room rather than the brand's dark plate; nothing is revealed on
hover, because on a gaze-pointed surface that means revealed only to whatever is being looked at;
and nothing needs a drag, because Dwell Control cannot hold and move.
`Sources/SurfacesVision/README.md` argues each of those, lists which measured television numbers
carry over and which are void, and names what a device still has to settle.

What is true of that surface and not of the other two: **it has never run anywhere.** There is no
Vision Pro here and no visionOS simulator either — on 2026-11-14 `xcrun simctl list runtimes`
answered iOS 26.5 and tvOS 26.5 and nothing else. The visionOS SDKs are installed, so the module is
compiled for visionOS, device and simulator, with
`swift build --triple arm64-apple-xros1.0[-simulator] --sdk "$(xcrun --sdk xros[imulator] --show-sdk-path)"`,
and that compile is the only thing about visionOS anything here checks. `VisionTests` runs on the
iOS simulator, which is why the module is written to compile there too; what that proves is the
layout arithmetic, the words on every control and how large each control is laid out, and what it
cannot prove is anything about gaze, about a highlight the system draws out of process, or about a
window in space.

Part of an open video platform. Status: compiles and tests, never run on a device. A phone surface, a
television surface and a window surface in Swift behind this family's ports. `./check` holds the family to those ports with
`platform/tools/check_rings.py` and compiles the generated `Contract.swift` where swiftc is present, but it
does not build the app. The tests under `Tests/` run with
`xcodebuild test -scheme AppleFamily-Package`, on the iOS simulator and the television half on the tvOS one;
`./check` has run them since 2026-11-13, through `platform/tools/check_apple.py`, which resolves a simulator at run time and announces loudly when it cannot find Xcode rather than passing quietly. CI is Linux, so there it always announces. The root README's status table carries
this line for every platform.

One thing found while writing those tests applies to all three surfaces, not just the new one:
**SwiftUI builds no accessibility nodes for the contents of a `ScrollView` hosted in an offscreen
window**, so `Tests/Heard` cannot see them. Measured on the iOS simulator on 2026-11-14 —
`heard(ScrollView { Text("inside") })` is the empty list at every size from 390×820 to 1280×820 and
at settles up to 2 s, while `heard(VStack { Text("outside") })` returns the text at once; under the
hosting view the one element is the `HostingScrollView` itself, answering nil to
`accessibilityElements` and 0 to `accessibilityElementCount()`, so it is not the walker's doing.
Any test here that reads a whole scrolling screen is reading only the parts outside its scroller.
`Tests/Heard` was left alone rather than worked around: making the walk see more would change what
the other two families' tests find, and one of those surfaces is being edited right now.

A third surface is also a constraint on the other two, which is worth knowing before adding a
fourth: every target in this package is compiled for every platform the package declares, so
`-only-testing:TelevisionTests` still builds `SurfacesVision` for tvOS. On 2026-11-16 that turned
the television target red — `'hoverEffect' is only available in tvOS 18.0 or newer` against this
family's tvOS 17 floor — and `check_apple.py` reported TelevisionTests running no test at all while
VisionTests passed. The window surface now guards that one call (`SurfacesVision/app/Reach.swift`),
since a television has neither a pointer nor a gaze for it to act on. Without running all three
targets the regression would have been invisible: the new surface's own tests were green throughout.

`check_apple.py` sends a test target to the television simulator when it depends on `SurfacesTV`
and to the phone one otherwise, so `VisionTests` lands on the phone — which is where it has to run
and is not where it is aimed. The rule it wants is a third platform that resolves to a visionOS
simulator, finds none on any machine here, and says on every run that the window surface was not
run, the way it already does for a missing Xcode. Until then the line it prints names the iOS
simulator and under-reports what was checked.

No size claims here on purpose: `platform/tools/check_claims.py` refuses them, because a count in prose is a
promise nothing enforces. To count the code, run `git ls-files -- 'apps/apple/*.swift' | xargs wc -l`.

Licensed under either of MIT or Apache 2.0, at your option.
