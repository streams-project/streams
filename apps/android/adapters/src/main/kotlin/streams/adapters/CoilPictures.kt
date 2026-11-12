// How Android reads a picture, and the one file of this family that may name an image library.
package streams.adapters

import android.content.Context
import coil3.SingletonImageLoader
import coil3.request.ImageRequest
import coil3.request.allowHardware
import coil3.size.Precision
import coil3.toBitmap
import streams.ports.Painted
import streams.ports.Picture
import streams.ports.Pictures
import streams.ports.Stop
import streams.ports.Unpainted
import streams.ports.Want

/**
 * The [Pictures] port over Coil.
 *
 * Coil stays, because Coil is a good image loader and what was missing was not a better one: it was
 * a boundary. Before this, fifteen surface files named `coil3` themselves and three surface modules
 * declared `coil-network-okhttp`, which is how 78 `okhttp3` and 89 `okio` classes reached the APK
 * while the ring check's ban on the word `okhttp3` passed -- the check read surface source, and a
 * dependency is not a word anybody types. The port is the permanent part: it outlives the toolkit
 * the surfaces are written in, so the Views rewrite has a boundary to aim at instead of this hole
 * to reproduce in Java.
 *
 * What is not here yet, and is one line in two files the lead owns: the process's single loader is
 * still built and installed by `surfaces/shared/Loader.kt`, because that install is what gives this
 * app its 64 MB disk cache and its memory budget, and moving it here without `app/Main.kt`
 * constructing this class would take both away between the two commits. So this class uses the
 * one loader of the process rather than building a second one -- two loaders mean two memory
 * caches, which is the defect `Loader.kt` was written to close. The handoff is in the report.
 */
class CoilPictures(context: Context) : Pictures {
    // The application's context, not the activity's: a request outlives the screen that asked.
    private val app = context.applicationContext

    override fun fetch(want: Want, done: (Picture) -> Unit): Stop {
        val held = loader().enqueue(
            ask(want)
                .listener(
                    onSuccess = { _, result -> done(Painted(result.image.toBitmap())) },
                    onError = { _, result ->
                        done(Unpainted(result.throwable.message ?: result.throwable::class.java.simpleName))
                    },
                )
                .build(),
        )
        return Stop { held.dispose() }
    }

    override fun warm(want: Want, done: () -> Unit) {
        loader().enqueue(
            ask(want)
                .listener(onSuccess = { _, _ -> done() }, onError = { _, _ -> done() })
                .build(),
        )
    }

    /**
     * One request, built the way the surfaces built theirs.
     *
     * `INEXACT` lets the decoder take the nearest sample size, so a w342 poster is not re-encoded
     * to fit a box exactly. A size of 0 is left out of the request, which is Coil's own default of
     * "as large as the picture is".
     *
     * `allowHardware(false)` is set only for a caller that said it will read the pixels back. A
     * hardware bitmap lives on the graphics card and reading its pixels throws, so the tint on the
     * phone's billboard and the one behind the profile gate both need an ordinary one; everything
     * else is drawn and never read, and keeps the cheaper picture.
     */
    private fun ask(want: Want): ImageRequest.Builder = ImageRequest.Builder(app)
        .data(want.url)
        .precision(Precision.INEXACT)
        .apply {
            if (want.wide > 0 && want.high > 0) size(want.wide, want.high)
            if (want.pixels) allowHardware(false)
        }

    /**
     * The one loader of this process, with its memory and disk caches.
     *
     * Asked for per request rather than held: Coil keeps the singleton itself, and whoever installs
     * it has to have done so before the first picture is asked for, which is a rule about startup
     * order and not about this class. [Painted.picture] is an `android.graphics.Bitmap`, which is
     * what both of today's callers want, and what a `BitmapDrawable` wraps in one line for a caller
     * that only draws it.
     */
    private fun loader() = SingletonImageLoader.get(app)
}
