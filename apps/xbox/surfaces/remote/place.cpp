#include "place.h"

#include <algorithm>

namespace streams::place {
namespace {

// The wire prefix a series key carries. `shared/Addresses.kt:17`.
const char *const SeriesPrefix = "tv:";

} // namespace

bool Feed::Card::series() const { return key.rfind(SeriesPrefix, 0) == 0; }

std::vector<std::string> Feed::Row::keys() const {
    std::vector<std::string> out;
    out.reserve(cards.size());
    for (const Card &card : cards) {
        out.push_back(card.key);
    }
    return out;
}

const Feed::Card *Feed::Row::at(int index) const {
    if (index < 0 || static_cast<std::size_t>(index) >= cards.size()) {
        return nullptr;
    }
    return &cards[static_cast<std::size_t>(index)];
}

const Feed::Row *Feed::row(int index) const {
    if (index < 0 || static_cast<std::size_t>(index) >= rows.size()) {
        return nullptr;
    }
    return &rows[static_cast<std::size_t>(index)];
}

int Feed::lastRow() const { return static_cast<int>(rows.size()) - 1; }

const Feed::Tab *Feed::tab(int index) const {
    if (index < 0 || static_cast<std::size_t>(index) >= tabs.size()) {
        return nullptr;
    }
    return &tabs[static_cast<std::size_t>(index)];
}

int RowMemory::columnOf(const std::string &id, const std::vector<std::string> &titles) const {
    const auto found = columns_.find(id);
    if (found == columns_.end()) {
        return 0;
    }
    const Kept &kept = found->second;
    const int last = static_cast<int>(titles.size()) - 1;
    if (kept.at >= 0 && kept.at <= last && kept.titled &&
        titles[static_cast<std::size_t>(kept.at)] == kept.title) {
        return kept.at;
    }
    if (kept.titled) {
        for (int at = 0; at <= last; ++at) {
            if (titles[static_cast<std::size_t>(at)] == kept.title) {
                return at;
            }
        }
    }
    return std::min(std::max(kept.at, 0), std::max(last, 0));
}

void RowMemory::keep(const std::string &id, const std::vector<std::string> &titles, int at) {
    Kept kept;
    kept.at = at;
    kept.titled = at >= 0 && static_cast<std::size_t>(at) < titles.size();
    if (kept.titled) {
        kept.title = titles[static_cast<std::size_t>(at)];
    }
    columns_[id] = kept;
}

bool RowMemory::knows(const std::string &id) const { return columns_.find(id) != columns_.end(); }

void RowMemory::clear() { columns_.clear(); }

Place::Place(const Feed &feed, int active)
    : feed_(feed), tabs_(static_cast<int>(feed.tabs.size())), active_(active),
      hero_(feed.has_hero) {
    where_ = (hero_ || feed_.rows.empty()) ? above() : Where::Row;
    row_ = 0;
    tab_ = active;
    button_ = 0;
}

void Place::arrived(const Feed &feed) {
    feed_ = feed;
    tabs_ = static_cast<int>(feed_.tabs.size());
    hero_ = feed_.has_hero;
}

int Place::columnOf(int index) const {
    const Feed::Row *found = feed_.row(index);
    if (found == nullptr) {
        return 0;
    }
    return columns_.columnOf(found->id, found->keys());
}

int Place::column() const { return columnOf(row_); }

void Place::keep(int at) {
    const Feed::Row *found = feed_.row(row_);
    if (found == nullptr) {
        return;
    }
    columns_.keep(found->id, found->keys(), at);
}

int Place::lastColumn() const {
    const Feed::Row *found = feed_.row(row_);
    return found == nullptr ? -1 : static_cast<int>(found->cards.size()) - 1;
}

// A translation of `Browse.kt:107-151` and of `Place.java:press` branch for branch and in the same
// order, because in one place the order is the behaviour.
//
// Going right along the bar, the two negative places (WHO_TAB at -3 and SEARCH_TAB at -2) have to
// be tried before `tab < tabs - 1`, because both of them satisfy that test as well: a reordered
// chain would step -3 to -2 and then -2 to -1, which is no place at all, and the profile would walk
// sideways into nothing instead of onto search. Going left the branches are disjoint and the order
// does not matter, which is worth saying because it is the half a reader would assume was the
// dangerous one. Both halves are held by `place_test.cpp` and were proved by mutation.
bool Place::press(Word word) {
    switch (word) {
        case Word::Up:
            if (where_ == Where::Row && row_ > 0) {
                row_ -= 1;
            } else if (where_ == Where::Row) {
                where_ = above();
            } else if (where_ == Where::Hero) {
                where_ = Where::Top;
            } else {
                return false;
            }
            return true;
        case Word::Down:
            if (where_ == Where::Top && hero_) {
                where_ = Where::Hero;
            } else if ((where_ == Where::Hero || where_ == Where::Top) && !feed_.rows.empty()) {
                where_ = Where::Row;
                row_ = 0;
            } else if (where_ == Where::Row && row_ < feed_.lastRow()) {
                row_ += 1;
            } else {
                return false;
            }
            return true;
        case Word::Left:
            if (where_ == Where::Top && tab_ > 0) {
                tab_ -= 1;
            } else if (where_ == Where::Top && tab_ == 0) {
                tab_ = Feed::SEARCH_TAB;
            } else if (where_ == Where::Top && tab_ == Feed::SEARCH_TAB) {
                tab_ = Feed::WHO_TAB;
            } else if (where_ == Where::Hero) {
                button_ = 0;
            } else if (where_ == Where::Row && column() > 0) {
                keep(column() - 1);
            } else {
                return false;
            }
            return true;
        case Word::Right:
            if (where_ == Where::Top && tab_ == Feed::WHO_TAB) {
                tab_ = Feed::SEARCH_TAB;
            } else if (where_ == Where::Top && tab_ == Feed::SEARCH_TAB) {
                tab_ = 0;
            } else if (where_ == Where::Top && tab_ < tabs_ - 1) {
                tab_ += 1;
            } else if (where_ == Where::Hero) {
                button_ = 1;
            } else if (where_ == Where::Row && column() < lastColumn()) {
                keep(column() + 1);
            } else {
                return false;
            }
            return true;
        case Word::Back:
            if (where_ == Where::Row) {
                where_ = above();
            } else if (where_ == Where::Hero) {
                where_ = Where::Top;
            } else if (tab_ != active_) {
                // Focus wandered along the bar without choosing: Back brings it home to this page's
                // tab first, and only then leaves the page. Note 187's own row, `Browse.kt:143-146`.
                tab_ = active_;
            } else {
                return false;
            }
            return true;
        case Word::Ok:
        default:
            return false;
    }
}

Place::Chose Place::choose() const {
    if (where_ == Where::Top && tab_ == Feed::WHO_TAB) {
        return Chose::Menu;
    }
    if (where_ == Where::Top && tab_ == Feed::SEARCH_TAB) {
        return Chose::Search;
    }
    if (where_ == Where::Top) {
        return feed_.tab(tab_) == nullptr ? Chose::Nothing : Chose::Tab;
    }
    if (where_ == Where::Row) {
        return chosenCard() == nullptr ? Chose::Nothing : Chose::Title;
    }
    if (!feed_.has_hero) {
        return Chose::Nothing;
    }
    return (button_ == 1 || feed_.hero.series()) ? Chose::Title : Chose::Watch;
}

const Feed::Tab *Place::chosenTab() const {
    return where_ == Where::Top ? feed_.tab(tab_) : nullptr;
}

const Feed::Card *Place::chosenCard() const {
    if (where_ == Where::Row) {
        const Feed::Row *found = feed_.row(row_);
        return found == nullptr ? nullptr : found->at(column());
    }
    if (where_ == Where::Hero) {
        return feed_.has_hero ? &feed_.hero : nullptr;
    }
    return nullptr;
}

const Feed::Card *Place::open() const {
    const Feed::Row *found = feed_.row(row_);
    return found == nullptr ? nullptr : found->at(column());
}

bool Place::wantsMore() const {
    return feed_.rows.empty() || (where_ == Where::Row && row_ >= feed_.lastRow() - Reach);
}

} // namespace streams::place
