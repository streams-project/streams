# android

Compose apps for Android phones, tablets, Android TV and cars, with Media3.

Part of an open video platform. Status: runs on real devices, built by hand. Screens behind this family's
ports: the television's `browse` is Java and native Views, the rest is Compose in Kotlin. It builds with
`./gradlew` on a machine that has a JDK and the Android SDK. `./check` holds the family to its ports with
`platform/tools/check_rings.py` but does not run gradle, so the unit tests under
`surfaces/*/src/test` are run by `./gradlew test` or by CI, never by `./check`. The root README's status
table carries this line for every platform.

No size claims here on purpose: `platform/tools/check_claims.py` refuses them, because a count in prose is a
promise nothing enforces. To count the code, run `git ls-files -- 'apps/android/*.kt' | xargs wc -l`.

Licensed under either of MIT or Apache 2.0, at your option.
