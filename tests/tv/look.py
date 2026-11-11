#!/usr/bin/env python3
"""Signs in a new test account and takes pictures of the television page at chosen key presses.

    uv run --with playwright python tests/tv/look.py http://127.0.0.1:8080 <folder>
"""
import json, os, secrets, sys, urllib.request
from pathlib import Path
from playwright.sync_api import sync_playwright

BASE = sys.argv[1].rstrip("/")
OUT = Path(sys.argv[2])
OUT.mkdir(parents=True, exist_ok=True)
PLAN = [("home", []), ("row1", ["ArrowDown"]), ("row1-right", ["ArrowRight", "ArrowRight"]), ("row2", ["ArrowDown"]),
        ("top", ["ArrowUp", "ArrowUp", "ArrowUp"])]
email = f"look-{secrets.token_hex(5)}@example.test"
password = secrets.token_urlsafe(18)
request = urllib.request.Request(BASE + "/v1/auth/signup", json.dumps(
    {"email": email, "password": password, "first_name": "Key", "last_name": "Test"}).encode(),
    {"content-type": "application/json", "api-version": "2026-11-05"})
urllib.request.urlopen(request, timeout=15)
with sync_playwright() as p:
    browser = p.chromium.launch(channel=os.environ.get("BROWSER_CHANNEL", "chrome-beta"), headless=True)
    tv = browser.new_context(viewport={"width": 1920, "height": 1080}).new_page()
    tv.on("pageerror", lambda error: print("PAGE ERROR", error, flush=True))
    tv.on("console", lambda line: print("console:", line.text[:160], flush=True))
    tv.on("response", lambda answer: print("answer", answer.status, answer.url[-60:], flush=True) if "/v1/profiles" in answer.url else None)
    tv.goto(BASE + "/tv/")
    tv.wait_for_selector("#door:not(.off)", timeout=20000)
    tv.wait_for_timeout(1200)
    tv.keyboard.press("Enter")
    tv.wait_for_selector(".code-value", timeout=20000)
    code = tv.inner_text(".code-value")
    phone = browser.new_context(viewport={"width": 390, "height": 844}).new_page()
    phone.goto(f"{BASE}/activate?code={code}")
    phone.wait_for_selector("input", timeout=20000)
    phone.locator("input").nth(0).fill(email)
    phone.locator("input").nth(1).fill(password)
    phone.keyboard.press("Enter")
    phone.get_by_role("button", name="Approve").click(timeout=20000)
    tv.wait_for_selector(".who, .browse", timeout=30000)
    if tv.locator(".who").count():
        tv.wait_for_timeout(2500)
        tv.screenshot(path=OUT / "who-1.jpg", type="jpeg", quality=85)
        tv.keyboard.press("ArrowDown")
        tv.wait_for_timeout(2500)
        tv.screenshot(path=OUT / "who-2.jpg", type="jpeg", quality=85)
        tv.keyboard.press("ArrowUp")
        tv.wait_for_timeout(500)
        tv.keyboard.press("Enter")
        tv.wait_for_selector(".browse", timeout=20000)
    tv.wait_for_timeout(2500)
    for name, keys in PLAN:
        for key in keys:
            tv.keyboard.press(key)
            tv.wait_for_timeout(350)
        tv.wait_for_timeout(1800)
        tv.screenshot(path=OUT / f"{name}.jpg", type="jpeg", quality=85)
        print(name, flush=True)
    browser.close()
