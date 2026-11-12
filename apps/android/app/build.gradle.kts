// The one place that plugs adapters into ports, and the app people install. Its name and id come
// from brand.json, so a copy under another name is its own app.
import groovy.json.JsonSlurper

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

val brand = JsonSlurper().parse(rootDir.resolve("../../brand.json")) as Map<*, *>
// The NDK this app is built with, declared once in `core/ndk-version.txt`. This module is the one
// that packages the native libraries, so without it Android Gradle has no `llvm-strip` to run and
// its strip task passes the library through unchanged: the APK then carries the debug information
// of the core, uncompressed, which measured 9,439,528 bytes for arm64 against 3,302,872 stripped.
val declaredNdk: String = rootDir.resolve("core/ndk-version.txt").readText().trim()
// A second copy on one device: -PappSuffix=.owner installs beside the normal app, with its own sign in.
val suffix = providers.gradleProperty("appSuffix").orNull

android {
    namespace = "streams.app"
    compileSdk = 37
    ndkVersion = declaredNdk
    defaultConfig {
        applicationId = "${brand["command"]}.app"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
        applicationIdSuffix = suffix
        resValue("string", "app_name", "${brand["name"]}${suffix.orEmpty()}")
        // The window and the system splash before the first frame take the brand's background, so
        // the opening starts on the same colour with no flash (res/values*/themes.xml).
        resValue("color", "brand_background", (brand["colors"] as Map<*, *>)["background"] as String)
    }
    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            // The debug information the strip above removes is kept per ABI, outside the app, so a
            // crash address from a device still resolves to a function. A bundle writes it to
            // build/outputs/native-debug-symbols/release/native-debug-symbols.zip.
            ndk { debugSymbolLevel = "FULL" }
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "keep.pro")
            // Until the release key exists, so a release build can run on a test device.
            signingConfig = signingConfigs.getByName("debug")
        }
    }
    buildFeatures {
        compose = true
        resValues = true
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    implementation(project(":core"))
    implementation(project(":adapters"))
    implementation(project(":surfaces:phone"))
    implementation(project(":surfaces:tv"))
    val bom = platform("androidx.compose:compose-bom:2026.09.00")
    implementation(bom)
    implementation("androidx.compose.ui:ui")
    implementation("androidx.activity:activity-compose:1.13.0")
    // Hands ART the profile this build already ships in `assets/dexopt/baseline.prof`. It needs
    // no call: its own manifest merges a `ProfileInstallerInitializer` into
    // `androidx.startup.InitializationProvider`, which about six seconds after the first frame
    // copies the profile to ART's folder for this app on a background thread.
    //
    // Handing it over is not compiling it. ART compiles what it was handed at its next background
    // dexopt, which waits for an idle device on a charger, so a person who installs this APK and
    // opens it stays at ART's `verify` filter until then, and so does every measurement taken
    // that way. What compiles the profile at install time is the metadata file beside the APK:
    //
    //     adb install-multiple -r -d app-release.apk app-release.dm
    //
    // with the `.dm` from `build/outputs/apk/release/baselineProfiles/`, whose folder per API
    // range `output-metadata.json` names, kept under the APK's own basename. A Play install gets
    // the same effect from the store's profile; a sideloaded one never does, and this app is
    // handed to people directly. Worth 46 to 65 ms of cold start on a Pixel 8 and nothing on a
    // Google TV Streamer, where compiling more makes it slower.
    implementation("androidx.profileinstaller:profileinstaller:1.4.1")
}
