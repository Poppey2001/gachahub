# GachaHub v0.15.0 — Rust rewrite

## Komplett auf Rust umgebaut

- Das React/TypeScript-Frontend und Tauri wurden entfernt. GachaHub ist jetzt eine
  native Rust-App (egui/eframe) ohne Node.js, WebView oder npm.
- Backend-Logik als eigenständige Bibliothek `gachahub-core` (Cargo-Workspace).
  Fortschritts-Events laufen über ein `EventSink` statt `AppHandle::emit`.
- Einstellungen werden als JSON im Konfigurationsordner gespeichert statt im `localStorage`.

## Neu: Neverness to Everness (NTE)

- NTE im Katalog (Hotta Studio / Perfect World), Import einer bestehenden Installation
  mit automatischer EXE-Erkennung.

## Spiel-Icons

- Eigene SVG-Icons für alle 8 Spiele in Seitenleiste, Katalog, Mods, Downloads und Einstellungen.
- Offizielles Icon wird automatisch aus der installierten Spiel-EXE extrahiert und zwischengespeichert.

## Windows

- Automatische UAC-Elevation, wenn ein Spiel Administratorrechte verlangt (Spielzeit wird weiter erfasst).
- Windows-Standardpfade (`%LOCALAPPDATA%\GachaHub`), App-Icon in der EXE, kein Konsolenfenster.
- CI baut und testet unter Windows und Linux.

## Neue Oberfläche

- Animierte, farbige Spiel-Bühne mit Glow, Partikeln und großem Gradient-Play-Button.
- Seitenleiste mit Spiele-Bibliothek (Sortieren/Entfernen per Rechtsklick) und Katalog-Dialog.
- Überarbeitete Seiten für Downloads (inkl. ETA), Mods (Toggles, Suche), Runner und Einstellungen.
- Inter-Schrift, Toast-Benachrichtigungen, Deutsch/Englisch umschaltbar.
- Spielzeit-Tracking und automatische EXE-Erkennung beim Import bestehender Installationen.

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

## v0.14.3
- Added project-wide i18n audit script with strict missing-translation, fallback, placeholder, duplicate, unused-key and hard-coded UI checks.
