// Where the viewer is on a page of rows, and what each of the six words does from there.
//
// This is the second thing on this surface that needs no Windows at all. `paint.h` is the numbers a
// drawn part needs; this is the answer to "which stop does a direction reach, what does Left do at
// an edge, and where does a row open when the viewer comes back to it". None of that touches
// Win2D, WinRT, a swap chain or a controller: it is a state machine over a flattened page, so it is
// built and checked on whatever machine builds the tree, exactly as the grammar in `remote.h` is.
//
// The twins, file for file:
//
//     Feed       browse/Feed.java         a page flattened: tabs, a face, rows of cards, a hero
//     RowMemory  views/RowMemory.java     the tile a row was left on, kept by the title on it
//     Place      browse/Place.java        the three stops, the key map, what OK chooses
//
// and behind all three, `tv/browse/Browse.kt:55-230`, which is what the Java was ported from.
//
// The key map is not re-derived here. It is the one in note 187, translated branch for branch and
// in the same order, because in one place the order is the behaviour: see `press` in `place.cpp`.
//
// It holds no words a viewer reads. `says` on the Java side builds the sentence a screen reader is
// given; the Xbox equivalent of that has no provider to hand it to yet (note 223 open item 7), so
// this file answers *what* the remote is on and the screen above it will decide what is said.
#ifndef STREAMS_XBOX_PLACE_H
#define STREAMS_XBOX_PLACE_H

#include <cstdint>
#include <map>
#include <string>
#include <vector>

#include "remote.h"

namespace streams::place {

using streams::remote::Word;

// A page of rows, flattened. The twin of `browse/Feed.java`.
//
// Android needs this type for a reason that does not apply here: Java source cannot call the
// contract's name-mangled getters, so one Kotlin file flattens the contract and every other file in
// the package reads plain Java types (note 210 section 4; `browse/Reading.kt`). C++ has no such
// problem -- `reading.h` reads `Contract.hpp` straight into this -- so `Feed` survives for the
// other reason the Java gives: it is one arrival, built fresh each time the core publishes a view,
// and what outlives an arrival is where the viewer was, which lives in `Place` and `RowMemory`.
struct Feed {
    // Where the top bar's focus goes to the left of the first tab. `tv/browse/TopBar.kt:31`.
    static constexpr int SEARCH_TAB = -2;

    // And to the left of search. `tv/browse/TopBar.kt:34`.
    static constexpr int WHO_TAB = -3;

    // One tab of the bar.
    struct Tab {
        // The name the server sent with it, so it is never written in the code.
        std::string label;

        // The screen it opens, for the shell. `NavItem.screen`.
        std::string screen;
    };

    // Who is watching, for the face at the left of the bar.
    struct Who {
        std::string name;

        // The picture, already made absolute against the server, or empty for the letter.
        std::string picture;

        // True for a child's profile, which changes the plate behind the letter.
        bool kids = false;
    };

    // One title in a row.
    struct Card {
        // The title's key as a plain string: what a row's place is kept under and what the screen
        // turns back into an address when the viewer chooses.
        std::string key;

        // The title's own name, from the catalogue.
        std::string name;

        // The picture for a tall place and for a wide one. `shared/Art.kt:19-22`.
        std::string tall;
        std::string wide;

        // The marks on this tile, as the contract carries them.
        //
        // One deliberate difference from `Feed.java`, which holds them as finished strings. The
        // Java has to: the words are made by the Kotlin seam, because the seam is the only file in
        // that package that can read a `Badge` at all. Here the kind and the rank travel as
        // themselves, and the words are made where the words live -- `badge.ranked` is a pattern
        // with a `{rank}` in it, and filling one carries a language's digits and its bidirectional
        // isolates (`shared/Theme.kt:66-96`). Restating that rule in this file would give a
        // language a second home, which `platform/tools/check_patterns.py` exists to prevent.
        struct Mark {
            // `BadgeKind`: `new` or `ranked`. The contract's own word, never an English label.
            std::string kind;

            // The place in a ranked row, when the mark carries one.
            bool has_rank = false;
            std::int64_t rank = 0;
        };

        std::vector<Mark> badges;

        // True for a series, which decides whether the hero's Play opens the title's page or starts
        // playing (`tv/browse/Browse.kt:227`). The test is `shared/Addresses.kt:17`'s,
        // `title.value.startsWith("tv:")` -- a wire prefix and not a word, so it is the same four
        // characters in every language.
        bool series() const;
    };

    // One row of titles.
    struct Row {
        // The section's own id, which is the key a row's place is kept under.
        std::string id;

        // The row's name, from the server.
        std::string title;

        std::vector<Card> cards;

        // The title keys of this row, in the order it draws them, for `RowMemory`.
        std::vector<std::string> keys() const;

        // The card at a place, or nothing past the end.
        const Card *at(int index) const;
    };

    std::vector<Tab> tabs;

    // Which tab this page is, or -1 when the page is not one of them.
    int active = -1;

    // Who is watching. `has_who` false is Java's null: a page drawn before a profile is known.
    bool has_who = false;
    Who who;

    // Where the pictures come from, for the face and the mark.
    std::string server;

    std::vector<Row> rows;

    // The title the top of the page opens with. `has_hero` false is a page with no hero of its own.
    bool has_hero = false;
    Card hero;

    // What a page with nothing at all says, already read from the words. Empty when it has rows.
    std::string empty;

    // The row at a place, or nothing past the end.
    const Row *row(int index) const;

    // The last row's place, or -1 for a page with no rows.
    int lastRow() const;

    // The tab at a place, or nothing, which is what the bar's negative places need.
    const Tab *tab(int index) const;
};

// Where each row of a page was left: the place, and the title that was on it.
//
// The twin of `views/RowMemory.java`, which is note 187's Android `fd7b104`. The defect it fixed,
// in the owner's words: after scrolling a row and moving up or down, the row showed from its first
// tile as if it had never been scrolled.
//
// Why the title and not just the place: returning to a page fetches it again and the server may
// reorder the rows and the tiles in them, so a place kept by number alone opens a row on a
// different title. The place is kept by the row's id together with the title key that was on it,
// and a fresh answer that moved the title follows the title.
class RowMemory {
public:
    // The tile a row was left on, or its first. The three answers of `Browse.kt:89-96` in order:
    // the kept place when the title is still there; where that title moved to; and the kept place
    // clamped into the row when the title has gone from it altogether.
    //
    // The bounds are answered before the title, which is `RowMemory.java`'s own correction of the
    // Kotlin: `Browse.kt:93` compares a null read past the end with a null kept title and they
    // compare equal, so it answers a place the row does not have. Unreachable today and still a
    // bounds answer rather than a focus answer, so the port does not carry the hole across.
    int columnOf(const std::string &id, const std::vector<std::string> &titles) const;

    // Keeps a row's place and the title on it. `Browse.kt:100-103`.
    void keep(const std::string &id, const std::vector<std::string> &titles, int at);

    // True when this row has a kept place, which is not the same as its place being 0.
    bool knows(const std::string &id) const;

    // Forgets everything, for a page whose answer is not its own.
    void clear();

private:
    struct Kept {
        int at = 0;
        bool titled = false;
        std::string title;
    };

    std::map<std::string, Kept> columns_;
};

// Where the viewer is on a page of rows. It knows nothing about drawing.
class Place {
public:
    // The three places the remote can be on a page of rows. `Browse.kt:55`.
    enum class Where { Top, Hero, Row };

    // What OK did, for the screen to turn into an address. `Browse.kt:218-230`.
    enum class Chose {
        // A page with no hero and no rows, where OK has nowhere to go.
        Nothing,
        // The face: the profile menu opens over the page.
        Menu,
        // The search button.
        Search,
        // A tab of the bar. `chosenTab` is which.
        Tab,
        // A title in a row, or the hero's More info, or the hero's Play on a series.
        Title,
        // The hero's Play on something that is not a series: it starts playing.
        Watch,
    };

    // A page the viewer has just arrived on. The opening stop is `Browse.kt:78`'s exactly: the hero
    // or the bar when there is a hero or there are no rows, and the first row otherwise. The tab
    // starts on the page's own tab (`:80`), which is what makes Back's "come home to this tab
    // first" branch mean something.
    Place(const Feed &feed, int active);

    // A fresh answer for the same page. `Browse.kt:179-181`: the rows, the tab count and whether
    // there is a hero are replaced, and where the viewer is, is not. The row is not clamped either,
    // for the same reason the Kotlin does not clamp it: `row()` is read through `Feed::row`, which
    // answers nothing past the end, so a page that shrank draws nothing rather than reading off the
    // end, and a page that grew back finds the viewer where they were.
    void arrived(const Feed &feed);

    Where where() const { return where_; }
    int row() const { return row_; }
    int tab() const { return tab_; }
    int button() const { return button_; }

    // The page's own tab, which Back comes home to.
    int active() const { return active_; }

    const Feed &feed() const { return feed_; }

    // The tile a row was left on, or its first. `Browse.kt:89-96`.
    int columnOf(int index) const;

    // And for the row the remote is on. `Browse.kt:98`.
    int column() const;

    // One word from the remote. True when something moved, which is the one thing that redraws.
    //
    // OK answers false here and is answered by `choose`, exactly as `Browse.kt:148` and `:232-236`
    // split it: choosing leaves the page, so it is not a move and must not be reported as one.
    //
    // A right to left page has already had its word turned by `streams::remote::turn` before it
    // reaches here, so nothing in this file counts in two directions.
    bool press(Word word);

    // What OK chooses from here. The five branches of `Browse.kt:219-229`, in order. The last two
    // are the hero's: More info, and Play on a series, both open the title's page; Play on anything
    // else starts playing.
    Chose choose() const;

    // Which tab OK chose, or nothing when it chose something else.
    const Feed::Tab *chosenTab() const;

    // Which title OK chose, or nothing: the tile under the remote in a row, or the hero.
    const Feed::Card *chosenCard() const;

    // The card the remote is on inside a row, for the open card and for what is said.
    const Feed::Card *open() const;

    // Whether the page should read its next rows now: the focus is within REACH of the last row the
    // page holds, or the page has no rows at all and can only grow by asking. `Browse.kt:203`.
    bool wantsMore() const;

    // `Browse.kt:53`. One row below the focused one is already partly on screen -- 82 reference
    // pixels of its tiles -- so the second row down is the first the viewer cannot see at all.
    static constexpr int Reach = 2;

private:
    // The page above the rows: the hero when there is one, the bar when there is not.
    // `Browse.kt:77`.
    Where above() const { return hero_ ? Where::Hero : Where::Top; }

    // `Browse.kt:100-103` and `:105`.
    void keep(int at);
    int lastColumn() const;

    Feed feed_;
    int tabs_ = 0;
    int active_ = -1;
    bool hero_ = false;

    Where where_ = Where::Top;
    int row_ = 0;
    int tab_ = -1;
    int button_ = 0;

    RowMemory columns_;
};

} // namespace streams::place

#endif // STREAMS_XBOX_PLACE_H
