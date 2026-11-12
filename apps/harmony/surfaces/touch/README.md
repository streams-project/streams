# surfaces/touch

The touch surface of the HarmonyOS family: Huawei phones and tablets.

Surfaces split by **input grammar**, not by platform or screen size — pointer, touch, remote (owner's
decision, 2026-11-11). A Huawei PC driven with a mouse is therefore **not** this surface; it belongs to
a pointer surface, which no family has built yet. That is an open item, not something to improvise
here: a phone layout stretched to a desktop is the same mistake as a desktop layout scaled up to a
television.

**20 files, 5,382 lines, written 2026-11-12. Not one line compiled.** No HarmonyOS SDK and no device
were on the machine, and `../../OPEN-QUESTIONS.md` says what that leaves and what would close it.
Every ArkTS and ArkUI call used is either in the reference documentation read that day, or marked
`ASSUMED` where it is used — there is no third category, and nothing here was written from memory of
an API.

## What is written

| File | What it draws | Read from |
|---|---|---|
| `Root.ets` | the one route switch, the window's size, the mirroring | `app/Root.kt` |
| `Home.ets` | a main page: tinted top, header, billboard, rows, floating bar, categories | `browse/Home.kt` |
| `Rows.ets` | one row and the page its name opens | `browse/Rows.kt:75-243,491-583` |
| `Tiles.ets` | the plain tile, the Top 10 tile, the Continue tile | `browse/Rows.kt:249-489` |
| `Billboard.ets` | the tall card, its logo, tags and two buttons | `browse/Billboard.kt` |
| `Header.ets` | the pinned top bar and the chip row | `browse/Header.kt` |
| `BottomBar.ets` | the floating bar | `parts/BottomBar.kt` |
| `Empty.ets` | the placeholder rows a page shows before it has anything | `browse/Empty.kt` |
| `Title.ets` | the title page, its facts, its actions, Episodes and More Like This | `title/Title.kt`, `title/Episodes.kt`, `title/MoreLike.kt` |
| `Gate.ets` | the profile chooser: carousel, arc sheet, grid, manage mode | `profiles/Gate.kt`, `profiles/GateGrid.kt`, `profiles/GateArt.kt` |
| `GateTint.ets` | the gate's colour model, HSV by hand | `profiles/GateTint.kt` |
| `Search.ets` | search — read its head before changing it | `tv/search/Search.kt` for behaviour only |
| `Face.ets` | a profile's picture, or its first letter | `shared/Face.kt` |
| `Pictures.ets` | `Art`, which draws what the `Pictures` port hands back | `shared/Art.kt` |
| `Paging.ets` | how a page and a row grow through `/v1/more/{cursor}` | `browse/Paging.kt` |
| `Details.ets` | a title's facts, read once per run | `shared/Details.kt` |
| `Words.ets` | `fill`, `number`, plural forms, counted words | `shared/Theme.kt:62-134` |
| `Sizes.ets` | **every measured number, each with its file and line** | the table below |
| `Theme.ets` | colours and corners, from `brand.json` | `shared/Theme.kt`, `brand.json` |
| `Heard.ets` | what a screen reader is handed | `shared/Says.kt` |
| `Pick.ets` | which picture a place wants, and where a tap goes | `shared/Art.kt`, `shared/Addresses.kt` |
| `Icons.ets` | the 23 icons, named in one place; **the files do not exist yet** | the Kotlin's `R.drawable.icon_*` |

## What is not written, and is not pretended

Five routes draw nothing, each named in `Root.ets` with the Kotlin that specifies it: the server
address screen, `start` (`door/Doors.kt`), `sign_in`, `watch` (`watch/Watch.kt` and the player's
controls) and the notifications screen (`notifications/Notifications.kt`). The opening
(`app/Opening.kt`, 309 lines) is not here either. Adding and editing a profile
(`profiles/AddProfile.kt`, `profiles/EditProfile.kt`, 1,505 lines between them) are raised by the
gate and go nowhere. A blank page is the honest state; a placeholder would be the `apps/roku`
mistake.

## The measured shape every screen here meets

Every line names where its number comes from. All of it was measured on the Netflix Android app on a
Pixel 8 on 2026-11-11. HarmonyOS and Android phones are the same form and the same input, so the
same numbers bind. Every figure below is in `Sizes.ets` as a named constant carrying its citation,
and a number not in that file is a number this surface may not draw with.

Every measured figure comes from the Netflix reference folder in the project's notes, which sits
outside this repository; paths like `netflix-reference/phone/...` are inside that folder.

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

ArkUI measures in `vp`, and the reference says of it: "Pixel unit specific to the screen density…
**This unit is used for values whose unit is not specified**", with `Length`'s number branch
documented as "The default unit is vp". Whether a vp is the same size as Android's dp is **assumed**:
no HarmonyOS page mentions dp or Android at all, and both are defined as a density-scaled reference
unit. If it turns out not to be, every number scales by one factor and `Sizes.ets` is the one file
that changes.

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

Paging exists in both native families, so there was nothing to invent here.
`GET /v1/more/{cursor}` is called from
`apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Paging.kt:75,120` and
`apps/apple/Sources/SurfacesPhone/browse/Home.swift:301`, and `Screen.cursor` is read at
`Paging.kt:59-61,115` and `Home.swift:297-314`. `Paging.ets` follows those two rather than writing a
third answer, and nothing new had to be built on the server for it.

## Two numbers this surface needed and does not have

Named here rather than chosen:

1. **A touch search screen's layout.** The Android phone surface has no search screen — it sends the
   viewer to `/search` and draws the server's answer as rows — and the only one in the project is the
   television's, built for a remote. So `Search.ets` is built from the two halves that *are*
   measured: the television's behaviour (a 350 ms settle, `/v1/search?q=`, the offered titles out of
   Home) and this surface's own grid of tiles (`browse/Rows.kt:556-561`). Where the field goes and
   how tall it is are not measured and are not invented. What would settle it: one measurement pass
   on the Netflix phone app's search screen.
2. **The arc of the profile sheet.** `profiles/Gate.kt:191` clips it with `ArcSheetShape(sagDp =
   15.2, radiusDp = 1465)`. Both numbers are in `Sizes.ets`; reproducing the curve wants `clipShape`
   with a `Path`, neither of which was read off the reference, so the sheet is drawn with rounded top
   corners instead. Right colour, right height, wrong curve, and said so where it is drawn.

## What the ring check would have to ban here

`platform/tools/check_rings.py` does not scan this family. When it does, the banned words for a
HarmonyOS surface are not the kit imports: `@kit.ImageKit` is imported by four files in this folder
for the **type name** `image.PixelMap`, which ArkTS requires because it has no structural typing, and
`@kit.ArkUI` is imported for `LengthMetrics`, which is how a layout mirrors. What reaches the device
is a **call**, and those are the words:

```
the network port    \bhttp\.createHttp\b|\bcreateHttp\(|@ohos\.net\.|@kit\.NetworkKit
the secrets port    \basset\.(add|query|remove|update)\b|@kit\.AssetStoreKit
the store port      \bfileIo\.|@kit\.CoreFileKit|@ohos\.file\.fs
the player port     \bAVPlayer\b|createAVPlayer|createMediaSourceWithUrl
the pictures port   createImageSource|createPixelMap
the address table   "/(home|browse|latest|login|sign-in|profiles|search|help|settings|kids|mine|games|activate)[/"?]
```

`effectKit.createColorPicker` in `Pictures.ets` is the one call worth arguing about: it reads a
colour out of a `PixelMap` the port already handed over, which is what `androidx.palette` does inside
the Android surface at `browse/Billboard.kt:134`. If it is to be banned, the colour moves into the
`Pictures` port for both families at once.
