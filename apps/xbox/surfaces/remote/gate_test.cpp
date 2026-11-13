// Where does the remote go on the profile gate, and what stands in front of a locked profile?
//
// A scripted key sequence against the state machine in `gate.h`. Nothing here needs an Xbox, a
// console, a Windows toolchain or a screen.
//
//     c++ -std=c++17 -Wall -Wextra apps/xbox/surfaces/remote/remote.cpp \
//         apps/xbox/surfaces/remote/gate.cpp apps/xbox/surfaces/remote/gate_test.cpp \
//         -o gate && ./gate
#include <cstdio>
#include <string>
#include <vector>

#include "gate.h"

using streams::gate::Did;
using streams::gate::Gate;
using streams::gate::Person;
using streams::gate::Pin;
using streams::gate::Step;
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

Word wordOf(char key) {
    switch (key) {
        case 'U': return Word::Up;
        case 'D': return Word::Down;
        case 'L': return Word::Left;
        case 'R': return Word::Right;
        case 'O': return Word::Ok;
        default: return Word::Back;
    }
}

template <typename T>
Did press(T &on, const char *keys) {
    Did did = Did::Nothing;
    for (const char *key = keys; *key != '\0'; ++key) {
        did = on.press(wordOf(*key));
    }
    return did;
}

// The panel, read through answers that cannot be mistaken for real ones when there is no panel.
//
// Without these a mutation that stops the panel opening at all crashes the suite on a null read
// instead of printing the checks it broke, and a test that crashes proves less than one that
// fails: the output is what says which rule went.
int panelAt(const Gate &gate) {
    const Pin *panel = gate.panel();
    return panel == nullptr ? -2 : panel->at();
}

int panelDigits(const Gate &gate) {
    const Pin *panel = gate.panel();
    return panel == nullptr ? -2 : panel->digits();
}

int panelDigit(const Gate &gate, int index) {
    const Pin *panel = gate.panel();
    return panel == nullptr ? -2 : panel->digit(index);
}

bool panelRefused(const Gate &gate) {
    const Pin *panel = gate.panel();
    return panel != nullptr && panel->refused();
}

bool panelAsking(const Gate &gate) {
    const Pin *panel = gate.panel();
    return panel != nullptr && panel->asking();
}

Person person(const std::string &id, bool locked = false) {
    Person made;
    made.id = id;
    made.name = id;
    made.has_pin = locked;
    return made;
}

// Three profiles, the second behind a PIN.
std::vector<Person> three() {
    return {person("a"), person("b", true), person("c")};
}

// The server's own limits, which are given to the gate and never written in it.
constexpr int ProfilesMax = 5;
constexpr int PinDigits = 4;

} // namespace

int main() {
    // ------------------------------------------------------------------ the column
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        ok(gate.step() == Step::Chooser && gate.at() == 0, "the gate opens on the first profile");
        ok(gate.canAdd() && gate.lastPlace() == 3,
           "an account with room has one more place than it has profiles: the Add row");
        ok(press(gate, "D") == Did::Moved && gate.at() == 1, "Down walks the column");
        ok(press(gate, "DDD") == Did::Nothing && gate.at() == 3,
           "and stops on the Add row, which is the last place there is");
        ok(gate.on() == nullptr, "no profile is under the remote there");
        ok(press(gate, "UUU") == Did::Moved && gate.at() == 0, "Up walks it back");
        ok(press(gate, "U") == Did::Nothing, "and stops at the first");

        Gate full(std::vector<Person>(ProfilesMax, person("x")), ProfilesMax, PinDigits, false);
        ok(!full.canAdd() && full.lastPlace() == ProfilesMax - 1,
           "an account with no room has no Add row");
    }

    // ------------------------------------------------------------------ the pencil
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        ok(press(gate, "L") == Did::Moved && gate.pencil(),
           "Left puts the pencil on the profile: the one left edge on this surface that is not a "
           "way out");
        ok(press(gate, "L") == Did::Nothing && gate.pencil(), "and Left again is nothing");
        ok(press(gate, "R") == Did::Moved && !gate.pencil(), "Right puts it away");
        ok(press(gate, "R") == Did::Nothing, "and Right again is nothing");
        press(gate, "L");
        ok(press(gate, "D") == Did::Moved && !gate.pencil(),
           "moving off the profile puts the pencil away with it");
        press(gate, "DD");
        ok(gate.at() == 3 && press(gate, "L") == Did::Nothing && !gate.pencil(),
           "the Add row has no pencil to put on");
    }

    // ------------------------------------------------------------------ what OK opens
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        ok(press(gate, "O") == Did::Enter && gate.entered() != nullptr &&
               gate.entered()->id == "a",
           "OK on an open profile enters it");
        ok(gate.pin().empty(), "with no digits, because it asked for none");

        press(gate, "L");
        ok(press(gate, "O") == Did::Edit && gate.step() == Step::Edit,
           "OK with the pencil on opens the editor");
        gate.closed();
        ok(gate.step() == Step::Chooser && !gate.pencil(),
           "and the editor closing puts the gate back with the pencil away");

        press(gate, "DDD");
        ok(press(gate, "O") == Did::Add && gate.step() == Step::Add,
           "OK on the Add row opens the adder");
        gate.closed("d");
        ok(gate.step() == Step::Chooser && gate.at() == 3,
           "a profile made is not in the list yet, so the column has not moved");
        std::vector<Person> four = three();
        four.push_back(person("d"));
        gate.arrived(four);
        ok(gate.at() == 3 && gate.on() != nullptr && gate.on()->id == "d",
           "and the answer that carries it moves the column onto it");
    }

    // ------------------------------------------------------------------ a locked profile
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        press(gate, "D");
        ok(press(gate, "O") == Did::Moved && gate.step() == Step::Pin,
           "OK on a locked profile opens the panel instead of entering it");
        ok(panelDigits(gate) == PinDigits,
           "with as many digits as the server asked for");
        ok(panelAt(gate) == 0, "on the first of them");

        ok(press(gate, "U") == Did::Moved && panelDigit(gate, 0) == 1, "Up counts a digit on");
        ok(press(gate, "D") == Did::Moved && panelDigit(gate, 0) == 0, "Down counts it back");
        ok(press(gate, "D") == Did::Moved && panelDigit(gate, 0) == 9,
           "and the digit is a wheel, so Down below zero reaches nine");
        ok(press(gate, "U") == Did::Moved && panelDigit(gate, 0) == 0, "and Up past nine wraps");

        ok(press(gate, "R") == Did::Moved && panelAt(gate) == 1, "Right walks the digits");
        ok(press(gate, "RRR") == Did::Nothing && panelAt(gate) == 3,
           "and stops at the last one");
        ok(press(gate, "LLL") == Did::Moved && panelAt(gate) == 0, "Left walks them back");
        ok(press(gate, "L") == Did::Moved && gate.step() == Step::Chooser,
           "and Left out of the first digit closes the panel: note 187's PIN row");
        ok(gate.at() == 1 && gate.on() != nullptr && gate.on()->id == "b",
           "leaving the column where it was, on the profile that is locked");
    }

    // ------------------------------------------------------------------ the digits and the server
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        press(gate, "DO");
        press(gate, "URUURUUUR");
        ok(panelDigit(gate, 0) == 1 && panelDigit(gate, 1) == 2 && panelDigit(gate, 2) == 3,
           "the digits are turned one at a time");
        ok(press(gate, "O") == Did::Asks && gate.pin() == "1230",
           "OK sends them, and what it sends is what was turned");
        ok(panelAsking(gate), "and the panel is waiting");
        ok(press(gate, "O") == Did::Nothing,
           "a second OK while it waits sends nothing, which a held A on a controller would "
           "otherwise do");

        ok(gate.answered(false) == Did::Nothing && panelRefused(gate),
           "a refusal keeps the panel and says so");
        ok(press(gate, "U") == Did::Moved && !panelRefused(gate),
           "and turning any digit takes the refusal off again");

        ok(press(gate, "O") == Did::Asks, "the panel can be answered again");
        ok(gate.answered(true) == Did::Enter && gate.step() == Step::Chooser,
           "and the right digits enter the profile");
        ok(gate.entered() != nullptr && gate.entered()->id == "b", "which profile is said");
        ok(!gate.pin().empty(), "with the digits to send with it");
        gate.forget();
        ok(gate.pin().empty(), "and the gate drops them when the shell has read them");
    }

    // ------------------------------------------------------------------ editing a locked profile
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        press(gate, "DL");
        ok(press(gate, "O") == Did::Moved && gate.step() == Step::Pin,
           "the pencil on a locked profile still asks for the PIN first");
        press(gate, "O");
        ok(gate.answered(true) == Did::Edit && gate.step() == Step::Edit,
           "and the right digits open the editor rather than entering the profile");
        ok(gate.pin().empty(), "the digits are not carried into the editor");
    }

    // ------------------------------------------------------------------ the panel keeps nothing
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        press(gate, "DO");
        press(gate, "UUU");
        ok(panelDigit(gate, 0) == 3, "a digit is turned");
        ok(press(gate, "O") == Did::Asks && !gate.pin().empty(),
           "they are sent, so the gate is holding them");
        gate.answered(false);
        press(gate, "B");
        ok(gate.step() == Step::Chooser, "Back closes the panel");
        ok(gate.pin().empty(),
           "and the digits go with it: a panel the viewer walked away from leaves nothing behind");
        press(gate, "O");
        ok(panelDigit(gate, 0) == 0 && panelAt(gate) == 0,
           "and opening it again starts from nothing: the panel keeps no digits between tries");

        // The other way out of a panel, and the one that matters: a profile that needs no PIN
        // must not be entered carrying the digits somebody typed for the profile beside it.
        press(gate, "UUUB");
        ok(gate.at() == 1, "the remote is back on the locked profile, panel closed");
        press(gate, "U");
        ok(press(gate, "O") == Did::Enter && gate.at() == 0 && gate.pin().empty(),
           "and the profile above it, which has no PIN, is entered with no digits at all");
    }

    // ------------------------------------------------------------------ the digits travel no
    //                                                                    further than one profile
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        press(gate, "DO");
        press(gate, "UO");
        gate.answered(true);
        ok(gate.entered() != nullptr && gate.entered()->id == "b" && !gate.pin().empty(),
           "a profile unlocked leaves its digits in hand for the shell to send with it");
        press(gate, "U");
        ok(press(gate, "O") == Did::Enter && gate.entered()->id == "a" && gate.pin().empty(),
           "and the profile above it, which needs no PIN, is entered with none of them, whether "
           "or not the shell remembered to drop them");
    }

    // ------------------------------------------------------------------ Back, and the one
    //                                                                    disagreement
    {
        Gate start(three(), ProfilesMax, PinDigits, false);
        ok(press(start, "B") == Did::Nothing,
           "Back at the start of a run does nothing: the app stays open, as Netflix does and as "
           "note 239 measured our own television doing");
        ok(press(start, "BBBB") == Did::Nothing && start.step() == Step::Chooser,
           "and pressing it again changes nothing, five presses deep");

        Gate inside(three(), ProfilesMax, PinDigits, true);
        ok(press(inside, "B") == Did::Went,
           "the same gate opened with Switch Profiles goes back to the page");
        press(inside, "DO");
        ok(inside.step() == Step::Pin && press(inside, "B") == Did::Moved &&
               inside.step() == Step::Chooser,
           "and Back from the panel closes the panel first, one level at a time");
    }

    // ------------------------------------------------------------------ a fresh list of profiles
    {
        Gate gate(three(), ProfilesMax, PinDigits, false);
        press(gate, "DD");
        ok(gate.at() == 2, "the remote is on the third profile");

        std::vector<Person> fewer = {person("a")};
        gate.arrived(fewer);
        ok(gate.at() == 1,
           "an answer with fewer profiles clamps the column into it, onto the Add row");

        Gate full(std::vector<Person>(ProfilesMax, person("x")), ProfilesMax, PinDigits, false);
        press(full, "DDDD");
        ok(full.at() == 4, "a full account walks to its last profile");
        std::vector<Person> four(4, person("x"));
        full.arrived(four);
        ok(full.at() == 4 && full.canAdd() && full.lastPlace() == 4,
           "and one deleted leaves the remote on the Add row that has just appeared");
    }

    // ------------------------------------------------------------------ the panel on its own
    {
        Pin panel(0);
        ok(panel.digits() == 1,
           "a panel of no digits is a panel of one: there would be nothing to answer otherwise");
        ok(panel.digit(0) == 0 && panel.digit(1) == -1 && panel.digit(-1) == -1,
           "and it answers nothing for a digit it has not got");
        ok(panel.value() == "0", "its value is what it shows");
    }

    std::printf("\n%d of %d held\n", checks - failures, checks);
    return failures == 0 ? 0 : 1;
}
