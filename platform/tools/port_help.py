#!/usr/bin/env python3
"""Port the help centre structure and translations into one JSON file per language."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOC_PREFIX = "doc:"

KEY_RE = re.compile(
    r"^(help-topic-[a-z0-9-]+|"
    r"help-group-[a-z0-9-]+|"
    r"help-art-[a-z0-9-]+-(?:title|summary|keywords|body)|"
    r"legal-[a-z0-9-]+-(?:title|updated|\d+-(?:title|body|b\d+)))\s*=\s*(.*)$"
)
BRAND_VAR_RE = re.compile(r"\{\s*\$brand\s*\}")
ANY_VAR_RE = re.compile(r"\{\s*\$([a-zA-Z0-9_-]+)\s*\}")
STRINGS_RE = re.compile(r'"([^"]+)"')


def read_brand_name(root: Path = ROOT) -> str:
    data = json.loads((root / "brand.json").read_text("utf-8"))
    return data["name"]


def read_banned_words(root: Path = ROOT) -> list[str]:
    clean_rs = root / "server" / "crates" / "server" / "tests" / "clean.rs"
    text = clean_rs.read_text("utf-8")
    block = text.split("fn banned()", 1)[1].split("}", 1)[0]
    pairs = re.findall(r'\[\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\]', block)
    return [a + b for a, b in pairs]


def find_default_source(root: Path = ROOT) -> Path:
    found = sorted(root.parent.glob("*/*frontend*/src/data/help.rs"))
    if not found:
        raise FileNotFoundError("could not locate help.rs under parent directory")
    return found[0].parents[2]


def parse_help_rs(text: str) -> dict:
    popular_m = re.search(r"pub const POPULAR:\s*&\[&str\]\s*=\s*&\[(.*?)\];", text, re.S)
    popular = STRINGS_RE.findall(popular_m.group(1)) if popular_m else []

    legal = []
    for m in re.finditer(
        r'LegalDoc\s*\{\s*id:\s*"([^"]+)",\s*path:\s*"[^"]+",\s*sections:\s*(\d+)\s*\}',
        text,
    ):
        legal.append({"id": m.group(1), "sections": int(m.group(2))})

    topics = []
    topics_m = re.search(r"pub const TOPICS:\s*&\[Topic\]\s*=\s*&\[(.*?)\n\];", text, re.S)
    if topics_m:
        for tm in re.finditer(
            r'Topic\s*\{\s*id:\s*"([^"]+)",.*?groups:\s*&\[(.*?)\],\s*\}',
            topics_m.group(1),
            re.S,
        ):
            groups = []
            for gm in re.finditer(
                r'Group\s*\{\s*id:\s*"([^"]+)",\s*articles:\s*&\[(.*?)\]\s*\}',
                tm.group(2),
                re.S,
            ):
                groups.append({"id": gm.group(1), "articles": STRINGS_RE.findall(gm.group(2))})
            topics.append({"id": tm.group(1), "groups": groups})

    articles = []
    articles_m = re.search(r"pub const ARTICLES:\s*&\[Article\]\s*=\s*&\[(.*?)\n\];", text, re.S)
    if articles_m:
        for am in re.finditer(
            r'Article\s*\{\s*slug:\s*"([^"]+)",\s*related:\s*&\[(.*?)\]\s*\}',
            articles_m.group(1),
            re.S,
        ):
            articles.append({"slug": am.group(1), "related": STRINGS_RE.findall(am.group(2))})

    return {"topics": topics, "popular": popular, "articles": articles, "policies": legal}


def parse_ftl(text: str) -> dict[str, str]:
    entries: dict[str, str] = {}
    cur_key: str | None = None
    cur_lines: list[str] = []

    def flush() -> None:
        nonlocal cur_key, cur_lines
        if cur_key is not None:
            while cur_lines and cur_lines[-1] == "":
                cur_lines.pop()
            entries[cur_key] = "\n".join(cur_lines)
            cur_key = None
            cur_lines = []

    for raw in text.splitlines():
        if raw == "" or raw.startswith(" ") or raw.startswith("\t"):
            if cur_key is not None:
                if raw == "":
                    cur_lines.append("")
                elif raw.startswith("    "):
                    cur_lines.append(raw[4:])
                else:
                    cur_lines.append(raw.lstrip())
            continue
        flush()
        m = KEY_RE.match(raw)
        if m:
            cur_key = m.group(1)
            first = m.group(2)
            cur_lines = [first] if first != "" else []

    flush()
    return entries


def convert_value(
    file_name: str,
    key: str,
    value: str,
    brand: str,
    banned: list[str],
    allow_exact_brand: bool = True,
) -> tuple[str, list[str]]:
    errors: list[str] = []
    out = BRAND_VAR_RE.sub("{brand}", value)

    seen_vars: set[str] = set()
    for vm in ANY_VAR_RE.finditer(out):
        var_name = vm.group(1)
        if var_name not in seen_vars:
            seen_vars.add(var_name)
            errors.append(f"{file_name}: {key}: unknown variable ${var_name}")

    low = out.lower()
    for word in banned:
        if word in low:
            errors.append(f"{file_name}: {key}: third party word {word}")

    if brand:
        if allow_exact_brand and brand in out:
            out = out.replace(brand, "{brand}")
        if brand.lower() in out.lower():
            errors.append(f"{file_name}: {key}: unsafe literal brand word {brand}")

    return out, errors


def build_lang(
    tree: dict,
    entries: dict[str, str],
    file_name: str,
    brand: str,
    banned: list[str],
    en_entries: dict[str, str] | None = None,
) -> tuple[dict, list[str]]:
    cleaned: dict[str, str] = {}
    errors: list[str] = []
    for k, v in sorted(entries.items()):
        en_has_exact = True if en_entries is None else (brand in en_entries.get(k, ""))
        conv, errs = convert_value(file_name, k, v, brand, banned, allow_exact_brand=en_has_exact)
        cleaned[k] = conv
        errors.extend(errs)

    policy_ids = {p["id"] for p in tree["policies"]}
    for t in tree["topics"]:
        for g in t["groups"]:
            for entry in g["articles"]:
                if entry.startswith(DOC_PREFIX):
                    pid = entry[len(DOC_PREFIX) :]
                    if pid not in policy_ids:
                        policy_ids.add(pid)
                        tree["policies"].append({"id": pid, "sections": 25})

    topics_out = []
    for t in tree["topics"]:
        tid = t["id"]
        groups_out = []
        for g in t["groups"]:
            gid = g["id"]
            g_obj: dict = {"id": gid, "articles": list(g["articles"])}
            g_title = cleaned.get(f"help-group-{gid}")
            if g_title is not None:
                g_obj["title"] = g_title
            groups_out.append(g_obj)
        t_obj: dict = {"id": tid, "groups": groups_out}
        t_title = cleaned.get(f"help-topic-{tid}")
        if t_title is not None:
            t_obj["title"] = t_title
        topics_out.append(t_obj)

    articles_out: dict[str, dict] = {}
    for a in tree["articles"]:
        slug = a["slug"]
        if slug.startswith(DOC_PREFIX):
            continue
        a_obj: dict = {}
        for part in ("title", "summary", "keywords", "body"):
            val = cleaned.get(f"help-art-{slug}-{part}")
            if val is not None:
                a_obj[part] = val
        if a_obj:
            a_obj["related"] = list(a["related"])
            articles_out[slug] = a_obj

    policies_out: dict[str, dict] = {}
    for p in tree["policies"]:
        pid = p["id"]
        p_obj: dict = {}
        title = cleaned.get(f"legal-{pid}-title")
        if title is not None:
            p_obj["title"] = title
        updated = cleaned.get(f"legal-{pid}-updated")
        if updated is not None:
            p_obj["updated"] = updated
        sections_out = []
        for n in range(1, p["sections"] + 1):
            s_title = cleaned.get(f"legal-{pid}-{n}-title")
            s_body = cleaned.get(f"legal-{pid}-{n}-body")
            bullets = []
            k = 1
            while f"legal-{pid}-{n}-b{k}" in cleaned:
                bullets.append(cleaned[f"legal-{pid}-{n}-b{k}"])
                k += 1
            if s_title is not None or s_body is not None or bullets:
                sec: dict = {}
                if s_title is not None:
                    sec["title"] = s_title
                if s_body is not None:
                    sec["body"] = s_body
                sec["bullets"] = bullets
                sections_out.append(sec)
        if p_obj or sections_out:
            p_obj["sections"] = sections_out
            policies_out[pid] = p_obj

    result = {
        "topics": topics_out,
        "popular": list(tree["popular"]),
        "articles": articles_out,
        "policies": policies_out,
    }
    return result, errors


def port_all(source_dir: Path, out_dir: Path, brand: str, banned: list[str]) -> list[str]:
    help_rs = (source_dir / "src" / "data" / "help.rs").read_text("utf-8")
    tree = parse_help_rs(help_rs)
    i18n_dir = source_dir / "i18n"
    ftl_files = sorted(i18n_dir.glob("*.ftl"))
    if not ftl_files:
        return [f"no .ftl files in {i18n_dir}"]

    en_path = i18n_dir / "en.ftl"
    en_entries = parse_ftl(en_path.read_text("utf-8")) if en_path.exists() else None

    all_errors: list[str] = []
    outputs: dict[str, dict] = {}
    for ftl_path in ftl_files:
        entries = parse_ftl(ftl_path.read_text("utf-8"))
        data, errs = build_lang(
            json.loads(json.dumps(tree)),
            entries,
            ftl_path.name,
            brand,
            banned,
            en_entries=en_entries,
        )
        all_errors.extend(errs)
        outputs[ftl_path.stem] = data

    if all_errors:
        return all_errors

    out_dir.mkdir(parents=True, exist_ok=True)
    for lang, data in outputs.items():
        text = json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
        (out_dir / f"{lang}.json").write_text(text, "utf-8")
    return []


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=None)
    parser.add_argument("--out", type=Path, default=ROOT / "help")
    args = parser.parse_args(argv)

    source = args.source or find_default_source(ROOT)
    brand = read_brand_name(ROOT)
    banned = read_banned_words(ROOT)
    errors = port_all(source, args.out, brand, banned)
    if errors:
        for line in errors:
            print(line)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
