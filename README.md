# GachaHub v0.13.2

Cross-platform Gacha launcher for Windows and Linux built with Tauri 2, React, TypeScript and Rust.

The v0.13.2 release completes the current Runner Manager milestone: detected runners can be installed/managed, selected globally or per game, persisted with their real filesystem path and used by the launch pipeline.

## Current highlights

- Flexible supported-game library and Twintail-inspired game rail.
- Existing Genshin installation import and executable/prefix detection.
- Real launch pipeline with native Windows, Wine and UMU/Proton handling.
- Runner Manager for GE-Proton, DWProton, Proton-CachyOS, Proton-EM, Proton-Sarek, Proton-Wineland, Valve Proton and UMU-managed Proton.
- Global and per-game runner selectors use detected installed versions and keep the exact runner path.
- Integrated XXMI manager and game-specific XXMI importers where an adapter exists.
- Mod library/profiles with explicit ownership safeguards.
- Download core with resume/progress/verification groundwork.
- Deutsch, English, Français and Español UI infrastructure.
- Linux/NVIDIA WebKitGTK development helper and stale Vite-port diagnostics.

## Linux development

```bash
npm install
npm run tauri:dev:linux
```

If an earlier development session still owns Vite port 1420:

```bash
npm run tauri:dev:stop
npm run tauri:dev:linux
```

After an upgrade that changes Tauri/Rust commands, clear the Rust build cache once:

```bash
rm -rf src-tauri/target
npm run tauri:dev:linux
```

## Build

```bash
npm run build
npm run tauri:build
```

## Important scope note

v0.13.2 finishes the current runner/settings/launch integration milestone. It is not the final GachaHub release: the remaining large downloader milestone is the full modern chunk/Sophon-style game installation pipeline and provider completion for additional games.

See `RUNNERS.md`, `DOWNLOADS.md`, `XXMI.md`, `I18N.md` and `ARCHITECTURE.md` for subsystem details.
