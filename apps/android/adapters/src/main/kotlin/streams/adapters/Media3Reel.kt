package streams.adapters

import android.content.Context
import android.graphics.Matrix
import android.view.TextureView
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player as Media
import androidx.media3.common.VideoSize
import androidx.media3.exoplayer.ExoPlayer
import streams.ports.Reel

/**
 * Brand films and sounds, each on its own engine so it can be handed back as soon as it is off
 * screen. A film draws into the app's own picture (a texture), so the screens can move over it every
 * frame without the system compositing a second layer; on a television that halves the frame rate.
 */
class Media3Reel : Reel {
    /** Every engine, and whether it was asked to play. */
    private val wanted = mutableMapOf<ExoPlayer, Boolean>()
    private var still = false

    override fun picture(context: Any, url: String, loop: Boolean, ready: () -> Unit, ended: () -> Unit): Any {
        val engine = ExoPlayer.Builder(context as Context).build()
        val view = TextureView(context)
        // Until the first frame, what is behind the film shows through.
        view.isOpaque = false
        view.tag = engine
        engine.repeatMode = if (loop) Media.REPEAT_MODE_ONE else Media.REPEAT_MODE_OFF
        engine.setVideoTextureView(view)
        engine.addListener(object : Media.Listener {
            private var told = false

            override fun onPlaybackStateChanged(state: Int) {
                if (state == Media.STATE_READY && !told) {
                    told = true
                    ready()
                }
                if (state == Media.STATE_ENDED) ended()
            }

            override fun onVideoSizeChanged(size: VideoSize) = fill(view, size)

            override fun onPlayerError(error: PlaybackException) = ended()
        })
        view.addOnLayoutChangeListener { _, _, _, _, _, _, _, _, _ -> fill(view, engine.videoSize) }
        engine.setMediaItem(MediaItem.fromUri(url))
        engine.prepare()
        engine.playWhenReady = false
        wanted[engine] = false
        return view
    }

    /** Scales the film to cover its box, cutting what spills over, as a card shows it. */
    private fun fill(view: TextureView, size: VideoSize) {
        if (size.width == 0 || size.height == 0 || view.width == 0 || view.height == 0) return
        val film = size.width * size.pixelWidthHeightRatio / size.height
        val box = view.width.toFloat() / view.height
        val wide = if (film > box) film / box else 1f
        val high = if (film > box) 1f else box / film
        view.setTransform(Matrix().apply { setScale(wide, high, view.width / 2f, view.height / 2f) })
    }

    override fun play(picture: Any, on: Boolean) {
        val engine = (picture as TextureView).tag as? ExoPlayer ?: return
        wanted[engine] = on
        engine.playWhenReady = on && !still
    }

    override fun release(picture: Any) {
        val view = picture as TextureView
        val engine = view.tag as? ExoPlayer ?: return
        wanted.remove(engine)
        engine.release()
        view.tag = null
    }

    override fun hold(still: Boolean) {
        this.still = still
        wanted.forEach { (engine, on) -> engine.playWhenReady = on && !still }
    }
}
