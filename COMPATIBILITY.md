# GachaHub compatibility baseline — 2026-09-18

This file documents the compatibility assumptions used by the v0.7 UI presets. These are not official Linux support statements from the game publishers. They are launcher/community compatibility observations and should be easy to update independently of the launcher core.

## Genshin Impact
- Windows: native.
- Linux: playable via current Proton-family runners according to current community/Twintail usage.
- Recommended approach: recent DWProton, Proton-CachyOS, Proton Hotfix or GE-Proton.
- Optional troubleshooting variable: `WINE_ENABLE_TIMEOUT_FIX=1` for token/timeout crashes; do not force globally.
- Mods: XXMI / GIMI.

## Honkai: Star Rail
- Windows: native.
- Linux: current Twintail releases report compatibility again across current Proton families.
- Older pinned Proton 10 builds had regressions; keep runner switching available.
- Mods: XXMI / SRMI.

## Zenless Zone Zero
- Windows: native.
- Linux: generally playable, but current 3.2-era community reports include launch regressions on some systems; keep the runner/rendering mode configurable.
- Mods: XXMI / ZZMI.
- ZZMI condition when auto-configuration is disabled: Character Quality = High; High-Precision Character Animation = Disabled.

## Wuthering Waves
- Windows/Steam: native Windows game.
- Linux: playable with current Proton builds; older media/webview issues make recent runners preferable.
- Mods: XXMI / WWMI.
- WWMI launch condition: `-DisableModule=streamline -dx11 -d3d11`.

## Arknights: Endfield
- Windows: native.
- Linux: playable with Proton builds carrying the required fixes. Current Twintail guidance explicitly warns against Proton Vanilla and Proton UMU for Endfield.
- GE-Proton 10-30 included Endfield-specific upstream patches; current DWProton/CachyOS builds are also candidates.
- Mods: XXMI / EFMI exists, but GachaHub keeps the game policy restricted/manual.

## Arknights PC
- Windows: native PC client.
- Linux: community reports show the new PC client working with GE-Proton and Proton Experimental; Twintail added original Arknights in its 2026 releases.
- Mods: no XXMI importer in the current XXMI list; GachaHub provides library-only management until a dedicated adapter exists.

## GIRLS' FRONTLINE 2: EXILIUM
- Windows/Steam: native Windows game.
- Linux: playable through Proton, but runner regressions exist.
- Known-good references include Proton 10.0-4 / Proton Experimental; GE-Proton 10-34 was reported as a video-working fallback when GE 11.1–11.3 regressed video playback.
- The launcher can require WebView2 in its prefix.
- Optional Steam workaround seen in compatibility notes: `UMU_ID=0`.
- Mods: no XXMI importer; library-only until a dedicated adapter exists.

## Mod backend policy
- `xxmi`: Genshin (GIMI), HSR (SRMI), ZZZ (ZZMI), Wuthering Waves (WWMI), Endfield (EFMI).
- `library-only`: Arknights, GFL2. Import/search/profile organization is allowed, but GachaHub does not pretend it has a working injection/deployment backend.
- GachaHub does not implement anti-cheat bypasses.

## Research references
- TwintailLauncher releases and supported-games documentation (2026 releases).
- XXMI Launcher README/wiki and importer package configuration.
- Valve Proton compatibility issues for Wuthering Waves and GFL2.
- GE-Proton issue/release notes for Endfield and GFL2.
- Recent community reports for Arknights PC and Genshin on Linux.
