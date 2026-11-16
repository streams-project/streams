#include "reading.h"

namespace streams::reading {
namespace {

// The tab that has nothing to show yet. The server builds no rows for it. `Page.kt:21`.
const char *const Games = "games";

// The item of the bar that is not a tab of it. `TopBar.kt:42`.
const char *const Genre = "genre";

// How many tabs the bar holds. `TopBar.kt:42`.
constexpr std::size_t Tabs = 6;

// The section kinds this surface draws as a row of tiles. Anything else is not a row.
// `Page.kt:24`.
bool isRow(const contract::SectionKind &kind) {
    return kind.value == contract::SectionKind::row || kind.value == contract::SectionKind::ranked ||
           kind.value == contract::SectionKind::continue_ ||
           kind.value == contract::SectionKind::grid;
}

// A section with no tiles is not drawn: a row with no titles in it is an empty line under a name.
// `Page.kt:59`.
bool filled(const contract::Section &section) {
    return section.tiles.has_value() && !section.tiles->empty();
}

void addFilled(std::vector<contract::Section> &into, const std::vector<contract::Section> &from) {
    for (const contract::Section &section : from) {
        if (filled(section)) {
            into.push_back(section);
        }
    }
}

// The first picture with this role, or nothing.
std::string roled(const std::optional<std::vector<contract::Image>> &images,
                  std::string_view role) {
    if (!images.has_value()) {
        return std::string();
    }
    for (const contract::Image &image : *images) {
        if (image.role.value == role) {
            return image.url;
        }
    }
    return std::string();
}

} // namespace

bool answerIsThisPage(const std::string &current, const std::optional<std::string> &asked,
                      const std::string &answerName) {
    return current != Games && (!asked.has_value() || *asked == answerName);
}

std::vector<contract::Section> rowsOn(const std::string &current,
                                      const std::optional<std::string> &asked,
                                      const std::string &answerName,
                                      const std::vector<contract::Section> &answer,
                                      const std::vector<contract::Section> &extra) {
    std::vector<contract::Section> held;
    if (current != Games) {
        addFilled(held, answer);
    }
    if (answerIsThisPage(current, asked, answerName)) {
        addFilled(held, extra);
    }
    std::vector<contract::Section> out;
    for (const contract::Section &section : held) {
        if (isRow(section.kind)) {
            out.push_back(section);
        }
    }
    return out;
}

std::optional<contract::Tile> heroOn(const std::string &current,
                                     const std::vector<contract::Section> &answer) {
    if (current == Games) {
        return std::nullopt;
    }
    for (const contract::Section &section : answer) {
        if (!filled(section)) {
            continue;
        }
        if (section.kind.value == contract::SectionKind::hero) {
            return (*section.tiles)[0];
        }
    }
    return std::nullopt;
}

std::string wide(const std::optional<std::vector<contract::Image>> &images) {
    std::string found = roled(images, contract::ImageRole::backdrop);
    if (!found.empty()) {
        return found;
    }
    found = roled(images, contract::ImageRole::still);
    if (!found.empty()) {
        return found;
    }
    return roled(images, contract::ImageRole::poster);
}

std::string tall(const std::optional<std::vector<contract::Image>> &images) {
    const std::string found = roled(images, contract::ImageRole::poster);
    return found.empty() ? wide(images) : found;
}

std::string face(const contract::Profile &person, const std::string &server) {
    if (!person.avatar.has_value()) {
        return std::string();
    }
    const std::string &avatar = *person.avatar;
    if (avatar.rfind('/', 0) == 0) {
        return server + avatar;
    }
    if (avatar.rfind("http", 0) == 0) {
        return avatar;
    }
    return std::string();
}

std::vector<contract::NavItem> tabs(const std::vector<contract::NavItem> &nav) {
    std::vector<contract::NavItem> out;
    for (const contract::NavItem &item : nav) {
        if (item.screen.value == Genre) {
            continue;
        }
        out.push_back(item);
        if (out.size() == Tabs) {
            break;
        }
    }
    return out;
}

place::Feed::Card cardOf(const contract::Tile &tile) {
    place::Feed::Card card;
    card.key = tile.title.value;
    card.name = tile.name;
    card.tall = tall(tile.images);
    card.wide = wide(tile.images);
    if (tile.badges.has_value()) {
        for (const contract::Badge &badge : *tile.badges) {
            place::Feed::Card::Mark mark;
            mark.kind = badge.kind.value;
            mark.has_rank = badge.rank.has_value();
            mark.rank = badge.rank.value_or(0);
            card.badges.push_back(mark);
        }
    }
    return card;
}

place::Feed feedOf(const std::vector<contract::NavItem> &bar, int active,
                   const std::optional<contract::Profile> &person, const std::string &server,
                   const std::vector<contract::Section> &rows,
                   const std::optional<contract::Tile> &lead, const std::string &empty) {
    place::Feed feed;
    for (const contract::NavItem &item : bar) {
        place::Feed::Tab tab;
        tab.label = item.label;
        tab.screen = item.screen.value;
        feed.tabs.push_back(tab);
    }
    feed.active = active;
    if (person.has_value()) {
        feed.has_who = true;
        feed.who.name = person->name;
        feed.who.picture = face(*person, server);
        feed.who.kids = person->kids;
    }
    feed.server = server;
    for (const contract::Section &section : rows) {
        place::Feed::Row row;
        row.id = section.id;
        row.title = section.title.value_or(std::string());
        if (section.tiles.has_value()) {
            for (const contract::Tile &tile : *section.tiles) {
                row.cards.push_back(cardOf(tile));
            }
        }
        feed.rows.push_back(row);
    }
    if (lead.has_value()) {
        feed.has_hero = true;
        feed.hero = cardOf(*lead);
    }
    feed.empty = empty;
    return feed;
}

watch::Window windowOf(const contract::TimeWindow &window) {
    watch::Window out;
    out.kind = window.kind.value;
    out.start_ms = window.start_ms;
    out.end_ms = window.end_ms;
    out.has_seek_to = window.seek_to_ms.has_value();
    out.seek_to_ms = window.seek_to_ms.value_or(0);
    out.text = window.text.value_or(std::string());
    return out;
}

std::vector<watch::Window> windowsOf(const std::vector<contract::TimeWindow> &windows) {
    std::vector<watch::Window> out;
    out.reserve(windows.size());
    for (const contract::TimeWindow &window : windows) {
        out.push_back(windowOf(window));
    }
    return out;
}

watch::Play playOf(const contract::PlayView &view) {
    watch::Play out;
    out.title = view.title.value;
    if (view.state.value == contract::PlayState::resolving) {
        out.state = watch::State::Resolving;
    } else if (view.state.value == contract::PlayState::starting) {
        out.state = watch::State::Starting;
    } else if (view.state.value == contract::PlayState::playing) {
        out.state = watch::State::Playing;
    } else if (view.state.value == contract::PlayState::paused) {
        out.state = watch::State::Paused;
    } else if (view.state.value == contract::PlayState::stalled) {
        out.state = watch::State::Stalled;
    } else if (view.state.value == contract::PlayState::switching) {
        out.state = watch::State::Switching;
    } else if (view.state.value == contract::PlayState::ended) {
        out.state = watch::State::Ended;
    } else if (view.state.value == contract::PlayState::failed) {
        out.state = watch::State::Failed;
    }
    if (view.episode.has_value()) {
        out.has_episode = true;
        out.episode.season = view.episode->season;
        out.episode.episode = view.episode->episode;
    }
    if (view.next_episode.has_value()) {
        out.has_next_episode = true;
        out.next_episode.season = view.next_episode->season;
        out.next_episode.episode = view.next_episode->episode;
    }
    out.still_asking = view.still_asking.value_or(false);
    if (view.windows.has_value()) {
        out.windows = windowsOf(*view.windows);
    }
    return out;
}

} // namespace streams::reading
