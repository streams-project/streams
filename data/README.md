# data

Builds the catalogue from nothing and proves it: pull, clean, embed, group, name, export, manifest.
It ships code and row vocabulary. It ships no built data, because each deployer builds their own.

Part of an open video platform. Licensed under either of MIT or Apache 2.0, at your option.

## Run

Python 3.13 and uv. Dependencies are pinned in `uv.lock`.

    cp config/settings.example.toml databuild.toml
    uv run --locked databuild --help

An empty setting turns its feature off, and the stage that needs it says which one. Key settings
hold a path to a file, never a key.

    ./scripts/check                 what must pass before each commit
    databuild sample                the open sample, from nothing
    databuild pull                  title details from TMDB
    databuild all                   clean, embed, group, name, export, manifest, verify
    databuild verify                check a set against its manifest
    databuild age                   fail when a set or a daily file is too old
    databuild keygen PRIVATE PUBLIC make a signing key pair
    databuild files                 print the table of files in a set
    databuild scan .                refuse brand words, addresses and keys

## The stages

| Stage | Reads | Writes |
|---|---|---|
| pull | TMDB, one key per deployer | `pull.db` |
| clean | `pull.db` | `titles.db`, with rows older than six months left out |
| embed | `titles.db`, or a file of open text | master rows of int8 values and a scale |
| group | master rows, optional review vectors | `groups.db`, `groups.npz` |
| name | groups, own vocabulary, optional text model | group names, rows, vocabulary vectors |
| export | everything above | the files a server reads |
| manifest | the output folder | `manifest.json`, `manifest.sig` |

`docs/files.md` lists every file of a set, what writes it, whether a stranger can rebuild it, and
what a build gives without it. The manifest stage refuses any file that table does not name.

## The open sample

`databuild sample` builds about 1,000 titles with no key and no private input. Wikidata (CC0) gives
film facts and the TMDB ids that key each title. TVmaze (CC BY-SA 4.0) gives series text, kept in
its own folder with credit. Cards are generated. It needs `open.wikidata_endpoint`,
`open.tvmaze_api_base` and `open.user_agent`.

The sample proves function. Open sources hold no ratings, so every title takes one neutral rating,
and with no certificates every title is refused below 18. Search and row quality on this text are
not measured.

## Scheduled jobs

`jobs/` holds systemd user units. New titles run every six hours at minute 23, and popularity runs
daily. Each job ends with `verify` and `age`, so a stale or wrong set fails the unit, and a failed
unit starts `jobs/alarm`. `~/.config/databuild/env` sets `DATABUILD_HOME`, `DATABUILD_SETTINGS` and
`ALARM_COMMAND`. A full rebuild is run by hand on a GPU machine.

## What needs the contract

See `docs/contract-request.md`.
