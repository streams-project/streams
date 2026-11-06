# tv

One TV app for Samsung, LG, Fire TV on Vega, Titan OS, Vizio and Hisense (VIDAA, now HomeOS). Each
is a web app on that system, so a new brand of television needs no new app.

Part of an open video platform. Status: groundwork. The signed out opening
scene and landing below run as a live page; nothing else runs yet.

Licensed under either of MIT or Apache 2.0, at your option.

## Signed out opening and landing

A generator that takes a folder of title images and a list of headlines and
makes:

1. The opening scene: the wordmark rests, collapses into the mark as it slides
   right and loses its letters, the mark rests, then stretches and folds into a
   column that bursts to the full width and parts into a violet family and a
   blue family, with thin white, pink and cyan lines, running outward from a
   dark gap at the centre on near black, then a fade and a ring on black.
   Both marks come from `brand/`; `landing/config.json` names them.
2. The landing: a rounded glass card slides in, a tilted wall of tiles, then a
   cycle of headlines on tilted tile layers that zoom about the card centre
   with blur on the way out and in, and the next card peeking at the right.
   There is no price and no plans page.

It runs as one WebGL 1 canvas, compiled to ES5 for the oldest supported sets.
A frame that is the same as the last one is not drawn.

### Run it

Needs Node 24 or newer (tested on 26.13.0). Frames, video and measurement need
a Chrome binary in `CHROME_PATH`.

```sh
npm ci
npm run landing -- serve            # live page at http://127.0.0.1:8090/
npm run landing -- frames --out preview          # key frames as PNG
npm run landing -- frames --out preview --fps 24 --from 1750 --to 4800
npm run landing -- video --out preview.webm --seconds 45 --fps 30
npm run landing -- measure --seconds 50 [--throttle 6]
npm run landing -- phases                        # measure every phase, writes test/phases.json
npm test && npm run check
```

Inputs, all optional:

| Option | Default | What |
|---|---|---|
| `--images DIR` | `landing/cards` | Title images: jpg, png, webp or svg. Up to 60 are used. |
| `--headlines FILE` | `landing/headlines.txt` | One headline per line; a bar splits it into two lines. |
| `--config FILE` | `landing/config.json` | The two marks, colours, buttons, type, render size, audio slot. |
| `--dist DIR` | `dist/landing` | Where the built page goes. |

`landing/cards` holds generated sample cards only (`npm run landing -- cards`
writes them again from the seed). No title art is kept here.

The audio slot is `audio.src` in the config. It is empty: no sound ships yet.
A path there, relative to the config, is copied in at build and starts with
the opening.

The font is Inter (SIL Open Font License), installed from npm at build time and
copied into `dist` with its licence; it is not stored in this repository.

The two marks are in `brand/`: the Streams wordmark and the stream play glyph.
See `brand/README.md`.

### Where the numbers come from

Timing, easing, blur, scale, card geometry, glass edge, stripe speeds, tile
grid and headline slots are frame by frame readings of a 24 fps recording of a
reference TV app on a real TV. Durations are good to plus or minus 42 ms. The
reference material is kept outside this repository. Values the readings could
not settle are marked `assumed` in `src/landing/timeline.ts`, and our own
design choices are marked `own`. The marks, colours, sample cards, headlines
and the face the headlines are set in are our own.

`test/timeline.test.ts` checks the timeline against the readings.

### Measuring the sequence against the readings

`npm run landing -- phases` renders the sequence one frame at a time at 24 fps,
reads each frame back off the canvas and reduces it to column and row profiles,
the card's own rows and a few means. Each phase boundary is then found in those
numbers alone: nothing in `tools/phases.ts` reads the timeline. It writes
`test/phases.json`, with a hash of the source it was made from.
`test/phases.test.ts` fails when any phase is more than one frame, 41.7 ms, from
its reading, and when the file was made from a different source than the one in
the tree, so a change to the timing has to be measured again.

Measured 2026-11-05, 1,000 frames, headless Chrome 158.0.8127.18 beta:

| Phase | Measured | Reading |
|---|---|---|
| wordmark rest | 108 f, 4,500 ms | 108 f |
| wordmark collapses to the mark | 28 f, 1,167 ms | 28 f |
| mark rest | 258 f, 10,750 ms | 258 f |
| mark stretches and folds into a column | 23 f, 958 ms | 23 f |
| column widens, 0.105 to 0.305 W | 6 f, 250 ms | 6 f |
| burst to full width | 3 f, 125 ms | 3 f |
| full width, one colour | 3 f, 125 ms | 3 f |
| colour split | 1 f, 42 ms | 1 f |
| stripes run outward from a dark centre gap | 15 f, 625 ms | 15 f |
| fade to black | 6 f, 250 ms | 6 f |
| black with ring | 10 f, 417 ms | 10 f |
| card slides in from the right | 8 f, 333 ms | 8 f |
| empty card rest | 8 f, 333 ms | 8 f |
| wall fades in | 3 f, 125 ms | 3 f |
| wall rest | 43 f, 1,792 ms | 44 f |
| wall out while the wordmark fades in | 15 f, 625 ms | 14 f |
| wordmark leaves and the first tile layer settles | 43 f, 1,792 ms | 42 f |
| first headline fades in | 9 f, 375 ms | 8 f |
| headline 1 to 2 | 15 f, 625 ms | 15 f |
| headline 2 to 3 | 15 f, 625 ms | 15 f |
| right peek fades in | 14 f, 583 ms | 14 f |
| headline 3 to 4, with the first zoom | 38 f, 1,583 ms | 38 f |
| one change every 70 frames | 70 f, 2,917 ms | 70 f |

Nineteen land on the reading exactly and four are one frame out. The four come
from where a phase is called finished: the reading itself numbers a rest and
the move after it one frame apart in places, and the line of a two line
headline that lags by three frames puts the whole arrival at nine.

### Measured frame time (2026-11-05)

`npm run landing -- measure --seconds 50`, headless Chrome 158.0.8127.18 beta,
ANGLE Metal on Intel UHD Graphics 630, Intel i7-9750H, canvas 1920x1080. One
warm up pass, then 50 s of real time animation from the first frame.

| What | Reading |
|---|---|
| Frame intervals | 1,501, median 33.3 ms, p99 33.4 ms, max 33.5 ms |
| Intervals over 34 ms | 0 |
| JS draw, opening | median under 0.05 ms, max 0.1 ms |
| JS draw, landing | median 0.1 ms, max 0.3 ms |
| Frames drawn, frames skipped as unchanged | 965, 537 |
| GPU per frame over the 41 key frames | median 1.16 ms, worst 1.88 ms |
| Startup after the mark shows | images 150 ms, compose 18 ms, texture upload 60 ms |

Headless Chrome drove the page at 30 Hz in this run; an earlier one on the same
machine ran at 60 Hz and gave the same draw costs at 16.7 ms intervals, so the
interval is the browser's, not the page's. GPU per frame: each key frame is
drawn 120 times, then the GPU is waited on once. The throttled and 1280x720
runs were not repeated for this version. Not yet measured on a TV.
