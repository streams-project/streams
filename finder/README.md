# finder

The service that finds video. It answers one published contract, `POST /v1/find`, `GET /v1/find/{job}` and `GET /v1/health`, and runs as its own program, so a deployment can run several and swap any of them without touching the server.

`service` is everything a finder needs except its sources: the token check, the deadline on each provider, partial answers, one shared search per title, and rest for a provider that keeps failing.

`open-films` is a complete finder: it serves films the operator lists in a file, each with its licence. Any other title is a real not found.

| Setting | Meaning | If missing |
|---|---|---|
| `FINDER_BIND` | Address to listen on | Does not start, says so |
| `FINDER_SERVICE_TOKEN` | Token the server sends | Finding is off, says so |
| `FINDER_SERVICE_TOKEN_NEXT` | Second token during a rotation | One token |
| `FINDER_FILMS` | Path to the film list (`open-films`) | No films, health is degraded |

## Adding a source pack

A pack is its own program in its own repository, in `packs/<name>/` here. That folder is ignored by git, so what a pack contains never enters this repository. A pack implements `Provider` for each source and its whole `main` is one call:

```rust
#[tokio::main]
async fn main() -> std::process::ExitCode {
    finder_service::run(vec![Arc::new(MySource)]).await
}
```

Licensed under either of MIT or Apache 2.0, at your option.
