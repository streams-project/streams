// How Android does what a port asks. The only module that may name Media3, the network, or an
// image library: a surface asks the `Pictures` port and never Coil.
plugins {
    id("com.android.library")
}

android {
    namespace = "streams.adapters"
    compileSdk = 37
    defaultConfig { minSdk = 26 }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    api(project(":ports"))
    implementation("androidx.media3:media3-exoplayer:1.11.1")
    implementation("androidx.media3:media3-exoplayer-hls:1.11.1")
    implementation("androidx.media3:media3-ui:1.11.1")

    // Pictures. `coil` holds the loader; the network fetcher and the SVG decoder are separate
    // artifacts that register themselves through `META-INF/services`, so they are found from
    // anywhere on the APK's classpath and belong on the module that owns the capability rather
    // than on three surface modules. `coil-network-okhttp` is where okhttp3 and okio enter this
    // app at all -- 78 okhttp3 and 89 okio classes ship -- and until today each surface module
    // declared it itself, under a ring check that banned the word `okhttp3` in surface source.
    implementation("io.coil-kt.coil3:coil:3.3.0")
    implementation("io.coil-kt.coil3:coil-network-okhttp:3.3.0")
    implementation("io.coil-kt.coil3:coil-svg:3.3.0")
}
