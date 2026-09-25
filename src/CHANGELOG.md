# GachaHub v0.14.1

## Build fixes

- Replaced the removed `GameManifest` type in `SettingsNav` with the current `Game` model.
- Fixed Tauri 2 `InvokeArgs` typing for `launch_game`.
- Added upgrade compatibility for stale legacy `Sidebar.tsx` files when applying the patch over older working trees.

## v0.14 downloader milestone

- Pause and resume direct-package downloads.
- Pause/resume all active downloads.
- HTTP Range recovery from `.part` files.
- Preserve partial files for later continuation.
- Reset speed accounting after a resume.
- Localized pause/resume controls for DE / EN / FR / ES.
