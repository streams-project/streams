// Plain Kotlin. Nothing here may know it runs on Android.
plugins {
    id("org.jetbrains.kotlin.jvm")
    id("org.jetbrains.kotlin.plugin.serialization")
}

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

kotlin {
    compilerOptions { jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17 }
    sourceSets["main"].kotlin.srcDir(rootDir.resolve("../../platform/contract/generated/types"))
    sourceSets["main"].kotlin.exclude("**/*.swift", "**/*.ts")
}

dependencies {
    api("org.jetbrains.kotlinx:kotlinx-serialization-json:1.11.0")
    api("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.11.0")
    testImplementation(kotlin("test"))
}

tasks.test { useJUnitPlatform() }
