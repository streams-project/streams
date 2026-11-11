# tv

One TV app for Samsung, LG, Fire TV on Vega, Titan OS, Vizio and Hisense. Each is a web app on that
system, so a new brand of television needs no new app.

Status: runs, work stopped on 2026-11-07. `page/` is the app, served at `/tv/`: 2,185 lines of plain
ES5 in 26 files with no build step, running the core from `core.wasm`, which `./up` builds and
publishes beside them. It signs in with a code a phone approves, then draws profiles, rows, a title
and the film. `tests/tv/press_through.py` drives all of that by key presses through a real browser,
which `./check` does not do. The root README's status table carries this line for every platform.

| File | What it does |
|---|---|
| `page/host.js` | Steps the core and performs what it asks |
| `page/ports.js` | The one way out of a surface: keys, network, secrets, player, clips |
| `page/adapters/` | What each port really does on a television |
| `page/surfaces/` | The screens: the door, the code, profiles, browse, a title, the film |

A brand puts `opening.mp4`, `showreel.mp4` and `peek.jpg` in its `media/` folder. The page runs without them.

Licensed under either of MIT or Apache 2.0, at your option.
