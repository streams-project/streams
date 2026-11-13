# Netflix on the television, measured

The layout this project's television surfaces are held to. Every number here was measured off
1920x1080 frames of a third-party capture by reading pixel coordinates, not read from any
published specification and not estimated from an angled photograph. Only measurements live in
this file. The method, the frames, the sampling and the open questions are the project's private
note; this is the part the code cites, so it lives with the code.

Status: verified on that capture. The content differs from the project owner's own account -- it
is a France catalogue, so the tabs read `Shows`/`Movies` -- while the layout is the same.

A number here is a **reference pixel** at 1920x1080. Android lays out at half of it, which is the
same arithmetic an Xbox uses at 200% over a 1080p panel.

## How close a surface has to be

A television surface holds to this reference when every measured size is within **2 px** of the
number here. The measurement's own precision is +/-1 px of video compression, so 2 px is one
pixel of real drift and not a tolerance with room in it.

## Safe margin

**68 px left, 69 px right = 3.5% of the width.** Netflix is tighter than the classic 5% TV safe
area. Row titles start at x 73, the hero card at x 67, the avatar at x 68, the mark ends at
x 1851.

This project uses **72**, decided in note 188 and held by every native television surface. It is
inside Microsoft's 48 epx (96 reference px) TV-unsafe band for Xbox by 24 px, which is recorded
rather than silently corrected: 72 is a measurement-led decision, not a guess.

## Top bar

Content band y 44-96, **53 px tall**, vertical centre at 6.5% of the height. Avatar 48 px square
at x 68-115; caret at 136-146; search icon 29 px at 557-585; five tab labels starting at x 642,
780, 927, 1079, 1230 (gaps 138, 147, 152, 151); the mark 29 px at 1823-1851. The active tab is a
`#2D2D2D` pill about 140 px wide.

## Home, first screen

Hero card **1790 x 772 px** at x 67-1856, y 140-911 -- **93.2% of the width and 71.5% of the
height** -- with a thin `#45494C` border. Under it one row title at y 951-975 and **79 px of the
first row's tiles, 15.6% of a tile**. Rows fully visible: **0**. The hero autoplays a trailer in
place with subtitles burnt in.

## A row screen, exact

| | px | as a fraction |
|---|---|---|
| focused tile | **906 x 510**, aspect **1.776** = 16:9 | 47.2% w, 47.2% h |
| closed tile | **311 x 505**, aspect **0.616** | 16.2% w, 46.8% h |
| focused / closed width | **2.91x** | |
| gutter | **17 px** | 5.2% of pitch |
| pitch | **328 px** | |
| tiles across the screen if all closed | **5.85** | |
| visible with one focused | 1 expanded + 3 closed + 1 clipped | |
| focus outline | `#FBFFFA`, 3 px | |

The pitch is the closed tile plus the gutter: 311 + 17 = 328. It is a derivation, not a separate
measurement, and a surface that resizes the tile without moving the pitch leaves the wrong gap at
every tile in the row.

## Vertical rhythm of a row screen

Row title 151-176, gap 23, tiles 199-708, gap 25, facts line 733-755, gap 20, two lines of
synopsis 775-842, gap 106, next row title 948-978, next row tiles from 998 with **82 px visible**.
Row title to row title: **797 px**. Synopsis text is capped at **990 px, 51.6% of the width**.

## The focused row is pinned to a fixed band

The focus outline sits at **y 199-708** in **24 of 24** sampled frames that show a focused row
(t=12 s to t=96 s, 2 s apart, +/-1 px of video compression). Focus never walks down the screen:
the content scrolls so the focused row always lands in the same place.

This is the single most important navigation fact for a television surface, and it is not how a
web page scrolls.

## The artwork swap

A closed tile is a **311 x 505** portrait poster; the same tile focused is a **906 x 510** 16:9
landscape still at the same height. Two assets per title per row.

## Top 10 rows

The same **311 px** tile, with the gutter widened to **146 px** (pitch **457**) so a large
outlined rank numeral can sit between tiles. Same tile, different pitch.

## Episodes screen

Two columns. Left: the mark, `SERIES`, the show logo, a facts line, a season list where each row
is `name` + `N episodes` with the chosen one on a grey band, and `Trailers & More` last. Right:
episode cards, each a **459 x 261 (16:9)** still with the episode label burnt in bottom left, then
title, synopsis and duration beside it.
