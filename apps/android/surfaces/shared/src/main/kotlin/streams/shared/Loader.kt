package streams.shared

import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.memory.MemoryCache
import coil3.request.ImageRequest
import coil3.size.Precision
import okio.Path.Companion.toOkioPath

/**
 * One loader for every picture in the app.
 *
 * Without this each screen that wanted a bitmap of its own built an `ImageLoader` where it stood, so
 * the gate downloaded artwork the rows already held, and nothing was kept between runs: no memory
 * cache was sized and no disk cache was configured at all.
 *
 * It is installed once, from the one place that draws a surface, and every `Art` and `AsyncImage`
 * in the app reads through it.
 */

/**
 * What the pictures may keep on disk. The reference's phone Home is 23 rows of up to 70 titles,
 * which is 1,610 poster slots; a poster is served at w342 and weighs about 40 kB, so 64 MB holds a
 * whole page of them and a title page besides.
 */
private const val DISK_BYTES = 64L * 1024 * 1024

/**
 * What they may keep in memory. One screen of this surface holds a billboard and about four rows of
 * four visible tiles; a tile decoded at 119 x 166.6 dp is around half a megabyte, so a screen is
 * about 9 MB and a quarter of the app's heap holds several screens of travel.
 */
private const val MEMORY_SHARE = 0.25

/** Points every picture in the app at one loader. A second call is ignored, which is why it is safe. */
fun installLoader() {
    SingletonImageLoader.setSafe { context -> loader(context) }
}

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
