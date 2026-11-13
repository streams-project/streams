// Do the drawing numbers of the remote surface hold? Nothing here needs an Xbox, Windows, Win2D or
// a screen: every number in `paint.h` is arithmetic over a measurement, so it is checked on
// whatever machine builds the tree.
//
//     c++ -std=c++17 -Wall -Wextra apps/xbox/surfaces/remote/paint.cpp \
//         apps/xbox/surfaces/remote/paint_test.cpp -o paint && ./paint
//
// Run from the repository root, or with the path to `brand.json` as the one argument. The brand
// checks read that file rather than a second copy of it, which is the whole reason they are worth
// running: a colour or a corner changed in `brand.json` and not here fails before anything is
// drawn. The pattern is `tv/.../NumbersTest.java`'s, which reads `Theme.kt` the same way.
#include <cstdio>
#include <cstdlib>
#include <string>

#include "paint.h"

namespace paint = streams::paint;

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

// A whole file. `brand.json` is 33 lines and `reference/netflix-on-the-tv.md` is 92; there is no
// reason to stream either.
std::string readWhole(const char *path) {
    std::string text;
    std::FILE *file = std::fopen(path, "rb");
    if (file == nullptr) {
        return text;
    }
    char buffer[4096];
    std::size_t got = 0;
    while ((got = std::fread(buffer, 1, sizeof buffer, file)) > 0) {
        text.append(buffer, got);
    }
    std::fclose(file);
    return text;
}

// The hex colour that follows a key, as 0xAARRGGBB with full alpha, or 0 when the key is absent.
//
// Not a JSON parser: a search for the quoted key, then the next `#`, then six hex digits. The file
// is flat, the keys are unique within it, and a dependency for this would be a dependency the
// whole folder does not otherwise have. A missing or malformed value answers 0, which fails the
// check that asked for it rather than passing quietly.
std::uint32_t colourAfter(const std::string &brand, const char *key) {
    std::string quoted = std::string("\"") + key + "\"";
    std::size_t at = brand.find(quoted);
    if (at == std::string::npos) {
        return 0;
    }
    std::size_t hash = brand.find('#', at);
    if (hash == std::string::npos || hash + 6 >= brand.size()) {
        return 0;
    }
    std::uint32_t value = 0;
    for (std::size_t i = 1; i <= 6; ++i) {
        char c = brand[hash + i];
        int digit;
        if (c >= '0' && c <= '9') {
            digit = c - '0';
        } else if (c >= 'a' && c <= 'f') {
            digit = c - 'a' + 10;
        } else if (c >= 'A' && c <= 'F') {
            digit = c - 'A' + 10;
        } else {
            return 0;
        }
        value = (value << 4) | static_cast<std::uint32_t>(digit);
    }
    return 0xFF000000u | value;
}

// The integer that follows a key, or -1 when it is absent.
int numberAfter(const std::string &brand, const char *key) {
    std::string quoted = std::string("\"") + key + "\"";
    std::size_t at = brand.find(quoted);
    if (at == std::string::npos) {
        return -1;
    }
    std::size_t digit = brand.find_first_of("0123456789", at + quoted.size());
    if (digit == std::string::npos) {
        return -1;
    }
    int value = 0;
    while (digit < brand.size() && brand[digit] >= '0' && brand[digit] <= '9') {
        value = value * 10 + (brand[digit] - '0');
        digit += 1;
    }
    return value;
}

// The first whole number written after a phrase, or -1 when the phrase is not there.
//
// For `reference/netflix-on-the-tv.md`, which is prose with the measurements in it rather than a
// file with keys. The same deliberate crudeness as `colourAfter`: a phrase, then digits, and a
// missing one answers -1 so the check that asked for it fails rather than passes quietly. The file
// is the one the television surfaces are held to and it is tracked beside the code, so a sentence
// reworded here is a failure a person reads, not a number that silently stops meaning anything.
int numberAfterPhrase(const std::string &text, const char *phrase) {
    std::size_t at = text.find(phrase);
    if (at == std::string::npos) {
        return -1;
    }
    std::size_t digit = text.find_first_of("0123456789", at + std::string(phrase).size());
    if (digit == std::string::npos) {
        return -1;
    }
    int value = 0;
    while (digit < text.size() && text[digit] >= '0' && text[digit] <= '9') {
        value = value * 10 + (text[digit] - '0');
        digit += 1;
    }
    return value;
}

bool near(float got, float want, float by) {
    float off = got - want;
    return (off < 0 ? -off : off) <= by;
}

} // namespace

int main(int argc, char **argv) {
    const char *brandPath = argc > 1 ? argv[1] : "brand.json";
    const char *referencePath = argc > 2 ? argv[2] : "reference/netflix-on-the-tv.md";

    // The reference scale.
    {
        ok(paint::Ref::W == 1920 && paint::Ref::H == 1080, "the canvas is 1920 by 1080");
        ok(near(paint::Ref::dp(paint::Ref::W), static_cast<float>(paint::Ref::LAYOUT_W), 0.0f),
           "and half of it is the 960 dp layout a television really lays out in");
        ok(near(paint::Ref::dp(paint::Ref::H), static_cast<float>(paint::Ref::LAYOUT_H), 0.0f),
           "and 540 dp high");
        ok(near(paint::Ref::dp(311), 155.5f, 0.0f), "a reference pixel is half a dp, half kept");
        ok(near(paint::Ref::type(44), 22.0f, 0.0f), "and a reference type size is halved the same");

        // The rounding, which is Compose's, and the one place a sum must not be split.
        ok(paint::Ref::px(311, 2.0f) == 311, "at density 2 a reference pixel is a pixel");
        ok(paint::Ref::px(311, 1.0f) == 156, "and at density 1 it rounds up from 155.5");
        // `Ref.java` warns that a sum must be converted once and never summed after converting.
        // The three-part derivation at `tv/browse/Rows.kt:137` is 72 + 904 + 19, and at density 1
        // both ways land on 498 -- so that particular sum is safe either way, which is written
        // down here so nobody reads the warning as a bug report about those three numbers. Three
        // odd measurements are not safe, and the second check is what the warning is about.
        ok(paint::Ref::px(72 + 904 + 19, 1.0f) == 498, "995 reference pixels is 498 pixels");
        ok(paint::Ref::px(72, 1.0f) + paint::Ref::px(904, 1.0f) + paint::Ref::px(19, 1.0f) == 498,
           "and summing the three roundings happens to agree on these three numbers");
        ok(paint::Ref::px(19 + 19 + 19, 1.0f) == 29 &&
               3 * paint::Ref::px(19, 1.0f) == 30,
           "but three halves converted separately are a pixel out, which is why the rule exists");

        // Xbox, from the archived Designing for Xbox and TV page.
        ok(near(paint::Ref::XBOX_SCALE, 2.0f, 0.0f),
           "Xbox scales a XAML app 200%, so an effective pixel is two reference pixels");
        ok(paint::Ref::px(paint::Ref::W, paint::Ref::XBOX_SCALE) == 1920,
           "so the whole reference canvas is the whole 1920 pixel panel, pixel for pixel");
        ok(near(paint::Ref::dip(904), 452.0f, 0.0f),
           "and a Win2D drawing session takes the dp number, 452 DIPs for the open card's width");
    }

    // The brand, held against brand.json and not against a copy of it.
    {
        std::string brand = readWhole(brandPath);
        ok(!brand.empty(), "brand.json was found and read");

        ok(paint::Ink::BACK == colourAfter(brand, "background"), "BACK is colors.background");
        ok(paint::Ink::PLATE == colourAfter(brand, "surface"), "PLATE is colors.surface");
        ok(paint::Ink::RAISED == colourAfter(brand, "raised"), "RAISED is colors.raised");
        ok(paint::Ink::MUTED == colourAfter(brand, "muted"), "MUTED is colors.muted");
        ok(paint::Ink::ACCENT == colourAfter(brand, "accent"), "ACCENT is colors.accent");
        ok(paint::Ink::JOIN == colourAfter(brand, "join"), "JOIN is colors.join");

        int cardRadius = numberAfter(brand, "card_radius");
        int panelRadius = numberAfter(brand, "panel_radius");
        int fieldRadius = numberAfter(brand, "field_radius");
        ok(cardRadius > 0 && panelRadius > 0 && fieldRadius > 0,
           "the three brand corners were read out of brand.json");
        ok(paint::Corner::card() == paint::Corner::tv(cardRadius),
           "the card corner is shape.card_radius at television scale");
        ok(paint::Corner::panel() == paint::Corner::tv(panelRadius),
           "the panel corner is shape.panel_radius at television scale");
        ok(paint::Corner::field() == paint::Corner::tv(fieldRadius),
           "the field corner is shape.field_radius at television scale");
    }

    // The three shares, which are the eight call sites of `views/Ink.java` in one place.
    {
        ok(paint::Ink::white(100) == 0xFFFFFFFFu, "white at full strength is opaque white");
        ok(paint::Ink::white(0) == 0x00FFFFFFu, "and at nothing it is transparent white");
        ok(paint::Ink::alphaOf(paint::Ink::white(28)) == 71,
           "0.28 of white rounds to 71 of 255, as Compose rounds it");
        ok(paint::Ink::alphaOf(paint::Ink::black(80)) == 204, "and 0.8 of black to 204");
        ok(paint::Ink::black(55) == 0x8C000000u, "the hero's lower shade is 0.55 black");
        ok(paint::Ink::alphaOf(paint::Ink::dim(paint::Ink::MUTED, 50)) == 128,
           "a colour dimmed to half keeps its own alpha halved");
        ok((paint::Ink::dim(paint::Ink::ACCENT, 50) & 0x00FFFFFFu) == 0x002F6BFFu,
           "and keeps its own three channels");
        ok(paint::Ink::redOf(paint::Ink::JOIN) == 0xE5 && paint::Ink::greenOf(paint::Ink::JOIN) == 0x35 &&
               paint::Ink::blueOf(paint::Ink::JOIN) == 0x2B,
           "the channels split the way a Direct2D colour wants them");
    }

    // The corners, including the two that are shares of a third.
    {
        ok(paint::Corner::card() == 21, "a card corner is 14 at 1.5, which rounds to 21");
        ok(paint::Corner::panel() == 36, "a panel corner is 36");
        ok(paint::Corner::field() == 18, "a field corner is 18");
        ok(paint::Corner::key() == 12, "a key is two thirds of a field");
        ok(paint::Corner::chip() == 9, "a chip is half of one");
        ok(paint::Corner::inside(paint::Corner::panel(), 8) == 28,
           "a corner inside another loses the inset");
        ok(paint::Corner::inside(paint::Corner::field(), 40) == paint::Corner::chip(),
           "and never falls below a chip");
    }

    // The TV-safe band, which is Microsoft's number and not ours.
    {
        ok(paint::Safe::TOP == 54 && paint::Safe::BOTTOM == 54,
           "27 effective pixels top and bottom is 54 reference pixels");
        ok(paint::Safe::LEFT == 96 && paint::Safe::RIGHT == 96,
           "and 48 effective pixels left and right is 96");
        ok(paint::Safe::holds(96, 54, 1728, 972), "the whole safe box holds");
        ok(!paint::Safe::holds(96, 54, 1729, 972), "a pixel past its right edge does not");
        ok(paint::Safe::over(0, 0, 1920, 1080) == 96,
           "a part drawn to the panel edges is 96 reference pixels over on its worst side");

        // The finding these checks exist to record. The page margin was measured off the Netflix
        // reference and is 72 reference pixels, which is 36 effective pixels: inside Microsoft's
        // 48. It is asserted rather than fixed, because the measurement is the measurement and
        // whether to move it is the owner's call -- note 223 open item 2.
        //
        // Both numbers are read out of `reference/netflix-on-the-tv.md` rather than written here.
        // They were literals until 2026-11-13 and a literal in a test is the same promise as a
        // literal in the code: it says 72 is the margin and proves only that two files agree about
        // a number neither of them owns.
        std::string tv = readWhole(referencePath);
        ok(tv.size() > 1000, "the television reference is there to be read");
        int ours = numberAfterPhrase(tv, "This project uses");
        int netflix = numberAfterPhrase(tv, "## Safe margin");
        int inside = numberAfterPhrase(tv, "TV-unsafe band for Xbox by");
        ok(ours == 72 && netflix == 68 && inside == 24,
           "and it still says what this surface was built from: 72 ours, 68 Netflix's, 24 inside");
        ok(paint::Corner::PAGE_EDGE == ours,
           "the page margin is the one that file names as this project's");
        ok(paint::Corner::PAGE_EDGE - netflix == 4,
           "four reference pixels wider than the margin measured off Netflix itself");
        ok(paint::Corner::PAGE_EDGE < paint::Safe::LEFT,
           "and inside Microsoft's TV-unsafe band, recorded not fixed");
        ok(paint::Safe::LEFT - paint::Corner::PAGE_EDGE == inside,
           "by the number the reference states, so neither can move alone");
        ok(paint::Corner::SUB_EDGE > paint::Safe::LEFT,
           "the sub-page column at 130 is clear of the band");
    }

    // The grid of posters, which is the one layout on this surface the reference does not measure.
    {
        namespace Grid = paint::Grid;
        const int room = paint::Ref::W - paint::Corner::PAGE_EDGE;
        ok(Grid::across(Grid::LEFT) == Grid::ACROSS,
           "four posters are what fits beside a title's name, which is what the page draws");
        ok(Grid::LEFT + Grid::ACROSS * Grid::WIDE + (Grid::ACROSS - 1) * Grid::GAP_X <= room,
           "four keep the page's right margin");
        ok(Grid::LEFT + (Grid::ACROSS + 1) * Grid::WIDE + Grid::ACROSS * Grid::GAP_X > room,
           "and five do not");
        ok(Grid::across(0) == 6,
           "a grid drawn from the page's own edge would hold six, which is what the arithmetic is "
           "for: the count follows the box rather than being written beside it");
    }

    // The easing curve and the rise it drives.
    {
        ok(near(paint::Glide::at(0.0f), 0.0f, 0.0f), "the curve starts at nothing");
        ok(near(paint::Glide::at(1.0f), 1.0f, 0.0f), "and ends at everything");
        ok(near(paint::Glide::at(-1.0f), 0.0f, 0.0f) && near(paint::Glide::at(2.0f), 1.0f, 0.0f),
           "and is clamped outside its own range");

        // Monotone, which is what a row that never goes backwards needs.
        bool rises = true;
        float last = -1.0f;
        for (int i = 0; i <= 100; ++i) {
            float y = paint::Glide::at(i / 100.0f);
            if (y < last) {
                rises = false;
            }
            last = y;
        }
        ok(rises, "and never goes backwards across a hundred samples");

        // It is an ease-out: fast first, slow last. The four control points say so and this is the
        // property a reader of the numbers would want held.
        ok(paint::Glide::at(0.5f) > 0.5f, "half the time is more than half the distance");
        ok(paint::Glide::at(0.25f) > 0.25f, "a quarter in, it is already past a quarter");
        ok(paint::Glide::at(0.9f) > 0.98f, "and the last tenth of the time is the last fiftieth");

        ok(paint::Glide::RISE == 810 && paint::Glide::RISE_MS == 320,
           "a row rises 810 reference pixels over 320 ms");
        ok(near(paint::Glide::remaining(0), 810.0f, 0.0f), "with all of it left at the first frame");
        ok(near(paint::Glide::remaining(320), 0.0f, 0.0f), "and none at the last");
        ok(near(paint::Glide::remaining(1000), 0.0f, 0.0f), "and none after it");
        ok(paint::Glide::remaining(160) < 405.0f,
           "half way through the rise, more than half the distance is done");

        // Sixteen frames at 60 Hz is 267 ms, so a 320 ms rise is about 19 frames. Every one of
        // them is a distinct position: a curve that repeated a position would read as a stutter.
        bool distinct = true;
        float previous = paint::Glide::remaining(0);
        for (int frame = 1; frame * 1000 / 60 < paint::Glide::RISE_MS; ++frame) {
            float now = paint::Glide::remaining(frame * 1000 / 60);
            if (!(now < previous)) {
                distinct = false;
            }
            previous = now;
        }
        ok(distinct, "and every 60 Hz frame of it is a nearer position than the one before");
    }

    if (failures > 0) {
        std::printf("%d of %d failed\n", failures, checks);
        return 1;
    }
    std::printf("%d of %d held\n", checks, checks);
    return 0;
}
