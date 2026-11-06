"""Every file a built set may hold: its role, its writer, its inputs, and what a stranger gets.

This table is the sidecar table of H132. The manifest stage refuses a file that is not listed here,
so a new file cannot join a set without saying where it comes from. `docs/files.md` is printed from
this table by `databuild files`.
"""

from __future__ import annotations

from dataclasses import dataclass

MANIFEST_FILE = "manifest.json"
SIGNATURE_FILE = "manifest.sig"
EXPORT_RECORD = "export.json"

TMDB_KEY = "Yes, with their own TMDB key, or from the open sample"
ANYONE = "Yes, from this repository alone"
TEXT_MODEL = "Yes, with a text model of their own"
PRIVATE = "No, a private input"


@dataclass(frozen=True)
class FileSpec:
    role: str
    path: str
    required: bool
    writer: str
    inputs: str
    stranger: str
    missing: str


FILES: tuple[FileSpec, ...] = (
    FileSpec(
        "catalog",
        "catalog.db",
        True,
        "export",
        "titles.db, groups.db, the build date",
        TMDB_KEY,
        "No set. The stage fails.",
    ),
    FileSpec(
        "vectors",
        "vectors.bin",
        True,
        "export",
        "master rows from embed, the catalogue view",
        TMDB_KEY,
        "No set. The stage fails.",
    ),
    FileSpec(
        "vector_keys",
        "vectors.keys",
        True,
        "export",
        "master keys from embed, the catalogue view",
        TMDB_KEY,
        "No set. The stage fails.",
    ),
    FileSpec(
        "seeds",
        "seeds.bin",
        True,
        "export",
        "world level 2 groups, vectors, popularity, ratings",
        TMDB_KEY,
        "No set. The stage fails.",
    ),
    FileSpec(
        "seed_names",
        "seeds.names.json",
        True,
        "export",
        "group names in seed order",
        TMDB_KEY,
        "No set. The stage fails.",
    ),
    FileSpec(
        "seed_groups",
        "seeds.groups.json",
        True,
        "export",
        "group ids in seed order",
        TMDB_KEY,
        "No set. The stage fails.",
    ),
    FileSpec(
        "row_requests",
        "rows.json",
        True,
        "name",
        "vocabulary/rows.json, events, titles, vectors",
        TMDB_KEY,
        "No set. The stage fails when no row has enough titles.",
    ),
    FileSpec(
        "events",
        "events.json",
        True,
        "name",
        "vocabulary/events.json, vocabulary/calendar.json",
        ANYONE,
        "No set. The stage fails.",
    ),
    FileSpec(
        "taxonomy_vectors",
        "taxonomy.bin",
        True,
        "name",
        "vocabulary/calendar.json, the embedding model",
        ANYONE,
        "No set. The stage fails.",
    ),
    FileSpec(
        "taxonomy_entries",
        "taxonomy.json",
        True,
        "name",
        "vocabulary/calendar.json",
        ANYONE,
        "No set. The stage fails.",
    ),
    FileSpec(
        "kids_taxonomy_vectors",
        "taxonomy_kids.bin",
        True,
        "name",
        "vocabulary/kids.json, the embedding model",
        ANYONE,
        "No set. The stage fails.",
    ),
    FileSpec(
        "kids_taxonomy_entries",
        "taxonomy_kids.json",
        True,
        "name",
        "vocabulary/kids.json",
        ANYONE,
        "No set. The stage fails.",
    ),
    FileSpec(
        "search_aliases",
        "search_aliases.json",
        True,
        "export",
        "vocabulary/search_aliases.json",
        ANYONE,
        "No set. The stage fails.",
    ),
    FileSpec(
        "kids_channels",
        "kids_channels.json",
        True,
        "export",
        "vocabulary/kids_channels.json",
        ANYONE,
        "No set. The stage fails. A set that names no such file leaves the channel part of the "
        "children's title rule off, and the server says so.",
    ),
    FileSpec(
        "removed_rows",
        "removed_rows.json",
        True,
        "export",
        "export.removed_rows, an operator list",
        ANYONE,
        "An empty list.",
    ),
    FileSpec(
        "title_profiles",
        "profiles.jsonl",
        False,
        "name",
        "titles.db, a text model",
        TEXT_MODEL,
        "No written profiles. Age rules use certificates, and an unrated title is refused "
        "below 18.",
    ),
    FileSpec(
        "categories",
        "categories.json",
        False,
        "name",
        "vocabulary/categories.json, vectors, a text model",
        TEXT_MODEL,
        "Category pages fall back to their keyword rules.",
    ),
    FileSpec(
        "topics",
        "topics.json",
        False,
        "name",
        "vocabulary/rows.json, vectors, a text model",
        TEXT_MODEL,
        "No topic rows.",
    ),
    FileSpec(
        "topics_meta",
        "topics_meta.json",
        False,
        "name",
        "vocabulary/rows.json",
        TEXT_MODEL,
        "No topic rows.",
    ),
    FileSpec(
        "popularity",
        "popularity.db",
        False,
        "popularity",
        "the daily TMDB export, catalog.db",
        "Yes, the exports need no key",
        "Popularity stays as built.",
    ),
    FileSpec(
        "fresh",
        "fresh.json",
        False,
        "fresh",
        "TMDB lists",
        TMDB_KEY,
        "No new or upcoming rows.",
    ),
    FileSpec(
        "open_vectors",
        "open/vectors.bin",
        False,
        "export",
        "a second vector set from open text",
        ANYONE,
        "No second set to score.",
    ),
    FileSpec(
        "open_vector_keys",
        "open/vectors.keys",
        False,
        "export",
        "a second vector set from open text",
        ANYONE,
        "No second set to score.",
    ),
    FileSpec(
        "awards",
        "awards.json",
        False,
        "export",
        "export.awards_file, an operator file; the old pipeline holds no writer",
        PRIVATE,
        "No award rows, no award lines.",
    ),
    FileSpec(
        "skip_marks",
        "skip_marks.jsonl",
        False,
        "export",
        "export.marks_file, an operator file from a video job",
        PRIVATE,
        "Skip buttons rely on the server's marks module.",
    ),
    FileSpec(
        "content_reasons",
        "content_reasons.jsonl",
        False,
        "export",
        "export.reasons_file, an operator file from a video job",
        PRIVATE,
        "The age card shows no reason words.",
    ),
    FileSpec(
        "cards",
        "cards.tar",
        False,
        "sample",
        "titles of the open sample",
        ANYONE,
        "The open sample has no images.",
    ),
    FileSpec(
        "series_text",
        "tvmaze/series.jsonl",
        False,
        "sample",
        "TVmaze text, CC BY-SA 4.0",
        ANYONE,
        "The open sample has no series.",
    ),
    FileSpec(
        "series_credit",
        "tvmaze/CREDIT.md",
        False,
        "sample",
        "the sample builder",
        ANYONE,
        "The open sample has no series.",
    ),
    FileSpec(
        "sources",
        "SOURCES.md",
        False,
        "sample",
        "the sample builder",
        ANYONE,
        "No open sample.",
    ),
)

BY_ROLE = {spec.role: spec for spec in FILES}
BY_PATH = {spec.path: spec for spec in FILES}

# What each file of the old engine became. Names that cannot appear here are described in words.
OLD_FILES: tuple[tuple[str, str], ...] = (
    ("the store database", "catalog.db"),
    ("the vector, key, seed, seed name and seed id files", "vectors.bin and the files after it"),
    ("the nearest neighbour index and its stamp", "Left out. The server builds it (note 152)."),
    ("calendar vocabulary, two files", "taxonomy.bin, taxonomy.json"),
    ("Kids vocabulary, two files", "taxonomy_kids.bin, taxonomy_kids.json"),
    ("vocabulary support cache", "Left out. The server works it out when it starts."),
    ("row requests, first version", "rows.json, rebuilt from own words"),
    ("row requests, second version", "Dropped. It carried names from captures."),
    ("capture tags", "Dropped. Derived from a third party service."),
    ("capture genre signals", "Dropped. Derived from a third party service."),
    ("capture page rhythm", "Dropped. Derived from a third party service."),
    ("events", "events.json"),
    ("fresh titles", "fresh.json"),
    ("popularity overlay", "popularity.db"),
    ("title profiles", "profiles.jsonl"),
    ("judged categories", "categories.json"),
    ("judged topics and their names", "topics.json, topics_meta.json, from own row words"),
    (
        "the first topic list",
        "Dropped. It came from a capture. Topics now come from own row words.",
    ),
    ("awards", "awards.json, from an operator file"),
    ("search aliases", "search_aliases.json"),
    ("blocked rows", "removed_rows.json, from an operator list"),
    ("skip marks", "skip_marks.jsonl, from an operator file"),
    ("content reasons", "content_reasons.jsonl, from an operator file"),
    ("feature switches and the games list", "Left out. Live server settings own them."),
)


OTHER_INPUTS: tuple[tuple[str, str, str], ...] = (
    (
        "Review vectors",
        "The operator, a private input",
        "No vibe groups and no mood pointers. Mood rows stay empty.",
    ),
    (
        "A text model",
        "The operator",
        "Names come from the data. No profiles, no judged pages, no topic rows.",
    ),
    (
        "An embedding model",
        "The operator",
        "The hashing back end proves function and knows nothing about meaning.",
    ),
    ("TMDB key", "The operator", "No full build. The open sample still builds."),
    ("Signing key", "The operator", "The set is unsigned, and the manifest stage says so."),
)


def table() -> str:
    """The sidecar table as Markdown."""
    lines = [
        "# Files in a built set",
        "",
        "Printed by `databuild files`. A test keeps this file equal to the table in the code, and",
        "the manifest stage refuses any file this table does not list.",
        "",
        "| Role | File | Required | Writer | Inputs | Can a stranger rebuild it | When missing |",
        "|---|---|---|---|---|---|---|",
    ]
    for f in FILES:
        required = "yes" if f.required else "no"
        lines.append(
            f"| {f.role} | `{f.path}` | {required} | {f.writer} | {f.inputs} | {f.stranger} "
            f"| {f.missing} |"
        )
    lines += [
        "",
        "The nearest neighbour index is not in a set. The server builds it from a set and stamps",
        "it with that set's manifest hash.",
        "",
        "## Inputs that are not files of a set",
        "",
        "| Input | Who holds it | What a build gives without it |",
        "|---|---|---|",
        *[f"| {what} | {who} | {without} |" for what, who, without in OTHER_INPUTS],
        "",
        "## What each old engine file became",
        "",
        "| Old file | Now |",
        "|---|---|",
    ]
    lines += [f"| {old} | {new} |" for old, new in OLD_FILES]
    return "\n".join(lines) + "\n"
