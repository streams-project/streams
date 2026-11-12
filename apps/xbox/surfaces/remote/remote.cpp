#include "remote.h"

namespace streams::remote {

Word turn(Word word) {
    switch (word) {
    case Word::Left:
        return Word::Right;
    case Word::Right:
        return Word::Left;
    default:
        return word;
    }
}

bool wordOf(Button button, Word &word) {
    switch (button) {
    case Button::DPadUp:
        word = Word::Up;
        return true;
    case Button::DPadDown:
        word = Word::Down;
        return true;
    case Button::DPadLeft:
        word = Word::Left;
        return true;
    case Button::DPadRight:
        word = Word::Right;
        return true;
    case Button::A:
        word = Word::Ok;
        return true;
    case Button::B:
        word = Word::Back;
        return true;
    // X, Y, View, Menu, both bumpers and both stick buttons say nothing, because the Xbox remote
    // control has none of them and a screen may not depend on a button half the viewers lack.
    default:
        return false;
    }
}

std::uint16_t bitOf(Button button) {
    switch (button) {
    case Button::DPadUp:
        return pad::DPadUp;
    case Button::DPadDown:
        return pad::DPadDown;
    case Button::DPadLeft:
        return pad::DPadLeft;
    case Button::DPadRight:
        return pad::DPadRight;
    case Button::A:
        return pad::A;
    case Button::B:
        return pad::B;
    case Button::X:
        return pad::X;
    case Button::Y:
        return pad::Y;
    case Button::View:
        return pad::View;
    case Button::Menu:
        return pad::Menu;
    case Button::LeftShoulder:
        return pad::LeftShoulder;
    case Button::RightShoulder:
        return pad::RightShoulder;
    case Button::LeftThumb:
        return pad::LeftThumb;
    case Button::RightThumb:
        return pad::RightThumb;
    }
    return 0;
}

void Heard::add(Word word) {
    if (count_ < Room) {
        words_[count_] = word;
        count_ += 1;
    }
}

bool Heard::holds(Word word) const {
    for (int i = 0; i < count_; i += 1) {
        if (words_[i] == word) {
            return true;
        }
    }
    return false;
}

bool Repeat::speaks(bool held, Word word, std::int64_t mono_ms) {
    if (!held) {
        on_ = false;
        return false;
    }
    if (!on_ || word_ != word) {
        on_ = true;
        word_ = word;
        next_ms_ = mono_ms + Hold;
        return true;
    }
    if (mono_ms >= next_ms_) {
        next_ms_ = mono_ms + Gap;
        return true;
    }
    return false;
}

bool Stick::pushed(int x, int y, Word &word) {
    // Squared, so the circle needs no square root and no floating point. The axes reach 32767, so
    // the largest sum is about 2.1e9 and the arithmetic is done in 64 bits.
    const std::int64_t far = static_cast<std::int64_t>(x) * x + static_cast<std::int64_t>(y) * y;
    const std::int64_t rest = static_cast<std::int64_t>(Rest) * Rest;
    const std::int64_t push = static_cast<std::int64_t>(Push) * Push;
    if (far <= rest) {
        // Back near the middle: the stick may speak again.
        spoken_ = false;
        return false;
    }
    if (far < push) {
        // Between the two: whatever the stick was doing it carries on doing, and a stick that has
        // not spoken yet stays quiet.
        return spoken_ ? holdsOn(x, y, word) : false;
    }
    spoken_ = true;
    return holdsOn(x, y, word);
}

// The direction a reading points. Separate so the band between `Rest` and `Push` can answer with
// the same arithmetic instead of a second copy of it.
bool Stick::holdsOn(int x, int y, Word &word) const {
    const int across = x < 0 ? -x : x;
    const int along = y < 0 ? -y : y;
    if (across == 0 && along == 0) {
        return false;
    }
    if (across > along) {
        word = x > 0 ? Word::Right : Word::Left;
    } else {
        // Positive y is up, as XINPUT_GAMEPAD states. A diagonal exactly on 45 degrees reads as up
        // or down rather than as nothing.
        word = y > 0 ? Word::Up : Word::Down;
    }
    return true;
}

Heard Pad::frame(std::uint16_t buttons, int x, int y, std::int64_t mono_ms) {
    Heard heard;
    const std::uint16_t went_down = static_cast<std::uint16_t>(buttons & ~before_);
    before_ = buttons;

    // OK and BACK: press edges only, the same rule as `Remote.kt:94`, which answers KeyDown and
    // ignores KeyUp. A held A must not open a title twice.
    if ((went_down & pad::A) != 0) {
        heard.add(Word::Ok);
    }
    if ((went_down & pad::B) != 0) {
        heard.add(Word::Back);
    }

    // One direction, from the D-pad if it is pushed and from the stick otherwise. Two opposite
    // D-pad buttons at once cancel, which is what a worn pad does under a thumb.
    bool held = false;
    Word way = Word::Up;
    const bool up = (buttons & pad::DPadUp) != 0;
    const bool down = (buttons & pad::DPadDown) != 0;
    const bool left = (buttons & pad::DPadLeft) != 0;
    const bool right = (buttons & pad::DPadRight) != 0;
    if (up != down) {
        held = true;
        way = up ? Word::Up : Word::Down;
    } else if (left != right) {
        held = true;
        way = left ? Word::Left : Word::Right;
    } else {
        held = stick_.pushed(x, y, way);
    }

    if (repeat_.speaks(held, way, mono_ms)) {
        heard.add(turned_ ? turn(way) : way);
    }
    return heard;
}

} // namespace streams::remote
