#!/usr/bin/env python3
"""Pick titles per genre from the catalogue and fetch their art from TMDB.

Art is written outside the repository. Keep only titles with a textless backdrop and an
English title logo, because a tile is that backdrop with the logo set on it.

    TMDB_READ_TOKEN=... fetch_art.py --db streams_v3.db --out DIR
"""
import argparse, json, os, sqlite3, time, urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

API = "https://api.themoviedb.org/3"
IMG = "https://image.tmdb.org/t/p"
SKIP = "Talk|News|Reality|Soap|Documentary"
POOLS = {
    # name: (where clause, keep count)
    "action": ("(genres like '%Action%')", 34),
    "drama": ("(genres like '%Drama%' and genres not like '%Comedy%' and genres not like '%Horror%')", 34),
    "comedy": ("(genres like '%Comedy%' and genres not like '%Animation%' and genres not like '%Horror%')", 34),
    "thriller": ("(genres like '%Thriller%' or genres like '%Crime%' or genres like '%Mystery%')", 34),
    "kids": ("(genres like '%Animation%' and genres like '%Family%')", 34),
    "world": ("(original_language in ('ko','es','ja','fr','de','tr','hi','it','da','no','sv','pt'))", 34),
    "wide": ("(1=1)", 90),
}

def get(url, token):
    req = urllib.request.Request(url, headers={"Authorization": f"Bearer {token}", "Accept": "application/json"})
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=25) as r:
                return json.load(r)
        except Exception:
            time.sleep(1 + attempt)
    return None

def save(url, path):
    if path.exists() and path.stat().st_size > 2000:
        return True
    for attempt in range(4):
        try:
            with urllib.request.urlopen(url, timeout=40) as r:
                data = r.read()
            path.write_bytes(data)
            return True
        except Exception:
            time.sleep(1 + attempt)
    return False

def best(items, test):
    good = [i for i in items if test(i)]
    return sorted(good, key=lambda i: (i.get("vote_average", 0), i.get("vote_count", 0), i.get("width", 0)), reverse=True)[0] if good else None

def fetch(row, token, out):
    media, tid = row["media_type"], row["tmdb_id"]
    art = get(f"{API}/{media}/{tid}/images?include_image_language=en,null", token)
    if not art:
        return None
    back = best(art.get("backdrops", []), lambda b: b.get("iso_639_1") in (None, "xx", "") and b.get("width", 0) >= 1280 and 1.7 < b.get("aspect_ratio", 0) < 1.85)
    logo = best(art.get("logos", []), lambda l: l.get("iso_639_1") == "en" and l.get("file_path", "").endswith(".png") and 1.0 < l.get("aspect_ratio", 0) < 9)
    if not back or not logo:
        return None
    d = out / f"{media}-{tid}"
    d.mkdir(parents=True, exist_ok=True)
    ok = save(f"{IMG}/w1280{back['file_path']}", d / "backdrop.jpg") and save(f"{IMG}/w500{logo['file_path']}", d / "logo.png")
    if row["poster_path"]:
        ok = ok and save(f"{IMG}/w500{row['poster_path']}", d / "poster.jpg")
    return {"key": d.name, "title": row["title"], "year": row["release_year"], "logo_ratio": logo["aspect_ratio"]} if ok else None

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    token = os.environ["TMDB_READ_TOKEN"]
    out = Path(a.out)
    db = sqlite3.connect(f"file:{a.db}?mode=ro", uri=True)
    db.row_factory = sqlite3.Row
    used, picked = set(), {}
    with ThreadPoolExecutor(10) as pool:
        for name, (where, keep) in POOLS.items():
            q = f"""select tmdb_id, media_type, title, release_year, poster_path, vote_count from titles
                    where {where} and adult = 0 and backdrop_path is not null and poster_path is not null
                      and vote_count >= 300 and cast(release_year as int) >= 2008
                      and genres not like '%Talk%' and genres not like '%News%' and genres not like '%Reality%'
                      and genres not like '%Soap%' and genres not like '%Documentary%'
                    order by (vote_count * (1 + (cast(release_year as int) - 2008) * 0.12)) desc limit {keep * 3}"""
            rows = [dict(r) for r in db.execute(q)]
            rows = [r for r in rows if (r["media_type"], r["tmdb_id"]) not in used]
            got = []
            for res, r in zip(pool.map(lambda r: fetch(r, token, out), rows), rows):
                if res and len(got) < keep:
                    got.append(res)
                    used.add((r["media_type"], r["tmdb_id"]))
            picked[name] = got
            print(name, len(rows), "asked", len(got), "kept", flush=True)
    (out.parent / "picked.json").write_text(json.dumps(picked, indent=1))

main()
