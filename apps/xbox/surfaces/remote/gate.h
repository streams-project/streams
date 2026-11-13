// Where the viewer is on the profile gate, and what each of the six words does from there.
//
// The fourth thing on this surface that needs no Windows: the first screen a viewer sees after the
// door, and the only one that stands between a television and somebody else's profile. A column of
// faces, a pencil beside the one in focus, and a panel of digits in front of a locked one. No
// Win2D, no WinRT, no swap chain, no controller.
//
// The twins, file for file:
//
//     Chooser  profiles/Profiles.kt:111-150   the column, the pencil, and what OK opens
//     Pin      profiles/PinPanel.kt:97-111    four digits, and Left out of the first one
//
// and beside each, `apps/apple/Sources/SurfacesTV/profiles/`, which is the same key map in Swift
// and disagrees about Back in one place. That disagreement is the whole of the hard part and it is
// written out at `Gate::press`.
//
// It holds no words a viewer reads and no limits of its own: how many profiles an account may have
// and how many digits a PIN has are the server's (`server/crates/server-accounts/src/limits.rs`,
// carried to a screen as `contract::Limits` by `/v1/hello`), so they are given to this class and
// not written in it. A screen drawn before that answer lands has to choose what to assume, and
// that choice belongs to the screen.
//
// One thing it does hold, briefly: the digits a viewer typed. `PinPanel.kt`'s own line is "the PIN
// goes to the server and is never kept", and here that is enforced rather than described --
// `Gate::forget` runs the moment the panel closes, whichever way it closes.
#ifndef STREAMS_XBOX_GATE_H
#define STREAMS_XBOX_GATE_H

#include <cstdint>
#include <string>
#include <vector>

#include "remote.h"

namespace streams::gate {

using streams::remote::Word;

// One profile, flattened. The twin of `contract::Profile` as the gate reads it.
struct Person {
    std::string id;
    std::string name;

    // The picture, already made absolute against the server by `reading::face`, or empty for the
    // letter.
    std::string picture;

    // True for a child's profile, which changes the plate behind the letter.
    bool kids = false;

    // True when this profile is behind a PIN.
    bool has_pin = false;

    // The language the shell speaks once this profile is entered. `Profiles.kt:133`.
    std::string language;
};

// What a press did. The shell above reads it and acts; nothing here speaks to a server.
enum class Did : std::uint8_t {
    // The press reached an edge, or the gate is busy. Nothing moved and nothing opened.
    Nothing,
    // The remote is somewhere else on the gate.
    Moved,
    // The gate is done and the shell goes back. Only ever answered when the gate was opened from
    // inside the app: see `press`.
    Went,
    // A profile is to be entered. `entered()` is who, and `pin()` is what they typed, which is
    // empty for a profile with no PIN and is gone as soon as it has been read once.
    Enter,
    // The profile under the remote is to be edited. `entered()` is who.
    Edit,
    // A new profile is to be made.
    Add,
    // The digits are to go to the server. `pin()` is what to send, `entered()` is whose profile.
    Asks,
};

// Which part of the gate is showing. `Profiles.kt:70-106`: three of the four are screens of their
// own that the gate opens and waits for.
enum class Step : std::uint8_t { Chooser, Pin, Edit, Add };

// The panel of digits in front of a locked profile.
//
// Every digit is a wheel: Up counts on, Down counts back, and both wrap, which is why there is no
// keyboard here and no way to be in an unfinished state. `PinPanel.kt:97-111`.
class Pin {
public:
    // `digits` is the server's `pin_digits`. One is the floor, as `PinPanel.kt:72` has it: a panel
    // of no digits could never be answered.
    explicit Pin(int digits);

    int digits() const { return static_cast<int>(digits_.size()); }
    int at() const { return at_; }
    int digit(int index) const;

    // What the panel would send. Read by the shell when the press answered `Did::Asks`.
    std::string value() const;

    // True once the server has refused these digits, until the next one is turned.
    // `PinPanel.kt:88`.
    bool refused() const { return refused_; }

    // True while the answer is still out. `PinPanel.kt:80-81`: a second OK must not post a second
    // time, which a held A on a controller would otherwise do.
    bool asking() const { return asking_; }

    // What the server said. True unlocks the profile; false lights the line that says so.
    void answered(bool held);

    Did press(Word word);

private:
    std::vector<int> digits_;
    int at_ = 0;
    bool refused_ = false;
    bool asking_ = false;
};

// The gate: a column of profiles, and what stands in front of a locked one.
class Gate {
public:
    // `profiles_max` and `pin_digits` are the server's limits. `from_inside` is true when the gate
    // was opened from within the app, with Switch Profiles, and false at the start of a run.
    Gate(const std::vector<Person> &people, int profiles_max, int pin_digits, bool from_inside);

    // A fresh list of profiles. The place in the column is kept by number and clamped into the
    // list, which is `Profiles.kt:65-68`, and is deliberately not what a row of titles does: a row
    // keeps its place by the title that was on it (`RowMemory`) because the server may reorder it,
    // and a profile column is the viewer's own and is not reordered by anything but them.
    //
    // A profile just made takes the focus once it is in the list. `Profiles.kt:58-64`.
    void arrived(const std::vector<Person> &people);

    Step step() const { return step_; }
    const std::vector<Person> &people() const { return people_; }

    // Where the remote is in the column. The last place is the row that makes a new profile, when
    // the account may have one. `Profiles.kt:65`.
    int at() const { return at_; }

    // The last place the column has: the profiles, and the Add row after them when there is room.
    int lastPlace() const;

    // True when the account has room for another profile. `Profiles.kt:48`.
    bool canAdd() const;

    // True when the pencil is on the profile under the remote, which changes what OK does rather
    // than where the remote is. `Profiles.kt:125`.
    bool pencil() const { return pencil_; }

    // The profile under the remote, or nothing when that is the Add row.
    const Person *on() const;

    // Who the last `Did::Enter`, `Did::Edit` or `Did::Asks` was about.
    const Person *entered() const { return has_entered_ ? &entered_ : nullptr; }

    // The digits to send, or to enter the profile with. Empty unless the last answer asked for
    // them, and emptied by `forget`.
    const std::string &pin() const { return pin_; }

    // The panel, while it is showing.
    Pin *panel() { return step_ == Step::Pin ? &pin_panel_ : nullptr; }
    const Pin *panel() const { return step_ == Step::Pin ? &pin_panel_ : nullptr; }

    // What the server said about the digits. True opens the profile, or the editor when the pencil
    // asked; false stays on the panel. `Profiles.kt:94-103`.
    Did answered(bool held);

    // The editor or the adder is done and the gate is the chooser again. `Profiles.kt:72-73`,
    // `:83-84`: the pencil goes out with them.
    //
    // `made` is the id of a profile the adder just made, and empty when it made none. The column
    // moves to it, and if the list in hand does not have it yet, the next `arrived` that does will
    // move there instead. `Profiles.kt:58-64, 71-79`.
    void closed(const std::string &made);
    void closed() { closed(std::string()); }

    // One word from the remote.
    //
    // A right to left page has already had its word turned by `streams::remote::turn` before it
    // reaches here, so nothing in this file counts in two directions.
    Did press(Word word);

    // Drops the digits. Called whenever the panel closes, and again by the shell once it has sent
    // them.
    void forget();

private:
    Did pressChooser(Word word);
    void wanted(const std::string &id);

    std::vector<Person> people_;
    int profiles_max_ = 0;
    int pin_digits_ = 1;
    bool from_inside_ = false;

    Step step_ = Step::Chooser;
    int at_ = 0;
    bool pencil_ = false;
    Pin pin_panel_;
    std::string pin_;
    bool has_entered_ = false;
    Person entered_;
    std::string wanted_;
};

} // namespace streams::gate

#endif // STREAMS_XBOX_GATE_H
