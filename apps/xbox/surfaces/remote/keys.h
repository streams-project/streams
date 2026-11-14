// A letter grid for a remote: where the viewer is on the keys, the text so far, and what each key
// does.
//
// The fifth thing on this surface that needs no Windows, and the one that is shared. Four screens
// draw a keyboard -- sign in, add profile, edit profile and the lock steps -- and every one of them
// is a screen a viewer cannot get past if a key does the wrong thing. No Win2D, no WinRT, no swap
// chain, no controller: a grid of keys and a cursor on it.
//
// The twins, file for file:
//
//     Keys       parts/Keys.java         the state machine with the drawing taken out
//     Keyboard   parts/Keyboard.kt       the same, with Compose around it
//     Keyboard   parts/Keyboard.swift    the same again, in SwiftUI
//
// `Keys.java` is the nearest of the three, because it is the only one that already separates the
// key map from the drawing, and it is the one this file is translated from branch for branch.
//
// It holds no words a viewer reads. Cancel and Done carry no text here, unlike all three twins,
// which are handed `cancel` and `done` so the cap can draw them: those words are the screen's and
// they come from `strings/`. A letter key does carry its text, because that text is the key's own
// character and not a translated word -- it is what OK types.
//
// ## The defect this file is built not to have
//
// On a television, column 9 of the fourth row plus Down threw `NoSuchElementException` on the sign
// in screen and closed it under a viewer typing a password. Reachable in eleven presses from where
// the keyboard opens, on all four screens that draw one. It was found by walking the keyboard,
// not by a test; `KeyboardReachTest.kt` holds it now and `Keyboard.kt:132-138` carries the fix.
//
// The cause is the shape of the grid rather than a slip. Three of the four letter rows are ten
// keys wide. The fourth is a two column shift key and then the page's last line, which is eight
// letters on the letters page and **seven** on the symbols and accented pages, so those two rows
// stop at column 8 while the rows above them reach column 9. A remote that comes down the right
// hand edge stands on a column that row has no key at.
//
// Two things follow, and this file takes a different answer from the twins on the second:
//
//  1. **`here` never answers nothing.** The key under the remote is the key whose span holds the
//     column, or the last key of the row. That is the twins' fix and it is kept, because the last
//     key of the row is where a viewer coming down the right hand edge expects to be.
//  2. **The painter asks the same question.** All three twins work out which key to light with a
//     second predicate -- `Keyboard.kt:118`, `Keyboard.swift:139` and `KeysView.java:107` each ask
//     `holds(column)` of every cap and light the one that answers -- so at the ragged stop nothing
//     is lit at all while OK types the row's last key. The fix of 2026-11-13 stopped the crash and
//     left a stop where the viewer cannot see where they are. Here `lit` is the one home for
//     "which key is the remote on", `press` and the painter both read it, and the two cannot
//     drift apart.
//
// `keys_test.cpp` walks every stop the keyboard can be in, from every stop, in all four
// directions, and asserts that no move leaves the grid and that every stop has a key under it.
// That is a property of the whole grid rather than one scripted sequence, so a page whose last row
// is shorter again cannot bring the defect back.
#ifndef STREAMS_XBOX_KEYS_H
#define STREAMS_XBOX_KEYS_H

#include <cstdint>
#include <string>
#include <vector>

#include "remote.h"

namespace streams::keys {

using streams::remote::Word;

// What a key does. `Keyboard.kt:31-40`.
enum class Press : std::uint8_t {
    Letter,
    Shift,
    Marks,
    Accents,
    Space,
    Erase,
    Cancel,
    Done,
};

// The mark a key wears instead of a word. The painter decides what it is drawn with; this names
// which of the three it is, so the picture is chosen where the pictures are. `Keys.java:62-67`.
enum class Mark : std::uint8_t { None, Shift, Space, Erase };

// Which set of letters is showing. `Keyboard.kt:42-44`.
enum class Page : std::uint8_t { Letters, Symbols, Accented };

// One key. `Cap` at `Keyboard.kt:40`.
struct Cap {
    Press press = Press::Letter;

    // Where it starts, and how many of the ten columns it covers.
    int column = 0;
    int span = 1;

    // What it types, for a letter key, and the sample a page key shows of the page it opens. Empty
    // for every key whose text is the screen's: shift, space, erase, Cancel and Done.
    std::string word;

    Mark mark = Mark::None;

    // True when the remote at this column is on this key.
    bool holds(int at) const { return at >= column && at < column + span; }
};

// What a press did. The screen above reads it and acts.
//
// Seven answers and not the four a screen of its own has, because this is a part: the screen that
// owns it has to tell apart a press that only redraws the keys from one that finished them, and a
// shift from a letter, without reading the whole state back.
enum class Did : std::uint8_t {
    // The word was used and nothing changed: Left at the left edge, Right at the right edge, Down
    // on the last row.
    Nothing,
    // The remote is on another key.
    Moved,
    // The text changed. `text()`.
    Typed,
    // The keys themselves changed: the shift went on or off, or another page came in.
    Turned,
    // The keyboard is done and brought nothing back: Cancel, or Back.
    Went,
    // The keyboard is done and `text()` is the answer.
    Done,
    // The word was **not** used, which happens for Up on the top row and nothing else. The page
    // above the keyboard takes it, which is how a screen gets the remote back.
    // `Keyboard.kt:136`, `Keys.java:224`.
    //
    // This is the one answer with no twin: the Kotlin, the Java and the Swift all say it by
    // returning false from a function that otherwise returns true, and a `Did` has room to name it.
    Above,
};

class Keys {
public:
    // How many columns a row of keys holds. `Keyboard.kt:27`.
    static constexpr int Columns = 10;

    // How many columns each of the two buttons at the foot holds. `Keyboard.kt:28`.
    static constexpr int Buttons = 5;

    // Four rows of letters, the modifier row, and the two buttons. `caps` at `Keyboard.kt:53-66`.
    static constexpr int Rows = 6;

    // Where the remote starts: the middle of the home row, which is the letter g.
    // `Keyboard.kt:74-75`.
    static constexpr int StartRow = 2;
    static constexpr int StartColumn = 4;

    // A keyboard holding what has been typed into it already, which is how the sign in screen
    // hands the email address back when the viewer cancels the password.
    explicit Keys(const std::string &start);
    Keys() : Keys(std::string()) {}

    const std::string &text() const { return text_; }

    // Replaces the text and leaves the remote where it is. The sign in screen's two steps are one
    // keyboard, and stepping between them is what this is for. `SignIn.kt:160-176`.
    void hold(const std::string &text) { text_ = text; }

    Page page() const { return page_; }
    bool upper() const { return upper_; }
    int row() const { return row_; }
    int column() const { return column_; }

    // Every key as it stands now, six rows of them. Built fresh, because the shift and the page
    // both change what a key says.
    std::vector<std::vector<Cap>> caps() const;

    // The key the remote is on. Never nothing: see the head of this file.
    Cap here() const;

    // Which key of a row the remote is on, as a place in that row's own list, or -1 for a row it
    // is not on. **The painter's question and `press`'s question are this one question**, so the
    // key that lights is the key that answers.
    int lit(int at_row) const;

    // One word from the remote.
    //
    // A right to left page has already had its word turned by `streams::remote::turn` before it
    // reaches here, so nothing in this file counts in two directions. The letters do not mirror:
    // qwerty is qwerty in every language, which is what `Remote.kt:39`'s one swap is for.
    Did press(Word word);

    // The letters of a page, four lines of them, for a test that wants to say which page is
    // showing without reading a key off the screen.
    static const std::vector<std::string> &lettersOf(Page page);

    // One letter at the shift's own case, by the rules `Locale.ROOT` applies and all three twins
    // already follow. Public because the test holds the handful of letters whose uppercase is not
    // the letter plus a constant.
    static std::string upperOf(const std::string &letter);

private:
    std::vector<Cap> rowOf(int at_row) const;

    std::string text_;
    Page page_ = Page::Letters;
    bool upper_ = false;
    int row_ = StartRow;
    int column_ = StartColumn;
};

} // namespace streams::keys

#endif // STREAMS_XBOX_KEYS_H
