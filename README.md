# Open video platform

One server, one small core, and a real app for each device. Run it for yourself, or change one folder
and make it yours.

Status: the core, server, data builder, finder and website run together. Apps for other devices are next.

## Run it

    ./up

That is the whole setup on macOS or Linux. It needs only Rust ([rustup.rs](https://rustup.rs)). It builds
everything, fetches a pinned database, and starts the server, the finder and the website at
`http://127.0.0.1:8080`. Add `--public` to serve other devices on your network, `--port N` for another port.
A TV signs in with a code that a phone opens, so `--public` also gives the server this machine's network
address and prints it. Behind a tunnel or a proxy, name the address with `--site-address https://...`.

Everything it writes stays in `~/.<command>`. After the first run the program is `bin/<command>`:

    bin/<command> doctor          check this install
    bin/<command> backup          save the database while the service is stopped
    bin/<command> build web       rebuild the website

Until you build a catalogue with `data`, the site shows a generated sample with invented titles.

## Make it yours

`brand.json` holds the name, the command, the domain, the colours, the fonts, the button roundness and the
spacing. Change that one file and run `./up` again. Put your own marks in a folder under `brand/` and point
`assets` at it.

Video comes from finders. `finder/open-films` plays films you list with their licence in
`~/.<command>/films.json`. A source pack is a separate program in `packs/<name>/`, see `finder/README.md`.

## What is here

| Folder | What it is |
|---|---|
| `platform` | The contract, the core, the offline layer, the bridges and the test kit |
| `server` | The one server every app talks to |
| `finder` | The service that finds video, and a source that plays open films from a list |
| `data` | Builds and proves the catalogue |
| `apps` | One app per device |
| `brand` | Marks. Add your own folder for your own product |
| `launcher` | The one command |
| `tests` | The parts running together for real |

Run every check with `./check`.

The code is under MIT or Apache 2.0, at your option. The Streams name and marks are not, see
`TRADEMARKS.md`.
