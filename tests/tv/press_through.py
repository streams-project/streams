#!/usr/bin/env python3
"""Drives the television page with key presses against a running stack, from the door to a playing
film, with a phone approving the code on the side. It needs nothing but a browser.

    uv run --with playwright python tests/tv/press_through.py http://127.0.0.1:8091 <folder for pictures>
"""
import json, os, secrets, sys, time, urllib.request
from pathlib import Path
from playwright.sync_api import sync_playwright

BASE = sys.argv[1].rstrip("/")
OUT = Path(sys.argv[2] if len(sys.argv) > 2 else ".")
OUT.mkdir(parents=True, exist_ok=True)
CHANNEL = os.environ.get("BROWSER_CHANNEL", "chrome-beta")

def post(path, body):
    request = urllib.request.Request(BASE + path, json.dumps(body).encode(), {"content-type": "application/json", "api-version": "2026-11-05"})
    with urllib.request.urlopen(request, timeout=15) as answer:
        return answer.status

def step(name):
    print(f"{time.strftime('%H:%M:%S')} {name}", flush=True)

email = f"tv-{secrets.token_hex(6)}@example.test"
password = secrets.token_urlsafe(18)
post("/v1/auth/signup", {"email": email, "password": password, "first_name": "Key", "last_name": "Test"})
step("account made")

with sync_playwright() as p:
    browser = p.chromium.launch(channel=CHANNEL, headless=True, args=["--autoplay-policy=no-user-gesture-required"])
    tv = browser.new_context(viewport={"width": 1920, "height": 1080}).new_page()
    tv.on("pageerror", lambda error: print("PAGE ERROR", error, flush=True))
    tv.on("console", lambda line: print("console:", line.text[:200], flush=True))
    def seen(answer):
        if "/v1/play" in answer.url:
            print("play answer", answer.status, answer.text()[:600], flush=True)
    tv.on("response", seen)
    tv.goto(BASE + "/tv/")
    tv.wait_for_selector("#door:not(.off)", timeout=20000)
    step("door")
    tv.wait_for_timeout(1500)
    tv.keyboard.press("Enter")
    tv.wait_for_selector(".code-value", timeout=20000)
    code = tv.inner_text(".code-value")
    address = tv.inner_text(".code-address")
    tv.screenshot(path=OUT / "1-code.png")
    step(f"code shown, address {address}")

    phone = browser.new_context(viewport={"width": 390, "height": 844}).new_page()
    phone.goto(f"{BASE}/activate?code={code}")
    phone.wait_for_selector("input", timeout=20000)
    fields = phone.locator("input")
    fields.nth(0).fill(email)
    fields.nth(1).fill(password)
    phone.keyboard.press("Enter")
    phone.get_by_role("button", name="Approve").click(timeout=20000)
    step("approved on the phone")

    tv.wait_for_selector(".who, .browse", timeout=30000)
    if tv.locator(".who").count():
        tv.screenshot(path=OUT / "2-profiles.png")
        tv.keyboard.press("Enter")
        tv.wait_for_selector(".browse", timeout=20000)
    tv.wait_for_timeout(800)
    tv.screenshot(path=OUT / "3-home.png")
    step("home")

    for key in ("ArrowDown", "ArrowRight", "ArrowRight", "ArrowDown", "ArrowRight"):
        tv.keyboard.press(key)
        tv.wait_for_timeout(260)
    tv.wait_for_timeout(300)
    tv.screenshot(path=OUT / "4-rows.png")
    tv.keyboard.press("Enter")
    tv.wait_for_selector(".title-text", timeout=20000)
    tv.wait_for_timeout(400)
    tv.screenshot(path=OUT / "5-title.png")
    step("title: " + tv.inner_text(".title-text h1"))

    tv.keyboard.press("Enter")
    tv.wait_for_selector("#movie", timeout=20000)
    first = 0
    for _ in range(90):
        first = tv.evaluate("document.getElementById('movie').currentTime")
        if first > 1:
            break
        tv.wait_for_timeout(500)
    else:
        print("state", tv.evaluate("(() => { var m = document.getElementById('movie'); return [m.readyState, m.networkState, m.currentSrc, m.error && m.error.code, document.body.innerText.slice(0, 200)] })()"))
        raise SystemExit("the film never reached 1 s")
    tv.wait_for_timeout(2500)
    later = tv.evaluate("document.getElementById('movie').currentTime")
    tv.screenshot(path=OUT / "6-film.png")
    step(f"film time {first:.1f} s then {later:.1f} s")
    assert later > first, "the film did not advance"

    tv.keyboard.press("Enter")
    tv.wait_for_timeout(700)
    assert tv.evaluate("document.getElementById('movie').paused"), "OK did not pause"
    tv.keyboard.press("Enter")
    tv.wait_for_timeout(700)
    assert not tv.evaluate("document.getElementById('movie').paused"), "OK did not resume"
    tv.keyboard.press("Escape")
    tv.wait_for_selector(".title-text", timeout=20000)
    step("back at the title")
    browser.close()
print("PASSED")
