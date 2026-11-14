// Where does the remote go on the door, and what does each of the six words do from there?
//
// Scripted key sequences against the two state machines in `door.h`: the landing, and the sign in
// screen with the keyboard of `keys.h` under one of its two ways. Nothing here needs an Xbox, a
// console, a Windows toolchain or a screen.
//
//     c++ -std=c++17 -Wall -Wextra apps/xbox/surfaces/remote/remote.cpp \
//         apps/xbox/surfaces/remote/keys.cpp apps/xbox/surfaces/remote/door.cpp \
//         apps/xbox/surfaces/remote/door_test.cpp -o door && ./door
#include <cstdio>
#include <string>
#include <vector>

#include "door.h"

using streams::door::Asked;
using streams::door::Ask;
using streams::door::Card;
using streams::door::Did;
using streams::door::Landing;
using streams::door::Opening;
using streams::door::SignIn;
using streams::door::Stop;
using streams::door::Way;
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

template <typename Screen, typename Answer>
Answer press(Screen &screen, const char *script) {
    Answer did = screen.press(wordOf(*script));
    for (const char *key = script + 1; *key != '\0'; ++key) {
        did = screen.press(wordOf(*key));
    }
    return did;
}

Did say(Landing &landing, const char *script) { return press<Landing, Did>(landing, script); }

Asked say(SignIn &screen, const char *script) { return press<SignIn, Asked>(screen, script); }

// The trending title in view, read through a helper so that a mutation which walks the cards off
// the end prints the checks it broke rather than crashing the suite on a null read.
std::string openCard(const Landing &landing) {
    const Card *card = landing.open();
    return card == nullptr ? std::string("<the showreel>") : card->key;
}

Card title(const std::string &key) {
    Card made;
    made.key = key;
    made.name = key;
    return made;
}

std::vector<Card> three() { return {title("tv:a"), title("mv:b"), title("tv:c")}; }

// What the server said about accounts. Given to the landing, never written in it: absence reads
// true, and applying that default is the contract reader's job.
constexpr bool Required = true;
constexpr bool Open = false;

// To the Cancel and the Done buttons of the keyboard from wherever the remote is on it: down to
// the last row, left to the first button, and for Done one step right.
const char *ToCancel = "DDDDDLLLLLLLLLLO";
const char *ToDone = "DDDDDLLLLLLLLLLRO";

} // namespace

int main() {
    // ================================================================== the landing
    // ------------------------------------------------------------------ where it opens
    {
        Landing landing(three(), Required);
        ok(landing.at() == Stop::Join && landing.page() == 0,
           "the landing opens on Get Started, on the showreel card, as the reference has it");
        ok(!landing.raised(),
           "with the brand's lockup still at the top left rather than the row of controls");
        ok(openCard(landing) == "<the showreel>", "and no title's card in view");
        ok(!landing.openServer(),
           "a server that asks for accounts says so, and the landing is given the answer");
    }

    // ------------------------------------------------------------------ the showreel card
    {
        Landing landing(three(), Required);
        ok(say(landing, "L") == Did::Moved && landing.at() == Stop::SignIn,
           "Left off Get Started is Sign In");
        ok(say(landing, "L") == Did::Moved && landing.at() == Stop::Help,
           "and Left again is Get Help, low on the card's left");
        ok(say(landing, "L") == Did::Nothing && landing.at() == Stop::Help,
           "Left at Get Help is the left edge of the card and it stays there: the landing is the "
           "first screen of the app and has nothing to fall out of sideways");
        ok(say(landing, "RR") == Did::Moved && landing.at() == Stop::Join,
           "Right walks back along the three");
        ok(say(landing, "D") == Did::Nothing && landing.at() == Stop::Join,
           "Down on the card does nothing: there is nothing under the buttons");
    }

    // ------------------------------------------------------------------ what each one opens
    {
        Landing landing(three(), Required);
        ok(say(landing, "O") == Did::Opens && landing.opening() == Opening::SignIn,
           "Get Started opens the sign in screen -- both twins send it there, so the join flow a "
           "browser has does not exist on a television yet");
        ok(say(landing, "LO") == Did::Opens && landing.opening() == Opening::SignIn,
           "and so does Sign In beside it");
        ok(say(landing, "LO") == Did::Opens && landing.opening() == Opening::Help,
           "Get Help opens get help");
    }

    // ------------------------------------------------------------------ the row above the cards
    {
        Landing landing(three(), Required);
        ok(say(landing, "U") == Did::Moved && landing.at() == Stop::Home,
           "Up from the showreel card is the house at the top left");
        ok(landing.raised(),
           "and the lockup gives way to the row of controls the first time the viewer goes up");
        ok(say(landing, "L") == Did::Nothing && landing.at() == Stop::Home,
           "Left at the house is the left edge of the row and of the page");
        ok(say(landing, "R") == Did::Moved && landing.at() == Stop::Tab,
           "Right is the tab of the trending titles");
        ok(say(landing, "R") == Did::Nothing && landing.at() == Stop::Tab,
           "and there is nothing past it while the showreel is the card in view: the way to sign "
           "in only moves up to the top right once a title's card is there");
        ok(say(landing, "U") == Did::Nothing && landing.at() == Stop::Tab,
           "Up from the row does nothing: it is the top of the page");
        ok(say(landing, "D") == Did::Moved && landing.at() == Stop::Join && landing.page() == 0,
           "Down comes back to the card in view, on that card's own stop");
    }

    {
        Landing landing(three(), Required);
        say(landing, "LL");
        ok(landing.at() == Stop::Help, "the remote is on Get Help");
        ok(say(landing, "UD") == Did::Moved && landing.at() == Stop::Join,
           "up to the row and straight back down lands on Get Started, not on Get Help: the "
           "landing remembers which card is in view and not which button of it");
    }

    {
        Landing landing(std::vector<Card>(), Required);
        say(landing, "U");
        ok(landing.at() == Stop::Home, "on a server with no trending titles the row is the house");
        ok(say(landing, "R") == Did::Nothing && landing.at() == Stop::Home,
           "and nothing else: the tab is not drawn when there is nothing in it");
        ok(say(landing, "D") == Did::Moved && landing.at() == Stop::Join,
           "Down still comes back to the card");
        ok(say(landing, "RRR") == Did::Nothing && landing.page() == 0,
           "and Right past Get Started brings no card in, because there is none");
    }

    // ------------------------------------------------------------------ the trending cards
    {
        Landing landing(three(), Required);
        ok(say(landing, "R") == Did::Moved && landing.page() == 1 && landing.at() == Stop::Watch,
           "Right past Get Started brings the first title's card in, on its one button");
        ok(openCard(landing) == "tv:a", "which is the first trending title");
        ok(say(landing, "RR") == Did::Moved && landing.page() == 3 && openCard(landing) == "tv:c",
           "and Right walks the rest of them");
        ok(say(landing, "R") == Did::Nothing && landing.page() == 3,
           "Right at the last card stays on it");
        ok(say(landing, "O") == Did::Opens && landing.opening() == Opening::SignIn,
           "OK on a title's card signs in: a server that asks for accounts has nothing else to "
           "offer a visitor who picked a title");
        ok(say(landing, "U") == Did::Moved && landing.at() == Stop::Tab,
           "Up from a title's card is the tab, not the house");
        ok(say(landing, "R") == Did::Moved && landing.at() == Stop::TopSignIn,
           "and the way to sign in is now at the top right, past the tab");
        ok(say(landing, "RO") == Did::Opens && landing.opening() == Opening::SignIn,
           "Right at it stays, and OK on it signs in");
        ok(say(landing, "LL") == Did::Moved && landing.at() == Stop::Home,
           "Left walks back along the row to the house");
        ok(say(landing, "D") == Did::Moved && landing.at() == Stop::Watch && landing.page() == 3,
           "and Down comes back to the title's card the viewer left");
        ok(say(landing, "LLL") == Did::Moved && landing.page() == 0 &&
               landing.at() == Stop::Join,
           "Left walks back along the cards and out onto the showreel, which is where the first "
           "title's card came in from -- there is no left edge on a title's card");
    }

    {
        Landing landing(three(), Required);
        say(landing, "URO");
        ok(landing.page() == 1 && landing.at() == Stop::Watch,
           "OK on the tab brings the first title's card in");
        say(landing, "RU");
        ok(landing.at() == Stop::Tab && landing.page() == 2, "the remote goes up to the tab");
        ok(say(landing, "O") == Did::Moved && landing.page() == 2 && landing.at() == Stop::Watch,
           "and OK on it comes back to the card that is already in view, not to the first");
        say(landing, "ULO");
        ok(landing.page() == 0 && landing.at() == Stop::Join,
           "and OK on the house brings the showreel back");
    }

    // ------------------------------------------------------------------ Back
    {
        Landing landing(three(), Required);
        ok(say(landing, "UB") == Did::Moved && landing.at() == Stop::Join,
           "Back from the row returns to the card in view");
        ok(say(landing, "RRB") == Did::Moved && landing.page() == 0 && landing.at() == Stop::Join,
           "Back from a title's card returns to the showreel, however many cards along it was");
        ok(say(landing, "B") == Did::Leaves,
           "and Back on the showreel's buttons has nowhere further back. Both twins hand that "
           "press to the host and both hosts close the app; on Xbox nothing above will do it, so "
           "the branch is named and the shell decides");
        ok(landing.at() == Stop::Join && landing.page() == 0,
           "the landing itself does not move when it says so");
    }

    // ------------------------------------------------------------------ a look round, where the server allows one
    {
        Landing landing(three(), Open);
        ok(landing.openServer(), "a server that asks for no accounts says so");
        ok(say(landing, "R") == Did::Moved && landing.at() == Stop::LookRound,
           "and Right off Get Started is a way straight in, before any card comes along");
        ok(say(landing, "O") == Did::Opens && landing.opening() == Opening::Home,
           "which opens the home page with no account at all. Neither television twin reads "
           "`Features.accounts_required`; the browser does, and the core routes on it");
        ok(say(landing, "L") == Did::Moved && landing.at() == Stop::Join,
           "Left off it is Get Started again");
        ok(say(landing, "RR") == Did::Moved && landing.page() == 1,
           "and Right past it still brings the first title's card in");
    }

    {
        Landing landing(three(), Required);
        ok(say(landing, "RRRR") == Did::Nothing && landing.page() == 3,
           "on a server that asks for accounts there is no such stop to walk through");
        Landing empty(std::vector<Card>(), Open);
        ok(say(empty, "R") == Did::Moved && empty.at() == Stop::LookRound,
           "and a server with no trending titles at all still offers it");
        ok(say(empty, "R") == Did::Nothing && empty.at() == Stop::LookRound,
           "with nothing past it");
    }

    // ------------------------------------------------------------------ a fresh answer
    {
        Landing landing(three(), Required);
        say(landing, "RRR");
        ok(landing.page() == 3, "the remote is on the last of three cards");
        landing.arrived({title("tv:a")}, Required);
        ok(landing.page() == 1 && openCard(landing) == "tv:a" && landing.at() == Stop::Watch,
           "an answer with fewer titles moves it to the last card there is");
        landing.arrived(std::vector<Card>(), Required);
        ok(landing.page() == 0 && landing.at() == Stop::Join,
           "and an answer with none puts it back on the showreel's own stop");
    }

    {
        Landing landing(three(), Required);
        say(landing, "RU");
        say(landing, "R");
        ok(landing.at() == Stop::TopSignIn, "the remote is on the way to sign in at the top right");
        landing.arrived(std::vector<Card>(), Required);
        ok(landing.at() == Stop::Home,
           "an answer with no titles takes both that stop and the tab away, and the house is what "
           "is left of the row");
    }

    {
        Landing landing(three(), Open);
        say(landing, "R");
        ok(landing.at() == Stop::LookRound, "the remote is on the look round");
        landing.arrived(three(), Required);
        ok(landing.at() == Stop::Join && !landing.openServer(),
           "and a server that turns out to want accounts after all takes it away from under them");
    }

    // ------------------------------------------------------------------ a held direction
    {
        Landing landing(three(), Required);
        say(landing, "LL");
        int moved = 0;
        for (int again = 0; again < 20; ++again) {
            if (landing.press(Word::Left) != Did::Nothing) {
                moved += 1;
            }
        }
        ok(moved == 0 && landing.at() == Stop::Help && landing.page() == 0,
           "a held Left at Get Help says the same word twenty times and nothing happens twenty "
           "times: a direction repeats on a controller, so every edge has to be idempotent");

        Landing crossing(three(), Required);
        say(crossing, "RRR");
        ok(crossing.page() == 3, "the remote is on the last card");
        ok(say(crossing, "LLLLL") == Did::Moved && crossing.page() == 0 &&
               crossing.at() == Stop::Help,
           "and a held Left crosses three cards and two buttons in five repeats and stops at Get "
           "Help, which is one screen with two key maps in it and one hold crossing both");
    }

    // ================================================================== sign in
    // ------------------------------------------------------------------ the two ways in
    {
        SignIn screen;
        ok(screen.way() == Way::Phone && !screen.onKeys(),
           "sign in opens on the phone, which is the way that needs no typing");
        ok(screen.ask() == Ask::Email, "with the email address as the first thing it asks for");
        ok(say(screen, "D") == Asked::Nothing && !screen.onKeys(),
           "Down on the phone does nothing: what is under it is a code the core keeps alive");
        ok(say(screen, "O") == Asked::Nothing, "and so does OK");
        ok(say(screen, "U") == Asked::Nothing, "and Up, which has nothing above it");
        ok(say(screen, "R") == Asked::Moved && screen.way() == Way::Remote,
           "Right is the other way in: the remote and the keyboard");
        ok(say(screen, "R") == Asked::Nothing && screen.way() == Way::Remote,
           "and there is no third");
        ok(say(screen, "L") == Asked::Moved && screen.way() == Way::Phone,
           "Left comes back to the phone");
        ok(say(screen, "L") == Asked::Went,
           "and Left past the first of the two goes back, which is the rule the PIN panel follows "
           "at its first digit");
        ok(say(screen, "B") == Asked::Went, "Back says the same from either of them");
    }

    // ------------------------------------------------------------------ down into the keys
    {
        SignIn screen;
        say(screen, "RD");
        ok(screen.onKeys(), "Down off the remote pill goes into the keys");
        ok(screen.keys().row() == 2 && screen.keys().column() == 4,
           "which open in the middle of the home row");
        ok(say(screen, "UU") == Asked::Moved && screen.keys().row() == 0,
           "Up walks to the top row of them");
        ok(say(screen, "U") == Asked::Moved && !screen.onKeys(),
           "and Up from the top row hands the remote back to the two pills -- the one word the "
           "keyboard does not take");
        ok(screen.way() == Way::Remote, "which is still the remote way");
        ok(say(screen, "O") == Asked::Moved && screen.onKeys(),
           "OK goes back down into them, as Down does");
    }

    // ------------------------------------------------------------------ the two steps
    {
        SignIn screen;
        say(screen, "RD");
        ok(say(screen, "O") == Asked::Moved && screen.keys().text() == "g",
           "a letter typed on the keys lands in the field");
        screen.keys().hold("  someone@example.com  ");
        ok(say(screen, ToDone) == Asked::Moved && screen.ask() == Ask::Password,
           "Done on the email step asks for the password next");
        ok(screen.email() == "someone@example.com",
           "and keeps the address, trimmed of the spaces the space key can put at either end");
        ok(screen.keys().text().empty(), "with an empty field under the keys for the password");
        ok(screen.onKeys(), "and the remote still on them: the two steps are one keyboard");

        screen.keys().hold("hunter2");
        ok(say(screen, ToCancel) == Asked::Moved && screen.ask() == Ask::Email,
           "Cancel on the password step goes back one step, not out of the keyboard");
        ok(screen.keys().text() == "someone@example.com",
           "and hands the address back so the viewer need not type it twice");
        ok(screen.onKeys(), "still on the keys");
        ok(say(screen, ToCancel) == Asked::Moved && !screen.onKeys(),
           "and Cancel on the email step leaves them for the two pills");
        ok(screen.email() == "someone@example.com",
           "the address that was finished is still the address that was finished");
    }

    // ------------------------------------------------------------------ what goes to the server
    {
        SignIn screen;
        say(screen, "RD");
        screen.keys().hold("someone@example.com");
        say(screen, ToDone);
        screen.keys().hold("hunter2");
        ok(say(screen, ToDone) == Asked::Asks, "Done on the password step asks the server");
        ok(screen.email() == "someone@example.com" && screen.password() == "hunter2",
           "with the address and the password it was given");
        ok(screen.keys().text().empty(),
           "and the field is emptied at once, so a server that refuses draws its notice under an "
           "empty field and the viewer types again");
        ok(screen.ask() == Ask::Password && screen.onKeys(),
           "the remote stays on the password step rather than starting over");
        ok(say(screen, "L") == Asked::Moved && screen.password().empty(),
           "and the password is gone by the next press, whatever the shell did with it");

        SignIn forgotten;
        say(forgotten, "RD");
        forgotten.keys().hold("a@b.c");
        say(forgotten, ToDone);
        forgotten.keys().hold("secret");
        ok(say(forgotten, ToDone) == Asked::Asks && forgotten.password() == "secret",
           "a second screen asks the same way");
        forgotten.forget();
        ok(forgotten.password().empty(), "and the shell can drop the password itself");
    }

    // ------------------------------------------------------------------ Back, one level at a time
    {
        SignIn screen;
        say(screen, "RD");
        screen.keys().hold("a@b.c");
        say(screen, ToDone);
        ok(screen.ask() == Ask::Password && screen.onKeys(), "the remote is on the password step");
        ok(say(screen, "B") == Asked::Moved && screen.ask() == Ask::Email,
           "Back on the keys is Cancel: the password step gives way to the email step");
        ok(say(screen, "B") == Asked::Moved && !screen.onKeys(),
           "the email step gives way to the two pills");
        ok(say(screen, "B") == Asked::Went,
           "and the pills give way to the page the viewer came from. Four levels, one press each");
    }

    std::printf("\n%d of %d held\n", checks - failures, checks);
    return failures == 0 ? 0 : 1;
}
