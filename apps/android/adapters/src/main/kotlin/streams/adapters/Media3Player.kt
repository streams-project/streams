package streams.adapters

import android.content.Context
import android.os.Handler
import android.os.Looper
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player as Media
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.ui.PlayerView
import contract.DriverCommand
import contract.DriverCommandClose
import contract.DriverCommandOpen
import contract.DriverCommandPause
import contract.DriverCommandPlay
import contract.DriverCommandSeek
import contract.DriverCommandVolume
import contract.DriverEvent
import contract.DriverEventEnded
import contract.DriverEventError
import contract.DriverEventFirstFrame
import contract.DriverEventPosition
import contract.DriverEventStalled
import contract.ErrorClass
import streams.ports.Player

/** The player port, on Media3. It reports facts, and the core picks every next step. */
class Media3Player(context: Context) : Player {
    private val main = Handler(Looper.getMainLooper())
    private val http = DefaultHttpDataSource.Factory()
    private val engine = ExoPlayer.Builder(context)
        .setMediaSourceFactory(DefaultMediaSourceFactory(context).setDataSourceFactory(http))
        .build()
    private var report: (Long, DriverEvent) -> Unit = { _, _ -> }
    private var generation = 0L
    private var firstFrame = false

    private val tick = object : Runnable {
        override fun run() {
            if (engine.isPlaying) {
                val length = engine.duration.takeIf { it > 0 }
                report(generation, DriverEventPosition(timeMs = engine.currentPosition, durationMs = length))
            }
            main.postDelayed(this, 1000)
        }
    }

    init {
        engine.addListener(object : Media.Listener {
            override fun onRenderedFirstFrame() {
                if (firstFrame) return
                firstFrame = true
                report(generation, DriverEventFirstFrame)
            }

            override fun onPlaybackStateChanged(state: Int) {
                when (state) {
                    Media.STATE_BUFFERING -> report(generation, DriverEventStalled)
                    Media.STATE_ENDED -> report(generation, DriverEventEnded)
                    else -> Unit
                }
            }

            override fun onPlayerError(error: PlaybackException) {
                val kind = when (error.errorCode) {
                    PlaybackException.ERROR_CODE_DECODING_FAILED -> ErrorClass.DECODE
                    PlaybackException.ERROR_CODE_PARSING_MANIFEST_MALFORMED -> ErrorClass.MANIFEST
                    else -> ErrorClass.NETWORK
                }
                report(generation, DriverEventError(`class` = kind))
            }
        })
        main.post(tick)
    }

    override fun listen(report: (Long, DriverEvent) -> Unit) {
        this.report = report
    }

    override fun picture(context: Any): Any =
        PlayerView(context as Context).apply {
            player = engine
            useController = false
        }

    override fun command(given: DriverCommand) {
        main.post {
            when (given) {
                is DriverCommandOpen -> open(given)
                DriverCommandPlay -> engine.play()
                DriverCommandPause -> engine.pause()
                is DriverCommandSeek -> engine.seekTo(given.toMs)
                is DriverCommandVolume -> engine.volume = given.percent / 100f
                DriverCommandClose -> engine.stop()
                else -> Unit
            }
        }
    }

    private fun open(given: DriverCommandOpen) {
        generation = given.generation
        firstFrame = false
        http.setDefaultRequestProperties(given.headers)
        engine.setMediaItem(MediaItem.fromUri(given.url), given.startAtMs)
        engine.prepare()
        engine.play()
    }
}
