#!/usr/bin/env python3
"""Mutation pass over the core and the offline layer.

Each mutant is one small change to a rule: a constant moved, a guard dropped, a
condition inverted. A mutant is killed when the tests notice it and survives
when they do not. A survivor names a rule no test holds.

A change that leaves behaviour the same cannot be killed by any test. Such a
mutant carries a written reason, is reported apart from the score, and fails the
run if a test does see it after all.

The pass never touches the working tree: it copies the tracked and untracked
files into a work folder, mutates the copy, and runs the tests there with one
shared target folder.

  python3 tools/mutate.py list
      Prints the catalogue, one line per mutant.

  python3 tools/mutate.py check
      Checks that every mutant's text is in its file exactly once, and that no
      mutant is a no-op. Reads no cargo.

  python3 tools/mutate.py run [--only M05,M14] [--work DIR] [--keep]
      Runs the baseline, then every mutant, and prints the score.

Exit status of `run` is 0 when every mutant was killed, 1 otherwise.
"""

import argparse
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The tests a mutant has to get past. These are the behaviour tests of the core
# and the layer, plus the scripted sessions, which run with the layer on and off.
TEST_COMMANDS = [
    ["cargo", "test", "--locked", "--offline", "-p", "core", "-p", "offline"],
    ["cargo", "test", "--locked", "--offline", "-p", "kit-core", "--test", "sessions"],
]

# Folders never copied into the work folder.
SKIP_DIRS = {".git", "target", "dist", "node_modules"}

PLAYER = "core/src/player.rs"
NET = "core/src/net.rs"
ROUTES = "core/src/routes.rs"
LIB = "core/src/lib.rs"
SESSION = "core/src/session.rs"
PAIRING = "core/src/pairing.rs"
CONFIG = "core/src/config.rs"
GATE = "core/src/gate.rs"
LAYER = "offline/src/lib.rs"
QUEUE = "offline/src/queue.rs"
CACHE = "offline/src/cache.rs"


def mutant(number, file, about, find, replace, equivalent=None):
    """One mutant. `equivalent` is a written reason why no test can see this change, for a
    change that leaves behaviour the same. Such a mutant is expected to survive and is
    counted apart from the score. A reason that turns out wrong fails the run."""
    return {
        "id": f"M{number:02d}",
        "file": file,
        "about": about,
        "find": find,
        "replace": replace,
        "equivalent": equivalent,
    }


MUTANTS = [
    mutant(
        1,
        PLAYER,
        "the heartbeat moves from 30 s to 5 s",
        "pub const HEARTBEAT_MS: u64 = 30_000;",
        "pub const HEARTBEAT_MS: u64 = 5_000;",
    ),
    mutant(
        2,
        PLAYER,
        "every position is worth saving, 5 s floor gone",
        "const WORTH_SAVING_MS: u64 = 5_000;",
        "const WORTH_SAVING_MS: u64 = 0;",
    ),
    mutant(
        3,
        PLAYER,
        "the no picture rule needs no time in, 2 s to 0",
        "const NO_PICTURE_AFTER_MS: u64 = 2_000;",
        "const NO_PICTURE_AFTER_MS: u64 = 0;",
    ),
    mutant(
        4,
        PLAYER,
        "no picture by the clock, 3 s to 0.5 s",
        "const NO_PICTURE_WALL_MS: u64 = 3_000;",
        "const NO_PICTURE_WALL_MS: u64 = 500;",
    ),
    mutant(
        5,
        PLAYER,
        "no picture by the video, 2.5 s to 0.5 s",
        "const NO_PICTURE_MEDIA_MS: u64 = 2_500;",
        "const NO_PICTURE_MEDIA_MS: u64 = 500;",
    ),
    mutant(
        6,
        PLAYER,
        "the age card stays up 1 s instead of 5 s",
        "const AGE_CARD_MS: u64 = 5_000;",
        "const AGE_CARD_MS: u64 = 1_000;",
    ),
    mutant(
        7,
        PLAYER,
        "a skip button hides at the end it moves to, not 1 s early",
        "const SKIP_HIDES_EARLY_MS: u64 = 1_000;",
        "const SKIP_HIDES_EARLY_MS: u64 = 0;",
    ),
    mutant(
        8,
        PLAYER,
        "the credits offer hides at the end, not 2 s early",
        "const CREDITS_HIDES_EARLY_MS: u64 = 2_000;",
        "const CREDITS_HIDES_EARLY_MS: u64 = 0;",
    ),
    # `.to_string()` in place of the lower casing does not compile: the core is no_std and
    # player.rs imports String but not ToString, so a mutant that read it would prove nothing.
    mutant(
        9,
        PLAYER,
        "host compare becomes case sensitive",
        """    url.split_once("://")
        .and_then(|(_, rest)| rest.split('/').next())
        .unwrap_or("")
        .to_ascii_lowercase()""",
        """    String::from(
        url.split_once("://")
            .and_then(|(_, rest)| rest.split('/').next())
            .unwrap_or(""),
    )""",
    ),
    mutant(
        10,
        PLAYER,
        "failover no longer needs another host",
        "        host_of(&next.url) != bad_host && !same_group_stream",
        "        !same_group_stream",
    ),
    mutant(
        11,
        PLAYER,
        "failover no longer skips other streams of the failed group",
        "        host_of(&next.url) != bad_host && !same_group_stream",
        "        host_of(&next.url) != bad_host",
    ),
    mutant(
        12,
        PLAYER,
        "failover walks from the first attempt again, so it wraps around",
        "    (failed + 1..attempts.len()).find(|&index| {",
        "    (0..attempts.len()).find(|&index| {",
    ),
    mutant(
        13,
        PLAYER,
        "a new source opens at the start, not at the kept position",
        "            start_at_ms: play.position_ms,",
        "            start_at_ms: 0,",
    ),
    mutant(
        14,
        PLAYER,
        "a failover while paused no longer re-sends Pause",
        """        if paused {
            tick.driver.push(DriverCommand::Pause);
        }
        tick.view = true;""",
        "        tick.view = true;",
    ),
    mutant(
        15,
        PLAYER,
        "a new source opens before the old one closes",
        """    ) {
        self.close(tick);
        self.generation += 1;""",
        """    ) {
        self.generation += 1;""",
    ),
    mutant(
        16,
        PLAYER,
        "a driver event from an older generation is taken",
        "        if generation != self.generation || !self.open {",
        "        if generation > self.generation || !self.open {",
    ),
    mutant(
        17,
        PLAYER,
        "a stall while paused is acted on",
        "            if play.frame_seen && !play.paused {",
        "            if play.frame_seen {",
    ),
    mutant(
        18,
        PLAYER,
        "the end no longer saves at the duration",
        """            DriverEvent::Ended => {
                play.position_ms = play.position_ms.max(play.duration_ms);""",
        "            DriverEvent::Ended => {",
    ),
    mutant(
        19,
        PLAYER,
        "autoplay is read the wrong way round",
        "                if play.autoplay_next {",
        "                if !play.autoplay_next {",
    ),
    mutant(
        20,
        PLAYER,
        "every frame counts as the first of the play",
        "        let first_of_play = !self.started;",
        "        let first_of_play = true;",
    ),
    mutant(
        21,
        PLAYER,
        "a position before the first frame replaces the kept one",
        """        if !self.frame_seen {
            return;
        }
""",
        "",
    ),
    mutant(
        22,
        PLAYER,
        "a stall never recovers when time moves again",
        """        if self.state == PlayState::Stalled && time_ms > self.position_ms {
            self.state = PlayState::Playing;
            self.deadline_mono = None;
            self.heartbeat_mono = Some(mono_ms + HEARTBEAT_MS);
            tick.view = true;
        }
""",
        "",
    ),
    mutant(
        23,
        PLAYER,
        "skip marks are asked for a film too",
        "        if self.marks_asked || self.episode.is_none() {",
        "        if self.marks_asked {",
    ),
    mutant(
        24,
        PLAYER,
        "skip marks are asked again on every position",
        "        if self.marks_asked || self.episode.is_none() {",
        "        if self.episode.is_none() {",
    ),
    mutant(
        25,
        PLAYER,
        "the credits offer shows on the last episode only",
        "        WindowKind::Credits if last_episode => return None,",
        "        WindowKind::Credits if !last_episode => return None,",
    ),
    mutant(
        26,
        PLAYER,
        "a window with nothing left to show is still shown",
        "    (window.start_ms < window.end_ms).then_some(window)",
        "    Some(window)",
    ),
    mutant(
        27,
        PLAYER,
        "a seek is not held to the video's length",
        """        let to_ms = if play.duration_ms > 0 {
            to_ms.min(play.duration_ms)
        } else {
            to_ms
        };
""",
        "",
    ),
    mutant(
        28,
        PLAYER,
        "a seek no longer starts the picture check again",
        """        play.position_ms = to_ms;
        play.no_picture_since = None;""",
        "        play.position_ms = to_ms;",
    ),
    mutant(
        29,
        PLAYER,
        "a pause saves nothing",
        """            tick.view = true;
        }
        save(play, tick);
    }

    pub(crate) fn unpause""",
        """            tick.view = true;
        }
    }

    pub(crate) fn unpause""",
    ),
    mutant(
        30,
        PLAYER,
        "a position is saved before any frame was seen",
        "    if play.started && play.position_ms > WORTH_SAVING_MS {",
        "    if play.position_ms > WORTH_SAVING_MS {",
    ),
    mutant(
        31,
        NET,
        "the request deadline moves from 60 s to 30 s",
        "pub const REQUEST_DEADLINE_MS: u32 = 60_000;",
        "pub const REQUEST_DEADLINE_MS: u32 = 30_000;",
    ),
    mutant(
        32,
        NET,
        "the first backoff wait is 1 s, not 2 s",
        "const BACKOFF_FIRST_MS: u64 = 2_000;",
        "const BACKOFF_FIRST_MS: u64 = 1_000;",
    ),
    mutant(
        33,
        NET,
        "a budget wait is no longer held to 30 s at the bottom",
        "const BUDGET_MIN_S: u32 = 30;",
        "const BUDGET_MIN_S: u32 = 1;",
    ),
    mutant(
        34,
        ROUTES,
        "next may start with two slashes, so another host can be named",
        """
        && !next.starts_with("//")""",
        "",
    ),
    mutant(
        35,
        ROUTES,
        "next may hold a backslash",
        """
        && !next.contains('\\\\')""",
        "",
    ),
    mutant(
        36,
        ROUTES,
        "next may hold control characters and spaces",
        """
        && !next.chars().any(|c| c.is_control() || c.is_whitespace())""",
        "",
    ),
    mutant(
        37,
        ROUTES,
        "next need not start with a slash, so it need not be local",
        "    let local = next.starts_with('/')",
        "    let local = true",
    ),
    mutant(
        38,
        ROUTES,
        "a watch address with only an episode is taken, season assumed",
        """    let season = query_value(query, &["season", "s"])?.parse().ok()?;""",
        """    let season = query_value(query, &["season", "s"])
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);""",
    ),
    mutant(
        39,
        ROUTES,
        "home counts as open to visitors, so it is no safe next",
        "        matches!(self, Self::Start | Self::SignIn | Self::Title(_))",
        "        matches!(self, Self::Start | Self::SignIn | Self::Title(_) | Self::Home)",
    ),
    mutant(
        40,
        ROUTES,
        "a trailing slash is kept, so the moved table misses the row",
        """
    let path = path.trim_end_matches('/');""",
        "",
    ),
    mutant(
        41,
        ROUTES,
        "the moved table becomes case sensitive",
        "        .find(|row| row.from.eq_ignore_ascii_case(path))",
        "        .find(|row| row.from == path)",
    ),
    mutant(
        42,
        ROUTES,
        "a pending title is read without decoding",
        """    TitleKey::parse(&decode(query_value(query, &["title"])?)?).ok()""",
        """    TitleKey::parse(query_value(query, &["title"])?).ok()""",
    ),
    mutant(
        43,
        ROUTES,
        "next is never followed, so a member stays on sign in",
        "    decide_from(address, who, true)",
        "    decide_from(address, who, false)",
    ),
    mutant(
        44,
        LIB,
        "a cancelled refresh is not answered, so a waiting request hangs",
        """                Purpose::Hello
                    | Purpose::Refresh""",
        "                Purpose::Hello",
    ),
    mutant(
        45,
        LIB,
        "a cancelled hello is not answered, so no retry is armed",
        """                Purpose::Hello
                    | Purpose::Refresh""",
        "                Purpose::Refresh",
    ),
    mutant(
        46,
        LIB,
        "an unsupported secret store is asked again and again",
        """        if let Some(wait) = self.secrets.remove(id) {
            self.secret_store = false;""",
        "        if let Some(wait) = self.secrets.remove(id) {",
    ),
    mutant(
        47,
        LIB,
        "a request near expiry goes out instead of waiting for the refresher",
        "        if request.takes_token() && self.session.must_wait(now) {",
        "        if request.takes_token() && self.session.refreshing() {",
    ),
    mutant(
        48,
        LIB,
        "a prepared request may name any path, not only /v1/",
        '                if let Some(id) = id.filter(|_| path.starts_with("/v1/")) {',
        "                if let Some(id) = id {",
    ),
    mutant(
        49,
        LIB,
        "the session ends with the gate state left behind",
        """        self.net.forget_session();
        self.gate.clear();""",
        "        self.net.forget_session();",
    ),
    mutant(
        50,
        LIB,
        "the title waiting in the address is dropped after the profile pick",
        """            Some(route) => {
                home.title = route.title;""",
        "            Some(route) => {",
    ),
    mutant(
        51,
        LIB,
        "a second 401 refreshes again instead of ending the session",
        "        if retried || !self.session.has_refresh() {",
        "        if !self.session.has_refresh() {",
    ),
    mutant(
        52,
        LIB,
        "entering a profile no longer stops what plays",
        """                let mut tick = Tick::default();
                self.player.stop(&mut tick);
                self.run_tick(now, tick, out);
                if let Ok(body) = serde_json::to_string(&ProfileEnter { pin }) {""",
        """                let mut tick = Tick::default();
                self.run_tick(now, tick, out);
                if let Ok(body) = serde_json::to_string(&ProfileEnter { pin }) {""",
    ),
    mutant(
        53,
        LIB,
        "a command goes out with no profile in use",
        "        if !self.session.signed_in() || self.session.profile().is_none() {",
        "        if !self.session.signed_in() {",
    ),
    mutant(
        54,
        LIB,
        "the last activity is written on every step, not at most every 30 s",
        """        if self.gate.take_moved() {
            self.keep_session(out);
        }""",
        "        self.keep_session(out);",
    ),
    mutant(
        55,
        LIB,
        "a hello with no ttl is read again at once",
        "        self.hello_next_mono = (ttl_ms > 0).then_some(now.mono_ms + ttl_ms);",
        "        self.hello_next_mono = Some(now.mono_ms + ttl_ms);",
    ),
    mutant(
        56,
        LIB,
        "facts are settled while the kept session is still unknown",
        """    fn settle(&mut self, now: Now, out: &mut Out) {
        if self.session.state() == SessionState::Starting {
            return;
        }
""",
        "    fn settle(&mut self, now: Now, out: &mut Out) {\n",
        equivalent=(
            "open_address holds the same rule, so with this guard gone settle still draws a "
            'wait. The only state left over is that `asked` becomes "/" instead of nothing, '
            'and every reader of `asked` falls back to "/" anyway.'
        ),
    ),
    mutant(
        57,
        SESSION,
        "renewal starts 60 s before expiry, not 120 s",
        "const RENEW_MARGIN_MS: u64 = 120_000;",
        "const RENEW_MARGIN_MS: u64 = 60_000;",
    ),
    mutant(
        58,
        SESSION,
        "a missed refresh is tried again after 60 s, not 300 s",
        "const CHECK_EVERY_MS: u64 = 300_000;",
        "const CHECK_EVERY_MS: u64 = 60_000;",
    ),
    mutant(
        59,
        CONFIG,
        "the fallback start deadline moves from 15 s to 5 s",
        "pub const DEFAULT_START_DEADLINE_MS: u32 = 15_000;",
        "pub const DEFAULT_START_DEADLINE_MS: u32 = 5_000;",
    ),
    mutant(
        60,
        GATE,
        "activity counts at most every 10 s, not every 30 s",
        "const WRITE_EVERY_MS: u64 = 30_000;",
        "const WRITE_EVERY_MS: u64 = 10_000;",
    ),
    mutant(
        61,
        LAYER,
        "a sign in by another account keeps what the last one left",
        """        let other_account =
            self.owner.is_some() && session.user_id.is_some() && session.user_id != self.owner;""",
        "        let other_account = false;",
    ),
    mutant(
        62,
        LAYER,
        "entering a scope that holds ops does not flush them",
        """            if self.queue.has_ops(&scope) {
                self.queue.flush_now(mono_ms);
            }
""",
        "",
    ),
    mutant(
        63,
        LAYER,
        "coming back pulls nothing",
        """                EventBody::Resume | EventBody::Visibility { visible: true } => {
                    if let Some(scope) = self.scope.clone() {
                        self.pull(&scope, events);
                    }
                    events.push(event);
                }""",
        """                EventBody::Resume | EventBody::Visibility { visible: true } => {
                    events.push(event);
                }""",
    ),
    mutant(
        64,
        LAYER,
        "a kept page is read again without its etag",
        """fn fetch_headers(etag: Option<String>, headers: &mut BTreeMap<String, String>) {
    if let Some(etag) = etag {
        headers.insert(String::from("If-None-Match"), etag);
    }
}""",
        "fn fetch_headers(_etag: Option<String>, _headers: &mut BTreeMap<String, String>) {}",
    ),
    mutant(
        65,
        LAYER,
        "a hidden page does not flush at once",
        """                EventBody::Suspend | EventBody::Visibility { visible: false } => {
                    *flush_after = true;""",
        """                EventBody::Suspend | EventBody::Visibility { visible: false } => {
                    *flush_after = false;""",
    ),
    mutant(
        66,
        QUEUE,
        "the wait after a change moves from 4 s to 1 s",
        "pub const DEBOUNCE_MS: u64 = 4_000;",
        "pub const DEBOUNCE_MS: u64 = 1_000;",
    ),
    mutant(
        67,
        QUEUE,
        "a batch holds at most 200 ops, not 400",
        "pub const BATCH_MAX: usize = 400;",
        "pub const BATCH_MAX: usize = 200;",
    ),
    mutant(
        68,
        QUEUE,
        "a position on its way is replaced by a newer one",
        "            same && kept.scope.as_deref() == scope && !self.in_flight.contains(&kept.op.op_id)",
        "            same && kept.scope.as_deref() == scope",
    ),
    mutant(
        69,
        CACHE,
        "freshness no longer needs the same boot",
        """        !self.stale
            && self.boot == boot""",
        "        !self.stale",
    ),
    mutant(
        70,
        CACHE,
        "the byte cap drops the oldest page, scope or not",
        "            .min_by_key(|(_, entry)| (entry.scope.is_none(), entry.seq))",
        "            .min_by_key(|(_, entry)| entry.seq)",
    ),
    mutant(
        71,
        LAYER,
        "the layer drops the device sign in view",
        "newest!(session, device_sign_in, route,",
        "newest!(session, route,",
    ),
    mutant(
        72,
        PAIRING,
        "slow_down adds nothing to the interval",
        "held.interval_ms += POLL_STEP_MS;",
        "held.interval_ms += 0;",
    ),
    mutant(
        73,
        PAIRING,
        "expired_token stops, so a dead code stays until a press",
        "DevicePoll::ExpiredToken => self.lapse(mono_ms),",
        "DevicePoll::ExpiredToken => self.clear(DeviceSignInState::Expired),",
    ),
    mutant(
        74,
        PAIRING,
        "access_denied keeps polling",
        "DevicePoll::AccessDenied => self.clear(DeviceSignInState::Denied),",
        "DevicePoll::AccessDenied => self.send_at_mono = Some(mono_ms + 5_000),",
    ),
    mutant(
        75,
        PAIRING,
        "an answer to a stopped sign in still lands",
        "let current = serial == self.serial && self.on_the_way;",
        "let current = self.on_the_way;",
    ),
    mutant(
        76,
        LIB,
        "tokens that arrive after a stop open a session",
        """                if self.pairing.is_current(serial) {
                    self.signed_in(now, answer, out);
                }""",
        "                self.signed_in(now, answer, out);",
    ),
    mutant(
        77,
        PAIRING,
        "a code past its life stays when no poll reaches the server",
        ".is_some_and(|held| mono_ms >= held.expires_mono)",
        ".is_some_and(|held| mono_ms >= held.expires_mono + 1_000_000)",
    ),
    mutant(
        78,
        LIB,
        "a cancelled poll is not answered, so polling stops",
        """                    | Purpose::DeviceCode(_)
                    | Purpose::DevicePoll(_)
""",
        "",
    ),
    mutant(
        79,
        PAIRING,
        "a retry can come inside the interval",
        "self.send_at_mono = Some(mono_ms + wait_ms.max(interval_ms));",
        "self.send_at_mono = Some(mono_ms + wait_ms);",
    ),
    mutant(
        80,
        PAIRING,
        "a network return cuts the interval and earns slow_down",
        "if self.failing && self.send_at_mono.is_some() {",
        "if self.send_at_mono.is_some() {",
    ),
    mutant(
        81,
        LIB,
        "a session leaves device sign in polling",
        """        self.pairing.stop();
        self.session.accept(answer, now);""",
        "        self.session.accept(answer, now);",
    ),
    mutant(
        82,
        LIB,
        "a server with device sign in off is asked again and again",
        "_ if code == ErrorCode::Unsupported => {",
        "_ if status == 1 => {",
    ),
    mutant(
        83,
        LIB,
        "polling arms no wake",
        """            self.pairing.next_wake(),
""",
        "",
    ),
]


def catalogue(only):
    if not only:
        return MUTANTS
    wanted = {name.strip().upper() for name in only.split(",") if name.strip()}
    chosen = [one for one in MUTANTS if one["id"] in wanted]
    missing = wanted - {one["id"] for one in chosen}
    if missing:
        sys.exit(f"no such mutant: {', '.join(sorted(missing))}")
    return chosen


def check(only):
    problems = []
    seen = set()
    for one in catalogue(only):
        if one["id"] in seen:
            problems.append(f"{one['id']}: the id is used twice")
        seen.add(one["id"])
        path = ROOT / one["file"]
        if not path.is_file():
            problems.append(f"{one['id']}: {one['file']} is missing")
            continue
        text = path.read_text()
        found = text.count(one["find"])
        if found != 1:
            problems.append(f"{one['id']}: its text is in {one['file']} {found} times")
        elif text.replace(one["find"], one["replace"], 1) == text:
            problems.append(f"{one['id']}: changes nothing")
        elif one["equivalent"] is not None and len(one["equivalent"]) < 40:
            problems.append(f"{one['id']}: marked equivalent with no real reason")
    for problem in problems:
        print(problem)
    print(f"{len(catalogue(only)) - len(problems)} of {len(catalogue(only))} mutants fit")
    return 1 if problems else 0


def copy_tree(work):
    """Copy the working tree, tracked and untracked, without the build folders."""
    source = work / "source"
    if source.exists():
        shutil.rmtree(source)
    shutil.copytree(
        ROOT,
        source,
        ignore=lambda folder, names: [name for name in names if name in SKIP_DIRS],
        symlinks=True,
    )
    return source


def run_tests(source, target):
    """Run every test command. Returns (passed, unviable, output)."""
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = str(target)
    # Debug info is the bulk of a build and no test reads it.
    env["CARGO_PROFILE_DEV_DEBUG"] = "0"
    env["CARGO_PROFILE_TEST_DEBUG"] = "0"
    env["CARGO_INCREMENTAL"] = "0"
    env["RUSTFLAGS"] = env.get("RUSTFLAGS", "")
    whole = []
    for command in TEST_COMMANDS:
        done = subprocess.run(
            command,
            cwd=source,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        whole.append(done.stdout + done.stderr)
        if done.returncode != 0:
            text = "".join(whole)
            unviable = "error[E" in text or "could not compile" in text
            return False, unviable, text
    return True, False, "".join(whole)


def run(only, work, keep):
    work = Path(work).resolve()
    work.mkdir(parents=True, exist_ok=True)
    source = copy_tree(work)
    target = work / "target"
    chosen = catalogue(only)
    pristine = {}
    for one in chosen:
        path = source / one["file"]
        pristine[one["file"]] = path.read_text()
        if pristine[one["file"]].count(one["find"]) != 1:
            sys.exit(f"{one['id']}: its text is in {one['file']} more or less than once")

    print(f"work folder {work}")
    started = time.time()
    passed, unviable, output = run_tests(source, target)
    if not passed:
        print(output[-4000:])
        sys.exit("the baseline fails, so nothing can be measured")
    print(f"baseline passes in {time.time() - started:.0f} s, {len(chosen)} mutants to run")

    killed, survived, broken, same, wrong = [], [], [], [], []
    for number, one in enumerate(chosen, start=1):
        path = source / one["file"]
        path.write_text(pristine[one["file"]].replace(one["find"], one["replace"], 1))
        began = time.time()
        passed, bad_build, output = run_tests(source, target)
        path.write_text(pristine[one["file"]])
        took = time.time() - began
        if bad_build:
            broken.append(one)
            verdict = "UNVIABLE"
        elif passed and one["equivalent"]:
            same.append(one)
            verdict = "EQUIVALENT"
        elif passed:
            survived.append(one)
            verdict = "SURVIVED"
        else:
            killed.append((one, first_failure(output)))
            verdict = "killed"
            if one["equivalent"]:
                wrong.append(one)
        line = f"[{number}/{len(chosen)}] {one['id']} {verdict} {took:.0f} s  {one['about']}"
        if verdict == "killed":
            line += f"\n    by {killed[-1][1]}"
        print(line, flush=True)

    print("")
    print(
        f"killed {len(killed)}, survived {len(survived)}, "
        f"equivalent {len(same)}, unviable {len(broken)}"
    )
    if survived:
        print("survivors:")
        for one in survived:
            print(f"  {one['id']} {one['file']}: {one['about']}")
    if same:
        print("equivalent, by the reason written with each:")
        for one in same:
            print(f"  {one['id']} {one['file']}: {one['about']}")
            print(f"    {one['equivalent']}")
    if wrong:
        print("marked equivalent and killed anyway, so the reason is wrong:")
        for one in wrong:
            print(f"  {one['id']} {one['file']}: {one['about']}")
    if broken:
        print("did not build:")
        for one in broken:
            print(f"  {one['id']} {one['file']}: {one['about']}")
    counted = len(killed) + len(survived)
    if counted:
        print(f"mutation score {len(killed)}/{counted} = {100 * len(killed) / counted:.1f}%")
    if not keep:
        shutil.rmtree(source, ignore_errors=True)
    return 0 if not survived and not broken and not wrong else 1


def first_failure(output):
    """The first failing test name, or the first error line."""
    for line in output.splitlines():
        stripped = line.strip()
        if stripped.startswith("test ") and stripped.endswith("FAILED"):
            return stripped[len("test ") : -len("... FAILED")].strip()
        if stripped.startswith("---- ") and stripped.endswith("stdout ----"):
            return stripped[len("---- ") : -len("stdout ----")].strip()
    for line in output.splitlines():
        if line.startswith("error"):
            return line.strip()
    return "a test command failed"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["list", "check", "run"])
    parser.add_argument("--only", default="", help="comma separated mutant ids")
    parser.add_argument(
        "--work",
        default=str(Path(os.environ.get("TMPDIR", "/tmp")) / "platform-mutate"),
        help="folder for the copy and its target folder",
    )
    parser.add_argument("--keep", action="store_true", help="keep the mutated copy")
    args = parser.parse_args()
    if args.mode == "list":
        for one in catalogue(args.only):
            print(f"{one['id']} {one['file']}: {one['about']}")
        return 0
    if args.mode == "check":
        return check(args.only)
    return run(args.only, args.work, args.keep)


if __name__ == "__main__":
    sys.exit(main())
