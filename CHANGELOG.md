# Changelog

## 0.7.0 — Compatibility & All-Game Settings

- Added researched per-game Linux compatibility presets (research date 2026-09-18).
- Added one-click "Empfohlene Linux-Werte anwenden" in every game settings page.
- Added runner candidates, known regression/block notes, optional environment workarounds, and XXMI-specific requirements.
- Added global + per-game Linux extras: Gamescope, GameMode, MangoHud, prevent sleep, FPS limit.
- Added dark native select styling for WebKitGTK/Tauri on Linux.
- Game page now shows current Linux compatibility state and effective runtime/runner.
- Mod library scanner/importer is no longer hard-coded to Genshin; every configured game gets a library.
- XXMI deployment remains enabled only for games with an explicit XXMI backend and allowed mod policy.
- Arknights and GFL2 are now library-only/restricted until a safe dedicated loader adapter exists.
- Generic/library-only mods no longer require a 3DMigoto INI file to be considered valid.
- Added COMPATIBILITY.md documenting the assumptions behind the presets.
