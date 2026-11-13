// The phone and tablet screens. They depend on ports and shared parts, so no player or network
// library is on the classpath here: a screen that tried to use one would not compile.
plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "streams.surfaces.phone"
    compileSdk = 37
    defaultConfig { minSdk = 26 }
    buildFeatures { compose = true }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    // The family's first tests. They run on the machine, not on a device: what is tested here is
    // the paging, which is plain Kotlin holding snapshot state, and a test that needed a device
    // would be a different thing with a different cost. No Android resources are read, so the
    // slower resource-backed runner is left off.
    testOptions {
        unitTests {
            // PROBE 2026-11-13: a composed tree needs the real resource table.
            isIncludeAndroidResources = true
            isReturnDefaultValues = true
        }
    }
}

// What the toolkit may treat as stable although it was generated elsewhere. Without it every tile
// on a page recomposed with the page, because a contract class is a type this module cannot annotate.
composeCompiler {
    stabilityConfigurationFiles.add(layout.projectDirectory.file("compose-stability.conf"))
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
    implementation("androidx.palette:palette-ktx:1.0.0")

    // JUnit 4 and nothing else. `ports` uses kotlin-test on the JUnit platform, which it can
    // because it is a plain Kotlin module and applies the Kotlin plugin itself; an Android library
    // gets its Kotlin from the Android plugin, its unit tests run on JUnit 4 by default, and
    // kotlin-test picks its framework from a rule the Kotlin plugin would have installed. Asking
    // for JUnit 4 directly is one dependency that needs no rule to work.
    testImplementation("junit:junit:4.13.2")
    // PROBE 2026-11-13: can a composed semantics tree be read in a unit test here?
    testImplementation("org.robolectric:robolectric:4.17")
    testImplementation(bom)
    testImplementation("androidx.compose.ui:ui-test-junit4")
    debugImplementation("androidx.compose.ui:ui-test-manifest")
}
