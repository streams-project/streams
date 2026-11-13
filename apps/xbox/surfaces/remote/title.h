// Where the viewer is on a title's own pages, and what each of the six words does from there.
//
// The third thing on this surface that needs no Windows. `place.h` answers it for a page of rows;
// this answers it for the page a row opens into -- the summary with its menu and its three
// reactions, the episodes of a series, and the grid of titles like this one. None of it touches
// Win2D, WinRT, a swap chain or a controller: it is a state machine over a flattened title, built
// and checked on whatever machine builds the tree.
//
// The twins, file for file. There is no Java twin here: the Android television surface converted
// its browse package to Views and left the title's pages in Compose, so the Kotlin is the twin and
// the Swift is the second opinion.
//
//     Summary   title/TitlePage.kt:98-171     the menu, the reactions, and what OK does
//     Episodes  title/Episodes.kt:131-180     the season column and the episode cards
//     Posters   title/MoreLike.kt:28-46       the grid, and Left at the first column
//     gridMove  title/PosterGrid.kt:36-43     one step in a grid of four across
//     Title     title/TitlePage.kt:87-96      which of the three is showing, and the kept line
//
// and beside each, `apps/apple/Sources/SurfacesTV/title/`, which is the same key map in Swift.
// Where the two disagree this file says so on the line, because twice they do and once it matters.
//
// The key map is not re-derived here. It is note 187's "Title page" and "Episodes" rows and the
// Kotlin's own branches, translated branch for branch and in the same order.
//
// It holds no words a viewer reads, and no wire word either. A rating travels as `Face`, a place
// in a row of three, exactly as `place.h` keeps a badge's kind rather than its English: the
// contract's words for the three are `Rating::down`, `Rating::up` and `Rating::love`
// (`Contract.hpp:824-826`) and turning one into the other belongs where the contract is read.
#ifndef STREAMS_XBOX_TITLE_H
#define STREAMS_XBOX_TITLE_H

#include <cstdint>
#include <string>
#include <vector>

#include "place.h"
#include "remote.h"

namespace streams::title {

using streams::place::Feed;
using streams::remote::Word;

// What a press did. The shell above reads it and acts; nothing here knows an address.
//
// `place.h` splits this the other way -- `press` answers whether anything moved and `choose`
// answers OK -- and it can, because on a page of rows OK never changes where the viewer is. Here
// it does: OK on the Episodes line opens a second page, OK on a reaction leaves a rating on the
// title, OK on My List is a write. So the one answer carries both, and OK is a press like any
// other.
enum class Did : std::uint8_t {
    // The press reached an edge. Nothing moved, nothing opened, nothing was written.
    Nothing,
    // The remote is somewhere else on this page.
    Moved,
    // This page is done and the shell goes back.
    //
    // On Xbox that is the surface's own job and nothing above it will do it: when an app handles
    // neither KeyDown nor KeyUp for B, `SystemNavigationManager::BackRequested` fires and
    // Microsoft's own words are "which should result in back navigation within the app. However,
    // you have to implement this yourself". The Kotlin can answer `false` and let the host's back
    // dispatcher have it (`TitlePage.kt:168`); the Swift already calls `shell.back()` itself
    // (`TitlePage.swift:155`), and that is the shape this port takes.
    Went,
    // A rating was left on the title, or the one that was there was taken off. `rating()`.
    Rated,
    // The title went into My List or came out of it. `inList()`.
    Listed,
    // Something is to be played. `watching()` says what.
    Watch,
    // Another title's page opens. `opened()` says which.
    Opened,
};

// The three ratings, in the order the row draws them. `title/Reactions.kt:21-25`.
enum class Face : std::uint8_t { Down, Up, Love };

// How many ratings the row offers. `Reactions.kt:28`'s `ReactionCount`, which is the size of that
// list and not a number written beside it.
constexpr int Faces = 3;

// Which rating the row opens on. `TitlePage.kt:109` and `TitlePage.swift:96`: the middle one.
constexpr int FirstFace = 1;

// One season of a series, as the title's own answer carries it. `contract::SeasonSummary`.
struct Season {
    std::int64_t number = 0;
    std::int64_t episode_count = 0;
};

// One episode of the season in hand. `contract::Episode`, cut to what a key map needs.
struct Episode {
    std::int64_t number = 0;
    std::string name;
};

// A title's page, flattened. The twin of `contract::TitleDetail` as the three screens read it.
struct Detail {
    // The title's key as a plain string, `tv:...` or `mv:...`.
    std::string key;

    std::string name;

    // True for a series, which decides what Play does and whether the menu has an Episodes line.
    // `shared/Addresses.kt:17`'s test, the same four characters in every language.
    bool series() const;

    // The seasons, in the order the server sent them. Empty for a film.
    std::vector<Season> seasons;

    // Titles like this one, already flattened by `reading::cardOf`.
    std::vector<Feed::Card> more;

    // Whether this profile has saved the title.
    bool in_list = false;

    // What this profile said about it, if anything.
    bool has_rating = false;
    Face rating = Face::Up;

    // Where this profile left off. `contract::Resume`: a position, and an episode when the title
    // is a series.
    bool has_resume = false;
    bool resume_episode = false;
    std::int64_t resume_season = 0;
    std::int64_t resume_number = 0;
    std::int64_t resume_position_s = 0;

    // The season at a place, or nothing past the end.
    const Season *season(int index) const;

    // Which place a season number is at, or -1. `Episodes.kt:139`'s `indexOfFirst`.
    int seasonAt(std::int64_t number) const;
};

// What Play asks for. The three branches of `TitlePage.kt:114-130`.
//
// The third branch is the reason this is a struct and not an address: a series with nothing to
// resume plays the first episode of its first season, and which episode that is, is not in the
// title's own answer -- the Kotlin reads `/v1/titles/<path>/seasons/<n>` for it. So this says the
// season and leaves `known` false, and the screen asks. Nothing here speaks to a server.
struct Play {
    // False for a film, which plays itself.
    bool episode = false;

    std::int64_t season = 0;
    std::int64_t number = 0;

    // False when `number` is not known yet and the season has to be read for its first episode.
    bool known = false;
};

// Which of a title's three pages is showing. `TitlePage.kt:53`.
enum class Page : std::uint8_t { Summary, Episodes, More };

// A line of the summary's menu. `TitlePage.kt:54`. Which of the four a title has is decided by the
// title: the Episodes line only for a series that has seasons, the More line only when the server
// sent titles like this one.
enum class Line : std::uint8_t { Play, Episodes, More, List };

// Which part of the summary the remote is on. `TitlePage.kt:52`.
enum class Part : std::uint8_t { Reactions, Menu };

// The summary: a menu of up to four lines, with three reactions above it.
class Summary {
public:
    explicit Summary(const Detail &detail);

    // A fresh answer for the same title. The lines are rebuilt, and where the viewer is, is kept.
    void arrived(const Detail &detail);

    Part part() const { return part_; }

    // The lines this title has, in the order they are drawn.
    const std::vector<Line> &lines() const { return lines_; }

    // Which line the remote is on, clamped into the menu.
    //
    // The clamp is the Swift's (`TitlePage.swift:103`) and not the Kotlin's: `TitlePage.kt:106`
    // keeps the line in a `remember(data.title)` across a fresh answer and reads `rows[row]` at
    // `:162-166` with no bounds, so a title whose answer drops a line under the viewer -- the
    // server stops sending `more_like_this` while they are on My List -- indexes past the end and
    // throws. Two presses on a live title are enough if the answer changes between them. The port
    // clamps, as `RowMemory::columnOf` clamps, and for the same reason: a bounds answer is not a
    // focus answer.
    int line() const;

    // Which of the three reactions the remote is on.
    int face() const { return face_; }

    // What this profile said about the title, and whether it said anything.
    bool hasRating() const { return has_rating_; }
    Face rating() const { return rating_; }

    // Whether the title is in My List.
    bool inList() const { return in_list_; }

    // What Play would ask for from here. `TitlePage.kt:114-130`.
    Play play() const;

    Did press(Word word);

private:
    void rebuild(const Detail &detail);

    Detail detail_;
    std::vector<Line> lines_;
    Part part_ = Part::Menu;
    int line_ = 0;
    int face_ = FirstFace;
    bool has_rating_ = false;
    Face rating_ = Face::Up;
    bool in_list_ = false;
};

// The episodes of a series: the seasons in a column on the left, that season's episodes as cards
// on the right.
//
// The episodes are not in the title's answer. They arrive per season from
// `/v1/titles/<path>/seasons/<n>`, so this page is told what it is drawing with `arrived` rather
// than holding it from the start, which is what `Episodes.kt:146-149`'s `produceState` does.
class Episodes {
public:
    explicit Episodes(const Detail &detail);

    // The season to ask the server for: the chosen one's number, or 1 when there are none.
    // `Episodes.kt:145`.
    std::int64_t asking() const;

    // That season's episodes, as they arrive.
    //
    // It carries `Episodes.kt:150-156`'s one effect with it: the page opens on the episode the
    // viewer is on, which cannot be done until the episodes are there. It happens once, and only
    // for the season the viewer is on, so choosing a season later starts at its first episode.
    void arrived(std::int64_t number, const std::vector<Episode> &episodes);

    // Which season is chosen, and which line of the column the remote is on.
    int chosen() const { return chosen_; }
    int line() const { return line_; }

    // Which episode card the remote is on.
    int card() const { return card_; }

    // True when the remote is in the season column rather than on the cards.
    bool inSeasons() const { return in_seasons_; }

    const std::vector<Episode> &episodes() const { return episodes_; }

    // Which season the episodes in hand belong to, which is 0 before any have arrived and is not
    // always `asking()`: see `play()`.
    std::int64_t holding() const { return number_; }

    // The episode under the remote, or nothing when the season is empty or has not arrived.
    const Episode *open() const;

    // What OK on a card asks for.
    Play play() const;

    Did press(Word word);

private:
    Detail detail_;
    std::vector<Episode> episodes_;
    std::int64_t number_ = 0;
    int chosen_ = 0;
    int line_ = 0;
    int card_ = 0;
    bool in_seasons_ = false;
    bool placed_ = true;
};

// A grid of portrait posters, `across` to a line: More Like This.
//
// `across` is given rather than held. The drawing number is `paint::Grid::ACROSS` and the page
// passes it in, so the key map is the same state machine at any width and the test can walk a grid
// three wide and five wide without moving a measurement.
class Posters {
public:
    Posters(const std::vector<Feed::Card> &cards, int across);

    int at() const { return at_; }

    // The poster under the remote, or nothing when the grid is empty.
    const Feed::Card *open() const;

    Did press(Word word);

    // One step in a grid, or the same place at an edge. `PosterGrid.kt:36-43`.
    //
    // Right at the last poster of a short last line stays, and Down from a line with nothing
    // directly below it stays: both are the Kotlin's own rule, and both are the reason this is one
    // function rather than four branches in `press`.
    static int step(int at, Word word, int last, int across);

private:
    std::vector<Feed::Card> cards_;
    int across_ = 1;
    int at_ = 0;
};

// A title, and which of its three pages the viewer is on.
//
// `TitlePage.kt:87-96`: a new title starts at its summary, and the line opened there is kept for
// coming back to it. The Kotlin says that with a `remember(data.title)`, which is a reset keyed on
// the title; here it is a key compared in `arrived`.
class Title {
public:
    // `across` is the width of the More Like This grid, which the page gives it: see `Posters`.
    Title(const Detail &detail, int across);

    // A fresh answer. The same title keeps the page, the line and the reaction; a different title
    // starts over at its summary, on the first line.
    void arrived(const Detail &detail);

    Page page() const { return page_; }
    const Detail &detail() const { return detail_; }

    Summary &summary() { return summary_; }
    const Summary &summary() const { return summary_; }

    // The episodes page, which exists only while it is open.
    Episodes *episodes() { return page_ == Page::Episodes ? &episodes_ : nullptr; }
    Posters *posters() { return page_ == Page::More ? &posters_ : nullptr; }

    // One word from the remote, routed to the page that is showing.
    //
    // A right to left page has already had its word turned by `streams::remote::turn` before it
    // reaches here, so nothing in this file counts in two directions.
    Did press(Word word);

    // What the last `Did::Watch` asked for.
    const Play &watching() const { return watching_; }

    // Which title the last `Did::Opened` chose, or nothing when nothing has been chosen. A copy,
    // not a pointer into the grid: a fresh answer rebuilds the grid and the shell may read this
    // after it.
    const Feed::Card *opened() const { return has_opened_ ? &opened_ : nullptr; }

private:
    Detail detail_;
    int across_ = 1;
    Page page_ = Page::Summary;
    Summary summary_;
    Episodes episodes_;
    Posters posters_;
    Play watching_;
    bool has_opened_ = false;
    Feed::Card opened_;
};

} // namespace streams::title

#endif // STREAMS_XBOX_TITLE_H
