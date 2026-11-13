# Netflix on the web, measured

The layout this project's web surface is held to. Every number here was read from the live page
with `getBoundingClientRect` and `getComputedStyle` in a signed-in browser on 2026-11-13, not read
from any published specification, not estimated from a screenshot and not taken from a photograph.
Only measurements live in this file. The method, the session and the open questions are the
project's private note; this is the part the code cites, so it lives with the code.

Provenance: Chrome 159 on macOS, `devicePixelRatio` 2, `www.netflix.com/browse` (Home), the United
Kingdom catalogue, signed in. An account-state banner 118.5 px tall sat above the top bar; it was
hidden locally (`display: none`) before measuring, so every y below is the page a reader without
that banner sees. Viewport height was 899 px at every width: the screen is 1792 x 1120 and the
window chrome takes the rest, so no reading here was taken on a 1080 px tall viewport.

A number here is a **CSS pixel at the stated viewport width**. Unlike the television, the web
layout has no single reference resolution: nine widths were measured and the rule that connects
them is given below.

Status: verified on that session.

## How close a surface has to be

A web surface holds to this reference when every measured size is within **1 px** of the number
here. `getBoundingClientRect` reports sub-pixel values and nothing here passed through a video
codec, so there is no measurement noise to absorb: 1 px is one pixel of real drift.

Tile widths are fractional because they come from a division. What a surface has to match is the
derivation below, not a rounded copy of one row of the table.

## The one rule that produces a row

At every width measured, the tiles fill the content box exactly:

**tile width = (viewport - 2 x page margin - (across - 1) x 8) / across**

The gutter is **8 px at every width measured, from 570 to 1920**. The page margin and the number
across are the two step functions; everything else follows from them.

| viewport | page margin | across | tile | pitch | hero |
|---|---|---|---|---|---|
| 1920 | 60 | 7 | 250.3 x 140.5 | 258.29 | 1800 x 825.7 |
| 1800 | 60 | 6 | 273.3 x 153.4 | 281.33 | 1680 x 770.6 |
| 1600 | 60 | 6 | 240.0 x 134.7 | 248.00 | 1480 x 678.9 |
| 1536 | 48 | 5 | 281.6 x 158.1 | 289.60 | 1440 x 660.5 |
| 1440 | 48 | 5 | 262.4 x 147.3 | 270.40 | 1344 x 616.5 |
| 1280 | 48 | 5 | 230.4 x 129.3 | 238.40 | 1184 x 543.1 |
| 1024 | 36 | 4 | 232.0 x 130.2 | 240.00 | 952 x 436.7 |
| 768 | 36 | 3 | 226.7 x 127.3 | 234.67 | 696 x 838.5 |
| 570 | 24 | 2 | 257.0 x 144.3 | 265.00 | 522 x 779.1 |

Pitch is the tile plus the gutter; it is a derivation, not a separate measurement.

A tile never falls below **226.7 px** and never rises above **281.6 px** across that whole range:
the count across changes before the tile is allowed to drift far. The steps measured are 2 across
at 570, 3 at 768, 4 at 1024, 5 from 1280 to 1536, 6 from 1600 to 1800, 7 at 1920. The page margin
steps 24, 36, 48, 60 over the same range. The exact width where each step happens was not
bracketed; the stylesheet carries `min-width` queries at 1280, 1600 and 1920, which is where three
of the steps were seen.

## Tile shape

The image inside a tile is **exactly 16:9** (250.3 x 140.8 at 1920, natural asset 332 x 187). The
tile box measured 0.3 px shorter, 140.5, at 1920 and the same 0.5614 of its width at every other
width; the image overflows its box by that much. Corner radius **12 px**.

## Page margin

**60 px at 1920**, and the top bar, the hero and the first tile all start on it: the logo at x 60,
the hero card at x 60, the first tile at x 60, the profile control ending at x 1860. The margin is
one number per width class (24 / 36 / 48 / 60), not a percentage: at 1920 it is 3.1% of the width,
at 570 it is 4.2%.

## Top bar

**80 px tall at every width measured**, `position: sticky` at y 0, **transparent at scroll 0 and
`rgb(0, 0, 0)` once the page scrolls**. Horizontal padding equals the page margin.

At 1920: logo 76 x 20 at x 60-136, y 30-50. Seven navigation labels, 16 px / 400, boxes 40.5 tall
at y 19.8, the first at x 160, 6 px between boxes. On the right, three controls 40 px apart in a
row ending on the page margin: search 40 x 40 at x 1594, notifications 40 x 40 at x 1646, profile
68 x 36 at x 1792-1860.

## Home, first screen

Hero card **1800 x 825.7 px** at x 60-1860, y 92-917.7, corner radius **24 px**, margin
`12px 60px 32px` under the 80 px bar.

**Hero height = 0.4587 x hero width** (aspect 2.180:1), constant to four decimals at seven widths
from 1024 to 1920: 825.7/1800, 770.6/1680, 678.9/1480, 660.5/1440, 616.5/1344, 543.1/1184,
436.7/952.

Content sits 40 px inside the card (x 100). Title art 576 x 230.4 ending y 639.7; synopsis 691.2
wide, 24 px / 30 px line, clamped to **3 lines**, ending y 797.7; a button row 48 px tall at
y 829.7 with a white pill 108 wide, a 12 px gap, then a grey `rgba(128,128,128,0.4)` pill 120.2
wide, both fully rounded; a 36 px callout block right-aligned to x 1820. A trailer plays in place
behind the card.

On a 899 px tall viewport the hero is 18.7 px taller than the screen and **0 rows are visible**
before scrolling. That part is the same as the television.

Below 1024 the hero stops being a wide card: it measured 696 x 838.5 at 768 and 522 x 779.1 at
570, taller than wide. ~~Its height there is set by its content, not by a ratio.~~ **Corrected
2026-11-13**: it is a ratio there too, in two further bands — 1.2048 x its width from 600 to 768,
and 1.4925 x its width from 160 to 599. Both are reproduced from the same two numbers quoted here:
838.5/696 = 1.2048 and 779.1/522 = 1.4925. See "Hero" under "The phone widths" below.

## Vertical rhythm of the rows, at 1920

Row title y 949.7-980.2 (**30.5 tall, 24 px / 500**), gap 10, scroller y 990.2-1138.7 (**148.5**:
4 px inset, the 140.5 tile, 4 px inset), then the next section's `margin-top` of **32 px**.

**Row title to row title: 221.0 px.** Hero bottom to the first row title: 32 px.

A Top 10 row's section is 227.3 (its tile is 178.8 tall) and a Continue Watching row's is 197 (8 px
of progress bar under the tile), so those rows pitch 259.3 and 229.0 to the row after them.

## Paging, which is what the web does instead of focus

A row scrolls by **one full page of 7 tiles = 1808 px at 1920**, which is the content width plus
one gutter. The control is an arrow 28 x 56 inside a **60 px scrim** `rgba(20, 20, 20, 0.5)` at
x 1860-1920 spanning the scroller's full 148.5 height. The left arrow has zero width on page 1.
A row held 13 tiles in the document before the first press and 22 after it.

Nothing is pinned. The web has no fixed band like the television's y 199-708: the row stays where
it is and its contents translate sideways.

## Hover opens a card over the row; it does not open the row

**556 ms** after the pointer enters a tile (one sample, measured in page from `mouseover` to the
node being inserted), a preview appears as an absolutely positioned overlay at `z-index: 3`:

| | value |
|---|---|
| width | **375 px**, set inline, = **1.498 x the tile** |
| art area | 375 x 211.5, **16:9** |
| total height | not fixed: 369.5 and 384.6 in two samples, the info block varying 158 to 173.1 |
| centre | the tile's centre, within 0.5 px in both axes |
| corner radius | 6 px |
| shadow | `rgba(0, 0, 0, 0.75) 0 3px 10px` |
| controls | round buttons 34-38 px at 46 px pitch, 16 px inside the card, one at the right edge |

**The row does not reflow.** Every tile in the row keeps its position and its size while the
preview is open; the preview is drawn over its neighbours. This is the opposite of the television,
where the focused tile expands in place and pushes the row.

Keyboard focus is a different state from hover: `outline: 2px solid white`, `outline-offset: 0`,
on the tile's 12 px corner. No size change, no preview.

## The artwork swap

The tile's image is a 16:9 landscape still (natural 332 x 187 at 1920). The preview's image is a
**different asset** (a different path on the image host), natural 665 x 374, also 16:9, replaced
after a moment by a 1280 x 720 trailer. So the web swaps artwork the way the television does, but
both of its assets are landscape: there is no portrait poster on a standard web row.

## Top 10 rows

Same **250.3 px** tile width and the same **258.29 px** pitch as every other row at 1920, but
**178.8 px tall**, split into two halves of 125.1: the rank numeral in the left half and a
**portrait poster 125.1 x 178.8** (natural 426 x 597) in the right half.

The television widens the gutter to 146 and keeps one tile. The web keeps the pitch and splits the
tile. Only here does a portrait poster appear on the web home page.

## Continue Watching rows

The same 250.3 x 140.5 tile with the same 16:9 still, and **8 px** of progress bar added under it,
which is the whole of the difference between a 189 px section and a 197 px one.

## Colours read off the page

Page background `rgb(20, 20, 20)`. Top bar `rgb(0, 0, 0)` once scrolled, transparent at rest.
Keyboard focus ring white. Row scrim `rgba(20, 20, 20, 0.5)`.

## What is not measured here

* ~~**Any viewport narrower than 570 CSS px.**~~ Superseded on 2026-11-13 by "The phone widths"
  below: 160 to 599 CSS px are now measured, in a script-opened popup window that is not bound by
  the main window's 626 outer / 570 inner floor. What a phone **user agent** is served is still not
  established for the signed-in page; see the last part of that section for what was established.
* Any viewport taller than 899 px, and any width above 1920.
* ~~The exact width at which each step in the margin and the count across happens.~~ The first step
  is bracketed to 1 px on 2026-11-13: it is at exactly 600. The steps at 1024, 1280, 1600 and 1920
  are still unbracketed.
* Every screen except Home: the title page, search, My List and the episodes screen.
* The hover delay is a single sample, and it was taken at 1920 only.

## The phone widths

Added 2026-11-13, same session and same method as everything above, with one difference in how the
viewport was obtained. The main browser window on this machine cannot be made narrower than 570 CSS
px, so these widths were read in a **script-opened popup window** (`window.open` with explicit
dimensions), which is not bound by that floor and reaches **160 CSS px** of inner width. The popup
runs the same browser, the same profile and the same signed-in session.

Provenance: Chrome 159 on macOS, `devicePixelRatio` **2** at every width below, `www.netflix.com/browse`
(Home), the United Kingdom catalogue, signed in, 2026-11-13. The **user agent was unchanged desktop
Chrome** at every width:
`Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/159.0.0.0 Safari/537.36`.
`navigator.maxTouchPoints` was 0, `(hover: hover)` and `(pointer: fine)` both matched, and the page's
own `browserInfo.isMobile` was `false`. So this is the desktop document at phone widths, not the
document a phone is served. An account-state banner, 151 px tall at 390, sat above the top bar and was
hidden locally (`display: none`) before measuring, so every y below is the page without it. Viewport
height was 844 where a y is quoted.

Tolerance is the same 1 px as the rest of this file.

**Cross-check.** The popup reproduced the main window's two narrowest readings exactly: at 570,
tile 257.00 x 144.27 and hero 522 x 779.10; at 768, tile 226.66 and hero 696 x 838.55. Both agree
with the table above to 0.05 px.

### The one rule still holds

Same rule, same 8 px gutter, at every width from 160 to 599:

**tile width = (viewport - 2 x page margin - (across - 1) x 8) / across**

The page margin is **24 px** and the count across is **2** at every one of these widths. The layout
never falls to one tile across, down to 160.

| viewport | page margin | across | tile | pitch | hero width |
|---|---|---|---|---|---|
| 160 | 24 | 2 | 52.0 x 29.19 | 60.0 | 112 |
| 200 | 24 | 2 | 72.0 x 40.41 | 80.0 | 152 |
| 240 | 24 | 2 | 92.0 x 51.65 | 100.0 | 192 |
| 280 | 24 | 2 | 112.0 x 62.88 | 120.0 | 232 |
| 300 | 24 | 2 | 122.0 x 68.48 | 130.0 | 252 |
| 320 | 24 | 2 | 132.0 x 74.10 | 140.0 | 272 |
| 360 | 24 | 2 | 152.0 x 85.33 | 160.0 | 312 |
| 375 | 24 | 2 | 159.5 x 89.54 | 167.5 | 327 |
| 390 | 24 | 2 | 167.0 x 93.75 | 175.0 | 342 |
| 414 | 24 | 2 | 179.0 x 100.48 | 187.0 | 366 |
| 430 | 24 | 2 | 187.0 x 104.98 | 195.0 | 382 |
| 480 | 24 | 2 | 212.0 x 119.02 | 220.0 | 432 |
| 540 | 24 | 2 | 242.0 x 135.86 | 250.0 | 492 |
| 570 | 24 | 2 | 257.0 x 144.27 | 265.0 | 522 |
| 575 | 24 | 2 | 259.5 x — | 267.5 | 527 |
| 580 | 24 | 2 | 262.0 x — | 270.0 | 532 |
| 590 | 24 | 2 | 267.0 x — | 275.0 | 542 |
| 595 | 24 | 2 | 269.5 x — | 277.5 | 547 |
| 599 | 24 | 2 | 271.5 x — | 279.5 | 551 |

The measured tile matched the rule to **0.00 px** at every width in this table. Hero width is
always `viewport - 48`, which is the content box.

Tile height is **0.5614 x tile width**, the same constant as every width above: 29.19/52,
93.75/167, 144.27/257. The image inside is exactly 16:9 (167 x 93.91 at 390, natural asset
332 x 187) and overflows its 93.75 box by 0.16 px, the same way it does at 1920. Corner radius is
**8 px**, not the 12 px measured at 1920.

### The first step is at exactly 600

Bracketed to 1 px. At **599** the page margin is 24, the count across is 2, row titles are
16 px / 500 and the hero ratio is 1.4925. At **600** the page margin is 36, the count across is 3,
row titles are 18 px / 500 and the hero ratio is 1.2048. Tile at 600 is 170.66, matching the rule
on a 36 px margin.

### Top bar

**80 px tall** at every width from 160 to 768, unchanged from 1920. `position: sticky` at y 0,
transparent at scroll 0 and `rgb(0, 0, 0)` once the page scrolls. Horizontal padding equals the
page margin: **24 px**. Logo 76 x 20 at x 24, the same asset size as at 1920.

### Hero

The hero is a card at phone widths too, and its height is a **constant multiple of its width**, not
content-driven:

* **hero height = 1.4925 x hero width** at every width from 160 to 599 (510.45/342 at 390,
  779.10/522 at 570, 405.97/272 at 320).
* **hero height = 1.2048 x hero width** from 600 to 768 (636.14/528 at 600, 838.55/696 at 768).

At 390: card 342 x 510.45 at x 24, y 84; corner radius **12 px** (24 px at 1920); margin
`4px 24px 16px` (`12px 60px 32px` at 1920). Content sits **16 px** inside the card (x 40), against
40 px at 1920. Title art 310 x 124. Synopsis 310 wide, **14 px / 24 px line, clamped to 3 lines**
(24 px / 30 px at 1920). Two action pills, each the **full content width, 310 x 40**, stacked with
an 8 px gap, fully rounded: white, then `rgba(128,128,128,0.4)`. They do not sit side by side as
they do at 1920. The callout block measured 0 x 0 — it is absent at this width.

On a **390 x 844** viewport, `devicePixelRatio` 2: hero y 84 to 594.45, first row title at
y 626.45 — **32 px below the hero, the same gap as at 1920** — the first row's tiles fully on
screen at y 660.95 to 754.70, and the second row's title at y 790.7 with its tiles cut by the fold.
So unlike 1920, where 0 rows are visible before scrolling, **one full row is visible** at 390 x 844.

### Vertical rhythm of the rows, at 390

Row title y 626.45-646.95 (**20.5 tall, 16 px / 500**), gap **10**, scroller y 656.95-758.70
(**101.75**: 4 px inset, the 93.75 tile, 4 px inset), then the next section's `margin-top` of
**32 px**. Every one of those four numbers except the title box and the tile is identical to 1920.

**Row title to row title: 164.25 px.**

A Continue Watching row's section is 140.25 against a standard row's 132.25 — **8 px** of progress
bar under the tile, the same 8 px as at 1920. A Top 10 row's section is 157.78. A cloud-game row
measures the same as a standard row.

### Top 10 rows, at 390

Same **167 px** tile width and the same **175 px** pitch as every other row at this width, but
**119.28 px tall**, split into two halves of **83.5**: the rank numeral in the left half at x 24 and
a **portrait poster 83.5 x 119.28** (natural 426 x 597) in the right half at x 107.5. This is the
same construction as at 1920 and the only portrait artwork on the phone-width Home page. Standard,
Continue Watching and cloud-game rows all keep the landscape 332 x 187 still.

### Paging

One press of the forward control moves the row **350 px at 390**, which is the content width 342
plus one gutter, and also exactly 2 x the 175 pitch — the same derivation as the 1808 px measured at
1920. The control is an arrow **28 x 56** inside a scrim **24 px** wide at x 366-390, spanning the
scroller's full 101.75 height. **The scrim width equals the page margin** at both widths measured
(60 at 1920, 24 at 390). The left arrow has zero width on page 1 and is 28 x 56 after the first
press. A row held 5 tiles in the document before the first press and 7 after it.

### Hover preview

It still opens at phone widths, as an absolutely positioned overlay at `z-index: 3`, corner radius
**6 px**, and **the row does not reflow**: the three tiles measured kept x 24, 199 and 374 with the
preview open.

Its width is **max(320, 1.5 x tile)**:

| viewport | tile | 1.5 x tile | preview width | preview height | art |
|---|---|---|---|---|---|
| 320 | 132.0 | 198.0 | **320** | 338.45 | 320 x 180.45 |
| 390 | 167.0 | 250.5 | **320** | 338.45 | 320 x 180.45 |
| 430 | 187.0 | 280.5 | **320** | 338.45 | 320 x 180.45 |
| 570 | 257.0 | 385.5 | **386** | 375.67 | 386 x 217.67 |

So **320 px is a floor** on the preview, and above the width where 1.5 x tile passes 320 it is the
same 1.5 x tile ratio measured at 1920 (375 / 250.3 = 1.498). The art area is 16:9 in both cases and
the info block below it is **158 px** in both, which is the bottom of the 158 to 173.1 range
measured at 1920.

**It does not centre on its tile horizontally at these widths.** It centres vertically to within
0.5 px, as at 1920, but the preview is clamped inside the viewport: at 390 it sits at x 25-345 while
the tile's centre is at x 107.5, an offset of 67.5 px. At 320 the offset is 94 px. With a 320 px
preview in a 390 px viewport no tile can be centred on, so this is forced rather than a different
rule.

### What a phone user agent is served

Not answered for the signed-in page, and partly answered for the public one. Requested
`https://www.netflix.com/gb/` three times on 2026-11-13 with an iPhone 17.5 Safari UA, a Pixel 8
Chrome UA and the desktop Chrome UA above: all three returned **200**, the same `<title>`, the same
`<meta name="viewport" content="width=device-width,initial-scale=1.0,minimum-scale=1.0,maximum-scale=1.0">`
and the **same single script bundle** `nmhpFrameworkClient.f499ab2ebc193c5a92ba.js` (the set of
bundle URLs is byte-identical across the three). The only difference found is a server-rendered flag:
`isMobile":true` for both phone agents and `isMobile":false` for the desktop agent. `/browse`
returns a 302 to `/login` for every agent when unauthenticated, with `x-frame-options: DENY` on both
paths.

So Netflix does branch on the user agent, and on this page that branch does not change the document
or the code it loads. Whether it changes the signed-in `/browse` document is **not established**: it
could not be requested with a different user agent without reusing the session cookie, which was not
done.
