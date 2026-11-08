// What every Android surface shares: the brand, its font, and the icon set.
import groovy.json.JsonSlurper

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "streams.shared"
    compileSdk = 37
    defaultConfig {
        minSdk = 26
        // The brand's corners, read from brand.json so a renamed copy carries its own shape.
        val shape = (JsonSlurper().parse(rootDir.resolve("../../brand.json")) as Map<*, *>)["shape"] as Map<*, *>
        for (role in listOf("card", "panel", "field", "button", "chip", "chip_end")) {
            buildConfigField("int", "${role.uppercase()}_RADIUS", "${shape["${role}_radius"]}")
        }
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    api(project(":ports"))
    val bom = platform("androidx.compose:compose-bom:2026.09.00")
    api(bom)
    api("androidx.compose.ui:ui")
    implementation("androidx.compose.foundation:foundation")
    api("androidx.compose.material3:material3")
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.11.0")
    implementation("io.coil-kt.coil3:coil-compose:3.3.0")
    implementation("io.coil-kt.coil3:coil-network-okhttp:3.3.0")
}
