package streams.adapters

import android.content.Context
import android.graphics.Color
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player as Media
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.PlayerView
import streams.ports.Reel

/** Brand films, each on its own engine so it can be handed back as soon as it is off screen. */
class Media3Reel : Reel {
    override fun picture(context: Any, url: String, loop: Boolean, ended: () -> Unit): Any {
        val engine = ExoPlayer.Builder(context as Context).build()
        engine.repeatMode = if (loop) Media.REPEAT_MODE_ONE else Media.REPEAT_MODE_OFF
        engine.addListener(object : Media.Listener {
            override fun onPlaybackStateChanged(state: Int) {
                if (state == Media.STATE_ENDED) ended()
            }

            override fun onPlayerError(error: PlaybackException) = ended()
        })
        engine.setMediaItem(MediaItem.fromUri(url))
        engine.prepare()
        engine.playWhenReady = true
        return PlayerView(context).apply {
            player = engine
            useController = false
            // Until the first frame, what is behind the film shows through.
            setShutterBackgroundColor(Color.TRANSPARENT)
            setBackgroundColor(Color.TRANSPARENT)
        }
    }

    override fun release(picture: Any) {
        val view = picture as PlayerView
        view.player?.release()
        view.player = null
    }
}
