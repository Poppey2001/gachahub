# Managed XXMI integration

GachaHub v0.11 treats XXMI as a managed launcher component.

## Flow

```text
Game supports XXMI
        |
        v
XXMI installed?
  | no          | yes
  v             v
Download        Importer installed?
official ZIP      | no       | yes
  |               v          v
  v             Open XXMI    Toggle / Quick Launch
Extract          GUI once    --nogui --xxmi <IMPORTER>
  |
  v
Managed XXMI
```

GachaHub downloads the Portable ZIP from the official `SpectrumQT/XXMI-Launcher` GitHub Releases endpoint. XXMI is not bundled into the GachaHub source archive.

## Managed paths

Default Linux path:

```text
~/.local/share/gachahub/tools/xxmi
```

Default Windows path:

```text
%LOCALAPPDATA%\GachaHub\tools\xxmi
```

A custom managed location can be configured. Existing XXMI installations can still be selected through the optional custom Launcher path.

## Importers

- GIMI — Genshin Impact
- SRMI — Honkai: Star Rail
- ZZMI — Zenless Zone Zero
- WWMI — Wuthering Waves
- EFMI — Arknights: Endfield
- HIMI — Honkai Impact 3rd

The official XXMI application currently documents the first installation of a Model Importer through its GUI. GachaHub therefore opens XXMI for the one-time importer setup instead of copying internal XXMI package-management code or depending on undocumented command-line behavior. Once the importer is present, GachaHub can use Quick Launch.

## Linux

XXMI Portable is launched through the configured Wine executable. The upstream project recommends Wine 9.22 or newer for its Portable Linux setup.

## Update / repair

Managed XXMI installations support:

- check latest stable release
- update
- repair/reinstall core Portable files
- remove managed installation

Repair merges the official Portable files into the managed install so user-created/importer data that is not part of the Portable archive is not proactively deleted.
