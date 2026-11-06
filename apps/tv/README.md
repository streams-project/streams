# tv

One TV app for Samsung, LG, Fire TV on Vega, Titan OS, Vizio and Hisense. Each is a web app on that
system, so a new brand of television needs no new app.

Status: in progress. `page/` is the app, served at `/tv/`: plain ES5 files with no build step, running
the core from `core.wasm`, which `./up` builds and publishes beside them. It signs in with a code a
phone approves, then draws profiles, rows, a title and the film.

| File | What it does |
|---|---|
| `page/host.js` | Steps the core and performs what it asks |
| `page/tv.js` | The signed out door and the keys |
| `page/app.js` | Everything past the door |
| `page/qr.js` | A small QR code encoder |

A brand puts `opening.mp4`, `showreel.mp4` and `peek.jpg` in its `media/` folder. The page runs without them.

Licensed under either of MIT or Apache 2.0, at your option.
