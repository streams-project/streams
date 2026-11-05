# finder

The source finder service. It answers one published contract and runs on its own.

Part of an open video platform. Status: the `demo` crate runs and is tested; provider modules are not here.

## Demo finder

`demo` serves open films only, from a list the operator writes. Each entry names a title key, an http or https address, `hls` or `file`, and its licence. Entries with no licence are refused.

| Setting | Meaning | If missing |
|---|---|---|
| `FINDER_BIND` | Address to listen on | Does not start, says so |
| `FINDER_SERVICE_TOKEN` | Token the server sends | Finding is off, says so |
| `FINDER_SERVICE_TOKEN_NEXT` | Second token during a rotation | One token |
| `FINDER_DEMO_FILMS` | Path to the film list | No films, health is degraded |

Licensed under either of MIT or Apache 2.0, at your option.
