# The TV screens, by folder

Android TV (`apps/android/surfaces/tv/src/main/kotlin/streams/tv`) and Apple TV
(`apps/apple/Sources/SurfacesTV`) use the same folders and the same file names, so a screen found in
one is at the same path in the other and can be carried across part by part.

| Folder | What is in it |
|---|---|
| app | The root (Television), the remote and its key map rules (Remote), shapes, edges and direction (Round), the opening, title details, pictures |
| door | The landing, sign in, its code picture, the server address |
| profiles | Choosing, adding, editing and locking profiles, the profile menu, picture rows |
| browse | Pages of rows: the top bar, the hero, the rows |
| title | The title page, episodes, More Like This, reactions, the poster grid |
| search | Search and its letter keyboard (one set of letters per language, the globe switches to English) |
| watch | The player and its panels: choices, tracks, moments, side words |
| help | Get Help, the Help Centre, problem pages |
| settings | Settings and their rows |
| parts | Small pieces every screen uses: face, pill, menu, page head, badge, facts, keyboard, tick grid |

Words and numbers: a language's words come from `strings/{lang}.json`, its row names from
`strings/rows/{lang}.json`; direction, digits and text size from that file's `language.*` keys.
