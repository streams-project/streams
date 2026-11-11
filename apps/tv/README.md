# tv

The television app for the three systems whose only open runtime is a web engine: Samsung Tizen,
LG webOS and Vizio. It is one page of plain ES5 with no build step, running the core as WebAssembly.

**Status, 2026-11-11: it runs, served. It is not a package yet, and it cannot be one without the
changes in section 2 — measured, not guessed.** Note 179 stopped this folder on 2026-11-07 because
it was a page *served* by our server at `/tv/`, and Samsung refuses hosted apps. The owner has now
drawn the line differently: a packaged app, in the television's own format, carrying its assets and
the core locally and talking to a server address the viewer gives it, is not a hosted page. That is
what reopens this folder, and only for those three systems. The core is native code everywhere else.
Roku is settled later; nothing here plans around one.

26 JavaScript files, 2,198 lines, plus `index.html` and `tv.css`: 28 files and 98 KB in this folder,
measured 2026-11-11 with `find page -type f -exec wc -c` (100,802 bytes). The 164 KB in section 1 is
the *published* folder, which carries files this one does not. It signs in with a
code a phone approves, then draws profiles, rows, a title and the film. `tests/tv/press_through.py`
drives all of that by key presses through a real browser against a running stack, which `./check`
does not do.

| File | What it does |
|---|---|
| `page/host.js` | Steps the core and performs what it asks |
| `page/ports.js` | The one way out of a surface: keys, network, secrets, player, clips |
| `page/adapters/` | What each port really does on a television |
| `page/surfaces/` | The screens: the door, the code, profiles, browse, a title, the film |

A brand puts `opening.mp4`, `showreel.mp4` and `peek.jpg` in its `media/` folder. The page runs
without them.

---

## 1. What a package needs on each system, and what this folder already has

Read from the vendors' own pages on 2026-11-11. Nothing here was taken from an SDK, because none of
the three is installed on this machine; every line says which it is.

| | Samsung Tizen | LG webOS | Vizio |
|---|---|---|---|
| Package | `.wgt` | `.ipk`, built by `ares-package` | no package found — see below |
| Manifest | `config.xml` in the root, `<widget>` root element | `appinfo.json` in the root | — |
| Entry point | `<content src="...">`, default `index.html` | `main`, path relative to `appinfo.json`, default `index.html` | — |
| Required fields | application id `{package}.{project}`, `version` as `x.y.z`, `<icon src>` | `id` (reverse DNS, not `com.lge`/`com.webos`/`com.palm`), `title`, `type` (`web` only), `main`, `icon`, `version` | — |
| Icons | launcher icon named in `config.xml` | `icon` 80x80 PNG, `largeIcon` 130x130 PNG, `splashBackground` 1920x1080 PNG | — |
| Network | blocked by default under W3C WARP; the access policy must be declared | — | — |
| Signing | a certificate profile issued by Samsung; the TV's certificate and DUID must be registered to side-load | not read | — |

Status: **verified against the vendor pages** for the webOS field list, the Tizen `config.xml`
elements and the Tizen signing model
([webOS `appinfo.json`](https://webostv.developer.lge.com/develop/references/appinfo-json),
[Samsung, Configuring Web Applications](https://developer.samsung.com/tv/develop/guides/fundamentals/configuring-tv-applications)).
**Open** for whether webOS 27 and later demand `requiredACG` of every app, which one search result
claims and the reference page does not mention.

**Vizio is the odd one and may not fit the owner's rule at all.** `developer.vizio.com` answers 403
without an account. Every description found is third party, and all of them say a SmartCast app is
HTML5 **hosted at a URL** that the developer gives Vizio — pre-production and production addresses —
rather than a package installed on the set, and that the app must carry Vizio's own JavaScript
library for the remote. If that holds, Vizio is a hosted web app by definition, which is the thing
note 179 ruled out, and it belongs in a different decision from Tizen and webOS. Status: **assumed,
third party only**; note 171 recorded the same wall on 2026-11-06. Unblocks with: a Vizio developer
account.

**What this folder already satisfies.** The entry point is `index.html` at the root of `page/`, which
is both platforms' default. There is no build step, no bundler and no framework, so what ships is
what is in the folder. The core is already a local file next to the page. Everything else on the list
above is missing: there is no `config.xml`, no `appinfo.json`, no icon set, no splash image, no
signing, and no network-access declaration.

**Size, measured on the published site 2026-11-11** (`ls -la ~/.streams/site/tv`): page 164 KB, 
`core.wasm` 534 KB, `hls.light.min.js` 377 KB, `peek.jpg` 95 KB, two opening sounds 100 KB — about
**1.25 MB** of package — and then `showreel.mp4` at **78.6 MB**, which is 98% of the whole thing. A
package carrying the showreel is a 80 MB download on every update. Whoever builds the package has to
decide whether the showreel ships inside it or is fetched from the server on first run. Status:
verified sizes, open decision.

## 2. Every place the page assumes the server is its origin

Measured, not read: the page was copied to a folder with the core beside it and opened from
`file://` in Chrome 1920x1080. What happened, in order:

| Assumption | Where | What happened from local files |
|---|---|---|
| `/brand.css` | `page/index.html:7` | `ERR_FILE_NOT_FOUND`. Every colour, font and radius token is undefined; `--radius-card` read back empty |
| `/hls.light.min.js` | `page/index.html:32` | `ERR_FILE_NOT_FOUND`; `window.Hls` is `undefined`, so HLS dies on any set without native HLS |
| `/mark.svg` | `surfaces/parts/topbar.js`, `surfaces/people.js` | same path, same fate; not reached in this run because the app never started |
| `core.wasm` by `XMLHttpRequest` | `page/host.js:61-96`, the XHR at 80 | **blocked by CORS from origin `null`**. The app stopped at the door with "The app's core did not load from this server." This is the one that makes it fail completely |
| the API's address | `page/host.js:21` `base = given.base \|\| ""`; `surfaces/shell.js` never passes `base` | every `/v1/...` goes to the page's own origin. A package has no server at its origin |
| the content policy | `server/crates/server-api/src/site.rs` sends it as a header | a package gets no header; Tizen and webOS want it in their own manifest |
| the brand font | `page/tv.css` asks for `var(--font-body)` | `document.fonts.size` is **0**: the `@font-face` lives in the website's stylesheet, which this page never loads. The page has **always** fallen back to the set's own sans-serif, served or not |

What also worked, and is worth knowing: **`localStorage` works from a `file://` origin in Chromium**,
so the secrets port survives the move. Status: verified in Chrome; **open** for Tizen's and webOS's
own engines.

**What must change, in the order that matters:**

1. **The core must stop arriving over XHR.** `fetch` and `instantiateStreaming` hit the same wall. The
   portable answer is to load the module as a script — the bytes assigned by a generated `.js` file —
   because a script tag is not subject to the origin check. The cost is about a third more bytes
   (534 KB becomes roughly 740 KB base64) and a build step this folder does not have today.
   Alternative: keep XHR and rely on each vendor's engine allowing it for a packaged app. That is
   exactly the thing that cannot be settled without an SDK or a set. **Open.**
2. `brand.css`, `mark.svg`, `hls.light.min.js` and the brand font file must be copied into the package
   and referenced relatively. They sit at the site root today because `launcher/src/main.rs:189-213`
   publishes the page into the website. That file belongs to another lead; this folder cannot fix it
   alone. **Open.**
3. **A server address screen.** A packaged app has no idea where its server is. The Android app
   already asks for one (note 179). The core has the `base` seam for it (`host.js:330`) and the
   secrets port to keep it in. This is a new screen, so it is not in today's changes. **Open.**
4. The content policy moves into `config.xml` / `appinfo.json`. **Open.**

## 3. The remote surface against the measured reference

This is a remote surface, held to the same reference as the native televisions:
`pstream/notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md`. Measured in a real browser at
1920x1080 on 2026-11-11 by rendering the page's own parts and reading `getBoundingClientRect`.

| What the reference says | This page |
|---|---|
| The focused row is pinned to a fixed band, y 199-708, and the content scrolls under it; focus never walks down the screen | **holds.** `.open-card` sits at y 200, height 508, in every row. Moving between rows slides the old row out and the new one in; the band never moves. Within 2 px of the measurement |
| Focusing a tile swaps the artwork from a portrait poster to a 16:9 still — two assets per title | **holds.** `parts/tiles.js` asks for `poster` closed and `backdrop`/`still` focused |
| Closed tile 311 x 505, focused 906 x 510, gutter 17, pitch 328 | **holds within 2 px**: 311 x 504, 904 x 508, gutter **17 after today** (19 before), pitch 328 |
| 1 expanded + 3 closed + 1 clipped across the screen | **holds** |
| Back never exits the app | **does not hold, on purpose.** Note 187, a recorded decision of this project, says the root pages leave the app, and Home and Kids Home still let both television Back codes through. This page has no profile menu, so Netflix's way out (an Exit line there, note 184) does not exist here and taking Back everywhere would leave no way out at all. Open item below; the owner picks |
| Netflix cards carry no text: titles are composited into the artwork | **does not hold, on purpose.** Our artwork has no title burnt into it, so `.open-name` and the poster's fallback name are the only way to read a tile. Removing them would not reproduce Netflix's look, it would just lose the name. Left in, named here so nobody mistakes it for an oversight |
| Safe margin 68 px at 1920 (3.5%) | this project uses **72**, decided in note 188 and already held by the two native TV apps. Today's change brings this page onto it |
| While a tab loads the body is pure black, no skeleton | **holds** — nothing is drawn until data arrives |
| A cold start lands on a profile gate: a left column of faces, art filling the right | **holds in shape.** The art starts at x 480 here against the reference's rough "right two thirds" (x 640). That reading comes from an angled phone video, which that note forbids using for pixel sizes, so it was not moved |

## 4. What changed on 2026-11-11

Each line says what it traces to. Nothing was added that nobody asked for.

* **`surfaces/shell.js:83-92` — Back is unchanged, and note 187 is why.** This file briefly took Back
  on Home and Kids Home as well, on the strength of the reference's section 7 (four `BACK` presses,
  the app in front every time, n=4, measured on Netflix's profile gate) and section 10.8 ("Back never
  exits the app"). That measurement is real, but it does not outrank note 187, which is a recorded
  decision of this project and says the root pages leave the app. **An earlier version of this file
  claimed the owner had named "Back never exiting" binding for this surface today. No note records
  that instruction and it has been deleted rather than replaced.** Netflix's way out is an Exit line
  in the profile menu (note 184) and this page has no profile menu, so the change would have left a
  packaged app with no way out at all. Note 187 holds; the disagreement is an open item for the owner.
* **`tv.css:8-16` — one corner rule.** `--tv-card` 21, `--tv-panel` 36, `--tv-field` 18, each the
  brand's value times 1.5, in one place, replacing hard-coded 24, 20 and 16 and a phone-sized
  `var(--radius-card)` of 14. Traces to note 188, which set exactly these roles and which the two
  native TV apps already hold in `Round.kt` and `Round.swift`. Verified: computed radii read back
  21 / 36 / 18. Two places this rule does **not** reach, because note 188 does not name them:
  the QR picture (`tv.css:194`) keeps its own 24 — note 188's card role is posters, stills, episode
  pictures and the picture grid, and its field role is inputs, the code plate, PIN boxes and help
  boxes, and a 460 px code picture is none of those — and the landing card's corner is covered in
  the open items below.
* **`tv.css:275-285` — a face's corner is 16% of its size** (48 -> 7.7, 130 -> 20.8, 194 -> 31), so
  a face keeps its shape while it grows on the profile gate. Before, a fixed 18 px on a face
  animating 130 -> 194. Note 188. It is a CSS percentage on `.avatar`, not a pixel value written by
  JS, so it follows the size through the 200 ms grow instead of jumping to its end value at the
  start — which is the one moment the rule exists for. That retires `.avatar.square`'s 8 px (16% of
  the bar's 48 px face is 7.7, the same shape) and `.who-row .avatar`'s 18 px, and the `"square"`
  argument in `surfaces/parts/topbar.js` with them.
* **`tv.css:287-290` — the member top bar is one line.** Its four parts sat on centre y 70, 67, 67
  and 72; they now all sit on **68**, and the mark's right edge is **1848**, not 1860. Note 188.
* **`tv.css:66-85` — the landing's top row is one line of one height.** The home circle was 82 px and
  the tab 59 px, both on centre 119; both are now **72 px on centre 104**. Note 188 records the owner
  sending a shot of exactly this row with its parts "on three different lines and heights"; the two
  native apps were fixed then and this page was not.
* **`tv.css:105-123` — Get Help is a pill on the buttons' line.** It was 32 px below the pills' centre
  (measured: 961 against 929) and is now on **760**, their centre. Note 188 records the same fault at
  25 px and its fix as "a pill on their line"; the two native apps draw it there as a circle-shaped
  box 56 tall (`apps/android/.../door/Door.kt`: offset y 732, height 56, `CircleShape`, 24 px of
  horizontal padding), so this page takes 732, 56 and the button radius. It briefly had the field
  corner 18 and a height of 80, which nothing names; that was the lead's own choice and is gone.
  Still unlike the native apps: the words sit on this page's landing column of 96 with 14 px of
  padding, not on 72 with 24. That belongs to the landing-column open item below.
* **`tv.css:355-389` — the page edge is 72 and the right edge 1848** on the pages with the top bar:
  hero card, open card, the posters beside it, the facts, the synopsis, the next row. They were on 66,
  68, 70, 75 and 989. Note 188. A side effect worth naming: the row's gutters are now **17 px on both
  sides** of the focused card, which is the reference's measured gutter; they were 19.
* **`tv.css:305-325` — the hero card takes its measured band**, 72 x 140, 1776 x 772, against 75 x 143,
  1789 x 767. Reference section 6b gives 1790 x 772 at x 67, y 140; the left and right edges come from
  note 188 instead, which is a decision about this project and not about Netflix.
* **`tv.css:383` — the synopsis is capped at 990 px**, not 1000. Reference section 6b: "capped at
  990 px, 51.6% of the width".
* **`surfaces/parts/openrow.js:10-15` — a row travels 798 px when focus moves between rows**, not 810.
  798 is this page's own distance between the open row and the dim one below it (row title 146 to 944,
  open card 200 to next posters 998), which is what that number has to be: the row is translated by it
  to land on the dim row's place. It was briefly 797, the reference's row-title-to-row-title pitch
  (section 6b), which is not the same quantity — the reference's own rhythm is not one translation,
  since its titles are 797 apart (151 to 948) while its tiles are 799 (199 to 998). Bringing this
  page's rhythm onto the reference's is an open item below.

**How it was verified.** `tests/tv/press_through.py` needs a running stack and none was up, so it was
not run — stated plainly rather than claimed. Instead the page's own parts were rendered from a
harness outside this repository, at 1:1 on a 1920x1080 stage with the brand tokens stubbed at their
`brand.json` values, and every box was read back with `getBoundingClientRect`. Read back on
2026-11-11, after the review: Get Help 96 x 732, 56 tall, computed radius 999 px, centre y **760** —
the same centre as the two 80 px pills at 720; the bar's face 48 at 44 and the mark 64 at 1784, both
on centre **68**, right edge **1848**; faces of 48, 130 and 194 all computing a 16% corner, and a
corner pixel 3 px inside the 130 and 194 boxes falling outside the painted shape while one 25 px and
10 px in falls inside, which is the corner following the size; the open card's corner **21**; the QR
picture's **24**; row title 146 to 944 and open card 200 to next posters 998, so this page's own row
distance is **798** on both readings. The Back behaviour is unchanged from `505652c^`, so there was
nothing to press for it. `check_rings.py`, `check_private.py` and `check_strings.py` all exit 0.

## Open items

* **The core cannot load from local files** — CORS blocks XHR at a `file://` origin, measured.
  Unblocks with: a decision between a generated script that carries the bytes and a vendor engine
  that allows the read, and the latter needs a Tizen or webOS SDK or a real set.
* **Is Vizio a package at all** — every source says hosted URL. Unblocks with: a Vizio developer
  account.
* **The manifests, icons, splash and signing** do not exist. Unblocks with: the two SDKs, a Samsung
  seller certificate and a registered set.
* **Note 187 and the reference disagree about Back, and only the owner can settle it.** Note 187, a
  recorded decision, has the root pages leave the app, which is what this page does on Home and Kids
  Home. The reference measured the opposite on Netflix: four `BACK` presses on its profile gate with
  the app still in front (section 7, n=4; section 10.8 "Back never exits the app"), the way out being
  an Exit line in the profile menu (note 184). Nothing in `pstream/notes` records an instruction to
  prefer the measurement, so note 187 stands. Taking Back on the root pages as well would leave a
  packaged app with no way out at all, since this page has no profile menu. Unblocks with: the
  profile menu plus an Exit line, or the owner saying the root pages should stop leaving.
  Also unread: whether Tizen or webOS store certification requires Back to close the app from the
  first screen. Nothing here has checked that, and it could decide the question on its own.
* **The profile chooser does not leave the app at the start, and note 187 says it should.** That note
  has the chooser go back while switching and leave the app at the start. Here Back on the chooser has
  always reached `kit.back()`, which with an empty trail does `go("/home")` (`surfaces/kit.js:39-42`),
  so it navigates instead. Unchanged by 505652c and unchanged now; it needs the chooser to know
  whether a profile is already in use. Unblocks with: that state, or the owner saying the page may
  keep this.
* **The brand font never loads** — zero `@font-face` rules, measured. Unblocks with: the font file
  copied beside the page, which is `launcher/src/main.rs`, not this folder.
* **38 of the server's rows are unreachable from here.** The server builds up to 50 and this page asks
  for 12; `surfaces/browse.js` never reads `Screen.cursor` and never calls
  `GET /v1/more/{cursor}`. That route had one caller until 505652c and it was `apps/web`; the same
  commit added callers in `apps/android` (the phone's `browse/Paging.kt`, the television's
  `browse/Browse.kt`) and `apps/apple` (`SurfacesPhone/browse/Home.swift`,
  `SurfacesTV/browse/Browse.swift`), verified by reading both commits. So this page is now the only
  surface in the family without paging, which is the opposite of what the sentence here used to say.
  Unblocks with: a decision on how many rows a television should carry, which nothing has measured yet.
* **`opening.mp4` is never published.** `launcher/src/main.rs:161` copies `showreel.mp4`, `peek.jpg`
  and the two opening sounds, not `opening.mp4`, which `surfaces/door.js` asks for. The door survives
  it — the error jumps straight to the landing — so this has been silently dead. Note 186 decided the
  opening should be drawn live by the apps instead of a video; this page still plays a file.
* **The strings are hard-coded English.** `check_strings.py` does not cover this folder.
* **Note 188 gives the code picture no role.** Its card role is posters, stills, episode pictures and
  the picture grid; its field role is inputs, the code plate, PIN boxes and help boxes. The QR is a
  460 px picture that is neither, so it keeps its own 24 and is the one corner in this file that the
  one rule does not reach. Unblocks with: the owner or note 188 naming a role for it.
* **The landing's columns disagree with note 188**: the card is on 55 and the top row on 96, where
  that note put both on 72. The card's position is a measurement of the reference landing recorded in
  `tv.css`'s own first comment, so moving it would break a measured composition with nothing to check
  the result against. The same comment measured that card's corner at 24 and the file now draws it at
  note 188's panel 36 — the corner is this project's brand and the place is Netflix's composition, so
  they are treated differently on purpose, and the file says so where it happens rather than only
  here. Get Help's words also sit on this 96 column with 14 px of padding where the two native apps
  put them on 72 with 24. Unblocks with: the reference landing frame, or the owner's eye.
* **This page's row rhythm is its own, not the reference's.** Reference section 6b: row title 151-176,
  tiles 199-708, facts 733-755, synopsis 775-842, next title 948-978, next tiles from 998. This page:
  title at 146, card 200, facts 728, synopsis 768, next title 944, next tiles 998. The differences are
  a few pixels and partly an artefact of comparing a CSS box top against a measured ink box, but they
  have not been checked one by one. The `.high` variant is looser still: its two row titles are 800
  apart where the normal pair is 798, so the row that rises lands 2 px off there. Unblocks with: a
  reading of this page's own ink boxes against section 6b, which needs the harness and an hour.
* **The landing's active tab underline sits further from its words than it did.** The tab is 72 tall
  on centre 104 because note 188 says so, and the underline is `box-shadow: inset 0 -4px` on that box,
  so it moved down with the box's bottom edge, about 8 px. Nothing measures where the underline
  belongs. Unblocks with: a measurement of Netflix's landing tab, or the owner's eye.
* **Only Get Help carries a fallback for the button radius.** `tv.css` reads `var(--radius-button)`
  raw in `.tab`, `.pill`, `.button` and `.top-tab`, and every token is undefined in a package until
  `brand.css` is copied in (section 2, item 2), which squares off every pill. Get Help now falls back
  to 999 where it is drawn. Unblocks with: section 2 item 2, after which no fallback is needed, or one
  pass putting the button role in the `:root` block beside the three corner roles.
* **The root `README.md` still says work on this folder stopped on 2026-11-07.** That file belongs to
  another lead.

Licensed under either of MIT or Apache 2.0, at your option.
