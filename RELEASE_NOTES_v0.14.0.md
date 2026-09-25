# GachaHub v0.14.0 release notes

v0.14.0 starts the downloader milestone with a reliable pause/resume flow for real provider downloads.

## Pause / resume

- Each active download can be paused and resumed from the Downloads page.
- Pause is a backend state, not only a UI toggle.
- The active HTTP response is closed when the pause reaches the downloader.
- The `.part` file is flushed before pausing.
- Resume starts a fresh HTTP Range request from the exact committed byte offset.
- Long pauses therefore do not rely on keeping a CDN connection alive.
- Download speed is reset after resume so paused time does not distort the speed display.
- MD5 verification also respects pause/cancel checkpoints.
- Cancelling preserves the partial `.part` file so starting the same installation again can reuse it when the origin supports Range requests.

## Queue UI

- Real `paused` job state is emitted by the Rust backend.
- Per-download Pause / Resume controls.
- Pause all / Resume all controls.
- Running, paused and completed counters.
- Paused jobs have a dedicated visual state and show zero transfer speed.
- German, English, French and Spanish strings included.

## Scope

The direct-package downloader is now resumable and pause-aware. Modern HoYoPlay Sophon/chunk manifests are still detected but are not falsely presented as installable direct packages. Native Sophon assembly remains a separate downloader step.
