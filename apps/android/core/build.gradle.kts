// The core in its Android shell: the Rust core behind JNI, and the host that steps it.
import org.gradle.api.tasks.Exec

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.plugin.serialization")
}

val repoRoot: File = rootDir.resolve("../..").canonicalFile
val coreLib: File = repoRoot.resolve("target/aarch64-linux-android/native/libbridge_c.a")

android {
    namespace = "streams.core"
    compileSdk = 36
    ndkVersion = "27.2.12479018"
    defaultConfig {
        minSdk = 26
        ndk { abiFilters += "arm64-v8a" }
        externalNativeBuild { cmake { arguments += "-DCORE_LIB=$coreLib" } }
    }
    externalNativeBuild { cmake { path = file("src/main/cpp/CMakeLists.txt") } }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

val buildCore = tasks.register<Exec>("buildCore") {
    workingDir = repoRoot
    environment("CARGO_TARGET_DIR", repoRoot.resolve("target").path)
    commandLine("cargo", "build", "--locked", "--profile", "native", "-p", "bridge-c", "--target", "aarch64-linux-android")
    outputs.file(coreLib)
}

tasks.matching { it.name.startsWith("configureCMake") || it.name.startsWith("buildCMake") }
    .configureEach { dependsOn(buildCore) }

dependencies {
    api(project(":ports"))
}
