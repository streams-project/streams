// The Android family: one hexagon. `ports` is plain Kotlin with no Android in it. `surfaces` sees
// only `ports`, so a screen cannot reach a player or the network even by accident. `app` is the one
// place that plugs adapters into ports.
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "streams"
include(":ports", ":core", ":adapters", ":surfaces:shared", ":surfaces:phone", ":surfaces:tv", ":app")
