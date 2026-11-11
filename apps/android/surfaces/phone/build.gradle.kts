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
    implementation("io.coil-kt.coil3:coil-compose:3.3.0")
    implementation("io.coil-kt.coil3:coil-network-okhttp:3.3.0")
    // The brand's mark is an SVG the server serves; the page tint is read from the billboard's colours.
    implementation("io.coil-kt.coil3:coil-svg:3.3.0")
    implementation("androidx.palette:palette-ktx:1.0.0")
}
