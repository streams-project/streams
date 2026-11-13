// Plain Kotlin. Nothing here may know it runs on Android.
plugins {
    id("org.jetbrains.kotlin.jvm")
    id("org.jetbrains.kotlin.plugin.serialization")
}

// The one folder the generator writes, read twice: once as Kotlin and once as Java.
val generatedTypes = rootDir.resolve("../../platform/contract/generated/types")

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
    // Contract.java lives beside Contract.kt. Registering the folder on the Kotlin source set
    // alone let kotlinc read the Java for resolution and left javac with nothing to compile, so
    // no `contract.Contract` class ever reached the jar and Java call sites could not name the
    // contract at all. Measured 2026-11-13: build/classes/kotlin/main/contract held 786 classes
    // and zero `Contract$*`. The two files do not collide - Contract.kt puts its types at
    // `contract.Tile`, Contract.java nests the same shapes at `contract.Contract$Tile` - so
    // Kotlin callers keep the value classes and Java callers get plain getters. This is the one
    // source folder still; nothing is copied or duplicated.
    sourceSets["main"].java.srcDir(generatedTypes)
}

kotlin {
    compilerOptions { jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17 }
    sourceSets["main"].kotlin.srcDir(generatedTypes)
    sourceSets["main"].kotlin.exclude("**/*.swift", "**/*.ts")
}

dependencies {
    api("org.jetbrains.kotlinx:kotlinx-serialization-json:1.11.0")
    api("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.11.0")
    testImplementation(kotlin("test"))
}

tasks.test { useJUnitPlatform() }
