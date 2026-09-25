# GachaHub Download Integration v0.14

v0.14 turns pause/resume into a real backend download state instead of only a frontend control.

## Direct provider packages

When a provider exposes direct package URLs, GachaHub downloads them in Rust into:

`<game install path>/.gachahub-downloads/`

The active downloader supports:

- `.part` files
- HTTP Range resume
- backend pause/resume
- Pause all / Resume all in the queue
- cancel while preserving partial data
- download speed and byte progress
- Tauri progress events
- package-size validation
- MD5 verification when supplied by the provider

## How pause works

GachaHub does not keep a CDN body stream open indefinitely while paused.

1. The backend marks the job as `paused` and emits the new state immediately.
2. The downloader flushes the current `.part` file.
3. The active HTTP response is dropped.
4. While paused, no further chunks are requested or written.
5. Resume opens a new HTTP Range request from the last committed byte.
6. The speed baseline is reset so the pause duration is not included in the speed calculation.

If the origin does not support HTTP Range, GachaHub safely restarts that package instead of appending a full response to a partial file.

## Restart / cancel recovery

Partial `.part` files are intentionally retained. If GachaHub is closed or a job is cancelled, starting the same game download into the same destination can continue from that partial file when the origin supports Range requests.

## HoYoPlay / Sophon

GachaHub resolves current HoYoPlay package metadata and detects `res_list_url`/Sophon mode. v0.14 does not pretend Sophon metadata is a direct package download. Native resource-list parsing, manifest/chunk assembly, decompression and final-file verification remain the next provider-specific implementation stage.
