#include "title.h"

#include <algorithm>

namespace streams::title {
namespace {

// The wire prefix a series key carries. `shared/Addresses.kt:17`, the same test `place.cpp` makes.
const char *const SeriesPrefix = "tv:";

int clamp(int value, int least, int most) { return std::min(std::max(value, least), most); }

} // namespace

bool Detail::series() const { return key.rfind(SeriesPrefix, 0) == 0; }

const Season *Detail::season(int index) const {
    if (index < 0 || static_cast<std::size_t>(index) >= seasons.size()) {
        return nullptr;
    }
    return &seasons[static_cast<std::size_t>(index)];
}

int Detail::seasonAt(std::int64_t number) const {
    for (std::size_t at = 0; at < seasons.size(); ++at) {
        if (seasons[at].number == number) {
            return static_cast<int>(at);
        }
    }
    return -1;
}

// ------------------------------------------------------------------------------- the summary

Summary::Summary(const Detail &detail) { rebuild(detail); }

void Summary::rebuild(const Detail &detail) {
    detail_ = detail;
    // `TitlePage.kt:101-106`: Play and My List always, and the two middle lines only when the
    // title has what they open.
    lines_.clear();
    lines_.push_back(Line::Play);
    if (!detail_.seasons.empty()) {
        lines_.push_back(Line::Episodes);
    }
    if (!detail_.more.empty()) {
        lines_.push_back(Line::More);
    }
    lines_.push_back(Line::List);
    has_rating_ = detail_.has_rating;
    rating_ = detail_.rating;
    in_list_ = detail_.in_list;
}

void Summary::arrived(const Detail &detail) { rebuild(detail); }

int Summary::line() const {
    return clamp(line_, 0, static_cast<int>(lines_.size()) - 1);
}

Play Summary::play() const {
    Play asked;
    if (!detail_.series()) {
        // `TitlePage.kt:115-118`. A film plays itself, from wherever the core left it.
        return asked;
    }
    asked.episode = true;
    if (detail_.has_resume && detail_.resume_episode) {
        // `:120-124`. The episode the viewer is on, which the title's own answer carries.
        asked.season = detail_.resume_season;
        asked.number = detail_.resume_number;
        asked.known = true;
        return asked;
    }
    // `:126-130`. The first season's number, and an episode the screen has to go and ask for.
    const Season *first = detail_.season(0);
    asked.season = first == nullptr ? 1 : first->number;
    asked.known = false;
    return asked;
}

// A translation of `TitlePage.kt:143-171` branch for branch and in the same order.
Did Summary::press(Word word) {
    const int last = static_cast<int>(lines_.size()) - 1;
    const int row = line();
    switch (word) {
        case Word::Up:
            // `:145-149`: up the menu, then off the top of it onto the reactions.
            if (part_ == Part::Menu && row > 0) {
                line_ = row - 1;
                return Did::Moved;
            }
            if (part_ == Part::Menu) {
                part_ = Part::Reactions;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Down:
            // `:150-153`.
            if (part_ == Part::Reactions) {
                part_ = Part::Menu;
                return Did::Moved;
            }
            if (row < last) {
                line_ = row + 1;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Left:
            // `:155-158`. The menu sits at the left edge, so Left from it goes back, as Back does:
            // note 187's "Title page: Left on the menu goes back", pressed on the Streamer and
            // verified again in note 239 section 4.
            if (part_ == Part::Reactions && face_ > 0) {
                face_ -= 1;
                return Did::Moved;
            }
            if (part_ == Part::Menu) {
                return Did::Went;
            }
            return Did::Nothing;
        case Word::Right:
            // `:159`. Right at the last reaction stays: note 239 pressed it ("Title page
            // reactions, Left at 'Not for me': stays") and recorded it as not in note 187.
            if (part_ == Part::Reactions && face_ < Faces - 1) {
                face_ += 1;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Ok: {
            // `:160-167`.
            if (part_ == Part::Reactions) {
                // `:132-136`: the same rating again takes it off.
                const Face asked = static_cast<Face>(face_);
                if (has_rating_ && rating_ == asked) {
                    has_rating_ = false;
                } else {
                    has_rating_ = true;
                    rating_ = asked;
                }
                return Did::Rated;
            }
            if (row > last || row < 0) {
                return Did::Nothing;
            }
            switch (lines_[static_cast<std::size_t>(row)]) {
                case Line::Play: return Did::Watch;
                case Line::Episodes: return Did::Moved;
                case Line::More: return Did::Moved;
                case Line::List:
                    // `:138-141`.
                    in_list_ = !in_list_;
                    return Did::Listed;
            }
            return Did::Nothing;
        }
        case Word::Back:
            // `:168`. From the reactions Back returns to the menu first, then leaves the title.
            if (part_ == Part::Reactions) {
                part_ = Part::Menu;
                return Did::Moved;
            }
            return Did::Went;
        default:
            return Did::Nothing;
    }
}

// ------------------------------------------------------------------------------- the episodes

Episodes::Episodes(const Detail &detail) : detail_(detail) {
    // `Episodes.kt:139-143`. The page opens on the season the viewer is on, and on its first
    // episode until the episodes arrive and say where that viewer is.
    const std::int64_t season = detail_.has_resume && detail_.resume_episode ? detail_.resume_season : -1;
    const int found = detail_.seasonAt(season);
    chosen_ = found < 0 ? 0 : found;
    line_ = chosen_;
    card_ = 0;
    in_seasons_ = false;
    placed_ = !(detail_.has_resume && detail_.resume_episode);
}

std::int64_t Episodes::asking() const {
    const Season *season = detail_.season(chosen_);
    return season == nullptr ? 1 : season->number;
}

void Episodes::arrived(std::int64_t number, const std::vector<Episode> &episodes) {
    number_ = number;
    episodes_ = episodes;
    // `Episodes.kt:150-156`. Once, and only for the season the viewer is on.
    if (placed_ || episodes_.empty()) {
        return;
    }
    const Season *season = detail_.season(chosen_);
    if (season == nullptr || season->number != detail_.resume_season) {
        return;
    }
    card_ = 0;
    for (std::size_t at = 0; at < episodes_.size(); ++at) {
        if (episodes_[at].number == detail_.resume_number) {
            card_ = static_cast<int>(at);
            break;
        }
    }
    placed_ = true;
}

const Episode *Episodes::open() const {
    if (card_ < 0 || static_cast<std::size_t>(card_) >= episodes_.size()) {
        return nullptr;
    }
    return &episodes_[static_cast<std::size_t>(card_)];
}

Play Episodes::play() const {
    Play asked;
    const Episode *episode = open();
    if (episode == nullptr) {
        return asked;
    }
    asked.episode = true;
    // The season the episode in hand belongs to, which is not always the season that is chosen.
    //
    // `Episodes.kt:145` builds the address out of `number`, the chosen season, while `episodes`
    // comes from a `produceState` that keeps its old value until the new season answers. Choose a
    // season, press OK before the answer lands, and the Kotlin plays the new season's number with
    // the old season's episode: `/v1/.../2/7` where season 2 may have no episode 7. The Swift
    // reads the same two the same way. Here the arrival says which season it brought, so the
    // episode under the remote is played from the season it is actually in.
    asked.season = number_;
    asked.number = episode->number;
    asked.known = true;
    return asked;
}

// A translation of `Episodes.kt:159-180` branch for branch and in the same order.
Did Episodes::press(Word word) {
    const int lastSeason = static_cast<int>(detail_.seasons.size()) - 1;
    const int lastCard = static_cast<int>(episodes_.size()) - 1;
    switch (word) {
        case Word::Up: {
            // `:161`.
            const int was = in_seasons_ ? line_ : card_;
            const int now = std::max(0, was - 1);
            (in_seasons_ ? line_ : card_) = now;
            return now == was ? Did::Nothing : Did::Moved;
        }
        case Word::Down: {
            // `:162`, with the Swift's floor rather than the Kotlin's.
            //
            // `Episodes.kt:162` is `minOf(episodes.lastIndex, card + 1)`, and the last index of an
            // empty list is -1: a season whose episodes have not arrived, or a season the server
            // answers empty, puts the card at -1 and the page draws from -2. Nothing throws,
            // because the Kotlin reads the list with `getOrNull`, and nothing is right either.
            // `Episodes.swift:105` wraps the same line in `max(0, ...)` and that is what this
            // takes, for the reason `RowMemory::columnOf` gives: a bounds answer is not a focus
            // answer.
            const int was = in_seasons_ ? line_ : card_;
            const int most = in_seasons_ ? lastSeason : lastCard;
            const int now = std::max(0, std::min(most, was + 1));
            (in_seasons_ ? line_ : card_) = now;
            return now == was ? Did::Nothing : Did::Moved;
        }
        case Word::Left:
            // `:164-169`. From the cards Left opens the seasons at the one shown; from the seasons
            // it goes back, which is note 187's "Left on the seasons goes back to the title".
            if (in_seasons_) {
                return Did::Went;
            }
            in_seasons_ = true;
            line_ = chosen_;
            return Did::Moved;
        case Word::Right:
            // `:170`.
            if (!in_seasons_) {
                return Did::Nothing;
            }
            in_seasons_ = false;
            return Did::Moved;
        case Word::Ok:
            // `:171-177`.
            if (in_seasons_) {
                chosen_ = line_;
                card_ = 0;
                in_seasons_ = false;
                // Choosing the season already chosen still leaves the column, so the press is
                // never nothing.
                return Did::Moved;
            }
            return open() == nullptr ? Did::Nothing : Did::Watch;
        case Word::Back:
            // `:178`.
            return Did::Went;
        default:
            return Did::Nothing;
    }
}

// ------------------------------------------------------------------------------- the grid

Posters::Posters(const std::vector<Feed::Card> &cards, int across)
    : cards_(cards), across_(std::max(across, 1)), at_(0) {}

int Posters::step(int at, Word word, int last, int across) {
    const int wide = std::max(across, 1);
    switch (word) {
        case Word::Up: return at >= wide ? at - wide : at;
        case Word::Down: return at + wide <= last ? at + wide : at;
        case Word::Left: return at % wide > 0 ? at - 1 : at;
        case Word::Right: return (at % wide < wide - 1 && at < last) ? at + 1 : at;
        default: return at;
    }
}

const Feed::Card *Posters::open() const {
    if (at_ < 0 || static_cast<std::size_t>(at_) >= cards_.size()) {
        return nullptr;
    }
    return &cards_[static_cast<std::size_t>(at_)];
}

// A translation of `MoreLike.kt:33-45` branch for branch and in the same order.
Did Posters::press(Word word) {
    const int last = static_cast<int>(cards_.size()) - 1;
    if (word == Word::Ok) {
        return open() == nullptr ? Did::Nothing : Did::Opened;
    }
    if (word == Word::Back) {
        return Did::Went;
    }
    if (word == Word::Left && at_ % across_ == 0) {
        // The first column sits at the title's name, so Left there returns to the title: note
        // 187's "More Like This: Left at the first column goes back", verified on the Streamer.
        return Did::Went;
    }
    const int now = step(at_, word, last, across_);
    if (now == at_) {
        return Did::Nothing;
    }
    at_ = now;
    return Did::Moved;
}

// ------------------------------------------------------------------------------- the title

Title::Title(const Detail &detail, int across)
    : detail_(detail), across_(std::max(across, 1)), page_(Page::Summary), summary_(detail),
      episodes_(detail), posters_(detail.more, std::max(across, 1)) {}

void Title::arrived(const Detail &detail) {
    const bool same = detail.key == detail_.key;
    detail_ = detail;
    if (!same) {
        // `TitlePage.kt:89-90` and `TitlePage.swift:79-85`: a new title starts at its summary, on
        // its first line.
        page_ = Page::Summary;
        summary_ = Summary(detail_);
        episodes_ = Episodes(detail_);
        posters_ = Posters(detail_.more, across_);
        has_opened_ = false;
        return;
    }
    summary_.arrived(detail_);
}

Did Title::press(Word word) {
    if (page_ == Page::Episodes) {
        const Did did = episodes_.press(word);
        if (did == Did::Went) {
            // The page below is the summary, which kept the line that opened this one.
            page_ = Page::Summary;
            return Did::Moved;
        }
        if (did == Did::Watch) {
            watching_ = episodes_.play();
        }
        return did;
    }
    if (page_ == Page::More) {
        const Did did = posters_.press(word);
        if (did == Did::Went) {
            page_ = Page::Summary;
            return Did::Moved;
        }
        if (did == Did::Opened) {
            const Feed::Card *card = posters_.open();
            if (card == nullptr) {
                return Did::Nothing;
            }
            opened_ = *card;
            has_opened_ = true;
        }
        return did;
    }
    const Did did = summary_.press(word);
    if (did == Did::Watch) {
        watching_ = summary_.play();
        return did;
    }
    if (did != Did::Moved || word != Word::Ok) {
        return did;
    }
    // OK on the two lines that open a page of their own. The summary answers `Moved` for both and
    // says nothing about which, because which page a line opens is this class's question.
    const std::vector<Line> &lines = summary_.lines();
    const Line line = lines[static_cast<std::size_t>(summary_.line())];
    if (line == Line::Episodes) {
        episodes_ = Episodes(detail_);
        page_ = Page::Episodes;
    } else if (line == Line::More) {
        posters_ = Posters(detail_.more, across_);
        page_ = Page::More;
    }
    return Did::Moved;
}

} // namespace streams::title
