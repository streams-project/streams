# adapters

The six adapters of this family. Each one plugs into a port in `../ports/Ports.ets`, and the app
wiring is the only place that knows both sides, as in `apps/android/app/.../Main.kt`.

**None of them is written.** This file says which HarmonyOS API each one uses and how sure that is,
so the next person starts from a list and not from a search. Status words: **verified** (read off
Huawei's own reference, with the API version it arrived in), **noted** (a project note cites a
document for it), **assumed** (general knowledge, nothing checked).

Every `verified` row below was read on 2026-11-12 from `developer.huawei.com/consumer/en/doc/
harmonyos-references` or the OpenHarmony documentation it is generated from.

| Port | API | Status |
|---|---|---|
| `Network` | `http.createHttp().request(url, options)` from `@kit.NetworkKit`, module since API 6. `HttpRequestOptions` carries `method`, `header`, `extraData`, `connectTimeout` and `readTimeout` (both default 60000 ms) and `expectDataType`. Needs `ohos.permission.INTERNET`. For `watch`: `connection.createNetConnection()` then `on('netAvailable')` / `on('netLost')` **before** `register()`, which needs `ohos.permission.GET_NETWORK_INFO` | **verified** |
| `Secrets` | the Asset Store: `asset.add/query/remove/update` (and the `*Sync` forms, API 12) from `@kit.AssetStoreKit`, module since API 11. Its own description names "passwords, **app tokens**, and other critical data", AES256-GCM with the keys in a TEE. Tags `SECRET`, `ALIAS`, `ACCESSIBILITY`, `RETURN_TYPE` | **verified** |
| `Store` | `fileIo` from `@kit.CoreFileKit`, module since API 9: `openSync`, `writeSync`, `readTextSync`, `listFileSync`, `unlinkSync`, `accessSync`. The folder is `context.filesDir` on the ability's `Context` | **verified** |
| `Player` | `AVPlayer` from `@kit.MediaKit`, module since API 9, with per-source headers through `media.createMediaSourceWithUrl(url, headers)` + `setMediaSource(src, strategy?)`, both **API 12**. Its nine states are `AVPlayerState`. The picture goes to `avPlayer.surfaceId`, taken from an `XComponent`'s `onSurfaceCreated(surfaceId)` | **verified**; that headers reach the wire is **noted** (project note 151, line 450) and is the one device test Q5 leaves |
| `Reel` | a second `AVPlayer`, released when the title player wants the decoder | **assumed** |
| `Pictures` | `http.createHttp().request(url, { expectDataType: ARRAY_BUFFER })` for the bytes, then `image.createImageSource(buf)` and `createPixelMap({ desiredSize })` from `@kit.ImageKit` (the `ArrayBuffer` form of `createImageSource` is API 9). `Painted.picture` is the `image.PixelMap` | **verified** |

## Three traps the reference states outright

Each one costs a viewer something, and each is a sentence in the documentation rather than a thing to
find out on a device.

* **`asset.query` throws when nothing matches.** "an exception indicating that no asset is found is
  thrown instead of an empty query result list." A `Secrets.get` that let that exception out would
  take the app down on a first run. And the secret itself is capped at **1024 bytes**, which a
  session token fits and a kept page does not — which is why the store port is files.
* **`createPixelMapSync` blocks the thread it is called on**: "It should not be invoked from the main
  thread." The `Pictures` adapter decodes off the UI thread or uses the async `createPixelMap`. From
  API 15 the reference recommends `createPixelMapUsingAllocator` instead.
* **A `PixelMap` has to be released.** "Images occupy a large amount of memory. When you finish using
  a … instance, call release to free the memory promptly." There is no `releaseSync`. A surface that
  holds a tile's picture past the tile is a leak the `Pictures` port's `Stop` exists to prevent.

## Two rules that carry over from the families that are written

Both cost a viewer their session if they are got wrong:

- `Secrets.put` returns false when the store refused the write. It must not report a write it did not
  do: a session token that was never written signs the viewer out at the next launch with no signal.
- The `Player` reports facts and decides nothing. No adapter retries, re-ranks or picks a source.

## And one that is this family's own

`Pictures.fetch` answers once, and a read that was stopped answers neither way. `Art` in
`../surfaces/touch/Pictures.ets` holds the `Stop` and calls it when the tile goes off screen or its
url changes under it, so an adapter that answered a stopped read would be writing a picture into a
node that is gone.
