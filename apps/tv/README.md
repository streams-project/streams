# tv

One TypeScript TV app for Samsung, LG and Fire TV on Vega.

Part of an open video platform. Status: groundwork. The signed out opening
scene and landing below run as a live page; nothing else runs yet.

Licensed under either of MIT or Apache 2.0, at your option.

## Signed out opening and landing

A generator that takes a folder of title images and a list of headlines and
makes:

1. The opening scene: the logo from `landing/config.json` stretches, folds into
   a column and opens into violet and blue stripes with thin white, pink and
   cyan lines, on near black, then a ring on black.
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
npm run landing -- measure --seconds 45 [--throttle 6]
npm test && npm run check
```

Inputs, all optional:

| Option | Default | What |
|---|---|---|
| `--images DIR` | `landing/cards` | Title images: jpg, png, webp or svg. Up to 60 are used. |
| `--headlines FILE` | `landing/headlines.txt` | One headline per line; a bar splits it into two lines. |
| `--config FILE` | `landing/config.json` | Logo shapes, colours, buttons, font, render size, audio slot. |
| `--dist DIR` | `dist/landing` | Where the built page goes. |

`landing/cards` holds generated sample cards only (`npm run landing -- cards`
writes them again from the seed). No title art is kept here.

The audio slot is `audio.src` in the config. It is empty: no sound ships yet.
A path there, relative to the config, is copied in at build and starts with
the opening.

The font is Inter (SIL Open Font License), installed from npm at build time and
copied into `dist` with its licence; it is not stored in this repository.

### Where the numbers come from

Timing, easing, blur, scale, card geometry, glass edge and headline slots are
frame by frame readings of a 24 fps recording of a reference TV app on a real
TV. Durations are good to plus or minus 42 ms. The reference material is kept
outside this repository. Values the readings could not settle are marked
`assumed` in `src/landing/timeline.ts`, and our own design choices are marked
`own`. The logo, colours, sample cards, headlines and tile sizes are our own.
`test/timeline.test.ts` checks the timeline against the readings.

### Measured frame time (2026-11-05)

`npm run landing -- measure --seconds 45`, headless Chrome 158.0.8127.18 beta,
ANGLE Metal on Intel UHD Graphics 630, Intel i7-9750H, canvas 1920x1080. One
warm up pass, then 45 s of real time animation from the start of the sequence.

| Run | Frame intervals | Over 17.5 ms | JS draw, median and max | GPU per frame, median and worst |
|---|---|---|---|---|
| 1920x1080 | 2,701, median 16.7 ms, max 16.8 ms | 0 | 0.1 ms, 0.3 ms | 1.23 ms, 2.07 ms |
| 1920x1080, CPU slowed 6 times | 2,558, median 16.7 ms, p95 16.8 ms, p99 50.0 ms, max 166.7 ms | 58 | 0.1 ms, 127 ms | 1.69 ms, 2.45 ms |
| 1280x720 | 2,701, median 16.7 ms, max 16.8 ms | 0 | 0.1 ms, 0.2 ms | 0.73 ms, 1.26 ms |

GPU per frame: each of the 37 key frames is drawn 120 times, then the GPU is
waited on once. Startup after the logo shows: images 128 to 167 ms, compose 21
to 78 ms, texture upload 55 to 89 ms. Not yet measured on a TV.
