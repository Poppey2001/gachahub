# GachaHub v0.11 architecture

## XXMI

```text
React UI
  |
  +-- useXxmiStore
  |     +-- local status
  |     +-- install / update / repair / remove
  |     +-- progress
  |     +-- quick launch
  |
Tauri commands
  |
  +-- xxmi_manager_status
  +-- install_xxmi
  +-- repair_xxmi
  +-- remove_xxmi
  +-- launch_managed_xxmi
  |
Rust XXMI Manager
  |
  +-- official GitHub latest-release API
  +-- Portable ZIP selector
  +-- streaming download
  +-- optional SHA-256 verification
  +-- safe ZIP extraction
  +-- managed state file
  +-- importer detection
  +-- Windows native launch / Linux Wine launch
```

The official XXMI GUI remains responsible for the documented first-time Model Importer installation. Once GIMI/SRMI/ZZMI/WWMI/EFMI/HIMI is detected, GachaHub can use `--nogui --xxmi <IMPORTER>`.

## Settings precedence

```text
manual game value
      >
post-install detection (only fills unset fields)
      >
compatibility/global defaults
```

Game-specific settings are enabled by default in v0.11.

## Game download path

The game provider/downloader is separate from XXMI. Genshin's current provider integration can resolve HoYo package metadata and direct packages; Sophon chunk assembly is still a later milestone.
