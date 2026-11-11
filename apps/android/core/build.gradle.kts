// The core in its Android shell: the Rust core behind JNI, and the host that steps it.
import org.gradle.api.tasks.Exec

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.plugin.serialization")
}

val repoRoot: File = rootDir.resolve("../..").canonicalFile
val coreFolder: File = repoRoot.resolve("target")

// The one declaration of the NDK this app is built with, read by this module and by `:app`.
// It is a file and not a literal in two build scripts on purpose: `:app` packages the library this
// module links, and the version is what lets `:app` find `llvm-strip`. When the two disagreed,
// `:app` had no NDK at all, its strip task copied the library through untouched, and a 9.4 MB
// library with its DWARF still in it shipped inside the APK. platform/tools/check_android_native.py
// refuses a literal in either file.
val declaredNdk: String = rootDir.resolve("core/ndk-version.txt").readText().trim()

// Each Android CPU type and the Rust target that builds for it. A Google TV Streamer runs 32 bit
// apps only, and phones run 64 bit.
val cores = mapOf(
    "armeabi-v7a" to "armv7-linux-androideabi",
    "arm64-v8a" to "aarch64-linux-android",
)

android {
    namespace = "streams.core"
    compileSdk = 37
    ndkVersion = declaredNdk
    defaultConfig {
        minSdk = 26
        ndk { abiFilters += cores.keys }
        externalNativeBuild { cmake { arguments += "-DCORE_FOLDER=$coreFolder" } }
    }
    externalNativeBuild { cmake { path = file("src/main/cpp/CMakeLists.txt") } }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

val buildCores = cores.values.map { target ->
    tasks.register<Exec>("buildCore-$target") {
        workingDir = repoRoot
        environment("CARGO_TARGET_DIR", coreFolder.path)
        commandLine("cargo", "build", "--locked", "--profile", "native", "-p", "bridge-c", "--target", target)
        outputs.file(coreFolder.resolve("$target/native/libbridge_c.a"))
        // Cargo knows what the core is made of, so it decides whether anything changed.
        outputs.upToDateWhen { false }
    }
}

tasks.matching { it.name.startsWith("configureCMake") || it.name.startsWith("buildCMake") }
    .configureEach { dependsOn(buildCores) }

dependencies {
    api(project(":ports"))
}
