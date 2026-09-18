# GachaHub v0.7.0

Cross-platform Gacha launcher prototype built with Tauri 2, React, TypeScript and Rust.

## v0.7 focus

This release turns the settings system into a compatibility-oriented launcher configuration layer.
Each current game has a researched Linux preset and its own runtime/runner overrides.

### Current games
- Arknights
- Arknights: Endfield
- Genshin Impact
- Honkai: Star Rail
- Zenless Zone Zero
- Wuthering Waves
- GIRLS' FRONTLINE 2: EXILIUM

### New compatibility settings
Per game:
- Linux compatibility summary
- recommended runtime/runner
- candidate runners
- known bad/regressed runner notes
- one-click recommended preset
- launch arguments
- environment variables
- Gamescope
- GameMode
- MangoHud
- prevent sleep
- FPS limit
- optional troubleshooting workarounds
- XXMI-specific conditions where applicable

Global defaults are still inherited until a game override is enabled.

## Mod Manager model

v0.7 splits mod handling into two backend levels:

- **XXMI backend:** Genshin/GIMI, HSR/SRMI, ZZZ/ZZMI, Wuthering/WWMI, Endfield/EFMI.
- **Library-only:** Arknights and GFL2. Archive/folder import, search and organization work, but GachaHub does not pretend to have a safe automatic loader/deployment adapter yet.

No anti-cheat bypass is implemented.

## Development

```bash
npm install
npm run tauri:dev:linux
```

If Tauri permissions/build output is stale after patching:

```bash
rm -rf src-tauri/target
npm run tauri:dev:linux
```

For 7z/RAR mod import on Fedora/Nobara:

```bash
sudo dnf install 7zip
```

See `COMPATIBILITY.md` for the compatibility baseline used by the presets.
