// The television screens, driven by a remote. They take ports and shared parts, and never the
// phone screens.
plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "streams.tv"
    compileSdk = 37
    defaultConfig { minSdk = 26 }
    buildFeatures { compose = true }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    // The module's own tests, on this machine's JVM: `./gradlew :surfaces:tv:testDebugUnitTest`.
    // What they cover is what can be checked without a screen - which rows a page may draw
    // (browse/Page.kt) - because everything else here is drawing, and a drawing test needs a
    // device. `./check` does not run gradle at all, so nothing runs these but a person or CI.
    testOptions {
        unitTests.isReturnDefaultValues = true
        // Robolectric needs the real resource table, not the stubbed one.
        unitTests.isIncludeAndroidResources = true
    }
}

dependencies {
    api(project(":ports"))
    api(project(":surfaces:shared"))
    val bom = platform("androidx.compose:compose-bom:2026.09.00")
    implementation(bom)
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.11.0")
    // The fetcher and the decoders moved to `:adapters` with the `Pictures` port: they reach
    // the network and the disk, which is not a surface's business, and `coil-network-okhttp`
    // was linking okhttp3 into this module. They register through `META-INF/services`, so the
    // APK still has both. What is left here is the drawing binding, until these screens are
    // rewritten in Java and ask the port instead.
    implementation("io.coil-kt.coil3:coil-compose:3.3.0")
    implementation("com.google.zxing:core:3.5.3")
    testImplementation("junit:junit:4.13.2")
    // PROBE 2026-11-13: can a real accessibility tree be read in a unit test here?
    testImplementation("org.robolectric:robolectric:4.17")
}
