// Can a C++ surface read the contract, and does it read it into the right page?
//
// Held against the contract's own fixtures and not against a copy of them: the files under
// `platform/contract/fixtures/<wire>/valid/` are the ones every other arm of the contract is
// checked with, so a field that moves there fails here. That is the pattern `paint_test.cpp` uses
// against `brand.json` and `NumbersTest.java` uses against the television reference.
//
// Nothing here needs an Xbox, Windows, Win2D or a controller. What it proves is the half of note
// 220's blocker that is already gone: `gen/src/cpp.rs` emits `Contract.hpp`, C++ reads it directly,
// and this family needs no equivalent of the Kotlin seam the Android television surface must have.
//
//     c++ -std=c++17 -Wall -Wextra -I platform/contract/generated/types \
//         apps/xbox/surfaces/remote/remote.cpp apps/xbox/surfaces/remote/place.cpp \
//         apps/xbox/surfaces/remote/reading.cpp apps/xbox/surfaces/remote/reading_test.cpp \
//         -o reading && ./reading platform/contract/fixtures/2026-11-05/valid
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "place.h"
#include "reading.h"

using streams::place::Feed;
using streams::place::Place;
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

std::string slurp(const std::string &path) {
    std::ifstream file(path);
    if (!file) {
        std::printf("FAIL cannot read %s\n", path.c_str());
        failures += 1;
        return std::string();
    }
    std::ostringstream all;
    all << file.rdbuf();
    return all.str();
}

// The one card in a row, by its key, or nothing.
const Feed::Card *cardIn(const Feed::Row &row, const std::string &key) {
    for (const Feed::Card &card : row.cards) {
        if (card.key == key) {
            return &card;
        }
    }
    return nullptr;
}

} // namespace

int main(int count, char **args) {
    if (count < 2) {
        std::printf("usage: reading <contract fixtures>/valid\n");
        return 2;
    }
    const std::string folder = std::string(args[1]) + "/";

    // ------------------------------------------------- the contract reads, in C++, with no seam
    const std::string homeText = slurp(folder + "Screen.home.json");
    const contract::Screen home = contract::Screen::read(contract::Json::parse(homeText));
    ok(home.name.value == "home", "a screen fixture reads straight into contract::Screen");
    ok(home.sections.size() == 2, "with both of its sections");
    ok(home.sections[0].kind.value == contract::SectionKind::hero &&
           home.sections[1].kind.value == contract::SectionKind::row,
       "a hero and a row, read as the kinds the contract names");
    ok(home.ttl_s == 120 && home.cursor.has_value() && *home.cursor == "c-page-2",
       "and the numbers and the cursor beside them");

    // What a reader kept, measured rather than assumed: write it back out and compare the two
    // documents. A field this surface silently dropped would show here.
    ok(contract::Json::same(contract::Json::parse(contract::Json::render(home)),
                            contract::Json::parse(homeText)),
       "and what was read writes back as the same document, field for field");

    // The contract refuses what it should refuse. A title key is a shape, not a string.
    {
        bool refused = false;
        try {
            contract::TitleKey::read(contract::Json::parse("\"film:1001\""));
        } catch (const contract::Refused &) {
            refused = true;
        }
        ok(refused, "and a title key that is not one is refused rather than carried into a page");
    }

    const std::string listText = slurp(folder + "Screen.my_list.json");
    const contract::Screen list = contract::Screen::read(contract::Json::parse(listText));
    ok(list.sections.size() == 1 && list.sections[0].kind.value == contract::SectionKind::grid,
       "the saved list is one grid, which this surface draws as a row");

    const contract::Nav nav = contract::Nav::read(contract::Json::parse(slurp(folder + "Nav.basic.json")));
    const contract::Profile kid =
            contract::Profile::read(contract::Json::parse(slurp(folder + "Profile.kids.json")));
    const contract::Profile sam =
            contract::Profile::read(contract::Json::parse(slurp(folder + "Profile.main.json")));

    // ------------------------------------------------------------- which rows a page may draw
    {
        const std::vector<contract::Section> none;
        const std::optional<std::string> unasked;
        const std::optional<std::string> askedHome("home");

        ok(streams::reading::answerIsThisPage("home", unasked, "home"),
           "home names no screen on the route, so the answer's own name stands for it");
        ok(!streams::reading::answerIsThisPage("home", std::optional<std::string>("my-list"), "home"),
           "an answer for another screen is not this page's");
        ok(!streams::reading::answerIsThisPage("games", unasked, "games"),
           "and games never owns one, which is the defect of 2026-11-11");

        const std::vector<contract::Section> rows =
                streams::reading::rowsOn("home", askedHome, home.name.value, home.sections, none);
        ok(rows.size() == 1 && rows[0].id == "s1",
           "a home page draws its row and not its hero section");

        const std::vector<contract::Section> grown =
                streams::reading::rowsOn("home", askedHome, home.name.value, home.sections,
                                         list.sections);
        ok(grown.size() == 2 && grown[1].id == "s1" && grown[1].kind.value == "grid",
           "rows read past the first twelve are added when the answer in hand is this page's");

        const std::vector<contract::Section> elsewhere =
                streams::reading::rowsOn("home", std::optional<std::string>("my-list"),
                                         home.name.value, home.sections, list.sections);
        ok(elsewhere.size() == 1,
           "and are left out when it is not, so a page never grows with another page's rows");

        const std::vector<contract::Section> games =
                streams::reading::rowsOn("games", unasked, home.name.value, home.sections,
                                         list.sections);
        ok(games.empty(), "games draws nothing at all, whatever answer is held behind it");
        ok(!streams::reading::heroOn("games", home.sections).has_value(),
           "and has no hero either");

        const std::optional<contract::Tile> lead = streams::reading::heroOn("home", home.sections);
        ok(lead.has_value() && lead->title.value == "movie:1001",
           "the hero is the first tile of the first hero section");

        // A notice is a section and is not a row, and a section with no tiles is not drawn.
        const contract::Section notice =
                contract::Section::read(contract::Json::parse(slurp(folder + "Section.notice.json")));
        const std::vector<contract::Section> noticed = {notice};
        ok(streams::reading::rowsOn("home", askedHome, home.name.value, noticed, none).empty(),
           "a notice is not a row");
        ok(!streams::reading::heroOn("home", noticed).has_value(), "and is not a hero");

        std::vector<contract::Section> hollow = home.sections;
        hollow[1].tiles = std::vector<contract::Tile>();
        ok(streams::reading::rowsOn("home", askedHome, home.name.value, hollow, none).empty(),
           "and a row with no titles in it is an empty line under a name, so it is not drawn");
        hollow[0].tiles = std::vector<contract::Tile>();
        ok(!streams::reading::heroOn("home", hollow).has_value(),
           "nor is a hero section with nothing in it");
    }

    // ------------------------------------------------------------------- the bar, and the face
    {
        const std::vector<contract::NavItem> bar = streams::reading::tabs(nav.items);
        ok(nav.items.size() == 2 && bar.size() == 1 && bar[0].screen.value == "home",
           "the genre item of the menu is not a tab of the bar");

        ok(streams::reading::face(kid, "https://box.invalid").empty(),
           "an avatar that is neither a path nor an address is no picture");
        ok(streams::reading::face(sam, "https://box.invalid").empty(),
           "and a profile with no avatar at all is none either");

        // The same fixture with a path, and with a whole address, read again rather than built.
        const std::string text = slurp(folder + "Profile.kids.json");
        const std::string pathed = [&] {
            std::string made = text;
            const std::size_t at = made.find("\"a2\"");
            return at == std::string::npos ? made : made.replace(at, 4, "\"/pictures/a2.png\"");
        }();
        const contract::Profile onServer =
                contract::Profile::read(contract::Json::parse(pathed));
        ok(streams::reading::face(onServer, "https://box.invalid") ==
                   "https://box.invalid/pictures/a2.png",
           "a path is made absolute against the server, here and nowhere below here");

        const std::string whole = [&] {
            std::string made = text;
            const std::size_t at = made.find("\"a2\"");
            return at == std::string::npos ? made : made.replace(at, 4, "\"https://pics.invalid/a.png\"");
        }();
        const contract::Profile away = contract::Profile::read(contract::Json::parse(whole));
        ok(streams::reading::face(away, "https://box.invalid") == "https://pics.invalid/a.png",
           "and a whole address is left alone");
    }

    // ------------------------------------------------------------------ a tile becomes a card
    {
        const contract::Tile film =
                contract::Tile::read(contract::Json::parse(slurp(folder + "Tile.film.json")));
        const Feed::Card card = streams::reading::cardOf(film);
        ok(card.key == film.title.value && card.name == film.name,
           "a tile's key and name cross unchanged");
        ok(!card.tall.empty() && card.tall == card.wide,
           "a tile with only a poster uses it in both a tall place and a wide one");

        const contract::Tile series = contract::Tile::read(
                contract::Json::parse(slurp(folder + "Tile.series_in_progress.json")));
        const Feed::Card both = streams::reading::cardOf(series);
        ok(both.tall.empty() && both.wide.empty(),
           "and a tile with no pictures at all asks for none rather than for an empty address");
        ok(both.series(), "a tv: key is a series, which is what the hero's Play branch reads");
        ok(both.badges.size() == 1 && both.badges[0].kind == "ranked" && both.badges[0].has_rank &&
               both.badges[0].rank == 2,
           "and its mark crosses as the kind and the number, with no word made for it here");
    }

    // ------------------------------------------------- the whole way: a fixture to a key press
    {
        const std::optional<std::string> asked("home");
        const std::vector<contract::Section> rows = streams::reading::rowsOn(
                "home", asked, home.name.value, home.sections, list.sections);
        const std::optional<contract::Tile> lead = streams::reading::heroOn("home", home.sections);
        const std::vector<contract::NavItem> bar = streams::reading::tabs(nav.items);
        const Feed feed = streams::reading::feedOf(bar, 0, kid, "https://box.invalid", rows, lead,
                                                   std::string());

        ok(feed.tabs.size() == 1 && feed.tabs[0].label == "Home",
           "the page carries the bar's own label, which the server sent and nobody here wrote");
        ok(feed.has_who && feed.who.kids && feed.who.name == "Kids",
           "and who is watching, with the plate a child's profile asks for");
        ok(feed.has_hero && feed.hero.key == "movie:1001", "and the hero");
        ok(feed.rows.size() == 2 && feed.rows[0].cards.size() == 2,
           "and both rows, the first with both of its tiles");
        ok(cardIn(feed.rows[0], "tv:1001") != nullptr, "including the series in it");
        ok(feed.empty.empty(), "and a page with rows says nothing about being empty");

        Place place(feed, 0);
        ok(place.where() == Place::Where::Hero && place.choose() == Place::Chose::Watch,
           "the remote opens on the hero, whose Play starts a film");
        place.press(Word::Down);
        place.press(Word::Right);
        ok(place.where() == Place::Where::Row && place.column() == 1,
           "Down then Right reaches the second tile of the first row");
        const Feed::Card *chosen = place.chosenCard();
        ok(place.choose() == Place::Chose::Title && chosen != nullptr && chosen->key == "tv:1001",
           "and OK there chooses the series the fixture put second, read out of the contract");
        ok(place.wantsMore(), "a page of two rows is already asking for its next ones");
    }

    if (failures > 0) {
        std::printf("%d of %d failed\n", failures, checks);
        return 1;
    }
    std::printf("%d of %d held\n", checks, checks);
    return 0;
}
