// The core in its Android shell: the Rust core behind JNI, and the host that steps it.
import org.gradle.api.tasks.Exec

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.plugin.serialization")
}

val repoRoot: File = rootDir.resolve("../..").canonicalFile
val coreFolder: File = repoRoot.resolve("target")

// Each Android CPU type and the Rust target that builds for it. A Google TV Streamer runs 32 bit
// apps only, and phones run 64 bit.
val cores = mapOf(
    "armeabi-v7a" to "armv7-linux-androideabi",
    "arm64-v8a" to "aarch64-linux-android",
)

android {
    namespace = "streams.core"
    compileSdk = 37
    ndkVersion = "27.2.12479018"
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
    }
}

tasks.matching { it.name.startsWith("configureCMake") || it.name.startsWith("buildCMake") }
    .configureEach { dependsOn(buildCores) }

dependencies {
    api(project(":ports"))
}
