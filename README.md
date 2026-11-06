# Open video platform

One server, one small core, and a real app for each device. Run it for yourself, or change one folder
and make it yours.

Status: groundwork. The core, server and data builder are built and tested. The apps are not built yet.

| Folder | What it is |
|---|---|
| `platform` | The contract, the core, the offline layer, the bridges and the test kit |
| `server` | The one server every app talks to |
| `finder` | The service that finds video, and a source that plays open films from a list |
| `data` | Builds and proves the catalogue |
| `apps` | One app per device |
| `brand` | Name and marks. Add your own folder for your own product |
| `tests` | The parts running together for real |

Run every check with `./check`.

The code is under MIT or Apache 2.0, at your option. The Streams name and marks are not, see
`TRADEMARKS.md`.
