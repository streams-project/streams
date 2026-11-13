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
        val brand = JsonSlurper().parse(rootDir.resolve("../../brand.json")) as Map<*, *>
        // The brand's corners, read from brand.json so a renamed copy carries its own shape.
        val shape = brand["shape"] as Map<*, *>
        for (role in listOf("card", "panel", "field", "button", "chip", "chip_end")) {
            buildConfigField("int", "${role.uppercase()}_RADIUS", "${shape["${role}_radius"]}")
        }

        // `#rgb`, `#rrggbb` or `#rrggbbaa` as the one ARGB integer both toolkits take, written as a
        // Java hex literal because that is what `buildConfigField` pastes into generated Java.
        fun argb(value: String): String {
            val hex = value.removePrefix("#")
            val rgb = if (hex.length == 3) hex.map { "$it$it" }.joinToString("") else hex
            val order = if (rgb.length == 8) rgb.takeLast(2) + rgb.take(6) else "FF$rgb"
            return "0x${order.uppercase()}"
        }

        // And the brand's colours, the same way. Until 2026-11-12 `Theme.kt` held these six as
        // Compose literals and the television's Views surface restated them as ARGB ints, because a
        // Compose `Color` is a `@JvmInline value class` over a `ULong` and Java source cannot name
        // it. An ARGB int is a form both toolkits take -- `Color(Int)` and
        // `GradientDrawable.setColor` -- and `BuildConfig` is generated Java, so generating them
        // here gives the brand's colour one home for both languages, as the corners above have.
        // The six are the six the surfaces draw with; a role no family asks for is not emitted.
        val colors = brand["colors"] as Map<*, *>
        for (role in listOf("background", "surface", "raised", "muted", "accent", "join")) {
            buildConfigField("int", "${role.uppercase()}_COLOR", argb(colors[role] as String))
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

// What the toolkit may treat as stable although it was generated elsewhere. Without it every tile
// on a page recomposed with the page, because a contract class is a type this module cannot annotate.
composeCompiler {
    stabilityConfigurationFiles.add(layout.projectDirectory.file("compose-stability.conf"))
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
    // The fetcher and the decoders moved to `:adapters` with the `Pictures` port: they reach
    // the network and the disk, which is not a surface's business, and `coil-network-okhttp`
    // was linking okhttp3 into this module. They register through `META-INF/services`, so the
    // APK still has both. What is left here is the drawing binding, until these screens are
    // rewritten in Java and ask the port instead.
    implementation("io.coil-kt.coil3:coil-compose:3.3.0")

    // This module's first tests, on this machine's JVM. JUnit 4 and nothing else, for the reason
    // `surfaces/phone/build.gradle.kts` gives: an Android library gets its Kotlin from the Android
    // plugin and its unit tests run on JUnit 4 by default. The semantics and the dp scale these
    // tests read are plain Kotlin value classes, so they need no device, no resources and no
    // toolkit runner. `androidx.compose.ui:ui-test` is still not resolved, so a *composed* tree
    // cannot be read here and `Heard.kt` in the test source set says what that costs. The Views
    // half of that hole is closed: `org.robolectric` resolved on 2026-11-13 and
    // `surfaces/tv/src/test/java/streams/tv/views/HeardTest.java` walks a real
    // `AccessibilityNodeInfo` tree. This module stays on JUnit alone until something here needs
    // more than a value class.
    testImplementation("junit:junit:4.13.2")
}
