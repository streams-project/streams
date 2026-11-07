// The one place that plugs adapters into ports, and the app people install. Its name and id come
// from brand.json, so a copy under another name is its own app.
import groovy.json.JsonSlurper

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

val brand = JsonSlurper().parse(rootDir.resolve("../../brand.json")) as Map<*, *>

android {
    namespace = "streams.app"
    compileSdk = 37
    defaultConfig {
        applicationId = "${brand["command"]}.app"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
        resValue("string", "app_name", brand["name"].toString())
    }
    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "keep.pro")
            // Until the release key exists, so a release build can run on a test device.
            signingConfig = signingConfigs.getByName("debug")
        }
    }
    buildFeatures { compose = true }
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
    // Compiles the toolkit's profile on the device, so a fresh install scrolls smoothly.
    implementation("androidx.profileinstaller:profileinstaller:1.4.1")
}
