plugins {
    id("com.android.application") version "9.3.1" apply false
    id("com.android.library") version "9.3.1" apply false
    id("org.jetbrains.kotlin.jvm") version "2.3.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.3.21" apply false
    id("org.jetbrains.kotlin.plugin.serialization") version "2.3.21" apply false
}

// Every unit test here reads repository files at run time rather than taking them as test
// resources: the module's own sources as text, the word files under `strings/`, the brand, and
// the measured reference. Gradle cannot see a file a test opens through `File(...)`, so none of
// them is an input and the test task stays up to date after one changes -- it then reports the
// previous run's result, which is a pass that proves nothing.
//
// Measured on 2026-11-12: three separate mutations of `reference/netflix-on-the-tv.md` -- a tile
// size, the stated tolerance and the page margin -- were each reported green by
// `:surfaces:tv:test`. Forced to rerun, the same three failed 1, 4 and 1 tests. Declaring the
// files below makes a change to any of them rerun the tests that read it.
subprojects {
    tasks.withType<Test>().configureEach {
        val repo = rootProject.layout.projectDirectory.dir("../..")
        inputs.file(repo.file("reference/netflix-on-the-tv.md"))
            .withPropertyName("measuredReference")
            .withPathSensitivity(PathSensitivity.RELATIVE)
        inputs.dir(repo.dir("strings"))
            .withPropertyName("words")
            .withPathSensitivity(PathSensitivity.RELATIVE)
        inputs.file(repo.file("brand.json"))
            .withPropertyName("brand")
            .withPathSensitivity(PathSensitivity.RELATIVE)
    }
}
