// How does the remote operate the player, and what does a `TimeWindow` offer at each moment of
// playback?
//
// Scripted key sequences and clock ticks against `watch.h`, with all user-facing strings read out
// of `strings/en.json` through `Contract.hpp`'s JSON parser so no English label is hardcoded here
// or in `watch.cpp`.
//
//     c++ -std=c++17 -Wall -Wextra -I platform/contract/generated/types \
//         apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/watch.cpp \
//         apps/xbox/surfaces/remote/watch_test.cpp -o watch && ./watch strings/en.json
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "Contract.hpp"
#include "watch.h"

using streams::remote::Word;
using streams::watch::Choice;
using streams::watch::Control;
using streams::watch::Did;
using streams::watch::Episode;
using streams::watch::Moment;
using streams::watch::MomentKind;
using streams::watch::Play;
using streams::watch::Rate;
using streams::watch::Reach;
using streams::watch::State;
using streams::watch::Track;
using streams::watch::Watch;
using streams::watch::Window;
using streams::watch::Words;

namespace {

int checks = 0;
int failures = 0;

void ok(bool held, const char *what) {
    checks += 1;
    std::printf("%s %s\n", held ? "ok  " : "FAIL", what);
    if (!held) {
        failures += 1;
    }
}

std::string slurp(const char *path) {
    std::ifstream file(path);
    if (!file) {
        std::printf("FAIL cannot read %s\n", path);
        failures += 1;
        return std::string();
    }
    std::ostringstream all;
    all << file.rdbuf();
    return all.str();
}

Words readWords(const char *path) {
    const std::string raw = slurp(path);
    if (raw.empty()) {
        return {};
    }
    const contract::Json::Value doc = contract::Json::parse(raw);
    return contract::Json::map<std::string>(doc, [](const contract::Json::Value &v) {
        return contract::Json::as_text(v);
    });
}

Word wordOf(char key) {
    switch (key) {
        case 'U': return Word::Up;
        case 'D': return Word::Down;
        case 'L': return Word::Left;
        case 'R': return Word::Right;
        case 'O': return Word::Ok;
        default: return Word::Back;
    }
}

Did press(Watch &player, const char *script, const Words &words = Words()) {
    Did did = Did::Nothing;
    for (const char *key = script; *key != '\0'; ++key) {
        did = player.press(wordOf(*key), words);
    }
    return did;
}

Window window(const char *kind, std::int64_t start_ms, std::int64_t end_ms,
              std::int64_t seek_to_ms = -1, const char *text = "") {
    Window made;
    made.kind = kind;
    made.start_ms = start_ms;
    made.end_ms = end_ms;
    if (seek_to_ms >= 0) {
        made.has_seek_to = true;
        made.seek_to_ms = seek_to_ms;
    }
    made.text = text;
    return made;
}

Play seriesPlay() {
    Play made;
    made.title = "tv:42";
    made.name = "Signal";
    made.state = State::Playing;
    made.has_episode = true;
    made.episode = Episode{1, 2};
    made.has_next_episode = true;
    made.next_episode = Episode{1, 3};
    made.windows = {
        window("age_card", 0, 5000, -1, "15"),
        window("recap", 5000, 25000),
        window("intro", 30000, 90000, 92000),
        window("credits", 1200000, 1260000),
    };
    return made;
}

Track trackOf(const char *id, const char *lang, const char *label, bool selected) {
    Track made;
    made.id = id;
    made.lang = lang;
    made.label = label;
    made.selected = selected;
    return made;
}

} // namespace

int main(int argc, char **argv) {
    const char *stringsPath = argc > 1 ? argv[1] : "strings/en.json";
    const Words book = readWords(stringsPath);

    // --------------------------------------------------- every player key exists in strings/en.json
    {
        ok(!book.empty(), "strings/en.json was found and parsed");
        const char *const keys[] = {
            "common.play",
            "error.no_source",
            "player.audio",
            "player.back",
            "player.down",
            "player.hint",
            "player.love",
            "player.next",
            "player.other",
            "player.restart",
            "player.skip_intro",
            "player.skip_recap",
            "player.still",
            "player.still.no",
            "player.still.yes",
            "player.subtitles",
            "player.subtitles_off",
            "player.up",
            "player.with_subtitles",
            "say.chosen",
            "say.pause",
            "say.position",
        };
        bool all_present = true;
        for (const char *key : keys) {
            if (streams::watch::lookup(book, key).empty()) {
                all_present = false;
            }
        }
        ok(all_present, "every string key the player uses is present and non-empty in strings/en.json");
    }

    // --------------------------------------------------------- TimeWindow skip marks and boundaries
    {
        Watch player(seriesPlay());
        player.tick(2000, 1260000);
        ok(player.age() == "15", "an age_card window shows its rating text while playback is inside it");
        ok(!player.moment().active() && player.momentWord(book).empty(),
           "before the recap or intro starts, no skip pill is offered");

        // At 5,000 ms the recap window begins (start_ms is inclusive).
        player.tick(5000, 1260000);
        ok(player.age().empty(), "the age_card window hides at its exclusive end_ms");
        Moment recap = player.moment();
        ok(recap.kind == MomentKind::SkipRecap && recap.key == "player.skip_recap",
           "inside a recap TimeWindow the moment key is player.skip_recap");
        ok(player.momentWord(book) == book.at("player.skip_recap"),
           "and its label comes straight from strings/en.json");
        ok(recap.seek_to_ms == 25000,
           "a recap window with no seek_to_ms seeks to its own end_ms");
        ok(!recap.on && recap.raised && recap.bottomRef() == streams::watch::MomentRaisedBottom,
           "while the transport controls are shown, the pill is raised to 230 ref px and not lit");
        ok(player.press(Word::Ok, book) == Did::Toggle,
           "so OK while controls are shown toggles play/pause rather than skipping");

        // When the controls hide after 3,000 ms, the pill drops to 96 ref px and takes OK.
        player.hide();
        recap = player.moment();
        ok(!player.shown() && recap.on && !recap.raised &&
               recap.bottomRef() == streams::watch::MomentRestingBottom,
           "once the controls hide, the pill drops to 96 ref px and lights up");
        ok(player.saying(book) == book.at("player.skip_recap"),
           "and a screen reader is told the skip pill's word");
        ok(player.press(Word::Ok, book) == Did::Seek && player.seekToMs() == 25000,
           "OK while hidden inside a recap window seeks to the end of the recap");
        ok(player.shown(), "and wakes the controls");

        // The end boundary of a TimeWindow is exclusive.
        player.tick(25000, 1260000);
        player.hide();
        ok(!player.moment().active() && player.momentWord(book).empty(),
           "at the recap's end_ms the window has ended and no pill is offered");
        ok(player.saying(book) == "Signal",
           "with controls hidden and no active window, the screen reader hears the title's name");
        ok(player.press(Word::Ok, book) == Did::Toggle,
           "and OK while hidden outside any window toggles play/pause");

        // Inside the intro window (30,000..90,000, with seek_to_ms = 92,000).
        player.tick(30000, 1260000);
        player.hide();
        Moment intro = player.moment();
        ok(intro.kind == MomentKind::SkipIntro && intro.key == "player.skip_intro",
           "at the intro window's start_ms the moment key is player.skip_intro");
        ok(player.momentWord(book) == book.at("player.skip_intro"),
           "with the Skip Intro label from strings/en.json");
        ok(intro.seek_to_ms == 92000,
           "and when seek_to_ms is set on the TimeWindow, the skip target uses seek_to_ms");
        ok(player.press(Word::Ok, book) == Did::Seek && player.seekToMs() == 92000,
           "OK while hidden inside the intro window seeks to seek_to_ms");

        player.tick(90000, 1260000);
        player.hide();
        ok(!player.moment().active(), "at 90,000 ms the intro window has ended");

        // Inside the credits window (1,200,000..1,260,000) when a next episode exists.
        player.tick(1210000, 1260000);
        player.hide();
        Moment closing = player.moment();
        ok(closing.kind == MomentKind::NextEpisode && closing.key == "player.next" &&
               player.momentWord(book) == book.at("player.next"),
           "inside a credits TimeWindow with a next episode, the pill offers player.next");
        ok(player.press(Word::Ok, book) == Did::Next &&
               player.nextEpisode() == (Episode{1, 3}),
           "and OK while hidden advances to that next episode");
    }

    // -------------------------------------- credits without a next episode, and a failed play state
    {
        Play finale = seriesPlay();
        finale.has_next_episode = false;
        Watch player(finale);
        player.tick(1210000, 1260000);
        player.hide();
        ok(!player.moment().active(),
           "a credits TimeWindow on the last episode (or a film) offers no pill");
        ok(player.press(Word::Ok, book) == Did::Toggle,
           "so OK during credits on a finale pauses or unpauses instead");

        Play broken = seriesPlay();
        broken.state = State::Failed;
        Watch failedPlayer(broken);
        failedPlayer.tick(45000, 1260000);
        failedPlayer.hide();
        ok(!failedPlayer.moment().active() && failedPlayer.age().empty(),
           "a failed player offers neither a skip pill nor an age card");
        ok(failedPlayer.saying(book) == book.at("error.no_source"),
           "and speaks error.no_source when no server notice is set");
        ok(failedPlayer.press(Word::Ok, book) == Did::Moved &&
               failedPlayer.press(Word::Ok, book) == Did::Nothing,
           "OK on a failed player wakes the screen and never seeks or toggles");
        ok(failedPlayer.press(Word::Back, book) == Did::Went,
           "and Back still leaves a failed player");
    }

    // ------------------------------------------------------------- clock, transport bar, and seeks
    {
        ok(streams::watch::clock(0) == "0:00" && streams::watch::clock(65000) == "1:05",
           "positions under an hour format as m:ss");
        ok(streams::watch::clock(3605000) == "1:00:05" &&
               streams::watch::clock(7384000) == "2:03:04",
           "and positions an hour or longer format as h:mm:ss");

        Watch player(seriesPlay());
        player.tick(41000, 634000);
        ok(player.zone() == Reach::Bar && player.shown() &&
               player.wordsLift() == streams::watch::WordsHigh,
           "the player opens on the bar with controls shown and subtitles lifted");
        ok(player.saying(book) == book.at("say.pause"),
           "while playing, the transport control is named by say.pause");

        ok(player.press(Word::Right, book) == Did::Seek && player.seekToMs() == 51000,
           "Right on the bar seeks forward 10 seconds");
        ok(player.press(Word::Right, book) == Did::Seek && player.seekToMs() == 61000 &&
               player.displayedPositionMs() == 61000,
           "a second Right before the picture catches up adds onto the aimed position");
        player.settleSeek();
        ok(player.displayedPositionMs() == 41000,
           "and once settled, the bar displays the player's reported position again");
        ok(press(player, "LLLLL", book) == Did::Seek && player.seekToMs() == 0,
           "seeking left past zero clamps at 0 ms");
    }

    // --------------------------------------------------- top controls, and a Next control that goes
    {
        Watch player(seriesPlay());
        ok(player.controls().size() == 3,
           "a series episode with a next episode has Back, Start Over, and Next Episode");
        ok(press(player, "U", book) == Did::Moved && player.zone() == Reach::Top &&
               player.top() == 0,
           "Up from a playing bar reaches the first top control (Back)");
        ok(player.saying(book) == book.at("player.back"), "which is named by player.back");
        ok(press(player, "R", book) == Did::Moved && player.top() == 1 &&
               player.saying(book) == book.at("player.restart"),
           "Right reaches Start Over (player.restart)");
        ok(player.press(Word::Ok, book) == Did::Seek && player.seekToMs() == 0,
           "and OK on Start Over seeks to 0 ms");
        ok(press(player, "R", book) == Did::Moved && player.top() == 2 &&
               player.saying(book) == book.at("player.next"),
           "Right again reaches Next Episode (player.next)");
        ok(press(player, "R", book) == Did::Nothing && player.top() == 2,
           "Right at the last top control stays");
        ok(player.press(Word::Ok, book) == Did::Next &&
               player.nextEpisode() == (Episode{1, 3}),
           "and OK on Next Episode advances to the next episode");

        // If a fresh PlayView drops next_episode while the remote is on Next, top() clamps.
        Play noNext = seriesPlay();
        noNext.has_next_episode = false;
        player.arrived(noNext);
        ok(player.controls().size() == 2 && player.top() == 1,
           "when next_episode disappears under the remote, top() clamps to Start Over");
        ok(player.press(Word::Ok, book) == Did::Seek && player.seekToMs() == 0,
           "so OK presses Start Over rather than indexing past the end of controls");
        ok(press(player, "LO", book) == Did::Went, "and Left then OK on Back leaves the player");
    }

    // ----------------------------------------------------------------- rating pills while paused
    {
        Play paused = seriesPlay();
        paused.state = State::Paused;
        Watch player(paused);
        ok(player.saying(book) == book.at("common.play"),
           "while paused, the transport control is named by common.play");
        player.hide();
        ok(player.shown(), "a paused player never hides its controls when the timer expires");

        ok(press(player, "U", book) == Did::Moved && player.zone() == Reach::Rate &&
               player.rate() == streams::watch::FirstRate,
           "Up from a paused bar reaches the middle rating pill (player.up)");
        ok(player.saying(book) == book.at("player.up"), "which says player.up");
        ok(press(player, "L", book) == Did::Moved && player.rate() == 0 &&
               player.saying(book) == book.at("player.down"),
           "Left reaches Not for me (player.down)");
        ok(press(player, "RR", book) == Did::Moved && player.rate() == 2 &&
               player.saying(book) == book.at("player.love"),
           "Right walks across to Love this! (player.love)");
        ok(player.press(Word::Ok, book) == Did::Rated && player.hasRating() &&
               player.rating() == Rate::Love,
           "OK sets the rating under the remote");
        ok(player.press(Word::Ok, book) == Did::Rated && !player.hasRating(),
           "and OK again on the same rating clears it");
        ok(press(player, "U", book) == Did::Moved && player.zone() == Reach::Top,
           "Up from the rating pills reaches the top controls");
        ok(press(player, "D", book) == Did::Moved && player.zone() == Reach::Rate,
           "and Down from the top controls while paused returns to the rating pills");
    }

    // -------------------------------------------------------------- the "still watching?" prompt
    {
        Play prompt = seriesPlay();
        prompt.still_asking = true;
        Watch player(prompt);
        ok(player.asking() && player.stay() == 0 &&
               player.saying(book) == book.at("player.still.yes"),
           "still_asking opens on Continue Watching (player.still.yes)");
        ok(player.press(Word::Ok, book) == Did::Next &&
               player.nextEpisode() == (Episode{1, 3}),
           "OK on Continue Watching starts the next episode");
        ok(press(player, "R", book) == Did::Moved && player.stay() == 1 &&
               player.saying(book) == book.at("player.still.no"),
           "Right moves to I'm Done (player.still.no)");
        ok(player.press(Word::Ok, book) == Did::Went,
           "and OK on I'm Done stops and leaves the player");
    }

    // ---------------------------------------------------- the track strip and the full track panel
    {
        Watch player(seriesPlay());
        const std::vector<Track> audio = {
            trackOf("a-en", "en", "English", true),
            trackOf("a-ja", "ja", "Japanese", false),
        };
        const std::vector<Track> text = {
            trackOf("t-en", "en", "English", false),
        };
        const std::vector<Track> files = {
            trackOf("file:es", "es", "Spanish", false),
        };
        player.tracks(audio, text, files, "en");

        const std::vector<Choice> strip =
                streams::watch::choicesOf(audio, text, files, "en", book);
        ok(strip.size() == 3 && strip[0].ticked && !strip[1].ticked && strip[2].other,
           "two audio languages and two subtitle languages build three strip choices");
        ok(strip[1].word == streams::watch::fill(book.at("player.with_subtitles"), "audio", "English") &&
               strip[2].word == book.at("player.other"),
           "using player.with_subtitles and player.other from strings/en.json");

        ok(press(player, "D", book) == Did::Moved && player.zone() == Reach::Tracks &&
               player.track() == 0,
           "Down from the bar enters the track strip on the ticked choice");
        ok(press(player, "RO", book) == Did::Track && player.choseText() &&
               player.hasChosenText() && player.chosenText().id == "t-en",
           "Right then OK picks the paired subtitle track");
        ok(press(player, "RO", book) == Did::Moved && player.panel().open() &&
               player.panel().column() == 0 && player.panel().line() == 0,
           "Right then OK on Other opens the full track panel on the playing audio track");
        ok(press(player, "DO", book) == Did::Track && player.choseAudio() &&
               player.chosenAudio().id == "a-ja",
           "Down then OK in the audio column selects the second audio language");
        ok(press(player, "R", book) == Did::Moved && player.panel().column() == 1 &&
               player.panel().line() == 0 &&
               player.saying(book) == book.at("player.subtitles_off"),
           "Right moves to the subtitles column on Off (player.subtitles_off)");
        ok(press(player, "DDO", book) == Did::Track && player.choseText() &&
               player.hasChosenText() && player.chosenText().id == "file:es" &&
               player.chosenText().isFile(),
           "Down twice then OK picks the server sidecar subtitle track");
        ok(press(player, "L", book) == Did::Moved && player.panel().column() == 0 &&
               player.panel().open(),
           "Left in the subtitles column returns to the audio column");
        ok(press(player, "L", book) == Did::Moved && !player.panel().open(),
           "and Left in the audio column closes the panel, matching TrackPanel.swift");
    }

    if (failures > 0) {
        std::printf("\n%d of %d failed\n", failures, checks);
        return 1;
    }
    std::printf("\n%d of %d held\n", checks, checks);
    return 0;
}
