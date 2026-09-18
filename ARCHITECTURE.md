# GachaHub v0.7 architecture

## Settings resolution

```text
GlobalSettings
      |
      v
GameSettings override
      |
      v
ResolvedGameSettings
      |
      +-- runtime / runner / prefix
      +-- launch args / environment
      +-- gamescope / gamemode / mangohud
      +-- prevent sleep / fps limit
      +-- update / preload
      +-- mod paths / deployment
      +-- XXMI toggle
```

## Compatibility database

`src/data/compatibilityPresets.ts` is intentionally separate from the generic game manifest.
It contains changeable Linux compatibility guidance:

- status
- research date
- recommended runtime
- recommended runner family/version text
- candidate runners
- known regressions / runners to avoid
- optional environment workarounds
- XXMI launch conditions
- mod backend type

This allows compatibility data to be updated without redesigning the launcher.

## Mod backends

```text
Mod Library (all games)
  |
  +-- import folder/ZIP/7z/RAR
  +-- scan/search
  +-- profiles metadata
  |
  +-- XXMI backend (supported games)
  |     +-- Active Mods path
  |     +-- Copy/Symlink deployment
  |     +-- XXMI quick launch
  |
  +-- Library-only backend
        +-- no automatic injection/deployment
```

The Rust scanner is generic. INI validation is only required for the XXMI/3DMigoto families currently configured in GachaHub.
