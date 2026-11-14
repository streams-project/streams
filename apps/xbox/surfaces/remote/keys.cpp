#include "keys.h"

#include <cstdint>

namespace streams::keys {
namespace {

// The three pages, exactly as all three twins hold them: `Keyboard.kt:42-44`,
// `Keyboard.swift:27-29`, `Keys.java:98-100`.
//
// Four lines each, and the fourth line of two of them is one letter short of the other, which is
// the whole of the shape this surface had to be built around.
const std::vector<std::string> &lettersPage() {
    static const std::vector<std::string> page = {"1234567890", "qwertyuiop", "asdfghjkl-",
                                                  "zxcvbnm'"};
    return page;
}

const std::vector<std::string> &symbolsPage() {
    static const std::vector<std::string> page = {"1234567890", "!@#$%^&*()", "_=+[]{};:\"",
                                                  ",.?/\\|~"};
    return page;
}

const std::vector<std::string> &accentedPage() {
    static const std::vector<std::string> page = {"àáâäãåæçèé", "êëìíîïñòóô", "õöøùúûüýßÿ",
                                                  "œšžćčđğ"};
    return page;
}

// One line of a page, cut into its letters.
//
// By code point and not by byte, for the reason `Keys.java:174-177` gives for cutting by code
// point rather than by `char`: a page of letters is the one place a surface meets a letter that is
// not one unit wide, and splitting one would draw half a letter and type half a letter. Every
// letter on the accented page is two bytes of UTF-8, so a byte walk would cut all thirty-seven of
// them in half.
//
// A lead byte is any byte that is not a continuation byte, and a continuation byte is `10xxxxxx`.
std::vector<std::string> cut(const std::string &line) {
    std::vector<std::string> letters;
    for (std::size_t at = 0; at < line.size();) {
        std::size_t next = at + 1;
        while (next < line.size() &&
               (static_cast<unsigned char>(line[next]) & 0xC0u) == 0x80u) {
            next += 1;
        }
        letters.push_back(line.substr(at, next - at));
        at = next;
    }
    return letters;
}

// The code point a letter is, for the one letter a `cut` hands back. Zero for an empty string.
std::uint32_t pointOf(const std::string &letter) {
    if (letter.empty()) {
        return 0;
    }
    const unsigned char lead = static_cast<unsigned char>(letter[0]);
    if (lead < 0x80u) {
        return lead;
    }
    std::uint32_t point = 0;
    std::size_t more = 0;
    if ((lead & 0xE0u) == 0xC0u) {
        point = lead & 0x1Fu;
        more = 1;
    } else if ((lead & 0xF0u) == 0xE0u) {
        point = lead & 0x0Fu;
        more = 2;
    } else if ((lead & 0xF8u) == 0xF0u) {
        point = lead & 0x07u;
        more = 3;
    } else {
        return 0;
    }
    if (letter.size() < more + 1) {
        return 0;
    }
    for (std::size_t at = 1; at <= more; ++at) {
        point = (point << 6) | (static_cast<unsigned char>(letter[at]) & 0x3Fu);
    }
    return point;
}

// And back again.
std::string letterOf(std::uint32_t point) {
    std::string out;
    if (point < 0x80u) {
        out.push_back(static_cast<char>(point));
    } else if (point < 0x800u) {
        out.push_back(static_cast<char>(0xC0u | (point >> 6)));
        out.push_back(static_cast<char>(0x80u | (point & 0x3Fu)));
    } else {
        out.push_back(static_cast<char>(0xE0u | (point >> 12)));
        out.push_back(static_cast<char>(0x80u | ((point >> 6) & 0x3Fu)));
        out.push_back(static_cast<char>(0x80u | (point & 0x3Fu)));
    }
    return out;
}

// The letters whose uppercase is not the letter less 0x20. Nine of them, and they are all on the
// accented page. Unicode's own data and not a number of ours, so there is no second home here to
// keep: these are the letters the three pages hold that the rule below does not reach.
struct Odd {
    const char *lower;
    const char *upper;
};

// `ß` is two letters uppercased, which is what `Locale.ROOT` does and what all three twins
// therefore already do -- `Keys.java:180` is `toUpperCase(Locale.ROOT)`, Kotlin's `uppercase()`
// and Swift's `uppercased()` are both the same full casing. A key of the accented page is drawn
// with two letters on it when the shift is on, and it types two.
const Odd Odds[] = {
    {"ß", "SS"}, {"ÿ", "Ÿ"}, {"œ", "Œ"}, {"š", "Š"}, {"ž", "Ž"},
    {"ć", "Ć"},  {"č", "Č"}, {"đ", "Đ"}, {"ğ", "Ğ"},
};

Cap letterCap(const std::string &word, int column) {
    Cap cap;
    cap.press = Press::Letter;
    cap.column = column;
    cap.span = 1;
    cap.word = word;
    cap.mark = Mark::None;
    return cap;
}

Cap markedCap(Press press, int column, int span, Mark mark) {
    Cap cap;
    cap.press = press;
    cap.column = column;
    cap.span = span;
    cap.mark = mark;
    return cap;
}

Cap wordedCap(Press press, int column, int span, const std::string &word) {
    Cap cap;
    cap.press = press;
    cap.column = column;
    cap.span = span;
    cap.word = word;
    cap.mark = Mark::None;
    return cap;
}

// One row of letters, each its own key. `Keyboard.kt:46-50`.
std::vector<Cap> lineOf(const std::string &letters, int from, bool upper) {
    const std::vector<std::string> cuts = cut(letters);
    std::vector<Cap> caps;
    caps.reserve(cuts.size());
    for (std::size_t at = 0; at < cuts.size(); ++at) {
        const std::string word = upper ? Keys::upperOf(cuts[at]) : cuts[at];
        caps.push_back(letterCap(word, from + static_cast<int>(at)));
    }
    return caps;
}

} // namespace

const std::vector<std::string> &Keys::lettersOf(Page page) {
    switch (page) {
        case Page::Symbols:
            return symbolsPage();
        case Page::Accented:
            return accentedPage();
        default:
            return lettersPage();
    }
}

// The case `Locale.ROOT` gives a letter, which is the case all three twins type. Two rules and a
// table of nine:
//
//   * a to z, and the Latin-1 letters from `à` to `þ`, are their uppercase less 0x20. `÷` sits in
//     the middle of that block and is a sign rather than a letter, so it is stepped over -- no
//     page here carries one, and a rule that is right only for the letters we happen to draw is a
//     rule that breaks the first time a page changes.
//   * the nine above, which the rule does not reach.
//
// Anything else is itself, which is every digit and every symbol on the first two pages.
std::string Keys::upperOf(const std::string &letter) {
    for (const Odd &odd : Odds) {
        if (letter == odd.lower) {
            return odd.upper;
        }
    }
    const std::uint32_t point = pointOf(letter);
    const bool ascii = point >= 'a' && point <= 'z';
    const bool latin1 = point >= 0xE0u && point <= 0xFEu && point != 0xF7u;
    if (ascii || latin1) {
        return letterOf(point - 0x20u);
    }
    return letter;
}

// Every key as it stands. `caps` at `Keyboard.kt:53-66`, in the same order and with the same
// spans, and `Keys.java:150-170`.
std::vector<Cap> Keys::rowOf(int at_row) const {
    const std::vector<std::string> &letters = lettersOf(page_);
    switch (at_row) {
        case 0:
        case 1:
        case 2:
            return lineOf(letters[static_cast<std::size_t>(at_row)], 0, upper_);
        case 3: {
            // The shift takes the first two columns of the row, and the page's last line starts
            // after it. On two of the three pages that line is seven letters, so the row ends at
            // column 8 while the rows above it end at 9.
            std::vector<Cap> caps;
            caps.push_back(markedCap(Press::Shift, 0, 2, Mark::Shift));
            const std::vector<Cap> rest = lineOf(letters[3], 2, upper_);
            caps.insert(caps.end(), rest.begin(), rest.end());
            return caps;
        }
        case 4: {
            // The modifier row. The two page keys draw a sample of the page they open, which is
            // the page's own letters and not a translated word, so they carry it here.
            std::vector<Cap> caps;
            caps.push_back(wordedCap(Press::Marks, 0, 2, "!#$"));
            caps.push_back(wordedCap(Press::Accents, 2, 2, "àáâ"));
            caps.push_back(markedCap(Press::Space, 4, 3, Mark::Space));
            caps.push_back(markedCap(Press::Erase, 7, 3, Mark::Erase));
            return caps;
        }
        default: {
            // The two buttons at the foot. Their words are the screen's, out of `strings/`, so
            // nothing here says what they read.
            std::vector<Cap> caps;
            caps.push_back(markedCap(Press::Cancel, 0, Buttons, Mark::None));
            caps.push_back(markedCap(Press::Done, Buttons, Buttons, Mark::None));
            return caps;
        }
    }
}

std::vector<std::vector<Cap>> Keys::caps() const {
    std::vector<std::vector<Cap>> rows;
    rows.reserve(Rows);
    for (int at_row = 0; at_row < Rows; ++at_row) {
        rows.push_back(rowOf(at_row));
    }
    return rows;
}

// The one answer to "which key is the remote on", asked by `press` and by the painter alike.
//
// The fall back to the last key of the row is the twins' fix of 2026-11-13 and it is kept for
// their reason: a remote that came down the right hand edge of a row that stops at column 8 is
// held by the last key of that row rather than dropped. What is not kept is the twins' habit of
// asking a second question when they draw, which is what leaves the ragged stop with nothing lit
// on all three of them.
int Keys::lit(int at_row) const {
    if (at_row != row_) {
        return -1;
    }
    const std::vector<Cap> line = rowOf(at_row);
    for (std::size_t at = 0; at < line.size(); ++at) {
        if (line[at].holds(column_)) {
            return static_cast<int>(at);
        }
    }
    return static_cast<int>(line.size()) - 1;
}

Cap Keys::here() const {
    const std::vector<Cap> line = rowOf(row_);
    const int at = lit(row_);
    return line[static_cast<std::size_t>(at)];
}

Keys::Keys(const std::string &start) : text_(start) {}

// A translation of `Keys.java:214-252` branch for branch and in the same order, which is
// `Keyboard.kt:129-154`'s own order.
Did Keys::press(Word word) {
    const Cap at = here();
    switch (word) {
        case Word::Up:
            // The one word the keyboard does not take. A keyboard that took it would be a
            // keyboard a viewer on the top row could never leave upwards.
            if (row_ > 0) {
                row_ -= 1;
                return Did::Moved;
            }
            return Did::Above;
        case Word::Down:
            // Down on the last row is used and moves nothing, which is the twins' answer: a
            // keyboard that let Down fall through would drop the viewer out of the bottom of it.
            if (row_ < Rows - 1) {
                row_ += 1;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Left: {
            // Left at the left edge stays, and is still used. All four screens that draw a
            // keyboard have something to the left of it, and a keyboard that let Left escape
            // sideways would lose the viewer's place mid word.
            if (at.column <= 0) {
                return Did::Nothing;
            }
            const int to = at.column - 1;
            if (to == column_) {
                return Did::Nothing;
            }
            column_ = to;
            return Did::Moved;
        }
        case Word::Right: {
            if (at.column + at.span >= Columns) {
                return Did::Nothing;
            }
            const int to = at.column + at.span;
            // On a row that stops short, the column the remote stands at is already past the last
            // key, so the step the twins take lands where it already is. They report that as a
            // move; this reports it as nothing, which is what a painter needs to know.
            if (to == column_) {
                return Did::Nothing;
            }
            column_ = to;
            return Did::Moved;
        }
        case Word::Back:
            return Did::Went;
        case Word::Ok:
            break;
    }
    switch (at.press) {
        case Press::Letter:
            text_ += at.word;
            // One letter and the shift falls back, which is what a viewer typing a name expects
            // and what `Keyboard.kt:146` does. The keys all change with it; the screen is told the
            // text changed, which redraws both.
            upper_ = false;
            return Did::Typed;
        case Press::Shift:
            upper_ = !upper_;
            return Did::Turned;
        case Press::Marks:
            page_ = page_ == Page::Symbols ? Page::Letters : Page::Symbols;
            return Did::Turned;
        case Press::Accents:
            page_ = page_ == Page::Accented ? Page::Letters : Page::Accented;
            return Did::Turned;
        case Press::Space:
            text_ += " ";
            return Did::Typed;
        case Press::Erase: {
            if (text_.empty()) {
                return Did::Nothing;
            }
            // By code point, for the same reason the letters are cut that way: a letter of the
            // accented page is two bytes and dropping one of them leaves half a letter in the
            // field. The three twins disagree here and all three are reasonable in their own
            // language -- the Kotlin drops a UTF-16 unit, the Java a code point, the Swift a
            // grapheme -- and a code point is the Java's, which is the twin this file follows.
            std::size_t at_byte = text_.size() - 1;
            while (at_byte > 0 &&
                   (static_cast<unsigned char>(text_[at_byte]) & 0xC0u) == 0x80u) {
                at_byte -= 1;
            }
            text_.erase(at_byte);
            return Did::Typed;
        }
        case Press::Cancel:
            return Did::Went;
        case Press::Done:
            return Did::Done;
    }
    return Did::Nothing;
}

} // namespace streams::keys
