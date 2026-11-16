#include "watch.h"

#include <algorithm>
#include <cstdio>
#include <limits>

namespace streams::watch {
namespace {

const char *const KindRecap = "recap";
const char *const KindIntro = "intro";
const char *const KindCredits = "credits";
const char *const KindAgeCard = "age_card";
const char *const FileMark = "file:";

int clampInt(int value, int least, int most) {
    return std::min(std::max(value, least), most);
}

std::int64_t clampMs(std::int64_t value, std::int64_t least, std::int64_t most) {
    return std::min(std::max(value, least), most);
}

int playingIndex(const std::vector<Track> &tracks) {
    for (std::size_t at = 0; at < tracks.size(); ++at) {
        if (tracks[at].playing()) {
            return static_cast<int>(at);
        }
    }
    return -1;
}

} // namespace

std::string Moment::word(const Words &words) const {
    return key.empty() ? std::string() : lookup(words, key);
}

bool Track::isFile() const { return id.rfind(FileMark, 0) == 0; }

const Window *windowNow(const std::vector<Window> &windows, std::int64_t position_ms,
                        const std::vector<std::string_view> &kinds) {
    for (const Window &window : windows) {
        if (!window.holds(position_ms)) {
            continue;
        }
        for (std::string_view wanted : kinds) {
            if (window.kind == wanted) {
                return &window;
            }
        }
    }
    return nullptr;
}

Moment momentAt(const Play &play, std::int64_t position_ms, bool shown) {
    Moment offered;
    offered.on = !shown;
    offered.raised = shown;
    if (play.state == State::Failed) {
        return offered;
    }
    // `Watch.kt:257-258, 488-493`, `Watch.swift:209-210, 400-403`: an intro or recap window takes
    // precedence over credits, and credits are offered only when a next episode exists.
    const Window *part = windowNow(play.windows, position_ms, {KindIntro, KindRecap});
    if (part != nullptr) {
        if (part->kind == KindIntro) {
            offered.kind = MomentKind::SkipIntro;
            offered.key = "player.skip_intro";
        } else {
            offered.kind = MomentKind::SkipRecap;
            offered.key = "player.skip_recap";
        }
        offered.seek_to_ms = part->targetMs();
        return offered;
    }
    if (!play.has_next_episode) {
        return offered;
    }
    const Window *closing = windowNow(play.windows, position_ms, {KindCredits});
    if (closing != nullptr) {
        offered.kind = MomentKind::NextEpisode;
        offered.key = "player.next";
        offered.next_episode = play.next_episode;
    }
    return offered;
}

std::string ageAt(const Play &play, std::int64_t position_ms) {
    if (play.state == State::Failed) {
        return std::string();
    }
    const Window *card = windowNow(play.windows, position_ms, {KindAgeCard});
    return card == nullptr ? std::string() : card->text;
}

const char *controlKey(Control control) {
    switch (control) {
        case Control::Back: return "player.back";
        case Control::Restart: return "player.restart";
        case Control::Next: return "player.next";
    }
    return "player.back";
}

const char *rateKey(Rate rate) {
    switch (rate) {
        case Rate::Down: return "player.down";
        case Rate::Up: return "player.up";
        case Rate::Love: return "player.love";
    }
    return "player.up";
}

const char *transportKey(bool playing) { return playing ? "say.pause" : "common.play"; }

const char *stillKey(int stay) { return stay == 0 ? "player.still.yes" : "player.still.no"; }

std::string lookup(const Words &words, std::string_view key) {
    const auto found = words.find(std::string(key));
    return found == words.end() ? std::string() : found->second;
}

std::string fill(const std::string &pattern, std::string_view hole, const std::string &value) {
    const std::string token = "{" + std::string(hole) + "}";
    std::string out = pattern;
    std::size_t at = 0;
    while ((at = out.find(token, at)) != std::string::npos) {
        out.replace(at, token.size(), value);
        at += value.size();
    }
    return out;
}

std::string clock(std::int64_t ms) {
    const std::int64_t clamped = std::max<std::int64_t>(0, ms);
    const std::int64_t total_seconds = clamped / 1000;
    const long long hours = static_cast<long long>(total_seconds / 3600);
    const long long minutes = static_cast<long long>((total_seconds / 60) % 60);
    const long long seconds = static_cast<long long>(total_seconds % 60);
    char buffer[64];
    if (hours > 0) {
        std::snprintf(buffer, sizeof buffer, "%lld:%02lld:%02lld", hours, minutes, seconds);
    } else {
        std::snprintf(buffer, sizeof buffer, "%lld:%02lld", minutes, seconds);
    }
    return std::string(buffer);
}

std::string nameOf(const Track &track) {
    if (!track.label.empty()) {
        return track.label;
    }
    if (!track.lang.empty()) {
        return track.lang;
    }
    return track.id;
}

std::vector<Track> oneOfEach(const std::vector<Track> &tracks) {
    std::vector<std::string> order;
    std::map<std::string, std::vector<Track>> groups;
    for (const Track &track : tracks) {
        const std::string &key = track.lang.empty() ? track.id : track.lang;
        if (groups.find(key) == groups.end()) {
            order.push_back(key);
        }
        groups[key].push_back(track);
    }
    std::vector<Track> out;
    out.reserve(order.size());
    for (const std::string &key : order) {
        const std::vector<Track> &same = groups[key];
        const int active = playingIndex(same);
        out.push_back(same[static_cast<std::size_t>(active >= 0 ? active : 0)]);
    }
    return out;
}

std::vector<Track> linesOf(const std::vector<Track> &text, const std::vector<Track> &files) {
    std::vector<Track> own = oneOfEach(text);
    const bool file_chosen = playingIndex(files) >= 0;
    if (file_chosen) {
        for (Track &line : own) {
            line.selected = false;
        }
    }
    std::vector<Track> out = own;
    for (const Track &file : files) {
        bool already = false;
        for (const Track &stream_line : own) {
            if (stream_line.lang == file.lang) {
                already = true;
                break;
            }
        }
        if (!already) {
            out.push_back(file);
        }
    }
    return out;
}

std::vector<Choice> choicesOf(const std::vector<Track> &audio, const std::vector<Track> &text,
                              const std::vector<Track> &files, const std::string &language,
                              const Words &words) {
    const std::vector<Track> sounds = oneOfEach(audio);
    const std::vector<Track> lines = linesOf(text, files);
    if (sounds.size() < 2 && lines.empty()) {
        return {};
    }
    if (sounds.empty()) {
        return {};
    }
    const int sound_index = playingIndex(sounds);
    const Track &sound = sounds[static_cast<std::size_t>(sound_index >= 0 ? sound_index : 0)];
    const int reading_index = playingIndex(lines);
    const Track *reading =
        reading_index >= 0 ? &lines[static_cast<std::size_t>(reading_index)] : nullptr;
    const std::string name = nameOf(sound);

    const Track *same = nullptr;
    for (const Track &line : lines) {
        if (!sound.lang.empty() && line.lang == sound.lang) {
            same = &line;
            break;
        }
    }
    if (same == nullptr) {
        for (const Track &line : lines) {
            if (!language.empty() && line.lang == language) {
                same = &line;
                break;
            }
        }
    }

    std::vector<Choice> out;
    Choice bare;
    bare.word = name;
    bare.has_audio = true;
    bare.audio = sound;
    bare.ticked = (reading == nullptr);
    out.push_back(bare);

    if (same != nullptr) {
        Choice paired;
        paired.word = fill(lookup(words, "player.with_subtitles"), "audio", name);
        paired.has_audio = true;
        paired.audio = sound;
        paired.has_text = true;
        paired.text = *same;
        paired.ticked = (reading != nullptr && reading->id == same->id);
        out.push_back(paired);
    }

    const bool elsewhere = reading != nullptr && (same == nullptr || reading->id != same->id);
    if (sounds.size() > 1 || lines.size() > 1) {
        Choice other;
        other.word = lookup(words, "player.other");
        other.ticked = elsewhere;
        other.other = true;
        out.push_back(other);
    }
    return out;
}

// ------------------------------------------------------------------------------- the track panel

void TrackPanel::show(const std::vector<Track> &audio) {
    open_ = true;
    column_ = 0;
    line_ = std::max(0, playingIndex(oneOfEach(audio)));
}

Did TrackPanel::press(Word word, const std::vector<Track> &audio, const std::vector<Track> &text,
                      const std::vector<Track> &files, Track &chosen_audio, bool &chose_audio,
                      Track &chosen_text, bool &has_text, bool &chose_text) {
    const std::vector<Track> sounds = oneOfEach(audio);
    const std::vector<Track> lines = linesOf(text, files);
    const int last = column_ == 0 ? static_cast<int>(sounds.size()) - 1
                                  : static_cast<int>(lines.size());
    switch (word) {
        case Word::Back:
            open_ = false;
            return Did::Moved;
        case Word::Up:
            if (line_ > 0) {
                line_ -= 1;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Down:
            if (line_ < last) {
                line_ += 1;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Left:
            // `TrackPanel.swift:29-36`: from the subtitles column Left returns to the sounds
            // column; from the sounds column Left closes the panel.
            if (column_ == 1) {
                column_ = 0;
                line_ = std::max(0, playingIndex(sounds));
                return Did::Moved;
            }
            open_ = false;
            return Did::Moved;
        case Word::Right:
            if (column_ == 0) {
                column_ = 1;
                line_ = playingIndex(lines) + 1;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Ok:
            if (column_ == 0) {
                if (line_ < 0 || static_cast<std::size_t>(line_) >= sounds.size()) {
                    return Did::Nothing;
                }
                chosen_audio = sounds[static_cast<std::size_t>(line_)];
                chose_audio = true;
                return Did::Track;
            }
            chose_text = true;
            if (line_ <= 0 || static_cast<std::size_t>(line_ - 1) >= lines.size()) {
                has_text = false;
                chosen_text = Track();
                return Did::Track;
            }
            has_text = true;
            chosen_text = lines[static_cast<std::size_t>(line_ - 1)];
            return Did::Track;
        default:
            return Did::Nothing;
    }
}

std::string TrackPanel::saying(const std::vector<Track> &audio, const std::vector<Track> &text,
                               const std::vector<Track> &files, const Words &words) const {
    if (column_ == 0) {
        const std::vector<Track> sounds = oneOfEach(audio);
        if (line_ >= 0 && static_cast<std::size_t>(line_) < sounds.size()) {
            return nameOf(sounds[static_cast<std::size_t>(line_)]);
        }
        return std::string();
    }
    if (line_ == 0) {
        return lookup(words, "player.subtitles_off");
    }
    const std::vector<Track> lines = linesOf(text, files);
    if (line_ > 0 && static_cast<std::size_t>(line_ - 1) < lines.size()) {
        return nameOf(lines[static_cast<std::size_t>(line_ - 1)]);
    }
    return std::string();
}

// ------------------------------------------------------------------------------- the player

Watch::Watch(const Play &play) : play_(play) {}

void Watch::arrived(const Play &play) {
    const bool same = play.title == play_.title && play.episode == play_.episode;
    const bool was_paused = paused();
    play_ = play;
    if (!same) {
        has_aim_ = false;
        panel_.close();
    }
    if (paused() != was_paused) {
        shown_ = true;
        if (!paused() && zone_ == Reach::Rate) {
            zone_ = Reach::Bar;
        }
    }
}

void Watch::tick(std::int64_t position_ms, std::int64_t length_ms) {
    position_ms_ = position_ms;
    length_ms_ = length_ms;
}

void Watch::tracks(const std::vector<Track> &audio, const std::vector<Track> &text,
                   const std::vector<Track> &files, const std::string &language) {
    audio_ = audio;
    text_ = text;
    files_ = files;
    language_ = language;
}

void Watch::hide() {
    // `Watch.kt:268-272`, `Watch.swift:413-417`: a paused player keeps its controls up; a playing
    // one drops them after `ShownMs` and resets the zone to the bar so the next press starts on
    // the transport (or on a `TimeWindow` moment pill when one is active).
    if (!paused()) {
        shown_ = false;
        zone_ = Reach::Bar;
    }
}

std::vector<Control> Watch::controls() const {
    std::vector<Control> out = {Control::Back, Control::Restart};
    if (play_.has_next_episode) {
        out.push_back(Control::Next);
    }
    return out;
}

int Watch::top() const {
    const std::vector<Control> row = controls();
    return clampInt(top_, 0, static_cast<int>(row.size()) - 1);
}

void Watch::setRating(bool has_rating, Rate rating) {
    has_rating_ = has_rating;
    rating_ = rating;
}

void Watch::seekBy(std::int64_t delta_ms) {
    const std::int64_t end =
        length_ms_ > 0 ? length_ms_ : std::numeric_limits<std::int64_t>::max();
    const std::int64_t base = has_aim_ ? aim_ms_ : position_ms_;
    aim_ms_ = clampMs(base + delta_ms, 0, end);
    has_aim_ = true;
    seek_to_ms_ = aim_ms_;
}

Did Watch::press(Word word, const Words &words) {
    chose_audio_ = false;
    chose_text_ = false;
    has_chosen_text_ = false;

    if (panel_.open()) {
        return panel_.press(word, audio_, text_, files_, chosen_audio_, chose_audio_, chosen_text_,
                            has_chosen_text_, chose_text_);
    }

    if (word == Word::Back) {
        return Did::Went;
    }

    if (asking()) {
        switch (word) {
            case Word::Left:
                if (stay_ == 0) {
                    return Did::Nothing;
                }
                stay_ = 0;
                return Did::Moved;
            case Word::Right:
                if (stay_ == 1) {
                    return Did::Nothing;
                }
                stay_ = 1;
                return Did::Moved;
            case Word::Ok:
                if (stay_ == 0 && play_.has_next_episode) {
                    next_episode_ = play_.next_episode;
                    return Did::Next;
                }
                return Did::Went;
            default:
                return Did::Nothing;
        }
    }

    const bool was_hidden = !shown_;
    shown_ = true;
    if (failed()) {
        return was_hidden ? Did::Moved : Did::Nothing;
    }

    // `Watch.kt:372-379`, `Watch.swift:276-279`: when the controls were hidden and the remote is
    // on the bar, OK activates the active `TimeWindow` moment pill (`SkipIntro` / `SkipRecap` ->
    // `ActionSeek`, `NextEpisode` -> `goNext()`) before falling back to play/pause.
    if (word == Word::Ok && zone_ == Reach::Bar && was_hidden) {
        const Moment active = momentAt(play_, position_ms_, false);
        if (active.kind == MomentKind::SkipIntro || active.kind == MomentKind::SkipRecap) {
            seek_to_ms_ = active.seek_to_ms;
            return Did::Seek;
        }
        if (active.kind == MomentKind::NextEpisode) {
            next_episode_ = active.next_episode;
            return Did::Next;
        }
        return Did::Toggle;
    }

    const std::vector<Choice> choices = choicesOf(audio_, text_, files_, language_, words);
    switch (zone_) {
        case Reach::Bar:
            switch (word) {
                case Word::Ok:
                    return Did::Toggle;
                case Word::Left:
                    seekBy(-SeekMs);
                    return Did::Seek;
                case Word::Right:
                    seekBy(SeekMs);
                    return Did::Seek;
                case Word::Up:
                    if (!was_hidden) {
                        zone_ = paused() ? Reach::Rate : Reach::Top;
                        return Did::Moved;
                    }
                    return Did::Moved;
                case Word::Down:
                    if (!was_hidden && !choices.empty()) {
                        zone_ = Reach::Tracks;
                        track_ = 0;
                        for (std::size_t at = 0; at < choices.size(); ++at) {
                            if (choices[at].ticked) {
                                track_ = static_cast<int>(at);
                                break;
                            }
                        }
                        return Did::Moved;
                    }
                    return was_hidden ? Did::Moved : Did::Nothing;
                default:
                    return Did::Nothing;
            }
        case Reach::Tracks:
            switch (word) {
                case Word::Left:
                    if (track_ > 0) {
                        track_ -= 1;
                        return Did::Moved;
                    }
                    return Did::Nothing;
                case Word::Right:
                    if (track_ + 1 < static_cast<int>(choices.size())) {
                        track_ += 1;
                        return Did::Moved;
                    }
                    return Did::Nothing;
                case Word::Up:
                    zone_ = Reach::Bar;
                    return Did::Moved;
                case Word::Ok: {
                    if (track_ < 0 || static_cast<std::size_t>(track_) >= choices.size()) {
                        return Did::Nothing;
                    }
                    const Choice &choice = choices[static_cast<std::size_t>(track_)];
                    if (choice.other) {
                        panel_.show(audio_);
                        return Did::Moved;
                    }
                    if (choice.has_audio && !choice.audio.playing()) {
                        chose_audio_ = true;
                        chosen_audio_ = choice.audio;
                    }
                    chose_text_ = true;
                    has_chosen_text_ = choice.has_text;
                    chosen_text_ = choice.has_text ? choice.text : Track();
                    return Did::Track;
                }
                default:
                    return Did::Nothing;
            }
        case Reach::Rate:
            switch (word) {
                case Word::Left:
                    if (rate_ > 0) {
                        rate_ -= 1;
                        return Did::Moved;
                    }
                    return Did::Nothing;
                case Word::Right:
                    if (rate_ < Rates - 1) {
                        rate_ += 1;
                        return Did::Moved;
                    }
                    return Did::Nothing;
                case Word::Up:
                    zone_ = Reach::Top;
                    return Did::Moved;
                case Word::Down:
                    zone_ = Reach::Bar;
                    return Did::Moved;
                case Word::Ok: {
                    const Rate asked = static_cast<Rate>(rate_);
                    if (has_rating_ && rating_ == asked) {
                        has_rating_ = false;
                    } else {
                        has_rating_ = true;
                        rating_ = asked;
                    }
                    return Did::Rated;
                }
                default:
                    return Did::Nothing;
            }
        case Reach::Top: {
            const std::vector<Control> row = controls();
            const int at = top();
            switch (word) {
                case Word::Left:
                    if (at > 0) {
                        top_ = at - 1;
                        return Did::Moved;
                    }
                    return Did::Nothing;
                case Word::Right:
                    if (at + 1 < static_cast<int>(row.size())) {
                        top_ = at + 1;
                        return Did::Moved;
                    }
                    return Did::Nothing;
                case Word::Down:
                    zone_ = paused() ? Reach::Rate : Reach::Bar;
                    return Did::Moved;
                case Word::Ok:
                    switch (row[static_cast<std::size_t>(at)]) {
                        case Control::Back:
                            return Did::Went;
                        case Control::Restart:
                            seek_to_ms_ = 0;
                            return Did::Seek;
                        case Control::Next:
                            next_episode_ = play_.next_episode;
                            return Did::Next;
                    }
                    return Did::Nothing;
                default:
                    return Did::Nothing;
            }
        }
    }
    return Did::Nothing;
}

std::string Watch::saying(const Words &words) const {
    if (panel_.open()) {
        return panel_.saying(audio_, text_, files_, words);
    }
    if (asking()) {
        return lookup(words, stillKey(stay_));
    }
    if (failed()) {
        return !play_.notice.empty() ? play_.notice : lookup(words, "error.no_source");
    }
    if (!shown_) {
        const Moment active = moment();
        if (active.active()) {
            return active.word(words);
        }
        return play_.name;
    }
    const std::vector<Control> row = controls();
    switch (zone_) {
        case Reach::Top:
            return lookup(words, controlKey(row[static_cast<std::size_t>(top())]));
        case Reach::Rate:
            return lookup(words, rateKey(static_cast<Rate>(rate_)));
        case Reach::Tracks: {
            const std::vector<Choice> choices = choicesOf(audio_, text_, files_, language_, words);
            if (track_ >= 0 && static_cast<std::size_t>(track_) < choices.size()) {
                return choices[static_cast<std::size_t>(track_)].word;
            }
            return std::string();
        }
        case Reach::Bar:
            return lookup(words, transportKey(!paused()));
    }
    return std::string();
}

} // namespace streams::watch
