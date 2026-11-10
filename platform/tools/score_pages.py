#!/usr/bin/env python3
"""Scorecard for the Streams server.

Measures catalog freshness, row quality, duplication, genre purity, language rendering,
tier filtering, hero/gate art & logo coverage, and endpoint latencies across fixed profile
sessions (adult_en, adult_ar, kids, history).
Outputs markdown report to notes/scorecards/<date>-<label>.md.
"""

import argparse
import datetime
import json
import os
import re
import statistics
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from typing import Any, Dict, List, Optional, Set, Tuple

DEFAULT_SERVER = os.environ.get("STREAMS_SERVER", "http://127.0.0.1:8092")
API_VERSION = "2026-11-05"
PAGE_CAP = 50

GENRES = [
    ("Action", "film.action", ["action"]),
    ("Comedy", "film.comedies", ["comedy", "comedies"]),
    ("Drama", "film.dramas", ["drama", "dramas"]),
    ("Sci-Fi", "film.sci-fi", ["sci-fi", "science fiction"]),
    ("Thriller", "film.thriller", ["thriller"]),
    ("Horror", "film.horror", ["horror"]),
]


class Client:
    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip("/")

    def request(
        self,
        path: str,
        method: str = "GET",
        data: Optional[Dict[str, Any]] = None,
        token: Optional[str] = None,
        timeout: float = 30.0,
    ) -> Tuple[Dict[str, Any], float]:
        headers = {
            "Api-Version": API_VERSION,
            "Accept": "application/json",
        }
        if token:
            headers["Authorization"] = f"Bearer {token}"
        body = None
        if data is not None:
            headers["Content-Type"] = "application/json"
            body = json.dumps(data).encode("utf-8")

        req = urllib.request.Request(
            f"{self.base_url}{path}", data=body, headers=headers, method=method
        )
        t0 = time.perf_counter()
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            elapsed_ms = (time.perf_counter() - t0) * 1000.0
            resp_body = json.loads(resp.read().decode("utf-8"))
            return resp_body, elapsed_ms


def bench_timing(client: Client, path: str, token: str, calls: int = 20) -> Tuple[float, float]:
    times: List[float] = []
    for _ in range(calls):
        try:
            _, ms = client.request(path, token=token)
            times.append(ms)
        except Exception:
            pass
    if not times:
        return 0.0, 0.0
    times.sort()
    med = statistics.median(times)
    idx95 = int(round(0.95 * (len(times) - 1)))
    p95 = times[idx95]
    return med, p95


def fetch_full_screen(client: Client, path: str, token: str) -> Dict[str, Any]:
    screen, _ = client.request(path, token=token)
    sections = list(screen.get("sections", []))
    cursor = screen.get("cursor")
    while cursor and len(sections) < PAGE_CAP:
        try:
            more, _ = client.request(f"/v1/more/{cursor}", token=token)
            more_sections = more.get("sections", [])
            if not more_sections:
                break
            sections.extend(more_sections)
            cursor = more.get("cursor")
        except Exception:
            break
    screen["sections"] = sections
    return screen


def score_page(
    screen: Dict[str, Any],
    genre_matchers: Optional[List[str]] = None,
    is_arabic: bool = False,
    is_kids: bool = False,
) -> Dict[str, Any]:
    sections = screen.get("sections", [])
    rows = [s for s in sections if s.get("kind") != "hero"]
    hero = next((s for s in sections if s.get("kind") == "hero"), None)

    row_titles: List[str] = []
    duplicate_row_names = 0
    seen_row_names: Set[str] = set()

    for r in rows:
        title = r.get("title") or r.get("name") or ""
        clean = title.strip("\u200f\u200e ").strip()
        if clean:
            row_titles.append(clean)
            if clean in seen_row_names:
                duplicate_row_names += 1
            seen_row_names.add(clean)

    # Tiles analysis
    all_tiles: List[Dict[str, Any]] = []
    for r in rows:
        all_tiles.extend(r.get("tiles", []))

    total_tiles = len(all_tiles)
    tile_ids = [t.get("title") for t in all_tiles if t.get("title")]
    unique_tile_ids = set(tile_ids)
    repeat_count = total_tiles - len(unique_tile_ids)
    repeat_pct = (repeat_count / total_tiles * 100.0) if total_tiles > 0 else 0.0

    # Genre purity (if genre page)
    genre_purity: Optional[float] = None
    if genre_matchers and total_tiles > 0:
        matching = 0
        matchers_lower = [m.lower() for m in genre_matchers]
        for t in all_tiles:
            tags = [tag.lower() for tag in t.get("tags", [])]
            if any(any(m in tag for m in matchers_lower) for tag in tags):
                matching += 1
        genre_purity = (matching / total_tiles) * 100.0

    # Hero art and logo coverage
    hero_tiles = hero.get("tiles", []) if hero else []
    hero_count = len(hero_tiles)
    hero_logo_pct = 0.0
    hero_art_pct = 0.0
    if hero_count > 0:
        logo_count = sum(
            1 for t in hero_tiles if any(img.get("role") == "logo" for img in t.get("images", []))
        )
        art_count = sum(
            1
            for t in hero_tiles
            if any(img.get("role") in ("art", "backdrop") for img in t.get("images", []))
        )
        hero_logo_pct = (logo_count / hero_count) * 100.0
        hero_art_pct = (art_count / hero_count) * 100.0

    # Language check (for Arabic)
    arabic_purity_pct: Optional[float] = None
    if is_arabic and row_titles:
        ar_regex = re.compile(r"[\u0600-\u06FF]")
        ar_matches = sum(1 for title in row_titles if ar_regex.search(title))
        arabic_purity_pct = (ar_matches / len(row_titles)) * 100.0

    # Tier leak checks for kids
    tier_leaks = 0
    if is_kids:
        # Check if any title contains adult keywords
        for t in all_tiles:
            tags = [tag.lower() for tag in t.get("tags", [])]
            if "horror" in tags or "erotic" in tags:
                tier_leaks += 1

    return {
        "rows_served": len(rows),
        "total_tiles": total_tiles,
        "unique_tiles": len(unique_tile_ids),
        "repeat_pct": repeat_pct,
        "duplicate_row_names": duplicate_row_names,
        "genre_purity": genre_purity,
        "hero_logo_pct": hero_logo_pct,
        "hero_art_pct": hero_art_pct,
        "arabic_purity_pct": arabic_purity_pct,
        "tier_leaks": tier_leaks,
        "tile_keys": unique_tile_ids,
    }


def main():
    parser = argparse.ArgumentParser(description="Streams Page Quality Scorecard")
    parser.add_argument("--server", default=DEFAULT_SERVER, help="Server base URL")
    parser.add_argument("--label", default="current", help="Scorecard label")
    parser.add_argument(
        "--out-dir",
        default="/Users/ibrahim/code/pstream/notes/scorecards",
        help="Output directory",
    )
    parser.add_argument(
        "--timing-calls", type=int, default=20, help="Timing benchmark call count"
    )
    args = parser.parse_args()

    client = Client(args.server)
    today_str = datetime.date.today().isoformat()
    now_ts = int(time.time())

    # 1. Verify /v1/ready
    try:
        ready, _ = client.request("/v1/ready")
        data_hash = ready.get("data_hash", "unknown")[:8]
    except Exception as e:
        print(f"Error connecting to server {args.server}: {e}", file=sys.stderr)
        sys.exit(1)

    # 2. Setup account
    signup_email = f"scorecard-{now_ts}@example.test"
    auth, _ = client.request(
        "/v1/auth/signup",
        method="POST",
        data={
            "email": signup_email,
            "password": "Password123!",
            "first_name": "Score",
            "last_name": "Card",
        },
    )
    token = auth["access_token"]

    profiles_resp, _ = client.request("/v1/profiles", token=token)
    profiles = profiles_resp.get("profiles", [])
    adult_profile = next(p for p in profiles if not p.get("kids"))
    kids_profile = next(p for p in profiles if p.get("kids"))

    # Create Arabic profile
    ar_profile_resp, _ = client.request(
        "/v1/profiles",
        method="POST",
        data={"name": "Arabic", "kids": False},
        token=token,
    )
    ar_pid = ar_profile_resp["id"]
    client.request(
        f"/v1/profiles/{ar_pid}",
        method="PATCH",
        data={"language": "ar"},
        token=token,
    )

    # Create History profile
    hist_profile_resp, _ = client.request(
        "/v1/profiles",
        method="POST",
        data={"name": "History", "kids": False},
        token=token,
    )
    hist_pid = hist_profile_resp["id"]

    # Enter adult profile to find titles for history
    client.request(f"/v1/profiles/{adult_profile['id']}/enter", method="POST", data={}, token=token)
    sample_home, _ = client.request("/v1/screens/home", token=token)
    history_titles = []
    for s in sample_home.get("sections", []):
        for t in s.get("tiles", []):
            if t.get("title"):
                history_titles.append(t["title"])
            if len(history_titles) >= 4:
                break
        if len(history_titles) >= 4:
            break

    # Seed history on hist_profile
    client.request(f"/v1/profiles/{hist_pid}/enter", method="POST", data={}, token=token)
    if history_titles:
        ops = [
            {"command": "list", "in_list": True, "title": history_titles[0], "op_id": "op-list-1"},
            {
                "command": "position",
                "position_s": 1200,
                "duration_s": 6000,
                "title": history_titles[1],
                "op_id": "op-pos-1",
            },
        ]
        client.request("/v1/commands", method="POST", data={"ops": ops}, token=token)

    # Evaluate gate
    gate_data, _ = client.request("/v1/profiles/gate", token=token)
    gate_titles = gate_data.get("titles", [])
    gate_count = len(gate_titles)
    gate_logo_pct = 0.0
    gate_art_pct = 0.0
    if gate_count > 0:
        g_logo = sum(
            1
            for gt in gate_titles
            if any(img.get("role") == "logo" for img in gt.get("tile", {}).get("images", []))
        )
        g_art = sum(
            1
            for gt in gate_titles
            if any(img.get("role") == "art" for img in gt.get("tile", {}).get("images", []))
        )
        gate_logo_pct = (g_logo / gate_count) * 100.0
        gate_art_pct = (g_art / gate_count) * 100.0

    # 3. Score Pages in Adult EN Session
    client.request(f"/v1/profiles/{adult_profile['id']}/enter", method="POST", data={}, token=token)

    pages_to_score = [
        ("Home", "/v1/screens/home", None),
        ("Series", "/v1/screens/series", None),
        ("Films", "/v1/screens/films", None),
    ]
    for genre_name, genre_id, matchers in GENRES:
        pages_to_score.append((f"Genre: {genre_name}", f"/v1/screens/genre?genre={genre_id}", matchers))

    results = []
    session_all_titles: Set[str] = set()
    total_screen_titles = 0

    for name, path, matchers in pages_to_score:
        med_ms, p95_ms = bench_timing(client, path, token, calls=args.timing_calls)
        screen = fetch_full_screen(client, path, token)
        res = score_page(screen, genre_matchers=matchers)
        res["name"] = name
        res["path"] = path
        res["med_ms"] = med_ms
        res["p95_ms"] = p95_ms
        results.append(res)
        session_all_titles.update(res["tile_keys"])
        total_screen_titles += len(res["tile_keys"])

    cross_page_repeats = total_screen_titles - len(session_all_titles)
    cross_page_repeat_pct = (
        (cross_page_repeats / total_screen_titles * 100.0) if total_screen_titles > 0 else 0.0
    )

    # 4. Score Kids Home
    client.request(f"/v1/profiles/{kids_profile['id']}/enter", method="POST", data={}, token=token)
    kids_med, kids_p95 = bench_timing(client, "/v1/screens/kids", token, calls=args.timing_calls)
    kids_screen = fetch_full_screen(client, "/v1/screens/kids", token)
    kids_res = score_page(kids_screen, is_kids=True)
    kids_res["name"] = "Kids Home"
    kids_res["path"] = "/v1/screens/kids"
    kids_res["med_ms"] = kids_med
    kids_res["p95_ms"] = kids_p95
    results.append(kids_res)

    # 5. Score Arabic Home
    client.request(f"/v1/profiles/{ar_pid}/enter", method="POST", data={}, token=token)
    ar_med, ar_p95 = bench_timing(client, "/v1/screens/home", token, calls=args.timing_calls)
    ar_screen = fetch_full_screen(client, "/v1/screens/home", token)
    ar_res = score_page(ar_screen, is_arabic=True)
    ar_res["name"] = "Arabic Home"
    ar_res["path"] = "/v1/screens/home"
    ar_res["med_ms"] = ar_med
    ar_res["p95_ms"] = ar_p95
    results.append(ar_res)

    # 6. Score History Home
    client.request(f"/v1/profiles/{hist_pid}/enter", method="POST", data={}, token=token)
    hist_med, hist_p95 = bench_timing(client, "/v1/screens/home", token, calls=args.timing_calls)
    hist_screen = fetch_full_screen(client, "/v1/screens/home", token)
    hist_res = score_page(hist_screen)
    hist_res["name"] = "History Home"
    hist_res["path"] = "/v1/screens/home"
    hist_res["med_ms"] = hist_med
    hist_res["p95_ms"] = hist_p95
    results.append(hist_res)

    # Build markdown report
    cmd_line = f"python3 platform/tools/score_pages.py --server {args.server} --label {args.label}"
    md = []
    md.append(f"# Catalog and Page Quality Scorecard: {args.label}")
    md.append("")
    md.append(f"* Date: `{today_str}`")
    md.append(f"* Data Hash: `{data_hash}`")
    md.append(f"* Server: `{args.server}`")
    md.append(f"* Command: `{cmd_line}`")
    md.append(f"* Gate Art Coverage: `{gate_art_pct:.1f}%` (Logo: `{gate_logo_pct:.1f}%`)")
    md.append(f"* Session Cross-Page Repeats: `{cross_page_repeat_pct:.1f}%` ({cross_page_repeats} repeated titles)")
    md.append("")
    md.append(
        "| Page | Rows | Tiles | Repeats | Dup Rows | Genre Purity | Tier Leaks | Lang Match | Med (ms) | p95 (ms) |"
    )
    md.append(
        "|---|---|---|---|---|---|---|---|---|---|"
    )

    for r in results:
        g_pure = f"{r['genre_purity']:.1f}%" if r["genre_purity"] is not None else "-"
        lang = f"{r['arabic_purity_pct']:.1f}%" if r["arabic_purity_pct"] is not None else "-"
        md.append(
            f"| {r['name']} | {r['rows_served']} | {r['total_tiles']} | {r['repeat_pct']:.1f}% | {r['duplicate_row_names']} | {g_pure} | {r['tier_leaks']} | {lang} | {r['med_ms']:.1f} | {r['p95_ms']:.1f} |"
        )

    md.append("")
    md.append("### Key Quality Indicators")
    md.append(f"- Tier leaks on Kids Home: `{kids_res['tier_leaks']}` (must be 0).")
    md.append(f"- Arabic localization on Arabic Home: `{ar_res['arabic_purity_pct']:.1f}%` translated rows.")
    md.append(f"- Hero logo coverage (Adult Home): `{results[0]['hero_logo_pct']:.1f}%`.")
    md.append(f"- Hero art/backdrop coverage (Adult Home): `{results[0]['hero_art_pct']:.1f}%`.")
    md.append("")

    report_content = "\n".join(md)
    os.makedirs(args.out_dir, exist_ok=True)
    out_file = os.path.join(args.out_dir, f"{today_str}-{args.label}.md")
    with open(out_file, "w", encoding="utf-8") as f:
        f.write(report_content)

    print(report_content)
    print(f"\nWrote scorecard to {out_file}")


if __name__ == "__main__":
    main()
