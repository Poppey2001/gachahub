# GachaHub

**Ein Launcher für alle Gacha-Spiele – komplett in Rust.**
Schluss mit „download every gacha launcher on earth“: Genshin Impact, Honkai: Star Rail,
Zenless Zone Zero, Wuthering Waves, Neverness to Everness (NTE), Arknights, Arknights: Endfield und GIRLS' FRONTLINE 2
in einer einzigen, nativen App.

Ab v0.15 gibt es kein Web-Frontend mehr: Kein Node.js, kein WebView, kein Tauri. Die
Oberfläche ist eine native GPU-gerenderte Rust-GUI (egui/eframe), die Logik eine
eigenständige Rust-Bibliothek.

## Features

- **Bibliothek in der Seitenleiste** – Spiele per „+“ aus dem Katalog hinzufügen, per
  Rechtsklick sortieren/entfernen; laufende Spiele bekommen einen grünen Punkt.
- **Spielseite** – animierter Hintergrund in den Farben des Spiels (oder eigenes Artwork),
  großer *Spielen/Installieren*-Button, Live-Fortschritt, Status, Laufzeit und Spielzeit.
- **Downloads** – fortsetzbar (HTTP Range), Pause/Abbrechen, MD5-Prüfung, ETA.
- **Vorhandene Installation importieren** – Ordner wählen, die Spiel-EXE wird automatisch gefunden.
- **Runner (Linux)** – installierte Wine/Proton-Builds erkennen, GE-Proton, DW-Proton,
  CachyOS-Proton u. a. direkt herunterladen und entfernen.
- **Mods** – Bibliothek getrennt vom Spiel, Aktivieren per Kopie oder Symlink, ZIP/7z/RAR-Import,
  Konflikterkennung.
- **XXMI** – XXMI Launcher automatisch installieren/aktualisieren, Spiele mit GIMI/SRMI/ZZMI/WWMI/EFMI starten.
- **Spielzeit-Tracking**, **Deutsch/Englisch**, Einstellungen global und pro Spiel.

## Windows & Linux

- **Windows**: Spiele starten nativ. Verlangt ein Spiel Administratorrechte (Anti-Cheat),
  fragt GachaHub automatisch per UAC nach. Die `gachahub.exe` enthält das App-Icon und
  öffnet kein Konsolenfenster.
- **Linux**: Start über `umu-run`/Proton oder Wine, optional mit GameMode, Gamescope, MangoHud.

Spiele ohne eigenen GachaHub-Downloader (z. B. NTE, Wuthering Waves, Arknights) installierst
du einmal mit dem offiziellen Launcher und klickst dann auf **Spiel importieren** – die
Spiel-EXE wird automatisch gefunden.

## Download & Installation

Fertige Builds gibt es unter **Releases** auf GitHub.

**Windows:** `gachahub.exe` herunterladen und starten – portabel, keine Installation und
kein Rust nötig. (SmartScreen-Warnung: „Weitere Informationen“ → „Trotzdem ausführen“.)

**Linux** (x86_64), eine Zeile im Terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/Poppey2001/gachahub/main/packaging/linux/get.sh | bash
```

Oder `gachahub-linux-x86_64.tar.gz` herunterladen, entpacken und `./install.sh` ausführen
(`sudo ./install.sh --system` für alle Benutzer). GachaHub erscheint danach im Anwendungsmenü.
Deinstallieren: `~/.local/share/gachahub/uninstall.sh`.

### Neues Release erstellen

```bash
git tag v0.15.0 && git push origin v0.15.0
```

Der Workflow *Release* baut dann EXE + Linux-Paket und veröffentlicht sie als GitHub-Release.
Manuell gestartet (Actions → Release → *Run workflow*) landen die Dateien als Artefakte am Lauf.

## Selbst bauen

Voraussetzung: [Rust](https://rustup.rs) (stable).

```bash
cargo run --release
```

Das Binary liegt danach unter `target/release/gachahub` (Windows: `target\release\gachahub.exe`).
Unter Windows reicht die Rust-Installation mit den MSVC Build Tools (Visual Studio Installer →
„Desktopentwicklung mit C++“).

Unter Linux werden zur Laufzeit die üblichen Grafik-/Fenster-Bibliotheken
(X11 oder Wayland, OpenGL) benötigt – auf jedem Desktop-System sind diese vorhanden.
Für Datei-Dialoge wird das XDG-Desktop-Portal verwendet.

Tests:

```bash
cargo test --workspace
```

## Projektstruktur

```text
crates/
├── gachahub-core/   # Logik: Provider, Downloads, Runner, Mods, XXMI, Einstellungen
│   ├── game-manifests/   # games.json + compatibility.json
│   └── postinstall/      # Post-Install-Profile (Genshin)
└── gachahub-app/    # Native GUI (egui/eframe), Binary „gachahub“
    └── assets/           # Icon + Inter-Schrift (SIL OFL)
assets/
├── branding/        # App-Icon
└── game-assets/     # optionale Artworks pro Spiel
```

## Spiel-Icons

Jedes Spiel hat ein eigenes GachaHub-Icon (`assets/game-icons/*.svg`, eigene Gestaltung,
keine offiziellen Logos). Sobald ein Spiel installiert/importiert ist, liest GachaHub das
**offizielle Icon direkt aus der Spiel-EXE** aus (funktioniert auch unter Linux) und zeigt
dieses an. Reihenfolge: eigenes Artwork › Icon aus der EXE › GachaHub-Icon.

## Eigene Artworks

Lege Bilder unter `~/.local/share/gachahub/game-assets/<spiel-id>/` (oder `assets/game-assets/`) ab:
`background.webp|png|jpg`, `icon.*`, `logo.*`. Ohne Artwork nutzt GachaHub einen
generierten Hintergrund in den Akzentfarben des Spiels.

## Einstellungen

Alle Einstellungen werden als JSON gespeichert:
Linux `~/.config/gachahub/config.json`, Windows `%APPDATA%\gachahub\config\config.json`.

Weitere Doku: [ARCHITECTURE.md](ARCHITECTURE.md), [RUNNERS.md](RUNNERS.md),
[XXMI.md](XXMI.md), [COMPATIBILITY.md](COMPATIBILITY.md).
