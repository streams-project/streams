#include "door.h"

#include <algorithm>

namespace streams::door {
namespace {

// The spaces at either end of what the keyboard typed. The space key is the only key of the sixty
// that can put whitespace in a field, so one character is the whole of what there is to trim; both
// twins reach for a wider trim and neither can meet anything wider.
std::string trimmed(const std::string &text) {
    std::size_t from = 0;
    while (from < text.size() && text[from] == ' ') {
        from += 1;
    }
    std::size_t to = text.size();
    while (to > from && text[to - 1] == ' ') {
        to -= 1;
    }
    return text.substr(from, to - from);
}

} // namespace

Landing::Landing(const std::vector<Card> &cards, bool accounts_required)
    : cards_(cards), accounts_required_(accounts_required) {}

bool Landing::onTop() const {
    return at_ == Stop::Home || at_ == Stop::Tab || at_ == Stop::TopSignIn;
}

Stop Landing::cardStop() const { return page_ == 0 ? Stop::Join : Stop::Watch; }

const Card *Landing::open() const {
    if (page_ <= 0 || page_ > static_cast<int>(cards_.size())) {
        return nullptr;
    }
    return &cards_[static_cast<std::size_t>(page_ - 1)];
}

Did Landing::show(int card) {
    page_ = card;
    at_ = cardStop();
    return Did::Moved;
}

void Landing::settle() {
    const int last = static_cast<int>(cards_.size());
    page_ = std::max(0, std::min(page_, last));
    // The way to sign in only stands at the top right while a title's card is in view, and the
    // tab of the titles only stands at all while there are titles.
    if (page_ == 0 && at_ == Stop::Watch) {
        at_ = Stop::Join;
    }
    if (page_ == 0 && at_ == Stop::TopSignIn) {
        at_ = Stop::Tab;
    }
    if (cards_.empty() && at_ == Stop::Tab) {
        at_ = Stop::Home;
    }
    if (at_ == Stop::LookRound && !openServer()) {
        at_ = Stop::Join;
    }
}

void Landing::arrived(const std::vector<Card> &cards, bool accounts_required) {
    cards_ = cards;
    accounts_required_ = accounts_required;
    settle();
}

// The stops of the showreel card, left to right, which is the one place the look round is named.
//
// `Door.kt:280` writes the same ladder as two conditions -- Left off Get Started is Sign In and
// Left off anything else is Get Help -- and with three stops the two forms are the same function.
// A list says it once, so adding the fourth stop is a line here and nowhere else.
namespace {

std::vector<Stop> buttonsOf(bool open_server) {
    std::vector<Stop> row = {Stop::Help, Stop::SignIn, Stop::Join};
    if (open_server) {
        // The way straight in, where the server says accounts are not required.
        //
        // **The third disagreement this port found.** Neither television twin reads
        // `Features.accounts_required` at all: both load `/v1/public/trending` whatever the server
        // says and send every OK on the landing to the sign in screen. The browser does read it --
        // `apps/web/src/surfaces/door.rs:70-76` draws a `nav.home` button beside Get Started when
        // and only when `!hello.features.accounts_required` -- so on a server that asks for no
        // accounts, a visitor with a browser is let in and a visitor with a television is sent to
        // sign in to an account the server does not want them to have. The core agrees with the
        // browser: `platform/core/src/routes.rs:290` routes a visitor straight through on exactly
        // that flag.
        //
        // This takes the browser's shape and nothing more than its shape: one stop, after Get
        // Started, opening the home page.
        row.push_back(Stop::LookRound);
    }
    return row;
}

int placeOf(const std::vector<Stop> &row, Stop at) {
    for (std::size_t index = 0; index < row.size(); ++index) {
        if (row[index] == at) {
            return static_cast<int>(index);
        }
    }
    return 0;
}

} // namespace

// The row above the cards. `Door.kt:275-287`.
Did Landing::pressTop(Word word) {
    switch (word) {
        case Word::Left: {
            // `:277`. Left at the house is the left edge of the row and of the page, and it stays
            // there: there is nothing to its left and nothing below the landing to fall out of.
            const Stop to = at_ == Stop::TopSignIn ? Stop::Tab : Stop::Home;
            if (to == at_) {
                return Did::Nothing;
            }
            at_ = to;
            return Did::Moved;
        }
        case Word::Right:
            // `:278-282`. The tab is only there when there are titles, and the way to sign in at
            // the top right only while one of their cards is in view.
            if (at_ == Stop::Home && !cards_.empty()) {
                at_ = Stop::Tab;
                return Did::Moved;
            }
            if (at_ == Stop::Tab && page_ > 0) {
                at_ = Stop::TopSignIn;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Down:
            // `:283`. Down comes back to the card that is in view, on that card's own stop --
            // **not** on the stop the viewer left it on. Going up to the row and straight back
            // down from Get Help lands on Get Started. Both twins, and it is the whole of what
            // this screen remembers.
            at_ = cardStop();
            return Did::Moved;
        case Word::Ok:
            // `:284-288`.
            if (at_ == Stop::Home) {
                return show(0);
            }
            if (at_ == Stop::Tab) {
                return show(std::max(page_, 1));
            }
            opening_ = Opening::SignIn;
            return Did::Opens;
        case Word::Up:
        case Word::Back:
            break;
    }
    return Did::Nothing;
}

// The showreel card. `Door.kt:290-299`.
Did Landing::pressCard(Word word) {
    const std::vector<Stop> row = buttonsOf(openServer());
    const int place = placeOf(row, at_);
    switch (word) {
        case Word::Left:
            // `:291`.
            if (place <= 0) {
                return Did::Nothing;
            }
            at_ = row[static_cast<std::size_t>(place - 1)];
            return Did::Moved;
        case Word::Right:
            // `:293-297`. Past the last button the next card comes in, as on the reference.
            if (place + 1 < static_cast<int>(row.size())) {
                at_ = row[static_cast<std::size_t>(place + 1)];
                return Did::Moved;
            }
            if (cards_.empty()) {
                return Did::Nothing;
            }
            return show(1);
        case Word::Up:
            // `:298`.
            at_ = Stop::Home;
            return Did::Moved;
        case Word::Ok:
            // `:299`. Get Started opens the sign in screen too: see `Opening::SignIn`.
            opening_ = at_ == Stop::Help      ? Opening::Help
                       : at_ == Stop::LookRound ? Opening::Home
                                                : Opening::SignIn;
            return Did::Opens;
        case Word::Down:
        case Word::Back:
            break;
    }
    return Did::Nothing;
}

// A trending title's card. `Door.kt:302-308`.
Did Landing::pressTitle(Word word) {
    switch (word) {
        case Word::Left:
            // `:303`. There is no left edge here: the card to the left of the first title is the
            // showreel, so Left always moves and the viewer walks back out the way they came.
            return show(page_ - 1);
        case Word::Right:
            // `:304`.
            if (page_ >= static_cast<int>(cards_.size())) {
                return Did::Nothing;
            }
            return show(page_ + 1);
        case Word::Up:
            // `:305`. Up from a title's card goes to the tab, not to the house.
            at_ = Stop::Tab;
            return Did::Moved;
        case Word::Ok:
            // `:306`.
            opening_ = Opening::SignIn;
            return Did::Opens;
        case Word::Down:
        case Word::Back:
            break;
    }
    return Did::Nothing;
}

Did Landing::press(Word word) {
    Did did = Did::Nothing;
    if (word == Word::Back) {
        // `Door.kt:273-276`. Back walks out the way the viewer came in: the row to its card, a
        // title to the showreel, and only the showreel's buttons have nowhere further back.
        if (onTop()) {
            at_ = cardStop();
            did = Did::Moved;
        } else if (page_ > 0) {
            did = show(0);
        } else {
            did = Did::Leaves;
        }
    } else if (onTop()) {
        did = pressTop(word);
    } else if (page_ == 0) {
        did = pressCard(word);
    } else {
        did = pressTitle(word);
    }
    // `Door.kt:254`, which is a line of the composition and so runs after every press: the lockup
    // gives way to the row the first time the viewer goes up to it or along to a title, and
    // nothing ever puts it back.
    if (onTop() || page_ > 0) {
        raised_ = true;
    }
    return did;
}

SignIn::SignIn() : keys_(std::string()) {}

void SignIn::forget() { password_.clear(); }

// `SignIn.kt:160-176`, branch for branch and in the same order.
Asked SignIn::finish(bool has_text) {
    if (!has_text && ask_ == Ask::Password) {
        // `:162-165`. Cancelling the password goes back to the email step with the address that
        // was typed, rather than out of the keyboard: the viewer is one step back, not two.
        ask_ = Ask::Email;
        keys_.hold(email_);
        return Asked::Moved;
    }
    if (!has_text) {
        // `:166`. And cancelling the email leaves the keyboard for the two pills.
        on_keys_ = false;
        return Asked::Moved;
    }
    if (ask_ == Ask::Email) {
        // `:167-171`.
        email_ = trimmed(keys_.text());
        ask_ = Ask::Password;
        keys_.hold(std::string());
        return Asked::Moved;
    }
    // `:172-175`. The field is emptied at once and the remote stays on the password step, so a
    // server that refuses draws its notice under an empty field and the viewer types again.
    password_ = keys_.text();
    keys_.hold(std::string());
    return Asked::Asks;
}

Asked SignIn::press(Word word) {
    // Stricter than `gate.h`, which keeps its digits until the shell says `forget`: here the
    // password is gone by the next press whatever the shell did with it. It costs nothing, because
    // a shell reads what a press answered in the same turn it answered it.
    forget();
    if (on_keys_) {
        const streams::keys::Did did = keys_.press(word);
        switch (did) {
            case streams::keys::Did::Above:
                // `SignIn.kt:179`. The one word the keyboard does not take hands the remote back
                // to the two pills -- which is what makes Up on the top row of keys the way out,
                // and what a held Up does one word at a time until it is out.
                on_keys_ = false;
                return Asked::Moved;
            case streams::keys::Did::Went:
                return finish(false);
            case streams::keys::Did::Done:
                return finish(true);
            case streams::keys::Did::Nothing:
                return Asked::Nothing;
            case streams::keys::Did::Moved:
            case streams::keys::Did::Typed:
            case streams::keys::Did::Turned:
                return Asked::Moved;
        }
        return Asked::Nothing;
    }
    // The two pills. `SignIn.kt:183-190`.
    switch (word) {
        case Word::Left:
            // `:185`. Left past the first pill goes back, which is the rule `Pin::press` follows
            // at the first digit and note 187's "Left at the first stop goes back".
            //
            // One thing to know about it on a controller: this is a direction, and a direction
            // repeats while it is held (`remote.h`'s `Repeat`). A held Left on the Phone pill
            // therefore answers `Went` again every 120 ms. OK and Back are press edges and never
            // repeat, so this is the one way out of this screen that a hold can say twice, and the
            // shell has to be able to hear it twice. It is not fixed here: a rule about which
            // words may repeat belongs in `remote.h` beside the clock that repeats them, and
            // putting it in this screen would be a second home for it. See the README.
            if (way_ == Way::Phone) {
                return Asked::Went;
            }
            way_ = Way::Phone;
            return Asked::Moved;
        case Word::Right:
            // `:186`.
            if (way_ == Way::Remote) {
                return Asked::Nothing;
            }
            way_ = Way::Remote;
            return Asked::Moved;
        case Word::Down:
        case Word::Ok:
            // `:187`. Either word goes down into the keys, and neither does anything on the Phone
            // way, which has nothing below it but a code the core keeps alive.
            if (way_ != Way::Remote) {
                return Asked::Nothing;
            }
            on_keys_ = true;
            return Asked::Moved;
        case Word::Back:
            // `:188`.
            return Asked::Went;
        case Word::Up:
            // `:189`.
            break;
    }
    return Asked::Nothing;
}

} // namespace streams::door
