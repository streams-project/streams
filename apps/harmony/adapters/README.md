# adapters

The five adapters of this family. Each one plugs into a port in `../ports/Ports.ets`, and the app
wiring is the only place that knows both sides, as in `apps/android/app/.../Main.kt`.

**None of them is written.** This file says which HarmonyOS API each one is expected to use and how
sure that is, so the next person starts from a list and not from a search. Status words: **verified**
(read off real output on a device), **noted** (a project note cites a document for it), **assumed**
(from general knowledge, nothing in this repository checked it).

| Port | Expected API | Status |
|---|---|---|
| `Network` | `@ohos.net.http` for the call, `@ohos.net.connection` for `watch` | assumed |
| `Secrets` | the Asset Store (`@ohos.security.asset`) for tokens and the server address | assumed |
| `Store` | plain files under the ability context's own folder (`@ohos.file.fs`) | assumed |
| `Player` | `AVPlayer` from `@ohos.multimedia.media`, with per source headers through `createMediaSourceWithUrl` | noted: project note 151 (outside this repository), line 450, which cites an undated Huawei guide |
| `Reel` | a second `AVPlayer`, released when the title player wants the decoder | assumed |

Two rules carry over from the families that are written, and both cost a viewer their session if they
are got wrong:

- `Secrets.put` returns false when the store refused the write. It must not report a write it did not
  do: a session token that was never written signs the viewer out at the next launch with no signal.
- The `Player` reports facts and decides nothing. No adapter retries, re-ranks or picks a source.
