// Where does the remote go on a title's own pages, and what does each word do at an edge?
//
// A scripted key sequence against the state machine in `title.h`. Nothing here needs an Xbox, a
// console, a Windows toolchain or a screen. It is the companion of `place_test.cpp`: that one
// walks a page of rows, this one walks the page a row opens into.
//
//     c++ -std=c++17 -Wall -Wextra apps/xbox/surfaces/remote/remote.cpp \
//         apps/xbox/surfaces/remote/place.cpp apps/xbox/surfaces/remote/paint.cpp \
//         apps/xbox/surfaces/remote/title.cpp apps/xbox/surfaces/remote/title_test.cpp \
//         -o title && ./title
#include <cstdio>
#include <string>
#include <vector>

#include "paint.h"
#include "title.h"

using streams::place::Feed;
using streams::remote::Word;
using streams::title::Detail;
using streams::title::Did;
using streams::title::Episode;
using streams::title::Episodes;
using streams::title::Face;
using streams::title::Line;
using streams::title::Page;
using streams::title::Part;
using streams::title::Play;
using streams::title::Posters;
using streams::title::Season;
using streams::title::Summary;
using streams::title::Title;

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

// Presses a string of words, one character each: U D L R O B. Answers the last thing a press did.
template <typename T>
Did press(T &on, const char *keys) {
    Did did = Did::Nothing;
    for (const char *key = keys; *key != '\0'; ++key) {
        did = on.press(wordOf(*key));
    }
    return did;
}

Feed::Card card(const std::string &key, const std::string &name) {
    Feed::Card made;
    made.key = key;
    made.name = name;
    return made;
}

// A series with three seasons, four titles like it, nothing watched and nothing said about it.
Detail series() {
    Detail made;
    made.key = "tv:1";
    made.name = "a series";
    made.seasons = {Season{10, 1}, Season{8, 2}, Season{6, 3}};
    for (int at = 0; at < 4; ++at) {
        made.more.push_back(card("mv:" + std::to_string(at), "like it " + std::to_string(at)));
    }
    return made;
}

// A film with nothing beside it: the shortest menu there is.
Detail film() {
    Detail made;
    made.key = "mv:9";
    made.name = "a film";
    return made;
}

std::vector<Episode> episodesOf(int count) {
    std::vector<Episode> out;
    for (int at = 1; at <= count; ++at) {
        Episode made;
        made.number = at;
        made.name = "episode " + std::to_string(at);
        out.push_back(made);
    }
    return out;
}

} // namespace

int main() {
    // ------------------------------------------------------------------ what a title's menu holds
    {
        Summary summary(film());
        ok(summary.lines().size() == 2 && summary.lines()[0] == Line::Play &&
               summary.lines()[1] == Line::List,
           "a film with nothing beside it has Play and My List and no more");

        Summary full(series());
        ok(full.lines().size() == 4 && full.lines()[1] == Line::Episodes &&
               full.lines()[2] == Line::More,
           "a series with seasons and titles like it has all four lines, in order");

        Detail noMore = series();
        noMore.more.clear();
        Summary three(noMore);
        ok(three.lines().size() == 3 && three.lines()[2] == Line::List,
           "a title the server sent no more-like-this for has no More line");

        ok(full.part() == Part::Menu && full.line() == 0,
           "a title opens on the first line of its menu");
        ok(full.face() == streams::title::FirstFace,
           "and on the middle reaction, which is the one Up reaches first");
    }

    // ------------------------------------------------------------------ down the menu and off it
    {
        Summary summary(series());
        ok(press(summary, "D") == Did::Moved && summary.line() == 1, "Down walks the menu");
        ok(press(summary, "DD") == Did::Moved && summary.line() == 3, "to its last line");
        ok(press(summary, "D") == Did::Nothing && summary.line() == 3,
           "and Down at the last line stays, which is note 187's own row");
        ok(press(summary, "UUU") == Did::Moved && summary.line() == 0, "Up walks it back");
        ok(press(summary, "U") == Did::Moved && summary.part() == Part::Reactions,
           "and Up off the top of the menu reaches the reactions");
        ok(press(summary, "U") == Did::Nothing && summary.part() == Part::Reactions,
           "Up again stays: there is nothing above the reactions");
        ok(press(summary, "D") == Did::Moved && summary.part() == Part::Menu &&
               summary.line() == 0,
           "Down comes back to the line the menu was left on");
    }

    // ------------------------------------------------------------------ the three reactions
    {
        Summary summary(series());
        press(summary, "U");
        ok(summary.part() == Part::Reactions && summary.face() == 1, "the row opens on the middle");
        ok(press(summary, "L") == Did::Moved && summary.face() == 0, "Left walks the row");
        ok(press(summary, "L") == Did::Nothing && summary.face() == 0,
           "and Left at the first reaction stays, rather than leaving the title");
        ok(press(summary, "RR") == Did::Moved && summary.face() == 2, "Right walks it back");
        ok(press(summary, "R") == Did::Nothing && summary.face() == 2,
           "Right at the last one stays: note 239 pressed this and note 187 had not");

        ok(press(summary, "O") == Did::Rated && summary.hasRating() &&
               summary.rating() == Face::Love,
           "OK leaves the rating the remote is on");
        ok(press(summary, "O") == Did::Rated && !summary.hasRating(),
           "and OK again takes it off");
        press(summary, "OLL");
        ok(press(summary, "O") == Did::Rated && summary.hasRating() &&
               summary.rating() == Face::Down,
           "and a different reaction replaces the one that was there rather than clearing it");

        ok(press(summary, "B") == Did::Moved && summary.part() == Part::Menu,
           "Back from the reactions returns to the menu first");
        ok(press(summary, "B") == Did::Went,
           "and Back from the menu leaves the title");
        ok(press(summary, "L") == Did::Went,
           "as Left on the menu does: note 187's title page row, verified on the Streamer");
    }

    // ------------------------------------------------------------------ what a rating starts as
    {
        Detail liked = series();
        liked.has_rating = true;
        liked.rating = Face::Up;
        Summary summary(liked);
        ok(summary.hasRating() && summary.rating() == Face::Up,
           "a title this profile has rated opens with that rating");
        press(summary, "U");
        ok(press(summary, "O") == Did::Rated && !summary.hasRating(),
           "and OK on the same one takes it off");
    }

    // ------------------------------------------------------------------ My List
    {
        Detail saved = series();
        saved.in_list = true;
        Summary summary(saved);
        ok(summary.inList(), "a saved title opens saved");
        press(summary, "DDD");
        ok(summary.lines()[static_cast<std::size_t>(summary.line())] == Line::List,
           "the last line is My List");
        ok(press(summary, "O") == Did::Listed && !summary.inList(), "OK takes it out");
        ok(press(summary, "O") == Did::Listed && summary.inList(), "and OK puts it back");
    }

    // ------------------------------------------------------------------ what Play asks for
    {
        Summary one(film());
        Play asked = one.play();
        ok(!asked.episode, "a film plays itself and names no episode");

        Detail watching = series();
        watching.has_resume = true;
        watching.resume_episode = true;
        watching.resume_season = 8;
        watching.resume_number = 3;
        Summary two(watching);
        asked = two.play();
        ok(asked.episode && asked.season == 8 && asked.number == 3 && asked.known,
           "a series the viewer is part way through plays the episode they are on");

        Summary three(series());
        asked = three.play();
        ok(asked.episode && asked.season == 10 && !asked.known,
           "a series with nothing to resume names its first season and leaves the episode to ask for");

        Detail bare = series();
        bare.seasons.clear();
        Summary four(bare);
        asked = four.play();
        ok(asked.episode && asked.season == 1 && !asked.known,
           "and a series whose answer carries no seasons at all falls back to season one");
    }

    // ------------------------------------------------------------------ the episodes page
    {
        Detail watching = series();
        watching.has_resume = true;
        watching.resume_episode = true;
        watching.resume_season = 8;
        watching.resume_number = 4;
        Episodes episodes(watching);
        ok(episodes.chosen() == 1 && episodes.asking() == 8,
           "the episodes page opens on the season the viewer is on");
        ok(!episodes.inSeasons() && episodes.card() == 0,
           "on the cards, not the column, and on the first card until the episodes land");

        episodes.arrived(8, episodesOf(6));
        ok(episodes.card() == 3, "and lands on the episode the viewer is on once they arrive");
        ok(episodes.open() != nullptr && episodes.open()->number == 4, "which is that episode");

        ok(press(episodes, "D") == Did::Moved && episodes.card() == 4, "Down walks the cards");
        ok(press(episodes, "DD") == Did::Nothing && episodes.card() == 5,
           "Down at the last card stays");
        ok(press(episodes, "UUUUU") == Did::Moved && episodes.card() == 0, "Up walks them back");
        ok(press(episodes, "U") == Did::Nothing && episodes.card() == 0, "and stays at the first");

        ok(press(episodes, "L") == Did::Moved && episodes.inSeasons() && episodes.line() == 1,
           "Left opens the season column at the season that is chosen");
        ok(press(episodes, "L") == Did::Went,
           "and Left in the column goes back to the title: note 187's episodes row");
        ok(press(episodes, "U") == Did::Moved && episodes.line() == 0, "Up walks the column");
        ok(press(episodes, "UU") == Did::Nothing && episodes.line() == 0, "and stops at the top");
        ok(press(episodes, "DDDD") == Did::Nothing && episodes.line() == 2,
           "Down stops at the last season");
        ok(press(episodes, "R") == Did::Moved && !episodes.inSeasons(),
           "Right leaves the column for the cards");
        ok(press(episodes, "R") == Did::Nothing, "and Right on the cards is nothing at all");
    }

    // ------------------------------------------------------------------ choosing a season
    {
        Episodes episodes(series());
        episodes.arrived(10, episodesOf(3));
        press(episodes, "D");
        ok(episodes.card() == 1, "the cards are walked");
        press(episodes, "LDD");
        ok(episodes.inSeasons() && episodes.line() == 2, "the column is walked to the third season");
        ok(press(episodes, "O") == Did::Moved && episodes.chosen() == 2 && !episodes.inSeasons() &&
               episodes.card() == 0,
           "OK chooses it, leaves the column, and starts at its first card");
        ok(episodes.asking() == 6, "and the page now asks the server for that season");

        // The window the twins leave open: the chosen season has changed and the episodes in hand
        // are still the season before's.
        ok(episodes.holding() == 10 && episodes.asking() == 6,
           "the episodes in hand are not the season being asked for");
        Play asked = episodes.play();
        ok(asked.season == 10 && asked.number == 1,
           "so OK on a card plays the season those episodes are in, not the one chosen");
        episodes.arrived(6, episodesOf(2));
        ok(episodes.holding() == 6 && episodes.play().season == 6,
           "and once the right season lands, that is what plays");
        ok(press(episodes, "O") == Did::Watch, "OK on a card watches it");
        ok(press(episodes, "B") == Did::Went, "and Back leaves the page");
    }

    // ------------------------------------------------------------------ a season with no episodes
    {
        Episodes episodes(series());
        episodes.arrived(10, std::vector<Episode>());
        ok(press(episodes, "D") == Did::Nothing && episodes.card() == 0,
           "Down on a season that answered nothing keeps the card at the first place, not at -1");
        ok(episodes.open() == nullptr && press(episodes, "O") == Did::Nothing,
           "and OK there watches nothing");
    }

    // ------------------------------------------------------------------ the grid of posters
    {
        std::vector<Feed::Card> cards;
        for (int at = 0; at < 10; ++at) {
            cards.push_back(card("mv:" + std::to_string(at), "like it"));
        }
        Posters grid(cards, 4);
        ok(grid.at() == 0, "the grid opens at its first poster");
        ok(press(grid, "R") == Did::Moved && grid.at() == 1, "Right walks a line");
        ok(press(grid, "D") == Did::Moved && grid.at() == 5, "Down drops a whole line");
        ok(press(grid, "U") == Did::Moved && grid.at() == 1, "and Up lifts one");
        ok(press(grid, "LLL") == Did::Went,
           "Left at the first column goes back to the title, and only there");
        ok(grid.at() == 0, "after walking back along the line");

        press(grid, "DD");
        ok(grid.at() == 8, "the last line of ten is reached");
        ok(press(grid, "D") == Did::Nothing && grid.at() == 8,
           "Down with nothing under it stays");
        ok(press(grid, "R") == Did::Moved && grid.at() == 9, "Right reaches the last poster");
        ok(press(grid, "R") == Did::Nothing && grid.at() == 9,
           "and Right past the end of a short last line stays");
        ok(press(grid, "O") == Did::Opened, "OK opens the title under the remote");
        ok(press(grid, "B") == Did::Went, "and Back leaves the page");

        Posters narrow(cards, 3);
        press(narrow, "RRD");
        ok(narrow.at() == 5, "the same key map three across is three across");

        Posters empty(std::vector<Feed::Card>(), 4);
        ok(empty.open() == nullptr && press(empty, "O") == Did::Nothing,
           "OK on an empty grid opens nothing");
        ok(press(empty, "D") == Did::Nothing, "and nothing moves in it");
    }

    // ------------------------------------------------------------------ the three pages together
    {
        Title title(series(), streams::paint::Grid::ACROSS);
        ok(title.page() == Page::Summary, "a title opens on its summary");
        press(title, "D");
        ok(press(title, "O") == Did::Moved && title.page() == Page::Episodes,
           "OK on the Episodes line opens the episodes");
        ok(title.episodes() != nullptr && title.posters() == nullptr,
           "and only that page is there to be pressed");
        ok(press(title, "B") == Did::Moved && title.page() == Page::Summary,
           "Back from it comes home to the summary");
        ok(title.summary().line() == 1,
           "on the line that opened it, which is note 187's 'the line opened is kept'");

        press(title, "D");
        ok(press(title, "O") == Did::Moved && title.page() == Page::More,
           "OK on the More line opens the grid");
        ok(press(title, "L") == Did::Moved && title.page() == Page::Summary,
           "Left at its first column comes home to the summary");
        ok(title.summary().line() == 2, "again on the line that opened it");

        press(title, "UU");
        ok(press(title, "O") == Did::Watch && title.watching().episode,
           "OK on Play watches, and says what");
        ok(press(title, "B") == Did::Went, "Back from the summary leaves the title");
    }

    // ------------------------------------------------------------------ what a fresh answer does
    {
        Title title(series(), 4);
        press(title, "DD");
        ok(title.summary().line() == 2, "the viewer is on the third line");

        Detail again = series();
        again.in_list = true;
        title.arrived(again);
        ok(title.summary().line() == 2 && title.summary().inList(),
           "a fresh answer for the same title keeps the line and takes the new state");

        Detail other = film();
        title.arrived(other);
        ok(title.page() == Page::Summary && title.summary().line() == 0,
           "and a different title starts over at the first line of its own menu");
        ok(title.summary().lines().size() == 2, "with the menu that title has");
    }

    // ------------------------------------------------------------------ the line that went away
    {
        Title title(series(), 4);
        press(title, "DDD");
        ok(title.summary().line() == 3, "the viewer is on My List, the fourth line");

        Detail thinner = series();
        thinner.more.clear();
        thinner.seasons.clear();
        title.arrived(thinner);
        ok(title.summary().lines().size() == 2,
           "a fresh answer drops two lines out from under them");
        ok(title.summary().line() == 1,
           "and the line is clamped into the menu rather than left past its end");
        ok(press(title, "O") == Did::Listed,
           "so OK presses My List, where the Kotlin would read past the end of the list");
    }

    std::printf("\n%d of %d held\n", checks - failures, checks);
    return failures == 0 ? 0 : 1;
}
