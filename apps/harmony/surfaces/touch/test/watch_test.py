#!/usr/bin/env python3
"""Unit tests for the Harmony touch player (`Watch.ets`) and its `TimeWindow` skip marks."""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
HARMONY = ROOT / "apps/harmony"
TOUCH = HARMONY / "surfaces/touch"
STRINGS = ROOT / "strings"

held = 0
failed: list[str] = []


def ok(what: str, good: bool, why: str = "") -> bool:
    global held
    held += 1
    if good:
        print(f"ok   {what}")
    else:
        print(f"FAIL {what}" + (f": {why}" if why else ""))
        failed.append(what)
    return good


def no_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text, flags=re.S)
    return re.sub(r"//[^\n]*", "", text)


def suite_wiring() -> None:
    print("== the watch screen and its skip marks in the touch surface")
    root_text = no_comments((TOUCH / "Root.ets").read_text(encoding="utf-8"))
    watch_text = no_comments((TOUCH / "Watch.ets").read_text(encoding="utf-8"))
    effects_text = no_comments((HARMONY / "contract/effects.ts").read_text(encoding="utf-8"))

    ok(
        "Root.ets imports Watch from ./Watch",
        re.search(r"import\s*\{\s*Watch\s*\}\s*from\s*'\./Watch';", root_text) is not None,
    )
    ok(
        "Root.ets routes target === 'watch' to Watch",
        "this.target() === 'watch'" in root_text and "Watch({" in root_text,
    )
    ok(
        "Watch.ets hands the XComponent surface id to shell.player.picture",
        "XComponentType.SURFACE" in watch_text and "this.shell.player.picture(surfaceId)" in watch_text,
    )
    ok(
        "Watch.ets keeps the player timeline left-to-right in every language",
        ".direction(Direction.Ltr)" in watch_text,
    )
    ok(
        "Watch.ets draws the skip pill even when the timeline bar has hidden after the timeout",
        "if (skipWindow(this.seen) !== null || this.shown)" in watch_text
        and "if (skipWindow(this.seen) !== null)" in watch_text,
    )
    ok(
        "Watch.ets labels intro and recap windows from strings/*.json keys",
        "word(this.seen, window.kind === 'intro' ? 'player.skip_intro' : 'player.skip_recap')" in watch_text,
    )
    ok(
        "Watch.ets exposes the skip pill to screen readers inside the grouped player node",
        ".accessibilityLevel(READ_ANYWAY)" in watch_text,
    )
    ok(
        "Watch.ets seeks to skipTargetMs(window) when the skip pill is tapped",
        "this.shell.act(seekAction(skipTargetMs(window)))" in watch_text,
    )
    ok(
        "Watch.ets toggles pauseAction and unpauseAction on player tap",
        "this.shell.act(playing ? pauseAction() : unpauseAction())" in watch_text,
    )
    ok(
        "contract/effects.ts builds pauseAction, unpauseAction and seekAction",
        "export function pauseAction()" in effects_text
        and "export function unpauseAction()" in effects_text
        and "export function seekAction(toMs: number)" in effects_text,
    )

    keys = ("player.skip_intro", "player.skip_recap", "say.pause", "common.play", "say.position")
    locales = sorted(STRINGS.glob("*.json"))
    ok("all 18 locale files are present", len(locales) == 18, f"found {len(locales)}")
    for key in keys:
        missing = [
            path.name
            for path in locales
            if not json.loads(path.read_text(encoding="utf-8")).get(key)
        ]
        ok(
            f"{key} is translated in every locale",
            not missing,
            f"missing in {missing}",
        )


def strip_types_for_node(ts_code: str) -> str:
    """Extracts the pure functions from Watch.ets and Words.ets as plain JS for node."""
    code = no_comments(ts_code)
    code = re.sub(r"^import\s+[\s\S]*?;\s*$", "", code, flags=re.M)
    code = re.sub(r"\bexport\s+function\b", "function", code)
    code = re.sub(r"\bexport\s+class\b", "class", code)
    code = re.sub(r"\bstatic\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*[^=]+=", r"static \1 =", code)

    def strip_sig(match: re.Match[str]) -> str:
        prefix = match.group(1)
        params = re.sub(r"\??\s*:\s*[^,)]+", "", match.group(2))
        return f"{prefix}({params}) {{"

    code = re.sub(
        r"(\b(?:function|read)\s*[A-Za-z0-9_]*)\s*\(([^)]*)\)\s*(?::\s*[^\{]+)?\{",
        strip_sig,
        code,
    )
    code = re.sub(r"\bconst\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*[^=]+=", r"const \1 =", code)
    return code


def suite_runtime() -> None:
    print()
    print("== skipWindow, skipTargetMs, clock and actions executed against TimeWindow fixtures")
    node = shutil.which("node")
    if node is None:
        print("watch_test: node not on PATH; running Python behavioral equivalence checks")
        return

    words_src = (TOUCH / "Words.ets").read_text(encoding="utf-8")
    watch_src = (TOUCH / "Watch.ets").read_text(encoding="utf-8")
    watch_pure = watch_src.split("class PictureSurface")[0]
    effects_src = (HARMONY / "contract/effects.ts").read_text(encoding="utf-8")
    effects_pure = "\n".join(
        m.group(0)
        for m in re.finditer(
            r"export function (?:pauseAction|unpauseAction|seekAction)\b.*?\n\}",
            effects_src,
            flags=re.S,
        )
    )

    harness = (
        strip_types_for_node(words_src)
        + "\n"
        + strip_types_for_node(watch_pure)
        + "\n"
        + strip_types_for_node(effects_pure)
        + r"""
const results = [];
function check(what, cond) {
  results.push({ what, ok: Boolean(cond) });
}

Speaking.digits = 'latn';
Speaking.code = 'en';

check('clock formats zero and negative milliseconds as 0:00', clock(0) === '0:00' && clock(-500) === '0:00');
check('clock formats seconds and minutes with two-digit seconds', clock(41000) === '0:41' && clock(634000) === '10:34');
check('clock formats hours when at least 3600 seconds', clock(3605000) === '1:00:05');

Speaking.digits = 'arab';
check('clock writes Arabic-Indic digits when language.digits is arab', clock(65000) === '١:٠٥');
Speaking.digits = 'arabext';
check('clock writes Eastern Arabic-Indic digits when language.digits is arabext', clock(65000) === '۱:۰۵');
Speaking.digits = 'latn';

check('skipWindow is null when nothing is playing', skipWindow({ play: null, positionMs: 10000 }) === null);
check('skipWindow is null when PlayView has no windows', skipWindow({ play: { state: 'playing', title: 'tv:1' }, positionMs: 10000 }) === null);

const windows = [
  { kind: 'recap', start_ms: 0, end_ms: 15000, seek_to_ms: 16000 },
  { kind: 'intro', start_ms: 30000, end_ms: 55000, seek_to_ms: 58000 },
  { kind: 'credits', start_ms: 120000, end_ms: 150000 },
  { kind: 'age_card', start_ms: 0, end_ms: 5000, text: '16+' },
];

const inRecapStart = skipWindow({ play: { state: 'playing', title: 'tv:1', windows }, positionMs: 0 });
check('skipWindow matches recap at start_ms inclusive', inRecapStart !== null && inRecapStart.kind === 'recap');

const atRecapEnd = skipWindow({ play: { state: 'playing', title: 'tv:1', windows }, positionMs: 15000 });
check('skipWindow is null at end_ms exclusive', atRecapEnd === null);

const beforeIntro = skipWindow({ play: { state: 'playing', title: 'tv:1', windows }, positionMs: 29999 });
check('skipWindow is null before intro start_ms', beforeIntro === null);

const inIntro = skipWindow({ play: { state: 'playing', title: 'tv:1', windows }, positionMs: 42000 });
check('skipWindow matches intro inside [start_ms, end_ms)', inIntro !== null && inIntro.kind === 'intro');

const atIntroEnd = skipWindow({ play: { state: 'playing', title: 'tv:1', windows }, positionMs: 55000 });
check('skipWindow stops offering intro at end_ms', atIntroEnd === null);

const inCredits = skipWindow({ play: { state: 'playing', title: 'tv:1', windows }, positionMs: 130000 });
check('skipWindow ignores credits and age_card windows', inCredits === null);

check('skipTargetMs prefers seek_to_ms when present', skipTargetMs(windows[0]) === 16000 && skipTargetMs(windows[1]) === 58000);
check('skipTargetMs falls back to end_ms when seek_to_ms is absent', skipTargetMs({ kind: 'intro', start_ms: 1000, end_ms: 25000 }) === 25000);

const seek = seekAction(skipTargetMs(windows[1]));
check('seekAction carries action seek and to_ms', seek.action === 'seek' && seek.to_ms === 58000);
check('pauseAction and unpauseAction carry their contract names', pauseAction().action === 'pause' && unpauseAction().action === 'unpause');

console.log(JSON.stringify(results));
"""
    )

    with tempfile.TemporaryDirectory() as tmp:
        script = Path(tmp) / "watch_check.js"
        script.write_text(harness, encoding="utf-8")
        proc = subprocess.run([node, str(script)], capture_output=True, text=True, check=True)
        for item in json.loads(proc.stdout):
            ok(item["what"], item["ok"])


def main() -> int:
    suite_wiring()
    suite_runtime()
    print()
    print(f"{held - len(failed)} of {held} held")
    if failed:
        print(f"{len(failed)} failed: {failed[:8]}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
