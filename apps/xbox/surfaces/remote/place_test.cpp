// Where does the remote go, and where does a row open when the viewer comes back to it?
//
// A scripted key sequence against the state machine in `place.h`. Nothing here needs an Xbox, a
// console, a Windows toolchain or a screen: the key map is a state machine over a flattened page.
// It is the C++ arm of `PlaceTest.java` and of `tv/.../BrowseKeysTest.kt`, and it exists for the
// reason note 210 gives for the Java one -- the key map is the part of a television surface that
// can be wrong without anything looking wrong.
//
//     c++ -std=c++17 -Wall -Wextra apps/xbox/surfaces/remote/remote.cpp \
//         apps/xbox/surfaces/remote/place.cpp apps/xbox/surfaces/remote/place_test.cpp \
//         -o place && ./place
#include <cstdio>
#include <string>
#include <vector>

#include "place.h"

using streams::place::Feed;
using streams::place::Place;
using streams::place::RowMemory;
using streams::remote::Word;

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

Feed::Card card(const std::string &key, const std::string &name) {
    Feed::Card made;
    made.key = key;
    made.name = name;
    return made;
}

// A row of `count` cards, keyed `<id>-0`, `<id>-1`, and so on.
Feed::Row row(const std::string &id, int count) {
    Feed::Row made;
    made.id = id;
    made.title = id;
    for (int at = 0; at < count; ++at) {
        made.cards.push_back(card(id + "-" + std::to_string(at), id + " " + std::to_string(at)));
    }
    return made;
}

Feed::Tab tab(const std::string &label, const std::string &screen) {
    Feed::Tab made;
    made.label = label;
    made.screen = screen;
    return made;
}

// A page with a bar of three tabs, a hero, and three rows of four, five and six cards. The hero is
// a film, so OK on its Play starts playing.
Feed page() {
    Feed feed;
    feed.tabs = {tab("one", "home"), tab("two", "series"), tab("three", "films")};
    feed.active = 0;
    feed.has_who = true;
    feed.who.name = "A";
    feed.server = "https://example.invalid";
    feed.rows = {row("r0", 4), row("r1", 5), row("r2", 6)};
    feed.has_hero = true;
    feed.hero = card("mv:1", "a film");
    return feed;
}

// The same page with no hero, which is what every page but Home looks like.
Feed rowsOnly() {
    Feed feed = page();
    feed.has_hero = false;
    feed.hero = Feed::Card();
    return feed;
}

// Presses a string of words, one character each: U D L R O B. Answers how many of them moved.
int press(Place &place, const char *keys) {
    int moved = 0;
    for (const char *key = keys; *key != '\0'; ++key) {
        Word word = Word::Up;
        switch (*key) {
            case 'U': word = Word::Up; break;
            case 'D': word = Word::Down; break;
            case 'L': word = Word::Left; break;
            case 'R': word = Word::Right; break;
            case 'O': word = Word::Ok; break;
            case 'B': word = Word::Back; break;
            default: break;
        }
        if (place.press(word)) {
            moved += 1;
        }
    }
    return moved;
}

} // namespace

int main() {
    // ----------------------------------------------------------------- where a page opens
    {
        Place place(page(), 0);
        ok(place.where() == Place::Where::Hero, "a page with a hero opens on the hero");
        ok(place.tab() == 0, "and on its own tab, which is what Back comes home to");

        Place bare(rowsOnly(), 1);
        ok(bare.where() == Place::Where::Row && bare.row() == 0,
           "a page with rows and no hero opens on the first row");

        Feed nothing;
        Place empty(nothing, -1);
        ok(empty.where() == Place::Where::Top,
           "a page with neither a hero nor a row opens on the bar");
        ok(empty.choose() == Place::Chose::Nothing,
           "where OK has nowhere to go, because there is no tab under it either");
    }

    // ----------------------------------------------------------------- up and down the page
    {
        Place place(page(), 0);
        ok(press(place, "U") == 1 && place.where() == Place::Where::Top,
           "Up from the hero reaches the bar");
        ok(press(place, "U") == 0 && place.where() == Place::Where::Top,
           "and Up from the bar is the top of the page: nothing moves");
        ok(press(place, "D") == 1 && place.where() == Place::Where::Hero,
           "Down from the bar reaches the hero");
        ok(press(place, "D") == 1 && place.where() == Place::Where::Row && place.row() == 0,
           "and Down from the hero reaches the first row");
        ok(press(place, "DD") == 2 && place.row() == 2, "Down walks the rows");
        ok(press(place, "D") == 0 && place.row() == 2,
           "and stops on the last one rather than walking off the page");
        ok(press(place, "UUU") == 3 && place.where() == Place::Where::Hero,
           "Up walks back up them and over the top row onto the hero");
    }
    {
        // A page with no hero: the bar and the rows are next to each other.
        Place place(rowsOnly(), 1);
        ok(press(place, "U") == 1 && place.where() == Place::Where::Top,
           "Up from the first row of a page with no hero reaches the bar");
        ok(press(place, "D") == 1 && place.where() == Place::Where::Row && place.row() == 0,
           "and Down from the bar comes back to the first row, never to a hero that is not there");
    }

    // ----------------------------------------------------------------- the bar, and its order
    {
        Place place(page(), 2);
        ok(press(place, "U") == 1 && place.where() == Place::Where::Top && place.tab() == 2,
           "the bar is entered on the page's own tab");
        ok(press(place, "LL") == 2 && place.tab() == 0, "Left walks the tabs down to the first");
        ok(press(place, "L") == 1 && place.tab() == Feed::SEARCH_TAB,
           "and Left of the first tab is search");
        ok(press(place, "L") == 1 && place.tab() == Feed::WHO_TAB, "and Left of search is the face");
        ok(press(place, "L") == 0 && place.tab() == Feed::WHO_TAB,
           "and Left of the face is the edge of the page: nothing moves and nothing wraps");
        ok(place.choose() == Place::Chose::Menu, "OK on the face opens the profile menu");

        // The order trap. Both negative places also satisfy `tab < tabs - 1`, so a branch chain
        // that tested that first would step search to -1, which is no place at all.
        ok(press(place, "R") == 1 && place.tab() == Feed::SEARCH_TAB,
           "Right from the face reaches search");
        ok(place.choose() == Place::Chose::Search, "where OK opens search");
        ok(press(place, "R") == 1 && place.tab() == 0,
           "and Right from search reaches the first tab, not the nothing at -1");
        ok(place.choose() == Place::Chose::Tab && place.chosenTab() != nullptr &&
               place.chosenTab()->screen == "home",
           "where OK chooses that tab, and the screen it names is the server's own word");
        ok(press(place, "RR") == 2 && place.tab() == 2, "Right walks the rest of the tabs");
        ok(press(place, "R") == 0 && place.tab() == 2,
           "and stops on the last one rather than wrapping");
    }

    // ----------------------------------------------------------------- the hero's two buttons
    {
        Place place(page(), 0);
        ok(place.where() == Place::Where::Hero && place.button() == 0,
           "the hero opens on its first button, which is Play");
        ok(place.choose() == Place::Chose::Watch,
           "and OK on Play starts playing, because this hero is not a series");
        ok(press(place, "R") == 1 && place.button() == 1, "Right reaches More info");
        ok(place.choose() == Place::Chose::Title, "where OK opens the title's page");
        ok(press(place, "R") == 1 && place.button() == 1,
           "and Right again stays on it rather than walking off the hero");
        ok(press(place, "L") == 1 && place.button() == 0, "Left comes back to Play");

        Feed series = page();
        series.hero = card("tv:9", "a series");
        Place watching(series, 0);
        ok(watching.choose() == Place::Chose::Title,
           "OK on a series' Play opens its page instead, which is the one branch the key tells");
        ok(watching.chosenCard() != nullptr && watching.chosenCard()->key == "tv:9",
           "and the hero is what was chosen");
    }

    // ----------------------------------------------------------------- along a row
    {
        Place place(rowsOnly(), 0);
        ok(place.column() == 0, "a row the viewer has not touched opens on its first tile");
        ok(press(place, "L") == 0 && place.column() == 0,
           "Left on the first tile does not move, and does not reach the bar either");
        ok(press(place, "RRR") == 3 && place.column() == 3, "Right walks along the row");
        ok(press(place, "R") == 0 && place.column() == 3,
           "and stops on the last tile of a four tile row");
        ok(place.open() != nullptr && place.open()->key == "r0-3",
           "the card under the remote is the fourth of that row");
        ok(place.choose() == Place::Chose::Title && place.chosenCard() != nullptr &&
               place.chosenCard()->key == "r0-3",
           "and OK chooses it");
    }

    // ----------------------------------------------------------------- the row remembers
    {
        Place place(rowsOnly(), 0);
        press(place, "RR");
        ok(place.column() == 2, "a row is scrolled to its third tile");
        press(place, "D");
        ok(place.row() == 1 && place.column() == 0, "the next row down opens on its own first tile");
        press(place, "RRR");
        ok(place.column() == 3, "and is scrolled to its fourth");
        press(place, "U");
        ok(place.row() == 0 && place.column() == 2,
           "coming back up finds the first row where it was left, which is note 187's defect");
        press(place, "D");
        ok(place.row() == 1 && place.column() == 3, "and going down again finds the second");
    }

    // ------------------------------------------------- a fresh answer that moved the title
    {
        Feed feed = rowsOnly();
        Place place(feed, 0);
        press(place, "RR");
        ok(place.open() != nullptr && place.open()->key == "r0-2",
           "the viewer is on the third tile of the first row");

        // The server answers again and has put two new titles in front of it.
        Feed again = feed;
        again.rows[0].cards.insert(again.rows[0].cards.begin(), card("r0-new1", "new one"));
        again.rows[0].cards.insert(again.rows[0].cards.begin(), card("r0-new2", "new two"));
        place.arrived(again);
        ok(place.column() == 4 && place.open() != nullptr && place.open()->key == "r0-2",
           "a reordered row follows the title the viewer was on, not the place it was at");

        // And again, with that title gone from the row altogether.
        Feed gone = feed;
        gone.rows[0].cards = {card("x0", "x"), card("x1", "x")};
        place.arrived(gone);
        ok(place.column() == 1,
           "a row the title has left falls back to the kept place, clamped into the row");

        // And a page that shrank to nothing under the viewer's feet.
        Feed shrunk = feed;
        shrunk.rows.clear();
        place.arrived(shrunk);
        ok(place.row() == 0 && place.open() == nullptr,
           "a page that lost its rows draws nothing rather than reading off the end of one");
        ok(place.choose() == Place::Chose::Nothing, "and OK on it chooses nothing");
        place.arrived(feed);
        ok(place.open() != nullptr && place.open()->key == "r0-2",
           "and when the rows come back the viewer is still where they were");
    }

    // -------------------------------------------- a page that gains a hero gains a stop with it
    {
        Place place(rowsOnly(), 0);
        press(place, "B");
        ok(place.where() == Place::Where::Top, "the remote is on the bar of a page with no hero");
        place.arrived(page());
        ok(press(place, "D") == 1 && place.where() == Place::Where::Hero,
           "an answer that brought a hero puts it between the bar and the rows");
        ok(press(place, "D") == 1 && place.where() == Place::Where::Row, "with the rows under it");
        ok(press(place, "U") == 1 && place.where() == Place::Where::Hero,
           "and it is the first stop up from them as well");
        place.arrived(rowsOnly());
        ok(press(place, "U") == 1 && place.where() == Place::Where::Top,
           "and when a later answer takes the hero away the stop goes with it");
    }

    // --------------------------------------------------------- the memory kernel on its own
    {
        RowMemory memory;
        const std::vector<std::string> titles = {"a", "b", "c"};
        ok(!memory.knows("row") && memory.columnOf("row", titles) == 0,
           "a row with no kept place opens on its first tile");
        memory.keep("row", titles, 1);
        ok(memory.knows("row") && memory.columnOf("row", titles) == 1,
           "a kept place is answered while its title is still there");

        const std::vector<std::string> moved = {"z", "y", "a", "b"};
        ok(memory.columnOf("row", moved) == 3, "and is followed when the title moves");

        const std::vector<std::string> without = {"z", "y"};
        ok(memory.columnOf("row", without) == 1, "and clamped when the title has gone");

        const std::vector<std::string> one = {"z"};
        ok(memory.columnOf("row", one) == 0, "and clamped into a row of one");

        // The hole the Java port names and does not carry across: a place kept past the end of the
        // row, where the Kotlin compares two nulls and answers a place the row does not have.
        memory.keep("edge", titles, 9);
        ok(memory.columnOf("edge", titles) == 2,
           "a place kept past the end of a row is a bounds answer, clamped, and never a focus one");
        ok(memory.columnOf("edge", std::vector<std::string>()) == 0,
           "and a row with no tiles at all answers its first place rather than a negative one");

        memory.clear();
        ok(!memory.knows("row") && memory.columnOf("row", titles) == 0,
           "a page whose answer is not its own forgets every row");
    }

    // ----------------------------------------------------------------- Back, and what OK is not
    {
        Place place(page(), 1);
        press(place, "DD");
        ok(place.where() == Place::Where::Row, "the remote is down in the rows");
        ok(press(place, "B") == 1 && place.where() == Place::Where::Hero,
           "Back from a row comes up to the hero");
        ok(press(place, "B") == 1 && place.where() == Place::Where::Top,
           "Back from the hero comes up to the bar");
        press(place, "LL");
        ok(place.tab() == Feed::SEARCH_TAB, "the focus wanders along the bar without choosing");
        ok(press(place, "B") == 1 && place.tab() == place.active() && place.tab() == 1,
           "and Back brings it home to this page's own tab first");
        ok(press(place, "B") == 0,
           "and only then does Back leave the page, which the screen above hears as no move");
    }
    {
        Place place(page(), 0);
        ok(press(place, "OOO") == 0,
           "OK never reports a move, because choosing leaves the page rather than walking it");
        ok(place.where() == Place::Where::Hero, "and nothing on the page moved under it");
    }

    // ----------------------------------------------------------------- reading ahead
    {
        Feed feed;
        Place empty(feed, 0);
        ok(Place::Reach == 2, "the page reads ahead two rows");
        ok(empty.wantsMore(), "a page with no rows at all asks for more, or it can never grow");

        Place place(rowsOnly(), 0);  // three rows: 0, 1, 2
        ok(place.wantsMore(), "the first of three rows is already within reach of the last");

        Feed many = rowsOnly();
        many.rows = {row("r0", 2), row("r1", 2), row("r2", 2), row("r3", 2),
                     row("r4", 2), row("r5", 2)};
        Place deep(many, 0);
        ok(!deep.wantsMore(), "the first of six is not");
        press(deep, "DD");
        ok(deep.row() == 2 && !deep.wantsMore(), "nor is the third");
        press(deep, "D");
        ok(deep.row() == 3 && deep.wantsMore(),
           "the fourth is two from the last, and that is where the next rows are asked for");
    }

    // ----------------------------------------------------------------- the series prefix
    {
        ok(card("tv:1234", "s").series(), "a key that starts tv: is a series");
        ok(!card("mv:1234", "f").series(), "and a film's key is not");
        ok(!card("", "").series(), "and neither is nothing at all");
        ok(!card("atv:1", "x").series(), "and the prefix is the start of the key and not anywhere");
    }

    if (failures > 0) {
        std::printf("%d of %d failed\n", failures, checks);
        return 1;
    }
    std::printf("%d of %d held\n", checks, checks);
    return 0;
}
