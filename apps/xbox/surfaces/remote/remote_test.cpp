// Does a controller speak the six words a television remote speaks? Nothing here needs an Xbox, a
// console or a Windows toolchain: the grammar is arithmetic over a button mask and two axes, so it
// is checked on whatever machine builds the tree.
//
//     c++ -std=c++17 -Wall -Wextra apps/xbox/surfaces/remote/remote.cpp \
//         apps/xbox/surfaces/remote/remote_test.cpp -o remote && ./remote
#include <cstdio>

#include "remote.h"

using streams::remote::Button;
using streams::remote::Heard;
using streams::remote::Pad;
using streams::remote::Repeat;
using streams::remote::Stick;
using streams::remote::Word;
namespace pad = streams::remote::pad;

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

// One word and nothing else.
bool only(const Heard &heard, Word word) { return heard.count() == 1 && heard.at(0) == word; }

bool says(Button button, Word want) {
    Word word = Word::Up;
    return streams::remote::wordOf(button, word) && word == want;
}

bool silent(Button button) {
    Word word = Word::Up;
    return !streams::remote::wordOf(button, word);
}

} // namespace

int main() {
    // The mapping itself.
    ok(says(Button::DPadUp, Word::Up) && says(Button::DPadDown, Word::Down) &&
           says(Button::DPadLeft, Word::Left) && says(Button::DPadRight, Word::Right),
       "the D-pad says the four directions");
    ok(says(Button::A, Word::Ok), "A says OK, which is Select on the Xbox remote control");
    ok(says(Button::B, Word::Back), "B says BACK, which is Back on the Xbox remote control");
    ok(silent(Button::X) && silent(Button::Y) && silent(Button::View) && silent(Button::Menu) &&
           silent(Button::LeftShoulder) && silent(Button::RightShoulder) &&
           silent(Button::LeftThumb) && silent(Button::RightThumb),
       "the eight buttons the Xbox remote control lacks say nothing");
    ok(streams::remote::bitOf(Button::A) == pad::A &&
           streams::remote::bitOf(Button::View) == pad::View &&
           streams::remote::bitOf(Button::Menu) == pad::Menu,
       "every button carries its XINPUT_GAMEPAD bit");
    ok(pad::View == 0x0020 && pad::Menu == 0x0010,
       "the pad's own Back button is View, and is not our BACK word");

    // Right to left.
    ok(streams::remote::turn(Word::Left) == Word::Right &&
           streams::remote::turn(Word::Right) == Word::Left &&
           streams::remote::turn(Word::Up) == Word::Up && streams::remote::turn(Word::Ok) == Word::Ok,
       "a right to left page swaps left and right and leaves the rest alone");

    // OK and BACK are press edges, so a held button does not fire twice.
    {
        Pad controller;
        ok(only(controller.frame(pad::A, 0, 0, 0), Word::Ok), "A going down says OK once");
        ok(controller.frame(pad::A, 0, 0, 16).count() == 0, "A held says nothing more");
        ok(controller.frame(0, 0, 0, 32).count() == 0, "A coming up says nothing");
        ok(only(controller.frame(pad::A, 0, 0, 48), Word::Ok), "A pressed again says OK again");
        ok(only(controller.frame(pad::B, 0, 0, 64), Word::Back), "B says BACK");
    }

    // A held direction repeats, and the first step is at once.
    {
        Pad controller;
        ok(only(controller.frame(pad::DPadDown, 0, 0, 0), Word::Down),
           "a direction steps the moment it goes down");
        ok(controller.frame(pad::DPadDown, 0, 0, Repeat::Hold - 1).count() == 0,
           "a held direction waits out the hold before it repeats");
        ok(only(controller.frame(pad::DPadDown, 0, 0, Repeat::Hold), Word::Down),
           "a held direction repeats after the hold");
        ok(controller.frame(pad::DPadDown, 0, 0, Repeat::Hold + Repeat::Gap - 1).count() == 0,
           "a repeat waits out the gap");
        ok(only(controller.frame(pad::DPadDown, 0, 0, Repeat::Hold + Repeat::Gap), Word::Down),
           "and repeats again a gap later");
        ok(only(controller.frame(pad::DPadRight, 0, 0, Repeat::Hold + Repeat::Gap + 1), Word::Right),
           "rolling to another direction steps that one at once");
        ok(controller.frame(0, 0, 0, Repeat::Hold * 4).count() == 0,
           "letting go says nothing, however long it was held");
        ok(only(controller.frame(pad::DPadRight, 0, 0, Repeat::Hold * 4 + 1), Word::Right),
           "and the next press starts the hold over");
    }

    // Two opposite directions at once cancel.
    {
        Pad controller;
        ok(controller.frame(pad::DPadUp | pad::DPadDown, 0, 0, 0).count() == 0,
           "up and down together say nothing");
        ok(only(controller.frame(pad::DPadUp | pad::DPadDown | pad::A, 0, 0, 16), Word::Ok),
           "but A still reaches the screen through them");
    }

    // The stick's dead zone.
    {
        Stick stick;
        Word word = Word::Ok;
        ok(!stick.pushed(0, 0, word), "a stick at rest says nothing");
        ok(!stick.pushed(Stick::Push - 1, 0, word), "a stick inside the dead zone says nothing");
        ok(stick.pushed(Stick::Push, 0, word) && word == Word::Right,
           "a stick on the dead zone pushes right");
        ok(stick.pushed(0, -Stick::Full, word) && word == Word::Down,
           "negative y is down, as XINPUT_GAMEPAD states");
        ok(stick.pushed(0, Stick::Full, word) && word == Word::Up, "positive y is up");
        ok(stick.pushed(-Stick::Full, 100, word) && word == Word::Left,
           "the larger axis wins, so a diagonal says one word");
    }

    // The circle, not a square. Both checks start from a stick that has not spoken, because a
    // stick that has spoken stays held until it falls under `Rest`, which is the next block.
    {
        Stick fresh;
        Word word = Word::Ok;
        // 6000 on each axis is 8485 away, under the dead zone on either axis alone and over it
        // together: that is what the recommended circular form means.
        ok(fresh.pushed(6000, 6000, word), "a diagonal push counts on the circle");
    }
    {
        Stick fresh;
        Word word = Word::Ok;
        // 5000 on each axis is 7071 away, inside the dead zone on the circle though a square
        // reading of 5000 per axis would also have said nothing. 7000 each is 9899 and does push.
        ok(!fresh.pushed(5000, 5000, word), "a smaller diagonal does not");
        ok(fresh.pushed(7000, 7000, word), "a larger one does");
    }

    // Hysteresis: a hand resting at the edge must not shiver the screen.
    {
        Pad controller;
        ok(only(controller.frame(0, Stick::Push + 1, 0, 0), Word::Right), "the stick steps right");
        // Back into the band between Rest and Push: still counted as held, so the repeat clock
        // carries on rather than starting over.
        ok(controller.frame(0, Stick::Rest + 1, 0, 1).count() == 0,
           "drifting back into the band says nothing new");
        ok(only(controller.frame(0, Stick::Rest + 1, 0, Repeat::Hold), Word::Right),
           "and the hold it was already in still repeats");
        // Under Rest the stick is let go.
        ok(controller.frame(0, Stick::Rest - 1, 0, Repeat::Hold + 1).count() == 0,
           "under the rest mark the stick is let go");
        ok(only(controller.frame(0, Stick::Push + 1, 0, Repeat::Hold + 2), Word::Right),
           "and pushing again steps once more");
    }

    // The D-pad wins over the stick, so one hand on each does not double step.
    {
        Pad controller;
        ok(only(controller.frame(pad::DPadLeft, Stick::Full, 0, 0), Word::Left),
           "the D-pad decides when both are pushed");
    }

    // Right to left, through the whole pad.
    {
        Pad controller;
        controller.rightToLeft(true);
        ok(only(controller.frame(pad::DPadLeft, 0, 0, 0), Word::Right),
           "on a right to left page the D-pad's left says RIGHT");
        ok(only(controller.frame(pad::A, 0, 0, 16), Word::Ok), "and OK is still OK");
    }

    if (failures > 0) {
        std::printf("%d of %d failed\n", failures, checks);
        return 1;
    }
    std::printf("%d of %d held\n", checks, checks);
    return 0;
}
