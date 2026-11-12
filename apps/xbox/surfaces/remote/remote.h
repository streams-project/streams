// The input grammar of the remote surface, for a controller.
//
// A remote speaks six words and a controller is a remote, so this file adds no seventh. It is the
// C++ answer to `apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Remote.kt`, which turns
// one key event into one `Word` and hands it to the screen. Three things make a controller
// different from a television remote, and all three live here rather than in any screen:
//
//  1. A controller is polled, not delivered. A remote arrives as key events and the system repeats
//     a held key; a gamepad is read as a state every frame and nothing repeats it, so a held
//     direction has to be counted here (`Repeat`).
//  2. A controller has a second way to say the same four words: the left stick, which is analogue.
//     It needs a dead zone and a way back out of one, or a resting stick walks the screen.
//  3. A controller has buttons a remote has not: X, Y, both sticks' own buttons, both bumpers,
//     both triggers, View and Menu. None of them says a word. A screen that needed one of them
//     would not work on the Xbox remote control, so none of them is mapped.
//
// No screen is here. This file is the grammar; the screens wait on a C++ contract (see the
// folder's README). It holds no words a viewer reads and no English: `strings/` carries those.
#ifndef STREAMS_XBOX_REMOTE_H
#define STREAMS_XBOX_REMOTE_H

#include <cstdint>

namespace streams::remote {

// What a remote can say, in the order `Remote.kt:26` names them. A screen answers true when it
// used the word, exactly as on a television.
enum class Word : std::uint8_t { Up, Down, Left, Right, Ok, Back };

// Left and right swapped, for a right to left page. The same swap as `Remote.kt:39`: it happens
// once, here, so no screen counts in two directions. The player keeps its own left to right way.
Word turn(Word word);

// The buttons this surface knows by name. The four of the D-pad and A and B say words; the rest are
// named so that a reading of the pad can say "nothing here" about them rather than drop them
// silently.
enum class Button : std::uint8_t {
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    A,
    B,
    X,
    Y,
    View,
    Menu,
    LeftShoulder,
    RightShoulder,
    LeftThumb,
    RightThumb,
};

// The word a button says, or false when it says none. The whole mapping of the surface:
//
//     UP DOWN LEFT RIGHT   D-pad, and the left stick
//     OK                   A          (Select on the Xbox remote control)
//     BACK                 B          (Back on the Xbox remote control)
//
// Verified against Microsoft's own mapping table for Xbox: the arrow keys map to the D-pad and the
// left stick, Spacebar and Enter to A and to the remote's Select, Escape to B and to the remote's
// Back, and the remote control carries no X, Y, stick, trigger or bumper at all.
// https://learn.microsoft.com/en-us/windows/uwp/ui-input/gamepad-and-remote-interactions (2026-11-11)
bool wordOf(Button button, Word &word);

// The bits of `XINPUT_GAMEPAD.wButtons`, which is the one button layout Microsoft writes down with
// numbers. Verified from the structure's own reference page, which lists every bitmask:
// https://learn.microsoft.com/en-us/windows/win32/api/xinput/ns-xinput-xinput_gamepad (ms.date
// 2018-12-05, read 2026-11-12). The newer GameInput and the WinRT `Windows.Gaming.Input.Gamepad`
// carry their own flag values; neither is reachable from this machine, so neither is written down
// here from memory. An adapter converts; this header stays the one layout that is verified.
namespace pad {
inline constexpr std::uint16_t DPadUp = 0x0001;
inline constexpr std::uint16_t DPadDown = 0x0002;
inline constexpr std::uint16_t DPadLeft = 0x0004;
inline constexpr std::uint16_t DPadRight = 0x0008;
inline constexpr std::uint16_t Menu = 0x0010;  // START in the header
inline constexpr std::uint16_t View = 0x0020;  // BACK in the header, and not our BACK word
inline constexpr std::uint16_t LeftThumb = 0x0040;
inline constexpr std::uint16_t RightThumb = 0x0080;
inline constexpr std::uint16_t LeftShoulder = 0x0100;
inline constexpr std::uint16_t RightShoulder = 0x0200;
inline constexpr std::uint16_t A = 0x1000;
inline constexpr std::uint16_t B = 0x2000;
inline constexpr std::uint16_t X = 0x4000;
inline constexpr std::uint16_t Y = 0x8000;
} // namespace pad

// The bit a button is, so a reading can be built and checked without naming numbers twice.
std::uint16_t bitOf(Button button);

// What one reading of the pad said. Several words can land in one frame: a viewer can press A while
// a direction is held, and a direction can repeat in the same frame another button goes down.
class Heard {
public:
    // Six is every word at once, which cannot happen, so nothing is ever dropped.
    static constexpr int Room = 6;

    void add(Word word);
    int count() const { return count_; }
    Word at(int index) const { return words_[index]; }
    bool holds(Word word) const;

private:
    Word words_[Room] = {};
    int count_ = 0;
};

// A direction held down says its word again, so a viewer can cross a row without pressing five
// times. A television remote gets this from the system's own key repeat; a polled gamepad has
// nothing behind it, so the surface counts the time itself.
//
// `Hold` and `Gap` are **not measured**. They are the one pair of numbers in this file that no
// source in this tree or at Microsoft settles, and the folder's README carries them as an open
// item: they are matched against the reference on a television, or against a console.
class Repeat {
public:
    static constexpr std::int64_t Hold = 500;
    static constexpr std::int64_t Gap = 120;

    // True when the word should be said now. `held` false means nothing is pushed, which disarms
    // the repeat; a different direction starts over, so rolling from right to down steps once.
    bool speaks(bool held, Word word, std::int64_t mono_ms);

private:
    bool on_ = false;
    Word word_ = Word::Up;
    std::int64_t next_ms_ = 0;
};

// The left stick, read as four directions.
//
// A stick at rest does not report zero, so a dead zone decides when a push counts. `Push` is
// `XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE`, which Microsoft writes as 7849 of a 32767 axis, and the
// same page recommends the circular form used here rather than a per axis one:
// https://learn.microsoft.com/en-us/windows/win32/xinput/getting-started-with-xinput (ms.date
// 2025-07-14, read 2026-11-12). `Rest` is lower than `Push` on purpose: a stick that has spoken
// must fall back under `Rest` before it speaks again, so a hand resting at the edge of the dead
// zone cannot shiver a row back and forth. `Rest` is a choice, not a measurement.
class Stick {
public:
    static constexpr int Push = 7849;
    static constexpr int Rest = 5000;
    static constexpr int Full = 32767;

    // The direction this reading pushes, or false for none. Positive y is up and positive x is
    // right, as the XINPUT_GAMEPAD reference states. The larger axis wins, so a diagonal says one
    // word and never two.
    bool pushed(int x, int y, Word &word);

private:
    // The direction a reading points, with no dead zone in it.
    bool holdsOn(int x, int y, Word &word) const;

    bool spoken_ = false;
};

// One controller, read once a frame. It carries what the last frame held, so a press is a press and
// not a hold, and it owns the one repeat clock shared by the D-pad and the stick: a viewer uses one
// of the two at a time, and two clocks would step the screen twice.
class Pad {
public:
    // A right to left page swaps left and right once, here.
    void rightToLeft(bool on) { turned_ = on; }

    // `buttons` is an XINPUT_GAMEPAD.wButtons reading, `x` and `y` the left stick's two axes, and
    // `mono_ms` the monotonic time the core is stepped with. OK and BACK are press edges and never
    // repeat: a held A must not open a title twice.
    Heard frame(std::uint16_t buttons, int x, int y, std::int64_t mono_ms);

private:
    std::uint16_t before_ = 0;
    bool turned_ = false;
    Stick stick_;
    Repeat repeat_;
};

} // namespace streams::remote

#endif
