# GachaHub Runner Manager — v0.13.2

The Runner Manager is both a version manager and the source of truth for runner selection in global and per-game settings.

## Families

GachaHub can browse or detect:

- GE-Proton
- DWProton
- Proton-CachyOS
- Proton-EM
- Proton-Sarek
- Proton-Wineland
- Valve Proton / Proton Experimental / Proton Hotfix
- UMU-managed Proton
- Wine runners discovered from the system, Heroic, Lutris and Bottles

## Managed install directory

Linux runners installed by GachaHub live under:

`~/.local/share/gachahub/runners/`

Only directories inside this managed root may be removed by GachaHub. Steam, Heroic, Lutris, Bottles and system runners remain read-only.

## Selection model

A selected runner stores two values:

- a human-readable version/name for the UI;
- the concrete detected filesystem path used at launch time.

For Proton/UMU launches, GachaHub passes `PROTONPATH` only when a concrete detected runner path exists. If no path is selected, `umu-run` is allowed to select/manage its runtime rather than receiving a descriptive UI label as a fake path.

For Wine launches, the detected Wine executable itself is stored and launched directly.

Selecting a runner also synchronizes the runtime:

- Proton family -> UMU
- Wine family -> Wine
- native -> Native
- automatic / UMU-managed -> Auto

## Game settings

Each game can override the global runner independently. The selector includes a shortcut back to the Runner Manager and a refresh action to re-scan installed tools.

Compatibility presets only describe recommended runner families. Applying a preset does not claim a specific runner is installed; choose an installed version in the runner selector if you want to pin one.

## Genshin post-install

The Linux Genshin post-install detector scans GachaHub-managed runners before external compatibility-tool directories, stores the selected runner path when it finds one, and infers an existing Wine/Proton prefix from an executable located below `drive_c`.

## Archive support

The managed installer supports ZIP, tar.gz, tar.xz and tar.zst release archives. Remote release assets are filtered to avoid obvious checksum, torrent, source and debug artifacts.
