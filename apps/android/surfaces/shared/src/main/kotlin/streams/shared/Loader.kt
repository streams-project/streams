package streams.shared

import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.memory.MemoryCache
import coil3.request.ImageRequest
import coil3.size.Precision
import java.util.concurrent.atomic.AtomicBoolean
import okio.Path.Companion.toOkioPath

/**
 * One loader for every picture on the phone.
 *
 * Without this each screen that wanted a bitmap of its own built an `ImageLoader` where it stood, so
 * the gate downloaded artwork the rows already held, and nothing was kept between runs: no memory
 * cache was sized and no disk cache was configured at all.
 *
 * It is installed from `phone/app/Root.kt`, so every `Art` and `AsyncImage` the phone surface draws
 * reads through it. The television surface of this same APK has its own entry (`Main.kt` calls
 * `Television()` without passing through `Root`) and does not install it, so that surface still runs
 * on the toolkit's default loader. Installing it there is the television surface's change to make.
 */

/**
 * What the pictures may keep on disk. The reference's phone Home is 23 rows of up to 70 titles
 * (`netflix-reference/00-gaps.md:10,41`), which is 1,610 poster slots at the w342 this project
 * serves tiles from (`server/crates/server-rules/src/answer.rs`). What one of those posters weighs
 * has not been measured here, so this budget is a round 64 MB and not an arithmetic result.
 */
private const val DISK_BYTES = 64L * 1024 * 1024

/**
 * What they may keep in memory: a quarter of the app's heap, which is the toolkit's own habit for a
 * screen that scrolls through artwork. The decoded weight of a 119 x 166.6 dp tile has not been
 * measured here either; the share is a budget, not a calculation.
 */
private const val MEMORY_SHARE = 0.25

/**
 * Points every picture on the phone at one loader, once per process.
 *
 * `setSafe` is not the no-op its name suggests: it ignores a second call only while nothing has
 * asked for the singleton yet, and throws `IllegalStateException` once anything has. Moving this
 * call from the surface's composable body into a `SideEffect` was tried on 2026-11-11 and crashed
 * the app on its first frame, because side effects are dispatched after the composition they
 * belong to has composed -- by which time an `AsyncImage` has already asked Coil for the loader.
 * So it is called from the body, before any picture composes, and the flag keeps it to one call
 * whatever recomposes.
 */
fun installLoader() {
    if (installed.compareAndSet(false, true)) SingletonImageLoader.setSafe { context -> loader(context) }
}

private val installed = AtomicBoolean(false)

private fun loader(context: PlatformContext): ImageLoader = ImageLoader.Builder(context)
    .memoryCache { MemoryCache.Builder().maxSizePercent(context, MEMORY_SHARE).build() }
    .diskCache {
        DiskCache.Builder()
            .directory(context.cacheDir.resolve("art").toOkioPath())
            .maxSizeBytes(DISK_BYTES)
            .build()
    }
    .build()

/**
 * A picture asked for at the size of the box it will fill.
 *
 * The size is in the request and not left to the layout for two reasons: the request starts before
 * the first measure instead of a frame after it, and a tile read ahead of the viewer lands under the
 * same cache key as the tile that will be drawn, so reading ahead is worth something. `INEXACT` lets
 * the decoder take the nearest sample size, so a w342 poster is not re-encoded to fit exactly.
 */
fun sized(context: PlatformContext, url: String, wide: Int, high: Int): ImageRequest =
    ImageRequest.Builder(context)
        .data(url)
        .size(wide, high)
        .precision(Precision.INEXACT)
        .build()

/**
 * Reads a picture into the cache before anything draws it, at the size the tile will ask for.
 *
 * The reference's rows are sometimes already painted the instant a fling stops -- one of five
 * measured flings filled in 0.0 s -- because it fetched them before the viewer arrived.
 */
fun warm(context: PlatformContext, url: String?, wide: Int, high: Int) {
    if (url.isNullOrEmpty()) return
    SingletonImageLoader.get(context).enqueue(sized(context, url, wide, high))
}
