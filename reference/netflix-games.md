# Netflix games, observed

What Netflix's games product does, from a 4:41.60 screen recording of a signed-in session on the
web, made 2026-11-12 at 3584x2240. Every claim here is something seen happening in that recording,
and the timings were measured from it rather than estimated.

This file differs from its two neighbours on purpose. `netflix-on-the-tv.md` and
`netflix-on-the-web.md` hold geometry read with pixel coordinates. **This one holds behaviour and
timing**, because a screen recording is not a ruler: it was captured at a window size nobody
recorded, so a pixel here is not a reference pixel and no number below is a layout number. What it
can say exactly is what happened, in what order, and how long it took.

Status: verified on that one session. One session is one sample -- a load time is what it was that
evening on that connection, not a specification.

## The shape of a session

    browse  ->  title page  ->  launch  ->  game  ->  exit  ->  survey  ->  back to the title page

Nothing in it is modal over the catalogue. A game takes the whole window and hands it back.

## The chrome Netflix wraps around a running game

A bar along the bottom of the game, carrying four controls:

| control | what it opens |
|---|---|
| Play game | returns to the game |
| Controllers | a panel with a QR code: "Scan to connect your phone" |
| Achievements | a row of card-shaped tiles, a counter, "Keep playing to unlock more" |
| Exit game | a dialog: "Continue playing" / "Exit game" |

**The controller is reached by a QR code in the page, not by installing anything.** The phone
becomes the pad by scanning; the panel says so and says what happens if it disconnects. On mobile
the same feature is a separate app in the store. That difference is the whole argument for
building ours into one app: the web route has no install step at all, so the install on mobile is
a platform concession and not a product decision.

A toast on entry reads "To exit your game, press Shift + Tab", so the exit is also a keystroke and
not only a control.

## What it asks after the session

Two screens, in order, after Exit game:

1. "How was your experience playing <title>?" -- five steps, Poor to Excellent, and a toggle.
2. "What could we improve?" -- **Video quality, Delayed input, Audio quality, Connection quality**,
   Other, a free-text box, and Continue.

Those four named faults are a playback-quality instrument. They are the same four a cloud session
can be bad in, and they are asked of the viewer because the client cannot see all of them alone.

## Measured: how long a viewer waits at a black screen

Luminance over all 282 seconds of the recording, one sample per second, counting runs where the
whole frame is near black.

| when | duration |
|---|---|
| first launch | **22 s** |
| returning to the game's main menu | **23 s** |
| mid-launch, before the studio logo | 8 s |

So **roughly 40 s from pressing play to a playable menu, over half of it a black screen with
nothing on it**. No progress bar, no spinner, no artwork -- the wordmark fades in near the end and
that is all. This is the ceiling a self-hosted player is measured against, and it is a low one.

## The launch sequence, in order

    NETFLIX GAMES wordmark
    "To exit your game, press Shift + Tab"
    title wordmark over a gradient, "Now loading..."
    the studio's own animated logo
    black
    the game's own main menu

The platform's branding comes first, the studio's second, and the game's own menu is reached last.
The viewer sees three different brands before a control they can press.

## What the game keeps for itself

The recording's game draws its own options (subtitles, language, three volumes), its own pause
menu, its own map, its own collectibles grid and its own quit confirmation, all in its own art.
**Netflix wraps a game; it does not restyle one.** The only Netflix surface inside the session is
the bottom bar, the QR panel, the achievements panel and the survey.

One consequence worth stating: the game's pause menu and Netflix's exit control both exist, and
they do different things. Quitting to the game's main menu costs the 23 s black screen; exiting
through the bar leaves the game entirely.

## Open

* Every number here is one sample from one session on one connection.
* The capture's window size was not recorded, so nothing here is a layout measurement.
* The games browse rows were seen but not measured: `netflix-on-the-web.md`'s method would measure
  them properly, and that has not been done.
* Whether the QR controller path exists on a television, and what it does there, is unobserved.
