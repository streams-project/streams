// The door: the way into the app before anybody has signed in, and what each of the six words does
// from there.
//
// Two screens, because the twins are two files and the shell routes between them by address:
//
//   * the **landing**, which is a showreel card with a way in on it, a row of trending titles to
//     look through without an account, and -- where the server says accounts are not required -- a
//     way straight in with no account at all;
//   * **sign in**, which offers the two ways in: a phone and a code, or the remote and the
//     keyboard in `keys.h`.
//
// No Win2D, no WinRT, no swap chain, no controller, no film, no picture: the slideshow, the read
// ahead, the card slide and the 320 ms glide are all drawing and all of them wait on a Windows
// machine. What a screen *decides* does not, and this is that.
//
// The twins, file for file:
//
//     Landing   door/Door.kt:243-293        the stops, the key map, and what OK opens
//     Landing   door/Door.swift:320-357     the same key map in Swift
//     SignIn    door/SignIn.kt:177-191      the two ways, and the keyboard under one of them
//     SignIn    door/SignIn.swift:160-176   the same again
//     Landing   apps/web/src/surfaces/door.rs:66-105   the only twin that reads the server's flag
//
// Read all five, because where they disagree there is something to find. Three disagreements came
// out of this port and they are written on the lines they belong to: the look round at
// `Landing::press`, what Back does at the root at `Did::Leaves`, and the keyboard's ragged row at
// the head of `keys.h`.
//
// It holds no words a viewer reads. `door.sign_in`, `door.join`, `door.help`, `door.trending`,
// `door.watch`, `nav.home`, `signin.phone` and `signin.remote` are keys of `strings/` and the
// screen above fetches them; what is here is which stop the remote is on.
//
// It holds no limit of its own either. Whether this server lets a visitor look round without an
// account is `contract::Features::accounts_required`, and it is **given** to the landing. Its
// absence reads *true* and not false -- it is the one field in the contract whose default is not
// the zero value (`platform/contract-proto/proto/contract/v1/hello.proto:12-20`) -- and applying
// that default belongs to the contract reader, so this file neither applies it nor repeats it.
#ifndef STREAMS_XBOX_DOOR_H
#define STREAMS_XBOX_DOOR_H

#include <cstdint>
#include <string>
#include <vector>

#include "keys.h"
#include "remote.h"

namespace streams::door {

using streams::remote::Word;

// One trending title as a card on the landing. The twin of what `TitleCard` reads out of
// `contract::TitleDetail`, cut to what a key map needs: the drawing wants a logo, a backdrop, its
// stills, the facts and the story, and none of those decide where a press goes.
struct Card {
    // The title's key as a plain string, `tv:...` or `mv:...`. What `ForEach(id:)` keys the cards
    // on in the Swift and what the shell turns into an address.
    std::string key;

    // The title's own name, from the catalogue.
    std::string name;
};

// Which screen a press opens. Named rather than addressed, for the reason `title.h` gives: nothing
// in a key map knows an address, and `contract::address` is the shell's.
enum class Opening : std::uint8_t {
    // The sign in screen. Note that **Join opens it too**: both twins send Get Started to
    // `Address.SIGN_IN` (`Door.kt:277`, `Door.swift:344`), so the join flow a browser has
    // (`apps/web/src/surfaces/door.rs:95`) does not exist on a television yet.
    SignIn,
    // Get Help.
    Help,
    // The home page, with no account at all. Only reachable where the server says accounts are
    // not required: see `Landing::press`.
    Home,
};

// What a press on the landing did.
enum class Did : std::uint8_t {
    // The word was used and the landing is as it was.
    Nothing,
    // The remote is on another stop, or another card is in view.
    Moved,
    // A screen opens. `opening()` says which.
    Opens,
    // Back, with nothing on the landing further back.
    //
    // Both twins answer `false` here and hand the press to the host, which on Android is the back
    // dispatcher and on tvOS is the system, and both of those close the app. **On Xbox nothing
    // above will do it.** Microsoft: when an app handles neither `KeyDown` nor `KeyUp` for B,
    // `SystemNavigationManager::BackRequested` fires and "you have to implement this yourself".
    // So the branch is named and the shell decides, rather than this file deciding for it.
    //
    // `gate.h` took the other decision on the same question and said so: it answers `Nothing` at
    // the start of a run, because Netflix on a television never leaves on Back and note 239
    // measured our own television never leaving either. The two screens are not the same screen --
    // the gate is reached only after a profile exists and the landing is the first frame of the
    // app -- so this does not follow it, and it does not contradict it either: both leave the
    // decision where README open item 9 says it lives, with the owner.
    Leaves,
};

// The landing: the showreel card, the trending titles beside it, and the row above them.
//
// Where the remote can be. `Door.kt:111` and `Door.swift:31` name seven of these eight; the
// eighth is `LookRound` and it is new here -- see `press`.
enum class Stop : std::uint8_t {
    // The row above the cards: the house, the tab of the trending titles, and -- while a title
    // card is in view -- the way to sign in, which moves up to the top right.
    Home,
    Tab,
    TopSignIn,
    // On the showreel card: Get Help low on the left, then Sign In and Get Started on the right.
    Help,
    SignIn,
    Join,
    // And, only where the server says accounts are not required, a way straight in.
    LookRound,
    // On a trending title's card: its one button.
    Watch,
};

class Landing {
public:
    // `accounts_required` is the server's answer, already defaulted by the contract reader. The
    // landing is drawn before `/v1/hello` has necessarily landed, so the shell passes the safe
    // answer -- true -- until it has one, and calls `arrived` when it does.
    Landing(const std::vector<Card> &cards, bool accounts_required);

    // The trending titles, the server's answer, or both. `/v1/public/trending` is read under the
    // opening in both twins, so the cards really do turn up after the landing is on screen.
    //
    // Where the remote is, is kept, and then put somewhere that still exists: a list that shrank
    // under a viewer standing on its last card moves them to the last card there is, and a server
    // that turns out to require accounts after all takes the look round away from under them.
    // That is this port's own answer and not the twins': neither reads its titles twice, so
    // neither has ever met a list that changed, and `Door.kt:333` would simply draw nothing where
    // the card was. It is the same answer `Summary::line` and `RowMemory::columnOf` give, and for
    // the same reason -- a bounds answer is not a focus answer.
    void arrived(const std::vector<Card> &cards, bool accounts_required);

    Stop at() const { return at_; }

    // Which card is in view: 0 is the showreel, 1 and up are the trending titles.
    int page() const { return page_; }

    const std::vector<Card> &cards() const { return cards_; }

    // The trending title in view, or nothing when that is the showreel.
    const Card *open() const;

    // True when this server lets a visitor look round without an account, which is the one thing
    // that adds a stop to the showreel card.
    bool openServer() const { return !accounts_required_; }

    // True once the brand's lockup at the top left has given way to the row of controls.
    //
    // It is state and not arithmetic, because both twins make it stick: it goes true the first
    // time the viewer goes up to the row or along to a title, and nothing ever puts it back.
    // `Door.kt:254` sets it during composition and `Door.swift:352-354` at the end of a press;
    // those two reach the same answer by different routes and the port takes the Kotlin's, which
    // is the one that does not depend on a value read before the press.
    bool raised() const { return raised_; }

    // One word from the remote.
    //
    // A right to left page has already had its word turned by `streams::remote::turn` before it
    // reaches here, so nothing in this file counts in two directions. The cards really do run
    // leftwards in such a language -- `Door.kt:267`'s `along()` -- and that is the drawing's
    // business, not this one's.
    Did press(Word word);

    // What the last `Did::Opens` opens.
    Opening opening() const { return opening_; }

private:
    Did pressTop(Word word);
    Did pressCard(Word word);
    Did pressTitle(Word word);

    // Where a press from the row above lands when it comes down, and where Back from the row
    // returns to. `Door.kt:271`'s `cardStop`.
    Stop cardStop() const;

    // True when the remote is on the row above the cards.
    bool onTop() const;

    // Moves to a card and to that card's own stop, and raises the row if the move raises it.
    // `Door.kt:272-275`'s `show`.
    Did show(int card);

    // Puts the stop somewhere that exists, after the cards or the server's answer changed.
    void settle();

    std::vector<Card> cards_;
    bool accounts_required_ = true;
    Stop at_ = Stop::Join;
    int page_ = 0;
    bool raised_ = false;
    Opening opening_ = Opening::SignIn;
};

// Sign in: the two ways in.
//
// Which way in. `SignIn.kt:44`, `SignIn.swift:6`.
enum class Way : std::uint8_t {
    // A phone opens the address on the screen, or its camera reads the square, and approves the
    // code. The core keeps that code alive and this screen only draws it, so there is nothing on
    // this side of it for a key map to do.
    Phone,
    // The remote and the keyboard.
    Remote,
};

// Which of the two the keyboard is filling. `SignIn.kt:45`, `SignIn.swift:7`.
enum class Ask : std::uint8_t { Email, Password };

// What a press on the sign in screen did.
enum class Asked : std::uint8_t {
    // The word was used and the screen is as it was.
    Nothing,
    // Something moved: the way, the step, or a key of the keyboard.
    Moved,
    // The screen is done and the shell goes back to the landing. Back on the two pills, and Left
    // past the first of them, which is note 187's "Left at the first stop goes back" and is the
    // same rule `Pin::press` follows at the first digit.
    Went,
    // The email address and the password are to go to the server, as `ActionSignIn`. `email()` and
    // `password()` are what to send, and the password is gone as soon as it has been read once.
    Asks,
};

class SignIn {
public:
    SignIn();

    Way way() const { return way_; }
    Ask ask() const { return ask_; }

    // True while the remote is on the keyboard rather than on the two pills. `SignIn.kt:157`'s
    // `onPills`, the other way up.
    bool onKeys() const { return on_keys_; }

    // The keyboard, which exists whether it is drawn or not: the Remote way draws it and the Phone
    // way does not, and what has been typed into it survives a look at the other way, exactly as
    // the twins' `remember`ed and `@State` keyboard does.
    streams::keys::Keys &keys() { return keys_; }
    const streams::keys::Keys &keys() const { return keys_; }

    // The email address, once the first step is finished. Trimmed of the spaces the keyboard's own
    // space key can put at either end, which is all the trimming a keyboard of these sixty keys
    // can ever need: `SignIn.kt:166`'s `trim()` and `SignIn.swift:168`'s
    // `trimmingCharacters(in: .whitespaces)` are both wider and neither can meet anything wider.
    const std::string &email() const { return email_; }

    // The password to send. Empty unless the last answer was `Asked::Asks`, and emptied by
    // `forget`. `gate.h` holds the PIN digits the same way and for the same reason: `PinPanel.kt`
    // says the PIN is never kept, and here that is enforced rather than described.
    const std::string &password() const { return password_; }

    // Drops the password. Called by the shell once it has sent it, and by this class itself the
    // moment the keyboard is finished with.
    void forget();

    // One word from the remote.
    Asked press(Word word);

private:
    // What a finished keyboard answered: the text, or nothing when it was cancelled.
    // `SignIn.kt:160-176` branch for branch.
    Asked finish(bool has_text);

    Way way_ = Way::Phone;
    Ask ask_ = Ask::Email;
    bool on_keys_ = false;
    std::string email_;
    std::string password_;
    streams::keys::Keys keys_;
};

} // namespace streams::door

#endif // STREAMS_XBOX_DOOR_H
