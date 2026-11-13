#include "gate.h"

#include <algorithm>

namespace streams::gate {

Pin::Pin(int digits) : digits_(static_cast<std::size_t>(std::max(digits, 1)), 0) {}

int Pin::digit(int index) const {
    if (index < 0 || static_cast<std::size_t>(index) >= digits_.size()) {
        return -1;
    }
    return digits_[static_cast<std::size_t>(index)];
}

std::string Pin::value() const {
    std::string out;
    out.reserve(digits_.size());
    for (int digit : digits_) {
        out.push_back(static_cast<char>('0' + digit));
    }
    return out;
}

void Pin::answered(bool held) {
    asking_ = false;
    refused_ = !held;
}

// A translation of `PinPanel.kt:97-111` branch for branch and in the same order.
Did Pin::press(Word word) {
    const int last = digits() - 1;
    switch (word) {
        case Word::Left:
            // `:99`. Left past the first digit goes back, as Back does: note 187's "PIN: Left at
            // the first digit goes back".
            if (at_ > 0) {
                at_ -= 1;
                return Did::Moved;
            }
            return Did::Went;
        case Word::Right:
            // `:100`.
            if (at_ < last) {
                at_ += 1;
                return Did::Moved;
            }
            return Did::Nothing;
        case Word::Up:
            // `:101-104`. Every digit is a wheel, so the ends do not stop it.
            digits_[static_cast<std::size_t>(at_)] = (digits_[static_cast<std::size_t>(at_)] + 1) % 10;
            refused_ = false;
            return Did::Moved;
        case Word::Down:
            // `:105-108`.
            digits_[static_cast<std::size_t>(at_)] = (digits_[static_cast<std::size_t>(at_)] + 9) % 10;
            refused_ = false;
            return Did::Moved;
        case Word::Ok:
            // `:109`, and `:80-81`'s guard with it: a held A sends once.
            if (asking_) {
                return Did::Nothing;
            }
            asking_ = true;
            return Did::Asks;
        case Word::Back:
            // `:110`.
            return Did::Went;
        default:
            return Did::Nothing;
    }
}

Gate::Gate(const std::vector<Person> &people, int profiles_max, int pin_digits, bool from_inside)
    : people_(people), profiles_max_(profiles_max), pin_digits_(std::max(pin_digits, 1)),
      from_inside_(from_inside), pin_panel_(std::max(pin_digits, 1)) {}

bool Gate::canAdd() const { return static_cast<int>(people_.size()) < profiles_max_; }

int Gate::lastPlace() const {
    const int count = static_cast<int>(people_.size());
    return canAdd() ? count : count - 1;
}

const Person *Gate::on() const {
    if (at_ < 0 || static_cast<std::size_t>(at_) >= people_.size()) {
        return nullptr;
    }
    return &people_[static_cast<std::size_t>(at_)];
}

void Gate::wanted(const std::string &id) { wanted_ = id; }

void Gate::arrived(const std::vector<Person> &people) {
    people_ = people;
    if (!wanted_.empty()) {
        for (std::size_t index = 0; index < people_.size(); ++index) {
            if (people_[index].id == wanted_) {
                at_ = static_cast<int>(index);
                wanted_.clear();
                break;
            }
        }
    }
    const int last = lastPlace();
    if (last >= 0 && at_ > last) {
        at_ = last;
    }
    if (at_ < 0) {
        at_ = 0;
    }
}

void Gate::forget() { pin_.clear(); }

void Gate::closed(const std::string &made) {
    step_ = Step::Chooser;
    pencil_ = false;
    if (made.empty()) {
        return;
    }
    for (std::size_t index = 0; index < people_.size(); ++index) {
        if (people_[index].id == made) {
            at_ = static_cast<int>(index);
            wanted_.clear();
            return;
        }
    }
    wanted(made);
}

Did Gate::answered(bool held) {
    if (step_ != Step::Pin) {
        return Did::Nothing;
    }
    pin_panel_.answered(held);
    if (!held) {
        return Did::Nothing;
    }
    // `Profiles.kt:94-103`. Which of the two the digits were for was decided before the panel
    // opened, and the pencil is still on.
    pin_ = pin_panel_.value();
    const bool toEdit = pencil_;
    pin_panel_ = Pin(pin_digits_);
    if (toEdit) {
        step_ = Step::Edit;
        forget();
        return Did::Edit;
    }
    step_ = Step::Chooser;
    return Did::Enter;
}

// A translation of `Profiles.kt:111-150` branch for branch and in the same order.
Did Gate::pressChooser(Word word) {
    const Person *person = on();
    switch (word) {
        case Word::Up:
            // `:113-118`. Moving off a profile puts the pencil away.
            if (at_ <= 0) {
                return Did::Nothing;
            }
            at_ -= 1;
            pencil_ = false;
            return Did::Moved;
        case Word::Down:
            // `:119-124`.
            if (at_ >= lastPlace()) {
                return Did::Nothing;
            }
            at_ += 1;
            pencil_ = false;
            return Did::Moved;
        case Word::Left:
            // `:125`. Left puts the pencil on the profile, which is the only place on this surface
            // where Left at the left edge is not a way out: there is something there to go to. The
            // Add row has no pencil, so Left on it is nothing.
            if (person == nullptr || pencil_) {
                return Did::Nothing;
            }
            pencil_ = true;
            return Did::Moved;
        case Word::Right:
            // `:126`.
            if (!pencil_) {
                return Did::Nothing;
            }
            pencil_ = false;
            return Did::Moved;
        case Word::Ok:
            // `:127-135`, in the Kotlin's own order: a locked profile asks for its PIN before the
            // pencil is read, so editing a locked profile unlocks it first.
            if (person == nullptr) {
                step_ = Step::Add;
                return Did::Add;
            }
            has_entered_ = true;
            entered_ = *person;
            if (person->has_pin) {
                pin_panel_ = Pin(pin_digits_);
                forget();
                step_ = Step::Pin;
                return Did::Moved;
            }
            if (pencil_) {
                step_ = Step::Edit;
                return Did::Edit;
            }
            forget();
            return Did::Enter;
        case Word::Back:
            // `:146`, and the one line on this surface where the two twins disagree.
            //
            // Opened from inside with Switch Profiles, Back returns to the page. At the start of a
            // run `Profiles.kt` answers the press and does nothing, so the app stays open;
            // `Profiles.swift:103` answers `false`, which hands the press to tvOS, which leaves
            // the app. Three things say the Kotlin is right and this port follows it:
            //
            //   * Netflix on a television never leaves on Back. Four presses, verified, and the
            //     way out is an Exit line in the profile menu
            //     (`reference/netflix-on-the-tv.md`'s source note, section 10 item 8).
            //   * Note 239 section 4 pressed Back five times on our own television's Home with
            //     focus on the active tab and the app never exited, so note 187's "only the root
            //     pages leave the app" is not what the build does either.
            //   * On Xbox nothing would do it for us in any case. Microsoft: when an app handles
            //     neither KeyDown nor KeyUp for B, `SystemNavigationManager::BackRequested` fires
            //     and "you have to implement this yourself". Answering `Nothing` here is the
            //     difference between a gate that stays and a console that suspends the app.
            //
            // What is still the owner's: whether a television app should offer a way out at all.
            // Note 187's "Profile chooser: at the start it leaves" row contradicts both this and
            // the Kotlin and wants a dated correction; the Android file carries the same note.
            return from_inside_ ? Did::Went : Did::Nothing;
        default:
            return Did::Nothing;
    }
}

Did Gate::press(Word word) {
    if (step_ == Step::Pin) {
        const Did did = pin_panel_.press(word);
        if (did == Did::Went) {
            // `Profiles.kt:104`: the panel closes and the chooser is where it was, pencil and all.
            // The digits go with it, which is the half of "never kept" that a closing panel owns;
            // the fresh panel is built by the branch that opens one, and building a second one
            // here would be a line no test could ever see move.
            forget();
            step_ = Step::Chooser;
            return Did::Moved;
        }
        if (did == Did::Asks) {
            pin_ = pin_panel_.value();
        }
        return did;
    }
    if (step_ != Step::Chooser) {
        // The editor and the adder are screens of their own with their own key maps. The gate is
        // waiting for `closed()` and answers nothing in the meantime.
        return Did::Nothing;
    }
    return pressChooser(word);
}

} // namespace streams::gate
