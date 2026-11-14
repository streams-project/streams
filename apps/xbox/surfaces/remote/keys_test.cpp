// Where does the remote go on the keyboard, and what does each key do?
//
// A scripted key sequence against the state machine in `keys.h`, and then an exhaustive one: every
// stop the keyboard can be in, pressed in all four directions, asserting that no move leaves the
// grid and that every stop has a key under it. Nothing here needs an Xbox, a console, a Windows
// toolchain or a screen.
//
//     c++ -std=c++17 -Wall -Wextra apps/xbox/surfaces/remote/remote.cpp \
//         apps/xbox/surfaces/remote/keys.cpp apps/xbox/surfaces/remote/keys_test.cpp \
//         -o keys && ./keys
#include <cstdio>
#include <string>
#include <vector>

#include "keys.h"

using streams::keys::Cap;
using streams::keys::Did;
using streams::keys::Keys;
using streams::keys::Mark;
using streams::keys::Page;
using streams::keys::Press;
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

Did press(Keys &keys, const char *script) {
    Did did = Did::Nothing;
    for (const char *key = script; *key != '\0'; ++key) {
        did = keys.press(wordOf(*key));
    }
    return did;
}

// The key under the remote, read through a helper that answers something no real key could rather
// than reading off the end of a row.
//
// `gate_test.cpp` reads its panel the same way and for the same reason: without this, a mutation
// that breaks the fall back in `lit` crashes the suite on an index instead of printing the checks
// it broke, and a test that crashes proves less than one that fails -- the output is what says
// which rule went. It is the test that is careful rather than `Keys`, so no unreachable guard sits
// in the surface pretending to be a rule.
std::string on(const Keys &keys) { return keys.here().word; }

Press pressOf(const Keys &keys) { return keys.here().press; }

// What the painter would light, reached the way a painter reaches it: `lit` names a place in the
// row and the row is read at it. `on` above goes through `here`, which is what `press` acts on, so
// the two readers really are the two questions and a surface that let them drift apart would fail
// here. This one is bounds checked and answers something no real key could rather than reading off
// the end of a row, which is `gate_test.cpp`'s reason for reading its panel through helpers: a
// mutation that prints the checks it broke proves more than one that crashes the suite.
std::string drawn(const Keys &keys) {
    const std::vector<std::vector<Cap>> rows = keys.caps();
    const std::vector<Cap> &line = rows[static_cast<std::size_t>(keys.row())];
    const int at = keys.lit(keys.row());
    if (at < 0 || at >= static_cast<int>(line.size())) {
        return "<no key lit>";
    }
    return line[static_cast<std::size_t>(at)].word;
}

// To the page a test wants, from where the keyboard opens: down to the modifier row, along to the
// key that opens it, and OK.
void toPage(Keys &keys, Page page) {
    if (page == Page::Letters) {
        return;
    }
    press(keys, "DDLLLL");
    if (page == Page::Accented) {
        press(keys, "R");
    }
    press(keys, "O");
}

// Where a stop sits in the whole space of them, so the walk below can say it has been there.
constexpr int Pages = 3;
constexpr int Cases = 2;
constexpr int Stops = Pages * Cases * Keys::Rows * Keys::Columns;

int stateOf(const Keys &keys) {
    const int paged = static_cast<int>(keys.page()) * Cases + (keys.upper() ? 1 : 0);
    return (paged * Keys::Rows + keys.row()) * Keys::Columns + keys.column();
}

} // namespace

int main() {
    // ------------------------------------------------------------------ where it opens
    {
        Keys keys("");
        ok(keys.row() == Keys::StartRow && keys.column() == Keys::StartColumn,
           "the keyboard opens in the middle of the home row");
        ok(on(keys) == "g", "which is the letter g");
        ok(keys.page() == Page::Letters && !keys.upper(), "on the letters, in lower case");
        ok(keys.text().empty(), "with nothing typed");

        Keys held("someone@example.com");
        ok(held.text() == "someone@example.com",
           "and a keyboard can be opened holding what was typed into it before, which is how the "
           "sign in screen hands the address back when the password is cancelled");
    }

    // ------------------------------------------------------------------ the four directions
    {
        Keys keys("");
        ok(press(keys, "L") == Did::Moved && on(keys) == "f", "Left steps one key");
        ok(press(keys, "RR") == Did::Moved && on(keys) == "h", "and Right steps back and on");
        ok(press(keys, "U") == Did::Moved && keys.row() == 1 && on(keys) == "y",
           "Up steps a row and keeps the column");
        ok(press(keys, "D") == Did::Moved && keys.row() == 2 && on(keys) == "h",
           "and Down steps back down to the key it left");

        ok(press(keys, "LLLLLL") == Did::Nothing && keys.column() == 0 && on(keys) == "a",
           "Left stops at the left edge, and the word is still used: a keyboard that let Left "
           "escape sideways would lose the viewer's place mid word");
        ok(press(keys, "RRRRRRRRRR") == Did::Nothing && keys.column() == 9 && on(keys) == "-",
           "and Right stops at the right edge");
    }

    // ------------------------------------------------------------------ the one word it does not take
    {
        Keys keys("");
        ok(press(keys, "UU") == Did::Moved && keys.row() == 0, "the remote reaches the top row");
        ok(keys.press(Word::Up) == Did::Above && keys.row() == 0,
           "and Up from the top row is not taken: the page above gets the remote back");
        ok(keys.press(Word::Up) == Did::Above && keys.press(Word::Up) == Did::Above &&
               keys.row() == 0,
           "a held Up answers the same every time, one word per repeat, and moves nothing");

        press(keys, "DDDDD");
        ok(keys.row() == Keys::Rows - 1, "the remote reaches the last row");
        ok(keys.press(Word::Down) == Did::Nothing && keys.row() == Keys::Rows - 1,
           "and Down on the last row is used and moves nothing: it must not drop the viewer out "
           "of the bottom of the keyboard");
    }

    // ------------------------------------------------------------------ typing
    {
        Keys keys("");
        ok(press(keys, "O") == Did::Typed && keys.text() == "g", "OK on a letter types it");
        ok(press(keys, "RO") == Did::Typed && keys.text() == "gh", "and on the next one, that");
        ok(press(keys, "DDO") == Did::Typed && keys.text() == "gh ",
           "the space key on the modifier row types a space");
        ok(press(keys, "O") == Did::Typed && keys.text() == "gh  ",
           "and a second space makes it two");
        ok(press(keys, "RO") == Did::Typed && keys.text() == "gh ",
           "the erase key beside it takes one back");
        ok(press(keys, "OOO") == Did::Typed && keys.text().empty(), "and keeps taking");
        ok(keys.press(Word::Ok) == Did::Nothing && keys.text().empty(),
           "erase on an empty field is used and changes nothing");
    }

    // ------------------------------------------------------------------ the shift
    {
        Keys keys("");
        ok(press(keys, "DLLL") == Did::Moved && pressOf(keys) == Press::Shift,
           "the shift takes the first two columns of the fourth row");
        ok(keys.column() == 1 && on(keys).empty(),
           "it carries no word of its own: it wears a mark, and the painter chooses the picture");
        ok(keys.here().mark == Mark::Shift, "which is the shift's");
        ok(press(keys, "O") == Did::Turned && keys.upper(), "OK on it turns the keys upper case");
        ok(press(keys, "ULO") == Did::Typed && keys.text() == "A",
           "and the next letter is typed upper case");
        ok(!keys.upper(),
           "after which the shift falls back, which is what a viewer typing a name expects");
        ok(press(keys, "O") == Did::Typed && keys.text() == "Aa",
           "so the letter after it is lower again");
    }

    // ------------------------------------------------------------------ the three pages
    {
        Keys keys("");
        ok(press(keys, "DDLLLL") == Did::Nothing && pressOf(keys) == Press::Marks,
           "Left along the modifier row reaches the marks key");
        ok(on(keys) == "!#$", "which draws a sample of the page it opens");
        ok(press(keys, "O") == Did::Turned && keys.page() == Page::Symbols,
           "and OK brings the symbols in");
        ok(press(keys, "O") == Did::Turned && keys.page() == Page::Letters,
           "pressing it again puts the letters back");
        ok(press(keys, "RO") == Did::Turned && keys.page() == Page::Accented,
           "the accents key beside it brings the accented letters in");
        ok(press(keys, "LO") == Did::Turned && keys.page() == Page::Symbols,
           "and the marks key swaps one page straight for the other, never through the letters");
        ok(keys.row() == 4 && keys.column() == 1,
           "a page change leaves the remote standing where it was");
    }

    // ------------------------------------------------------------------ the ragged fourth row
    //
    // The whole reason this file exists. `KeyboardReachTest.kt` presses exactly this on the
    // television, where before 2026-11-13 it threw `NoSuchElementException` out of `press` and
    // closed the sign in screen under a viewer typing a password.
    {
        Keys keys("");
        press(keys, "DDLLLLO");
        ok(keys.page() == Page::Symbols, "to the symbols page, seven presses from where it opens");
        press(keys, "UUUU");
        ok(keys.row() == 0, "up the right hand edge to the top row");
        press(keys, "RRRRRRRRR");
        ok(keys.row() == 0 && keys.column() == 9, "and along it to the last column");
        press(keys, "DDD");
        ok(keys.row() == 3, "three rows down is the row that has no key at column 9");
        ok(keys.column() == 9, "and the remote stands on column 9 all the same");
        ok(on(keys) == "~",
           "the last key of the short row holds it, which is where a viewer coming down the right "
           "hand edge expects to be");
        ok(drawn(keys) == "~",
           "and the painter lights that same key: all three twins ask a second question when they "
           "draw and light nothing here at all");
        ok(press(keys, "O") == Did::Typed && keys.text() == "~", "OK types it");
        ok(press(keys, "L") == Did::Moved && keys.column() == 7 && on(keys) == "|",
           "Left off that stop steps from the key that held the remote, not from the empty column");
        ok(press(keys, "RRR") == Did::Nothing && keys.column() == 9,
           "and Right walks back to it and stops: there is no column ten");
    }

    {
        Keys keys("");
        press(keys, "DDLLLLRO");
        ok(keys.page() == Page::Accented,
           "the accented page's fourth row is seven letters for the same reason");
        press(keys, "UUUU");
        press(keys, "RRRRRRRRR");
        press(keys, "DDD");
        ok(keys.row() == 3 && keys.column() == 9 && on(keys) == "ğ", "and it holds the remote too");
        ok(drawn(keys) == "ğ", "and lights what it holds");
    }

    {
        Keys keys("");
        press(keys, "UU");
        press(keys, "RRRRRRRRR");
        press(keys, "DDD");
        ok(keys.row() == 3 && keys.column() == 9 && on(keys) == "'",
           "the letters page's fourth row does reach column 9, and is unchanged by any of this");
    }

    // ------------------------------------------------------------------ the two buttons
    {
        Keys keys("hello");
        press(keys, "DDD");
        ok(keys.row() == 5 && pressOf(keys) == Press::Cancel,
           "the last row is the two buttons, and the left half of it is Cancel");
        ok(on(keys).empty(),
           "which carries no word: Cancel and Done read out of `strings/` and the screen draws "
           "them, so no English is written here");
        ok(press(keys, "R") == Did::Moved && pressOf(keys) == Press::Done && keys.column() == 5,
           "Right crosses to Done in one step, because each button is five columns wide");
        ok(press(keys, "R") == Did::Nothing, "and there is nothing to its right");
        ok(press(keys, "O") == Did::Done && keys.text() == "hello",
           "OK on Done finishes the keyboard with what was typed");
        ok(press(keys, "L") == Did::Moved && press(keys, "O") == Did::Went,
           "and OK on Cancel finishes it with nothing");
        ok(keys.press(Word::Back) == Did::Went,
           "Back says the same as Cancel, from wherever the remote is standing");
    }

    // ------------------------------------------------------------------ the case of a letter
    {
        ok(Keys::upperOf("a") == "A" && Keys::upperOf("z") == "Z", "a to z shift by the rule");
        ok(Keys::upperOf("1") == "1" && Keys::upperOf("!") == "!" && Keys::upperOf("-") == "-",
           "a digit and a sign are themselves");
        ok(Keys::upperOf("à") == "À" && Keys::upperOf("ø") == "Ø" && Keys::upperOf("þ") == "Þ",
           "and the Latin-1 letters shift by the same rule");
        ok(Keys::upperOf("÷") == "÷",
           "the sign sitting in the middle of that block is stepped over");
        ok(Keys::upperOf("ß") == "SS",
           "the sharp s is two letters upper case, which is what Locale.ROOT does and so what all "
           "three twins already type");
        ok(Keys::upperOf("ÿ") == "Ÿ" && Keys::upperOf("œ") == "Œ" && Keys::upperOf("š") == "Š" &&
               Keys::upperOf("ž") == "Ž",
           "and four of the nine the rule does not reach");
        ok(Keys::upperOf("ć") == "Ć" && Keys::upperOf("č") == "Č" && Keys::upperOf("đ") == "Đ" &&
               Keys::upperOf("ğ") == "Ğ",
           "and the other four that are letters");

        // Every letter of every page, so a page that gains a letter the rule cannot case fails
        // here rather than on a television.
        int unchanged = 0;
        int letters = 0;
        const Page pages[Pages] = {Page::Letters, Page::Symbols, Page::Accented};
        for (const Page page : pages) {
            Keys keys("");
            toPage(keys, page);
            const std::vector<std::vector<Cap>> rows = keys.caps();
            for (std::size_t row = 0; row < 4; ++row) {
                for (const Cap &cap : rows[row]) {
                    if (cap.press != Press::Letter) {
                        continue;
                    }
                    letters += 1;
                    if (Keys::upperOf(cap.word) == cap.word) {
                        unchanged += 1;
                    }
                }
            }
        }
        ok(letters == 38 + 37 + 37, "the three pages carry a hundred and twelve keys of their own");
        ok(unchanged == 12 + 37,
           "twelve of the letters page's and all thirty-seven of the symbols page's are digits or "
           "signs and case to themselves, and every one of the accented page's thirty-seven "
           "letters cases to something else");
    }

    // ------------------------------------------------------------------ a letter that is two bytes
    {
        Keys keys("");
        toPage(keys, Page::Accented);
        press(keys, "UUUULL");
        ok(keys.row() == 0 && keys.column() == 0, "the first key of the accented page");
        ok(press(keys, "O") == Did::Typed && keys.text() == "à",
           "a letter of two bytes types whole");
        ok(keys.text().size() == 2, "and really is two bytes");
        press(keys, "DDDDRRR");
        ok(pressOf(keys) == Press::Erase, "down and along to the erase key");
        ok(press(keys, "O") == Did::Typed && keys.text().empty(),
           "which takes the whole letter and not one byte of it: one byte would leave half a "
           "letter in the field");
    }

    // ------------------------------------------------------------------ the exhaustive walk
    //
    // The check the television's defect was worth. Not a sequence: every stop the keyboard can
    // reach, pressed with every word, asserting that the remote is still on the grid afterwards
    // and that there is a key under it. A fourth row that grew shorter again, a span that stopped
    // covering a column, a page added with a short line in it -- none of them can pass here.
    {
        std::vector<bool> been(Stops, false);
        std::vector<Keys> queue;
        const Keys start("");
        been[static_cast<std::size_t>(stateOf(start))] = true;
        queue.push_back(start);

        const Word all[6] = {Word::Up,    Word::Down, Word::Left,
                             Word::Right, Word::Ok,   Word::Back};
        int stops = 0;
        int turns = 0;
        int presses = 0;
        int offGrid = 0;
        int unlit = 0;
        int ragged = 0;
        int wrongHeld = 0;

        for (std::size_t head = 0; head < queue.size(); ++head) {
            const Keys from = queue[head];
            stops += 1;
            if (from.lit(from.row()) < 0) {
                unlit += 1;
            }
            // A stop standing on a column its row has no key of its own at. There are four of
            // them on the whole keyboard and every one is the same defect the television shipped.
            const std::vector<Cap> line = from.caps()[static_cast<std::size_t>(from.row())];
            bool covered = false;
            for (const Cap &cap : line) {
                covered = covered || cap.holds(from.column());
            }
            if (!covered) {
                ragged += 1;
                const Cap held = from.here();
                if (held.column != line.back().column || held.span != line.back().span) {
                    wrongHeld += 1;
                }
            }
            for (const Word word : all) {
                Keys next = from;
                const Did did = next.press(word);
                presses += 1;
                if (word != Word::Ok && word != Word::Back) {
                    turns += 1;
                    // A direction may move, may do nothing, and on the top row may hand the word
                    // to the page above. It may never type, finish or cancel the keyboard.
                    if (did == Did::Typed || did == Did::Turned || did == Did::Went ||
                        did == Did::Done) {
                        offGrid += 1;
                    }
                }
                if (next.row() < 0 || next.row() >= Keys::Rows || next.column() < 0 ||
                    next.column() >= Keys::Columns) {
                    offGrid += 1;
                }
                if (next.lit(next.row()) < 0) {
                    unlit += 1;
                }
                const std::size_t at = static_cast<std::size_t>(stateOf(next));
                if (!been[at]) {
                    been[at] = true;
                    queue.push_back(next);
                }
            }
        }

        std::printf("     %d stops walked, %d of them standing on a column with no key of its "
                    "own; %d presses, %d of them turns\n",
                    stops, ragged, presses, turns);
        ok(stops == Stops,
           "every stop of the keyboard is reachable: three pages, two cases, six rows and ten "
           "columns, which is three hundred and sixty");
        ok(turns == stops * 4, "each of them pressed in all four directions");
        ok(offGrid == 0,
           "and no direction from any of them leaves the grid, types, or finishes the keys");
        ok(unlit == 0, "every stop walked has a key under it, before the press and after it");
        ok(ragged == 4,
           "exactly four of them stand on an empty column: the symbols page's fourth row and the "
           "accented page's, in each of the two cases");
        ok(wrongHeld == 0,
           "and at each of the four the remote is held by the last key of that row");
    }

    std::printf("\n%d of %d held\n", checks - failures, checks);
    return failures == 0 ? 0 : 1;
}
