package streams.adapters

import android.content.Context
import android.os.Handler
import android.os.Looper
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player as Media
import androidx.media3.common.TrackSelectionOverride
import androidx.media3.common.Tracks
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.ui.PlayerView
import contract.DriverCommand
import contract.DriverCommandClose
import contract.DriverCommandOpen
import contract.DriverCommandPause
import contract.DriverCommandPlay
import contract.DriverCommandRate
import contract.DriverCommandSeek
import contract.DriverCommandSelectAudio
import contract.DriverCommandSelectText
import contract.DriverCommandVolume
import contract.DriverEvent
import contract.DriverEventEnded
import contract.DriverEventError
import contract.DriverEventFirstFrame
import contract.DriverEventPosition
import contract.DriverEventStalled
import contract.DriverEventTracks
import contract.ErrorClass
import contract.Track
import streams.ports.Player

private const val LIFT_MS = 240L

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
    private var view: PlayerView? = null

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

            override fun onTracksChanged(tracks: Tracks) {
                report(generation, DriverEventTracks(audio = listed(tracks, C.TRACK_TYPE_AUDIO), text = listed(tracks, C.TRACK_TYPE_TEXT)))
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
            view = this
        }

    override fun lift(share: Float) {
        val words = view?.subtitleView ?: return
        words.animate().translationY(-share * words.height).setDuration(LIFT_MS).start()
    }

    override fun command(given: DriverCommand) {
        main.post {
            when (given) {
                is DriverCommandOpen -> open(given)
                DriverCommandPlay -> engine.play()
                DriverCommandPause -> engine.pause()
                is DriverCommandSeek -> engine.seekTo(given.toMs)
                is DriverCommandVolume -> engine.volume = given.percent / 100f
                is DriverCommandRate -> engine.setPlaybackSpeed(given.percent / 100f)
                is DriverCommandSelectAudio -> choose(C.TRACK_TYPE_AUDIO, given.track)
                is DriverCommandSelectText -> choose(C.TRACK_TYPE_TEXT, given.track)
                DriverCommandClose -> engine.stop()
                else -> Unit
            }
        }
    }

    /** A track's id is its place in the engine's list, so it holds only until the list changes. */
    private fun listed(tracks: Tracks, type: Int): List<Track> =
        tracks.groups.flatMapIndexed { group, found ->
            if (found.type != type) return@flatMapIndexed emptyList()
            (0 until found.length).filter(found::isTrackSupported).map { index ->
                val format = found.getTrackFormat(index)
                Track(id = "$group:$index", label = format.label, lang = format.language, selected = found.isTrackSelected(index))
            }
        }

    private fun choose(type: Int, id: String?) {
        val change = engine.trackSelectionParameters.buildUpon().clearOverridesOfType(type)
        change.setTrackTypeDisabled(type, id == null)
        if (id != null) {
            val (group, index) = id.split(':').map(String::toInt)
            val found = engine.currentTracks.groups.getOrNull(group) ?: return
            change.setOverrideForType(TrackSelectionOverride(found.mediaTrackGroup, index))
        }
        engine.trackSelectionParameters = change.build()
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
