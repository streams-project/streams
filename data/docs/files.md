# Files in a built set

Printed by `databuild files`. A test keeps this file equal to the table in the code, and
the manifest stage refuses any file this table does not list.

| Role | File | Required | Writer | Inputs | Can a stranger rebuild it | When missing |
|---|---|---|---|---|---|---|
| catalog | `catalog.db` | yes | export | titles.db, groups.db, the build date | Yes, with their own TMDB key, or from the open sample | No set. The stage fails. |
| vectors | `vectors.bin` | yes | export | master rows from embed, the catalogue view | Yes, with their own TMDB key, or from the open sample | No set. The stage fails. |
| vector_keys | `vectors.keys` | yes | export | master keys from embed, the catalogue view | Yes, with their own TMDB key, or from the open sample | No set. The stage fails. |
| seeds | `seeds.bin` | yes | export | world level 2 groups, vectors, popularity, ratings | Yes, with their own TMDB key, or from the open sample | No set. The stage fails. |
| seed_names | `seeds.names.json` | yes | export | group names in seed order | Yes, with their own TMDB key, or from the open sample | No set. The stage fails. |
| seed_groups | `seeds.groups.json` | yes | export | group ids in seed order | Yes, with their own TMDB key, or from the open sample | No set. The stage fails. |
| row_requests | `rows.json` | yes | name | vocabulary/rows.json, events, titles, vectors | Yes, with their own TMDB key, or from the open sample | No set. The stage fails when no row has enough titles. |
| events | `events.json` | yes | name | vocabulary/events.json, vocabulary/calendar.json | Yes, from this repository alone | No set. The stage fails. |
| taxonomy_vectors | `taxonomy.bin` | yes | name | vocabulary/calendar.json, the embedding model | Yes, from this repository alone | No set. The stage fails. |
| taxonomy_entries | `taxonomy.json` | yes | name | vocabulary/calendar.json | Yes, from this repository alone | No set. The stage fails. |
| kids_taxonomy_vectors | `taxonomy_kids.bin` | yes | name | vocabulary/kids.json, the embedding model | Yes, from this repository alone | No set. The stage fails. |
| kids_taxonomy_entries | `taxonomy_kids.json` | yes | name | vocabulary/kids.json | Yes, from this repository alone | No set. The stage fails. |
| search_aliases | `search_aliases.json` | yes | export | vocabulary/search_aliases.json | Yes, from this repository alone | No set. The stage fails. |
| kids_channels | `kids_channels.json` | yes | export | vocabulary/kids_channels.json | Yes, from this repository alone | No set. The stage fails. A set that names no such file leaves the channel part of the children's title rule off, and the server says so. |
| removed_rows | `removed_rows.json` | yes | export | export.removed_rows, an operator list | Yes, from this repository alone | An empty list. |
| title_profiles | `profiles.jsonl` | no | name | titles.db, a text model | Yes, with a text model of their own | No written profiles. Age rules use certificates, and an unrated title is refused below 18. |
| categories | `categories.json` | no | name | vocabulary/categories.json, vectors, a text model | Yes, with a text model of their own | Category pages fall back to their keyword rules. |
| topics | `topics.json` | no | name | vocabulary/rows.json, vectors, a text model | Yes, with a text model of their own | No topic rows. |
| topics_meta | `topics_meta.json` | no | name | vocabulary/rows.json | Yes, with a text model of their own | No topic rows. |
| popularity | `popularity.db` | no | popularity | the daily TMDB export, catalog.db | Yes, the exports need no key | Popularity stays as built. |
| fresh | `fresh.json` | no | fresh | TMDB lists | Yes, with their own TMDB key, or from the open sample | No new or upcoming rows. |
| open_vectors | `open/vectors.bin` | no | export | a second vector set from open text | Yes, from this repository alone | No second set to score. |
| open_vector_keys | `open/vectors.keys` | no | export | a second vector set from open text | Yes, from this repository alone | No second set to score. |
| awards | `awards.json` | no | export | export.awards_file, an operator file; the old pipeline holds no writer | No, a private input | No award rows, no award lines. |
| skip_marks | `skip_marks.jsonl` | no | export | export.marks_file, an operator file from a video job | No, a private input | Skip buttons rely on the server's marks module. |
| content_reasons | `content_reasons.jsonl` | no | export | export.reasons_file, an operator file from a video job | No, a private input | The age card shows no reason words. |
| cards | `cards.tar` | no | sample | titles of the open sample | Yes, from this repository alone | The open sample has no images. |
| series_text | `tvmaze/series.jsonl` | no | sample | TVmaze text, CC BY-SA 4.0 | Yes, from this repository alone | The open sample has no series. |
| series_credit | `tvmaze/CREDIT.md` | no | sample | the sample builder | Yes, from this repository alone | The open sample has no series. |
| sources | `SOURCES.md` | no | sample | the sample builder | Yes, from this repository alone | No open sample. |

The nearest neighbour index is not in a set. The server builds it from a set and stamps
it with that set's manifest hash.

## Inputs that are not files of a set

| Input | Who holds it | What a build gives without it |
|---|---|---|
| Review vectors | The operator, a private input | No vibe groups and no mood pointers. Mood rows stay empty. |
| A text model | The operator | Names come from the data. No profiles, no judged pages, no topic rows. |
| An embedding model | The operator | The hashing back end proves function and knows nothing about meaning. |
| TMDB key | The operator | No full build. The open sample still builds. |
| Signing key | The operator | The set is unsigned, and the manifest stage says so. |

## What each old engine file became

| Old file | Now |
|---|---|
| the store database | catalog.db |
| the vector, key, seed, seed name and seed id files | vectors.bin and the files after it |
| the nearest neighbour index and its stamp | Left out. The server builds it. |
| calendar vocabulary, two files | taxonomy.bin, taxonomy.json |
| Kids vocabulary, two files | taxonomy_kids.bin, taxonomy_kids.json |
| vocabulary support cache | Left out. The server works it out when it starts. |
| row requests, first version | rows.json, rebuilt from own words |
| row requests, second version | Dropped. It carried names from captures. |
| capture tags | Dropped. Derived from a third party service. |
| capture genre signals | Dropped. Derived from a third party service. |
| capture page rhythm | Dropped. Derived from a third party service. |
| events | events.json |
| fresh titles | fresh.json |
| popularity overlay | popularity.db |
| title profiles | profiles.jsonl |
| judged categories | categories.json |
| judged topics and their names | topics.json, topics_meta.json, from own row words |
| the first topic list | Dropped. It came from a capture. Topics now come from own row words. |
| awards | awards.json, from an operator file |
| search aliases | search_aliases.json |
| blocked rows | removed_rows.json, from an operator list |
| skip marks | skip_marks.jsonl, from an operator file |
| content reasons | content_reasons.jsonl, from an operator file |
| feature switches and the games list | Left out. Live server settings own them. |
