# brand

The two marks the signed out opening scene draws. Both are our own; they ship
under this repository's licence (MIT OR Apache-2.0, see `REUSE.toml`).

| File | What | Source |
|---|---|---|
| `mark.svg` | Stream play glyph: a blue gradient arrow with a salmon ribbon behind it. 1024 square. | The systemic icon set `source-icon.svg` (set D53). Copied byte for byte. |
| `wordmark.svg` | The Streams wordmark, white, exact outlines. | Sofia Sans Extra Condensed 800, tracking -0.008. SIL Open Font License, `OFL-SofiaSans.txt`. Chosen by the owner 2026-11-06. |
| `lockup-on-dark.svg` | Mark and word together. | Mark height 1.42 cap heights, gap 0.20 cap heights. |

The icon set carries no licence file of its own, so there was none to carry
over. Only these two files were taken: the rasterised favicons, touch icons,
tiles and the small and mono drawings are not needed here, and the earlier
pixel cube mark is not used anywhere. Correction 2026-11-06: that was wrong, `wordmark.svg` carries the cube. It is to be removed; the owner dropped that mark.

The opening uses them as two separate things, as the motion reading does: the
wordmark rests first and collapses into the mark, and the mark then stretches
and folds into the column of stripes. Both are loaded as SVG and drawn into one
square texture with `preserveAspectRatio="none"`, so the renderer sets the shape
from its own measured box rather than from the file.
