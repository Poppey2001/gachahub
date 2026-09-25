# GachaHub v0.14.2

## Upgrade repair

- Added repair for accidental `src/src/` trees created by extracting an in-place patch from inside `src/`.
- Removed obsolete legacy `src/components/Sidebar.tsx` from the clean source tree.
- Keeps the corrected `Game` settings model and Tauri 2 launch invoke arguments in the canonical `src/` tree.
- Added `scripts/repair-source-tree.sh` for already-contaminated working directories.
- Keeps all v0.14 download pause/resume functionality.

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
