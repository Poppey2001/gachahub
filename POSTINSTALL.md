# GachaHub v0.8 – Genshin Post-Install Defaults

v0.8 introduces the first post-install hook. Genshin is the initial implementation.

## Linux profile

After a completed Genshin install, GachaHub can run:

```bash
bash scripts/postinstall/genshin-linux.sh \
  "$HOME/Games/GachaHub/Games/genshin" \
  "$HOME/.local/share/gachahub/prefixes/genshin"
```

The script is intentionally conservative. It:

- detects an existing Proton-family runner from common Steam/compatibilitytools.d locations;
- detects GameMode, Gamescope, MangoHud and umu;
- enables GameMode only when `gamemoderun` is available;
- keeps Gamescope and MangoHud disabled by default;
- keeps mods and XXMI disabled by default;
- does not add anti-cheat bypasses or optional timeout workarounds;
- records the installation and prefix paths.

The Tauri backend embeds this script at compile time, executes it after install, parses its output, and applies the resulting values to the game-specific settings store.

Until the real downloader emits an installation-complete event, the same hook is available manually under:

**Settings → Genshin Impact → Post-Install Standardprofil → Erkennen & übernehmen**

## Windows profile

`scripts/postinstall/genshin-windows.ps1` applies conservative native defaults and records the detected `GenshinImpact.exe` path.
