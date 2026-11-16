// Where the viewer is on the player, what a `TimeWindow` offers at this moment of the film, and
// what each of the six words does from there.
//
// The sixth key map on this surface that needs no Windows. `place.h` is a page of rows, `title.h`
// is a title's three pages, `gate.h` is the profile gate, `door.h` is the landing and sign in, and
// `keys.h` is the shared keyboard. This is the player: the transport bar, the top row of icons,
// the three rating pills shown while paused, the strip of audio and subtitle choices under the bar,
// the full two-column track panel, the "Are you still watching?" prompt, and the moment pill
// (`TimeWindow`) that offers to skip a recap, skip an intro, or move on to the next episode over
// the closing credits.
//
// No Win2D, no WinRT, no `MediaPlayer`, no swap chain, no controller. The video frame server and
// the 3,000 ms hide timer belong to the host above; what a press decides and what window is active
// at a millisecond of playback are pure state and arithmetic here.
//
// The twins, file for file:
//
//     windowNow, Moment   watch/Moments.kt, watch/Moments.swift
//     oneOfEach, linesOf  watch/Choices.kt, watch/Choices.swift
//     choicesOf           watch/Choices.kt:33-52, watch/Choices.swift:51-69
//     TrackPanel          watch/TrackPanel.kt, watch/TrackPanel.swift
//     Watch               watch/Watch.kt, watch/Watch.swift
//
// Where the two television twins disagree, this file says so on the line:
//
//   1. **`TrackPanel.kt:46` and `TrackPanel.swift:34` disagree about Left in the audio column.**
//      In the Swift, Left from the subtitles column returns to the audio column and Left from the
//      audio column closes the panel, matching `Episodes` and `TitlePage` where Left at the left
//      edge goes back. In the Kotlin, Left in the audio column does nothing at all, so only Back
//      leaves the panel. This port takes the Swift's answer.
//   2. **Both twins index `controls[top]` without clamping (`Watch.kt:415`, `Watch.swift:322`).**
//      `controls` has three items when `next_episode` is present and two when it is not. If a fresh
//      `PlayView` drops `next_episode` while the remote is standing on `Next` (`top == 2`), OK in
//      either twin reads past the end of `controls`. `Watch::top()` clamps into `controls()`, for
//      the same reason `Summary::line()` clamps into `lines_`.
//   3. **`Watch.kt:354` answers `false` on Back and `Watch.swift:260` leaves the player itself.**
//      On Xbox nothing above the surface implements Back for it (`SystemNavigationManager`
//      requires the app to handle `BackRequested`), so `Watch::press` answers `Did::Went` when the
//      track panel is closed, and closes the panel first when it is open.
//
// It holds no English literals. Every label key (`player.skip_intro`, `player.skip_recap`,
// `player.next`, `player.back`, `player.restart`, `player.down`, `player.up`, `player.love`,
// `player.still`, `player.still.yes`, `player.still.no`, `player.audio`, `player.subtitles`,
// `player.subtitles_off`, `player.other`, `player.with_subtitles`, `say.pause`, `common.play`,
// `say.position`, `error.no_source`) is a key of `strings/*.json`, resolved against the `Words`
// table handed in from the active language file.
#ifndef STREAMS_XBOX_WATCH_H
#define STREAMS_XBOX_WATCH_H

#include <cstdint>
#include <map>
#include <string>
#include <string_view>
#include <vector>

#include "remote.h"

namespace streams::watch {

using streams::remote::Word;

// A language file's key-to-translation table (`strings/<lang>.json`).
using Words = std::map<std::string, std::string>;

// How long the controls stay up while playing before hiding themselves. `Watch.kt:72`.
constexpr std::int64_t ShownMs = 3000;

// How far one press of Left or Right on the bar moves the film. `Watch.kt:73`.
constexpr std::int64_t SeekMs = 10000;

// How long the bar waits for another press before seeking, and for the picture to catch up.
// `Watch.kt:76-77`.
constexpr std::int64_t SeekSettleMs = 350;
constexpr std::int64_t SeekCatchUpMs = 1500;

// How long an ended film waits before leaving, when not asking if the viewer is still there.
// `Watch.kt:80`.
constexpr std::int64_t EndedWaitMs = 800;

// How far subtitles rise as a share of the picture's height when controls are shown.
// `Watch.kt:83-84`.
constexpr float WordsLow = 0.0f;
constexpr float WordsHigh = 0.2f;

// Where the moment pill sits from the bottom of the 1080p reference canvas: raised above the bar
// when controls are shown, and resting lower when they are hidden. `Moments.kt:42`.
constexpr int MomentRaisedBottom = 230;
constexpr int MomentRestingBottom = 96;

// The states of `contract::PlayState`.
enum class State : std::uint8_t {
    Resolving,
    Starting,
    Playing,
    Paused,
    Stalled,
    Switching,
    Ended,
    Failed,
};

// Which band of the player the remote is on. `Watch.kt:86`, `Watch.swift:17`.
enum class Reach : std::uint8_t { Top, Rate, Bar, Tracks };

// The icons at the top left of the player. `Watch.kt:87`, `Watch.swift:18`.
enum class Control : std::uint8_t { Back, Restart, Next };

// The three rating pills shown while paused, in left-to-right order. `Watch.kt:120-124`.
enum class Rate : std::uint8_t { Down, Up, Love };

constexpr int Rates = 3;
constexpr int FirstRate = 1;

// Season and episode numbers, as `contract::EpisodeRef` carries them.
struct Episode {
    std::int64_t season = 0;
    std::int64_t episode = 0;

    bool operator==(const Episode &other) const {
        return season == other.season && episode == other.episode;
    }
    bool operator!=(const Episode &other) const { return !(*this == other); }
};

// A span of a video the surface compares to player time. The twin of `contract::TimeWindow`.
struct Window {
    // `contract::WindowKind`: `recap`, `intro`, `credits`, `age_card`, or `cue`.
    std::string kind;
    std::int64_t start_ms = 0;
    std::int64_t end_ms = 0;
    bool has_seek_to = false;
    std::int64_t seek_to_ms = 0;
    std::string text;

    // True when `position_ms` falls inside `[start_ms, end_ms)`. `Moments.kt:34`, `Moments.swift:9`.
    bool holds(std::int64_t position_ms) const {
        return position_ms >= start_ms && position_ms < end_ms;
    }

    // Where skipping this window seeks to: `seek_to_ms` when present, or `end_ms`.
    // `Watch.kt:374`, `Watch.swift:277`.
    std::int64_t targetMs() const { return has_seek_to ? seek_to_ms : end_ms; }
};

// What kind of moment pill is offered at this millisecond of playback.
enum class MomentKind : std::uint8_t {
    None,
    SkipIntro,
    SkipRecap,
    NextEpisode,
};

// The pill offered at a moment of the film: skip the intro, skip the recap, or go on to the next
// episode. `Moments.kt:36-52`, `Moments.swift:13-34`, `Watch.kt:257-260, 372-379, 488-495`.
struct Moment {
    MomentKind kind = MomentKind::None;

    // The key in `strings/*.json`: `"player.skip_intro"`, `"player.skip_recap"`, `"player.next"`,
    // or empty when no pill is offered.
    std::string key;

    // Where OK seeks when `kind` is `SkipIntro` or `SkipRecap`.
    std::int64_t seek_to_ms = 0;

    // Which episode OK opens when `kind` is `NextEpisode`.
    Episode next_episode;

    // True when the pill is lit and takes OK (`!shown`). `Moments.kt:44`, `Watch.kt:494`.
    bool on = false;

    // True when the controls are shown and the pill is lifted to `MomentRaisedBottom`.
    bool raised = false;

    bool active() const { return kind != MomentKind::None; }

    int bottomRef() const { return raised ? MomentRaisedBottom : MomentRestingBottom; }

    // The pill's label from the active language table.
    std::string word(const Words &words) const;
};

// One audio or subtitle track, from the stream or from a subtitle file on the server.
// `Choices.kt:11-18`, `SideWords.kt:39-40`.
struct Track {
    std::string id;
    std::string label;
    std::string lang;
    bool selected = false;

    bool playing() const { return selected; }
    bool isFile() const;
};

// One chip in the strip under the bar: a sound alone, that sound with subtitles, or `Other…`.
// `Choices.kt:9`.
struct Choice {
    std::string word;
    bool has_audio = false;
    Track audio;
    bool has_text = false;
    Track text;
    bool ticked = false;
    bool other = false;
};

// The playback state a screen reads. The flattened twin of `contract::PlayView`.
struct Play {
    std::string title;
    std::string name;
    State state = State::Resolving;
    bool has_episode = false;
    Episode episode;
    bool has_next_episode = false;
    Episode next_episode;
    bool still_asking = false;
    std::vector<Window> windows;
    std::string notice;
};

// What a press on the player did.
enum class Did : std::uint8_t {
    // The press reached an edge, or the player is in a failed state and ignores transport keys.
    Nothing,
    // Focus moved on the controls or inside the track panel, or hidden controls woke up.
    Moved,
    // The viewer left the player (`ActionStop` and back).
    Went,
    // Play/pause toggled (`ActionPause` when playing, `ActionUnpause` when paused).
    Toggle,
    // A seek was requested: from a skip-intro or skip-recap `TimeWindow`, from Start Over, or from
    // Left/Right on the bar. `seekToMs()` says where.
    Seek,
    // The next episode was chosen: from a credits `TimeWindow`, from the top Next icon, or from
    // Continue Watching on the still-watching prompt. `nextEpisode()` says which.
    Next,
    // A rating pill was pressed while paused. `hasRating()` and `rating()`.
    Rated,
    // An audio or subtitle choice was picked from the strip or the full panel.
    Track,
};

// The first window in `windows` whose `kind` is in `kinds` and that holds `position_ms`, or
// `nullptr`. `Moments.kt:33-34`, `Moments.swift:8-10`.
const Window *windowNow(const std::vector<Window> &windows, std::int64_t position_ms,
                        const std::vector<std::string_view> &kinds);

// The moment pill offered for `play` at `position_ms`, with `on = !shown` and `raised = shown`.
// `Watch.kt:257-258, 486-495`, `Watch.swift:209-210, 398-404`.
Moment momentAt(const Play &play, std::int64_t position_ms, bool shown);

// The age rating card's text when `position_ms` is inside an `age_card` window and playback has
// not failed, or empty. `Watch.kt:259, 487`, `Watch.swift:211, 399`.
std::string ageAt(const Play &play, std::int64_t position_ms);

// The `strings/*.json` key for a top control, a rating pill, the transport button, or a
// still-watching choice. `Watch.kt:94-98, 109, 120-124, 441`.
const char *controlKey(Control control);
const char *rateKey(Rate rate);
const char *transportKey(bool playing);
const char *stillKey(int stay);

// Looks up `key` in `words`, or answers empty when the key is absent.
std::string lookup(const Words &words, std::string_view key);

// Replaces `{hole}` with `value` in `pattern`.
std::string fill(const std::string &pattern, std::string_view hole, const std::string &value);

// Formats a millisecond position as `m:ss` or `h:mm:ss`. `Watch.kt:126-130`, `Watch.swift:42-46`.
std::string clock(std::int64_t ms);

// What a viewer reads for a track: its label, its language tag, or its id. `Choices.kt:15-18`.
std::string nameOf(const Track &track);

// One track per language (or id when `lang` is empty), keeping the playing track when several
// share a language. `Choices.kt:21-22`, `Choices.swift:29-38`.
std::vector<Track> oneOfEach(const std::vector<Track> &tracks);

// The stream's own subtitle tracks, followed by server subtitle files for languages the stream
// lacks. When a server file is playing, the stream's own tracks are deselected. `Choices.kt:25-30`.
std::vector<Track> linesOf(const std::vector<Track> &text, const std::vector<Track> &files);

// The short list of choices under the bar: this sound alone, this sound with subtitles, and
// `Other…` when more than one sound or more than one subtitle language exists. `Choices.kt:33-52`.
std::vector<Choice> choicesOf(const std::vector<Track> &audio, const std::vector<Track> &text,
                              const std::vector<Track> &files, const std::string &language,
                              const Words &words);

// The full two-column audio and subtitle panel (`TrackPanel.kt`, `TrackPanel.swift`).
class TrackPanel {
public:
    bool open() const { return open_; }
    int column() const { return column_; }
    int line() const { return line_; }

    void show(const std::vector<Track> &audio);
    void close() { open_ = false; }

    Did press(Word word, const std::vector<Track> &audio, const std::vector<Track> &text,
              const std::vector<Track> &files, Track &chosen_audio, bool &chose_audio,
              Track &chosen_text, bool &has_text, bool &chose_text);

    // What the remote is on inside the panel, for a screen reader. `TrackPanel.kt:69-74`.
    std::string saying(const std::vector<Track> &audio, const std::vector<Track> &text,
                       const std::vector<Track> &files, const Words &words) const;

private:
    bool open_ = false;
    int column_ = 0;
    int line_ = 0;
};

// The player state machine. `Watch.kt:230-501`, `Watch.swift:181-462`.
class Watch {
public:
    explicit Watch(const Play &play);

    // A fresh `PlayView` from the core.
    void arrived(const Play &play);

    // Updates the player clock (`position_ms` and `length_ms`).
    void tick(std::int64_t position_ms, std::int64_t length_ms);

    // Updates the stream's audio and subtitle tracks and the server's sidecar subtitle files.
    void tracks(const std::vector<Track> &audio, const std::vector<Track> &text,
                const std::vector<Track> &files, const std::string &language);

    // Called when `ShownMs` elapses without a press: hides the controls and returns focus to the
    // bar when playing; keeps the controls up while paused. `Watch.kt:266-273`.
    void hide();

    // Clears the accumulated seek aim once `SeekCatchUpMs` has elapsed. `Watch.kt:305-311`.
    void settleSeek() { has_aim_ = false; }

    const Play &play() const { return play_; }
    std::int64_t positionMs() const { return position_ms_; }
    std::int64_t lengthMs() const { return length_ms_; }
    std::int64_t displayedPositionMs() const { return has_aim_ ? aim_ms_ : position_ms_; }

    bool paused() const { return play_.state == State::Paused; }
    bool playing() const { return play_.state == State::Playing; }
    bool failed() const { return play_.state == State::Failed; }
    bool asking() const { return play_.still_asking; }

    bool shown() const { return shown_; }
    float wordsLift() const { return shown_ ? WordsHigh : WordsLow; }

    Reach zone() const { return zone_; }

    // The top row of controls: Back and Start Over always, and Next Episode when `next_episode`
    // is present. `Watch.kt:261-265`.
    std::vector<Control> controls() const;

    // Clamped into `controls()`.
    int top() const;
    int rate() const { return rate_; }
    int stay() const { return stay_; }
    int track() const { return track_; }

    bool hasRating() const { return has_rating_; }
    Rate rating() const { return rating_; }
    void setRating(bool has_rating, Rate rating);

    const TrackPanel &panel() const { return panel_; }
    TrackPanel &panel() { return panel_; }

    // The active `TimeWindow` moment pill (`SkipIntro`, `SkipRecap`, or `NextEpisode`), or
    // `MomentKind::None` when no window covers `position_ms_` or playback has failed.
    Moment moment() const { return momentAt(play_, position_ms_, shown_); }

    // The localized label for the active moment pill from `words`, or empty when none is active.
    std::string momentWord(const Words &words) const { return moment().word(words); }

    // The active age card text when `position_ms_` is inside an `age_card` window, or empty.
    std::string age() const { return ageAt(play_, position_ms_); }

    // What the last `Did::Seek` asked to seek to, in milliseconds.
    std::int64_t seekToMs() const { return seek_to_ms_; }

    // Which episode the last `Did::Next` asked to open.
    const Episode &nextEpisode() const { return next_episode_; }

    // What the last `Did::Track` selected.
    bool choseAudio() const { return chose_audio_; }
    const Track &chosenAudio() const { return chosen_audio_; }
    bool choseText() const { return chose_text_; }
    bool hasChosenText() const { return has_chosen_text_; }
    const Track &chosenText() const { return chosen_text_; }

    // One word from the remote. Note: the player keeps left-to-right in every language because its
    // timeline and seek keys follow the film rather than the reading direction (`Watch.kt:452`).
    Did press(Word word, const Words &words = Words());

    // What the remote is on now, for a screen reader. `Watch.kt:439-448`.
    std::string saying(const Words &words) const;

private:
    void seekBy(std::int64_t delta_ms);

    Play play_;
    std::int64_t position_ms_ = 0;
    std::int64_t length_ms_ = 0;
    bool shown_ = true;
    Reach zone_ = Reach::Bar;
    int top_ = 0;
    int rate_ = FirstRate;
    bool has_rating_ = false;
    Rate rating_ = Rate::Up;
    int stay_ = 0;
    int track_ = 0;
    bool has_aim_ = false;
    std::int64_t aim_ms_ = 0;
    std::int64_t seek_to_ms_ = 0;
    Episode next_episode_;

    std::vector<Track> audio_;
    std::vector<Track> text_;
    std::vector<Track> files_;
    std::string language_ = "en";

    TrackPanel panel_;
    bool chose_audio_ = false;
    Track chosen_audio_;
    bool chose_text_ = false;
    bool has_chosen_text_ = false;
    Track chosen_text_;
};

} // namespace streams::watch

#endif // STREAMS_XBOX_WATCH_H
