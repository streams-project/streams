# finder

The service that finds video. It answers one published contract, `POST /v1/find`, `GET /v1/find/{job}` and `GET /v1/health`, and runs as its own program, so a deployment can run several and swap any of them without touching the server.

`service` is everything a finder needs except its sources: the token check, the deadline on each provider, partial answers, one shared search per title, and rest for a provider that keeps failing.

`open-films` is a complete finder: it serves films the operator lists in a file, each with its licence. Any other title is a real not found.

## Not found, and nothing to look with

These are two different answers and the service keeps them apart.

A provider that was asked and held nothing is a not found: a two hundred, an empty source list, and the server turns it into a four hundred and four, which a device draws as a dead end. That is the right answer for a title an operator does not carry.

A finder with nothing able to look has not searched at all. It answers five hundred and three with a line saying why, and the server reads that as no finder and retries. Two cases reach it: a finder started with no providers, which is what `open-films` becomes when its list is missing or refused, and every provider resting after five failures each. Saying not found here told every viewer that nothing in the catalogue exists.

A job handle is read by its own rule. It names a search that already ran, so `GET /v1/find/{job}` still answers with whatever that search found, however the sources are doing now.

## More than one address for a film

One address for a whole catalogue is one outage away from nothing playing. A film list may hold as many rows for a title as the operator has addresses for it, and every one of them is answered; the server ranks them and a device falls over from one to the next. The same address listed twice is not a second address and the list is refused. An address that is not `https` is refused too: the server drops those before they reach a device, so a list that took them passed its own check and then played nothing.

| Setting | Meaning | If missing |
|---|---|---|
| `FINDER_BIND` | Address to listen on | Does not start, says so |
| `FINDER_SERVICE_TOKEN` | Token the server sends | Finding is off, says so |
| `FINDER_SERVICE_TOKEN_NEXT` | Second token during a rotation | One token |
| `FINDER_FILMS` | Path to the film list (`open-films`) | No films: health is degraded and a search answers five hundred and three, not a not found |

## Adding a source pack

A pack is its own program in its own repository, in `packs/<name>/` here. That folder is ignored by git, so what a pack contains never enters this repository. A pack implements `Provider` for each source and its whole `main` is one call:

```rust
#[tokio::main]
async fn main() -> std::process::ExitCode {
    finder_service::run(vec![Arc::new(MySource)]).await
}
```

Licensed under either of MIT or Apache 2.0, at your option.
