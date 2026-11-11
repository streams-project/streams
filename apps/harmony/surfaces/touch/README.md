# surfaces/touch

The touch surface of the HarmonyOS family: Huawei phones and tablets.

Surfaces split by **input grammar**, not by platform or screen size — pointer, touch, remote (owner's
decision, 2026-11-11). A Huawei PC driven with a mouse is therefore **not** this surface; it belongs to
a pointer surface, which no family has built yet. That is an open item, not something to improvise
here: a phone layout stretched to a desktop is the same mistake as a desktop layout scaled up to a
television.

**No screen is written.** What follows is the measured shape any screen here has to meet, so that
whoever writes the first one does not invent it. Every line names where its number comes from. All of
it was measured on the Netflix Android app on a Pixel 8 on 2026-11-11. HarmonyOS and Android phones
are the same form and the same input, so the same numbers bind.

Every measured figure below comes from the Netflix reference folder in the project's notes, which
sits outside this repository; paths like `netflix-reference/phone/...` are inside that folder.

| Thing | Number | Where it was measured |
|---|---|---|
| Tile | 119 x 166.6 dp, ratio 1 : 1.400 | phone measurements |
| Tiles across a row | 3.29 visible | phone measurements |
| Rows on the first screen | exactly **one**, under a **still** billboard — not a video | phone measurements |
| Top bar on scroll | does **not** hide | phone measurements |
| Text on a card | **none**: the title and every badge are composited into the artwork server side, so a title string under a poster cannot reproduce the look | `netflix-reference/00-gaps.md` |
| Artwork scale | nothing is served at 2x; a row tile is about 1.3x its own box | `00-gaps.md` |
| Rows on Home | 23 on the phone | phone measurements |
| Titles in a row | 70 on the phone | `00-gaps.md`, counted by flinging to the end |

ArkUI measures in `vp`, which is the same unit as Android's `dp` (assumed, not checked here).

Every figure in that table is a **phone** measurement, and this surface also carries tablets. What a
tablet does with them is a recorded decision, not a guess here, and not a second layout:
`netflix-reference/08-account-settings-hierarchy-android.md:142` — Netflix's tablet app is the phone app
drawn wider, with the same header, the same chips row, the same floating bottom pill and the same My
Netflix and Profile screens, so tablet widths (768 to 1023 px) use the phone tree and the phone screens
centred in a wider column, with no separate tablet tree. The note marks that **verified** for the home
layout, from one App Store image, and **assumed** for everything behind the profile sheet. No figure
above has been measured on a tablet. What would settle it: the same measurement pass, on a tablet.

One gap this surface must not repeat, and it is narrower than it was. `00-gaps.md` gap 1 measures
`ROWS_CAP = 60` and `INITIAL_ROWS = 12` in `server/crates/server-pages/src/plan.rs:14-16`; the 50 rows
the note also quotes is what `platform/tools/score_pages.py:94` reached on Home by following the cursor,
not a server cap, and the note's 38 unreachable rows is that 50 less the 12 the app is given. Gap 2
measures a row at 15 titles plain and 24 at the head of a composed row against `MAX_ROW_DEPTH = 120` on
the server.

What has changed since that note: paging exists in both native families as of this commit, so there is
nothing to invent here. `GET /v1/more/{cursor}` is called from
`apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Paging.kt:75,120` and
`apps/apple/Sources/SurfacesPhone/browse/Home.swift:301`, and `Screen.cursor` is read at
`Paging.kt:59-61,115` and `Home.swift:297-314`. A first screen here follows those two rather than
writing a third answer, and nothing new has to be built on the server for it.
