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
// The CPU types this app is packaged for, which are the only ones it can run on at all: `:core`
// builds the Rust core for these two and no others, and names them in the same order
// (`core/build.gradle.kts`). Saying so here as well is what stops the native libraries of other
// modules riding along for a CPU the core was never built for: `libandroidx.graphics.path.so`
// came out of Compose for x86 and x86_64 too, 20,044 bytes packaged into every APK for two CPUs
// where the first `System.loadLibrary` of the core would have failed.
val packagedAbis = listOf("armeabi-v7a", "arm64-v8a")
// A second copy on one device: -PappSuffix=.owner installs beside the normal app, with its own sign in.
val suffix = providers.gradleProperty("appSuffix").orNull
// Android Gradle 9.3.1 will not build an app bundle and several APKs from one configuration: with
// the ABI splits below turned on, `bundleRelease` stops at `buildReleasePreBundle` with "Multiple
// shrunk-resources files found [...] Please disable building multiple APKs when building an Android
// app bundle" (https://issuetracker.google.com/402800800). A bundle already delivers one APK per
// CPU type by itself, so the two are never wanted together, and asking for a bundle turns the
// splits off instead of failing. This reads the task names the invocation asked for, so a bundle
// task reached only as some other task's dependency would still fail: name it on the command line.
val wantsBundle = gradle.startParameter.taskNames.any { it.lowercase().contains("bundle") }
val deploymentFile = rootDir.resolve("../../deployment.json")
val deploymentServer = if (deploymentFile.isFile) {
    ((JsonSlurper().parse(deploymentFile) as? Map<*, *>)?.get("server") as? String).orEmpty()
} else {
    ""
}
val serverUrl = (providers.gradleProperty("server").orNull ?: deploymentServer).trim().trimEnd('/')
val releaseKeystore = providers.gradleProperty("keystore").orNull
val releaseKeystorePassword = providers.gradleProperty("keystorePassword").orNull
val releaseKeyAlias = providers.gradleProperty("keyAlias").orNull
val releaseKeyPassword = providers.gradleProperty("keyPassword").orNull

abstract class PrepareBoundNetworkSecurity : DefaultTask() {
    @get:Input
    abstract val boundServer: Property<String>

    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty

    @TaskAction
    fun generate() {
        val url = boundServer.get()
        val xmlDir = outputDir.get().asFile.resolve("xml")
        xmlDir.mkdirs()
        val lower = url.lowercase()
        val policyXml = if (url.isNotEmpty()) {
            if (!lower.startsWith("https://") && !lower.startsWith("http://")) {
                throw GradleException("Bound server URL must start with https:// or http://; got: $url")
            }
            val host = hostOf(url)
            if (host.isEmpty()) {
                throw GradleException("Bound server URL has no host: $url")
            }
            if (lower.startsWith("http://")) {
                if (!isOwnNetworkHost(host)) {
                    throw GradleException(
                        "Plain http:// is permitted only on own-network hosts " +
                            "(localhost, *.local, ::1, 127/8, 10/8, 172.16/12, 192.168/16, 169.254/16); " +
                            "refused: $url",
                    )
                }
                val escapedHost = xmlEscape(host)
                """
                |<?xml version="1.0" encoding="utf-8"?>
                |<network-security-config>
                |    <base-config cleartextTrafficPermitted="false" />
                |    <domain-config cleartextTrafficPermitted="true">
                |        <domain includeSubdomains="false">$escapedHost</domain>
                |    </domain-config>
                |</network-security-config>
                |""".trimMargin()
            } else {
                """
                |<?xml version="1.0" encoding="utf-8"?>
                |<network-security-config>
                |    <base-config cleartextTrafficPermitted="false" />
                |</network-security-config>
                |""".trimMargin()
            }
        } else {
            """
            |<?xml version="1.0" encoding="utf-8"?>
            |<network-security-config>
            |    <base-config cleartextTrafficPermitted="false" />
            |</network-security-config>
            |""".trimMargin()
        }
        xmlDir.resolve("network_security_config.xml").writeText(policyXml)
    }

    private fun hostOf(address: String): String {
        val afterScheme = address.substringAfter("://", "")
        val authority = afterScheme.substringBefore('/').substringBefore('?').substringBefore('#')
        val hostPort = if ('@' in authority) authority.substringAfterLast('@') else authority
        if (hostPort.startsWith("[")) {
            return hostPort.substringAfter('[').substringBefore(']')
        }
        return if (hostPort.count { it == ':' } == 1) {
            hostPort.substringBefore(':')
        } else {
            hostPort
        }
    }

    private fun isOwnNetworkHost(host: String): Boolean {
        val lower = host.lowercase()
        if (lower == "localhost" || lower.endsWith(".local") || lower == "::1") return true
        val parts = lower.split('.')
        if (parts.size != 4) return false
        val numbers = parts.map { it.toIntOrNull() ?: return false }
        if (numbers.any { it !in 0..255 }) return false
        return when {
            numbers[0] == 127 || numbers[0] == 10 -> true
            numbers[0] == 192 && numbers[1] == 168 -> true
            numbers[0] == 169 && numbers[1] == 254 -> true
            numbers[0] == 172 && numbers[1] in 16..31 -> true
            else -> false
        }
    }

    private fun xmlEscape(value: String): String =
        value.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&apos;")
}

val requireBoundServer = tasks.register("requireBoundServer") {
    inputs.property("serverUrl", serverUrl)
    doLast {
        if (serverUrl.isBlank()) {
            throw GradleException(
                "No server address is bound to :app. Run ./up --public first or pass " +
                    "-Pserver=<url> (or --server <url> to the launcher build command).",
            )
        }
    }
}

val wantsAppPackage = gradle.startParameter.taskNames.any { raw ->
    val name = raw.lowercase()
    val targetsApp = name.startsWith(":app:") || !name.contains(":")
    val isPackageTask = listOf("assemble", "bundle", "install", "package").any { name.contains(it) }
    targetsApp && isPackageTask
}

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
        ndk { abiFilters += packagedAbis }
        buildConfigField("String", "SERVER", "\"${serverUrl.replace("\\", "\\\\").replace("\"", "\\\"")}\"")
        resValue("string", "app_name", "${brand["name"]}${suffix.orEmpty()}")
        // The window and the system splash before the first frame take the brand's background, so
        // the opening starts on the same colour with no flash (res/values*/themes.xml).
        resValue("color", "brand_background", (brand["colors"] as Map<*, *>)["background"] as String)
    }
    signingConfigs {
        if (!releaseKeystore.isNullOrBlank()) {
            create("release") {
                storeFile = rootDir.resolve(releaseKeystore)
                storePassword = releaseKeystorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }
    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            // The debug information the strip above removes is kept per ABI, outside the app, so a
            // crash address from a device still resolves to a function. Both `assembleRelease` and
            // `bundleRelease` write it to
            // build/outputs/native-debug-symbols/release/native-debug-symbols.zip, measured on
            // 2026-11-12 at 2,563,077 bytes holding both `libstreamscore.so.dbg`.
            ndk { debugSymbolLevel = "FULL" }
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "keep.pro")
            signingConfig = if (!releaseKeystore.isNullOrBlank()) {
                signingConfigs.getByName("release")
            } else {
                signingConfigs.getByName("debug")
            }
        }
    }
    // One APK per CPU type, instead of one APK that carries both cores. A phone runs `arm64-v8a`
    // and a Google TV Streamer runs 32 bit apps only, so in the single APK every device carried a
    // core it cannot execute: measured on 2026-11-12, that is 2,569,324 bytes of armv7 core dead on
    // a phone and 3,304,152 bytes of arm64 core dead on the Streamer, out of an APK of 9,008,215.
    //
    // `isUniversalApk` keeps the both-cores APK as well, because this app is handed to people
    // directly and not through a store: the per-ABI APK is the smaller download once somebody knows
    // what the device is, and the universal one is what to send when nobody does. All of them carry
    // the same `versionCode`, so a device can be moved from one to another by reinstalling, which a
    // per-ABI version code offset would refuse as a downgrade. A store needs the opposite; an app
    // bundle is the answer there, and it reads this same block.
    splits {
        abi {
            isEnable = !wantsBundle
            reset()
            include(*packagedAbis.toTypedArray())
            isUniversalApk = true
        }
    }
    buildFeatures {
        compose = true
        resValues = true
        buildConfig = true
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

androidComponents {
    onVariants { variant ->
        val taskName = "prepareBoundNetworkSecurity${variant.name.replaceFirstChar { it.uppercase() }}"
        val prepareTask = tasks.register<PrepareBoundNetworkSecurity>(taskName) {
            boundServer.set(serverUrl)
        }
        variant.sources.res?.addGeneratedSourceDirectory(prepareTask, PrepareBoundNetworkSecurity::outputDir)
    }
}

tasks.matching { task ->
    val name = task.name
    name.startsWith("assemble") ||
        name.startsWith("bundle") ||
        name.startsWith("install") ||
        name.startsWith("package") ||
        (wantsAppPackage && name.startsWith("pre") && name.endsWith("Build"))
}.configureEach {
    dependsOn(requireBoundServer)
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
