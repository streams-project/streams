package streams.tv

import contract.Track
import java.util.Locale
import streams.ports.Seen

/** One choice under the bar: a sound, with or without subtitles, or the way to the full lists. */
class Choice(val word: String, val audio: Track?, val text: Track?, val ticked: Boolean, val other: Boolean = false)

/** True for the track in play. */
val Track.playing: Boolean get() = selected == true

/** What a viewer reads for a track: the name the source gave it, or its language. */
fun nameOf(track: Track, language: String): String {
    track.label?.takeIf { it.isNotBlank() }?.let { return it }
    val tag = track.lang ?: return track.id
    return Locale.forLanguageTag(tag).getDisplayLanguage(Locale.forLanguageTag(language)).ifEmpty { tag }
}

/** One line per language, keeping the track in play when several share a language. */
fun oneOfEach(tracks: List<Track>): List<Track> =
    tracks.groupBy { it.lang ?: it.id }.values.map { same -> same.firstOrNull { it.playing } ?: same.first() }

/** The short list: this sound alone, this sound with its subtitles, and everything else. */
fun choicesOf(seen: Seen): List<Choice> {
    val sounds = oneOfEach(seen.audio)
    val lines = oneOfEach(seen.text)
    if (sounds.size < 2 && lines.isEmpty()) return emptyList()
    val sound = sounds.firstOrNull { it.playing } ?: sounds.firstOrNull() ?: return emptyList()
    val reading = lines.firstOrNull { it.playing }
    val name = nameOf(sound, seen.language)
    val same = lines.firstOrNull { it.lang == sound.lang } ?: lines.firstOrNull { it.lang == seen.language }
    return buildList {
        add(Choice(name, sound, null, reading == null))
        same?.let {
            val word = seen.words["player.with_subtitles"].orEmpty().replace("{audio}", name)
            add(Choice(word, sound, it, reading?.id == it.id))
        }
        val elsewhere = reading != null && reading.id != same?.id
        if (sounds.size > 1 || lines.size > 1) {
            add(Choice(seen.words["player.other"].orEmpty(), null, null, elsewhere, other = true))
        }
    }
}
