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

One gap this surface must not repeat. `00-gaps.md` gap 1 and gap 2: the server builds up to 50 rows and
the apps ask for 12, `GET /v1/more/{cursor}` has exactly one caller in the whole tree (`apps/web`), and
neither native family reads `Screen.cursor` — so 38 built rows and 96 of 120 titles in a row are
unreachable on a phone today. A first screen here reads the cursor from the start. Nothing new has to
be built on the server for it.
