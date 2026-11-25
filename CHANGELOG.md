# GachaHub v0.19.0 (in Arbeit)

## Spiel-EXE zuverlässig erkennen (alle Spiele)
- Die EXE wird jetzt in drei Stufen gesucht (bis 8 Ordner tief): bekannter Dateiname → **Versionsinfos
  der EXE** (Produktname, Beschreibung, Originaldateiname – bleiben über Updates gleich, anders als
  ein Hash) → größte plausible EXE. Hinterlegt für alle 15 Spiele.
- Genshin-Import unter Windows ohne PowerShell-Skript (das nur `GenshinImpact.exe` kannte und von
  Virenscannern als verdächtig gewertet werden kann).
- Wird nichts gefunden, nennt die Meldung die Zahl der geprüften EXE-Dateien, und ein Button
  „Spiel-EXE selbst auswählen“ erscheint direkt darunter.

## Icons unter Windows
- Spiel-Icons aus der EXE und eigene Artworks wurden unter Windows nicht geladen (rotes ▲ in der
  Seitenleiste): die Datei-Adresse war `file://C:\…` statt `file:///C:/…`. Behoben; schlägt ein Bild
  trotzdem fehl, zeigt GachaHub sein eigenes Spiel-Icon.

## Installer
- Der Setup beendet laufende GachaHub-Instanzen selbst und schiebt die alte `gachahub.exe` vor dem
  Kopieren zur Seite – behebt „MoveFile schlug fehl; Code 5. Zugriff verweigert“ beim Update über eine
  bestehende Installation. Im CI wird jetzt auch ein Update über eine laufende Installation getestet.
- Alte Startmenü-/Desktop-Verknüpfungen werden vor der Installation entfernt und neu angelegt.

# GachaHub v0.18.0 — Steam, Wuthering Waves & neue Spiele

## Steam-Integration
- GachaHub liest die lokalen Steam-Bibliotheken (`libraryfolders.vdf`, `appmanifest_*.acf`) und
  **erkennt installierte Steam-Spiele automatisch** – sie landen mit Pfad und EXE in der Bibliothek.
- Steam-Spiele starten **über Steam** (Spielzeit wird über den Spielprozess erfasst).
- **„Über Steam installieren“** öffnet das Spiel im Steam-Client; danach sucht GachaHub
  zwei Stunden lang automatisch nach der neuen Installation.
- Gilt u. a. für GFL2, Wuthering Waves, Blue Archive, Limbus Company.

## Spanisch & Französisch
- GachaHub gibt es jetzt auf **Deutsch, Englisch, Spanisch und Französisch** – umschaltbar über 🌐 oben rechts
  oder unter *Einstellungen → Oberfläche → Sprache*. Übersetzt sind alle Texte der Oberfläche sowie
  Spielbeschreibungen und Kompatibilitätshinweise; ein Test stellt sicher, dass kein Text fehlt.

## Selbst-Update ohne Antivirus-Fehlalarm
- Der Updater kopiert sich unter Windows nicht mehr nach `%TEMP%` und startet keine Hilfsprozesse
  mehr (das Muster „Kopie in Temp + Selbstlöschung“ wurde z. B. von Bitdefender als verdächtig blockiert).
  Die alte EXE wird jetzt nur neben der neuen in `gachahub.exe.old` umbenannt und beim nächsten Start gelöscht.

## GFL2 direkt herunterladen: Darkwinter & Haoplay
- Beim Installieren von GFL2 wählst du die Version: **Global (Darkwinter)** oder **Asien (Haoplay)**.
- GachaHub lädt das Spiel **selbst** von den offiziellen Servern: Launcher-Konfiguration
  (`flexible_config`), Client-Paket `GF2_Exilium_Origin.zip` und alle Asset-Bundles aus der
  `Version.txt` – mit MD5-Prüfung, Fortsetzen, Updates (nur geänderte Dateien) und „Dateien prüfen“.
  Sprachpakete lädt das Spiel selbst; GachaHub behält sie bei Updates.
- Jede Version bekommt einen eigenen Ordner, ein Wechsel ist jederzeit über ••• möglich.
- Alternativ: offiziellen Installer über GachaHub starten (danach automatische Erkennung) oder Steam.

## Wuthering Waves direkt herunterladen
- Neuer Kuro-Downloader: Dateiliste aus dem offiziellen Launcher-Index, paralleler Download mit
  MD5-Prüfung und Fortsetzen; derselbe Weg macht Updates (nur geänderte Dateien) und „Dateien prüfen“.
- Unterstützt das aktuelle Index-Format mit Ressourcen-Paketen (Qualität HD wie der offizielle Launcher,
  Start mit `-krqlv=hd`), gzip-komprimierte Indexdateien und mehrere CDNs mit Ausweich-Server.

## Live-Prüfung gegen die echten Server
- Neuer Workflow *Live check*: prüft bei Änderungen und wöchentlich, ob die Downloader noch zu den
  Servern von Sunborn/Darkwinter, Haoplay und Kuro passen (Konfiguration, Dateiliste, Client-Paket,
  Stichprobe mit MD5).

## Neue Spiele
- Blue Archive, Limbus Company, Punishing: Gray Raven, Snowbreak: Containment Zone,
  Reverse: 1999, Tower of Fantasy – jeweils mit eigenem Icon.
- Katalog mit Suchfeld und Scrollen.

# GachaHub v0.17.0 — Installer & Verbesserungen

## Windows-Installer
- Neuer **`gachahub-setup.exe`**: installiert nach `%LOCALAPPDATA%\Programs\GachaHub` (ohne Adminrechte),
  mit Startmenü-Eintrag, optionaler Desktop-Verknüpfung und Deinstallation über „Apps & Features“.
  Der Selbst-Updater funktioniert in der installierten Version weiter.
- Der Release-Workflow baut den Installer und testet stille Installation + Deinstallation automatisch.

## Spielseite
- **Spiel beenden** (⏹) für laufende Spiele.
- Menü **•••**: Spielordner öffnen, Spiel-Einstellungen, **Deinstallieren**. Gelöscht werden nur Ordner,
  die GachaHub selbst installiert hat – fremde Installationen werden nur aus GachaHub entfernt.

## Spielzeit
- Verfolgt jetzt das eigentliche Spiel, auch wenn ein Starter (umu-run, Launcher, XXMI) sich vorher beendet.

## Mods
- **Mod-Profile**: aktuelles Setup unter einem Namen speichern und später mit einem Klick anwenden.
- **Import aus dem GIMI Mod Manager**: Bibliotheks- und Mods-Ordner aus dessen Konfiguration übernehmen.

## Weiteres
- **Honkai Impact 3rd** im Katalog (Download über HoYoPlay, XXMI/HIMI).
- Spieltexte und Kompatibilitätshinweise auf Deutsch und Englisch.
- Mehr Tests: EXE-Erkennung, Mod-Manager (Aktivieren, Deaktivieren, Profile, Konflikte), Prozesserkennung.

# GachaHub v0.16.0 — Installieren, Updaten, Reparieren

## GachaHub aktualisiert sich selbst
- Neuer **Auto-Updater**: prüft beim Start die GitHub-Releases, lädt die neue Version,
  prüft die SHA-256-Prüfsumme und ersetzt die laufende EXE (Windows & Linux). Ein Klick auf
  „Neu starten“ aktiviert sie. Einstellungen → Allgemein → GachaHub.
- **Update-Kanal** wählbar: „Stabil“ oder „Vorabversionen“.
- Jeder Push auf `main` erzeugt automatisch ein **Pre-Release** (`vX.Y.Z-pre.N`); die 5 neuesten bleiben erhalten.

## Spiele wirklich installieren
- **Sophon-Downloader** für Genshin Impact, Honkai: Star Rail und ZZZ (HoYoPlay-Chunk-Verteilung):
  Dateien werden aus MD5-geprüften, zstd-komprimierten Chunks zusammengesetzt, mehrere parallel.
- **Updates**: Beim Start wird die neueste Version abgefragt. Ein Update lädt nur geänderte Dateien;
  „Automatische Updates“ (global oder pro Spiel) installiert es direkt.
- **Dateien prüfen**: vergleicht alle Spieldateien mit dem Manifest und lädt beschädigte neu.
- **Direkte Pakete** werden nach dem Download entpackt (auch aufgeteilte `.zip.001…`, `.7z` über 7-Zip),
  der Download-Cache wird gelöscht und die Spiel-EXE automatisch eingetragen.
- Abgebrochene Installationen lassen sich mit **„Fortsetzen“** weiterführen.
- Netzwerkfehler werden bis zu 5× automatisch wiederholt; einstellbare Parallelität und Bandbreitenlimit.
- Sprachpaket (Englisch, Japanisch, Chinesisch, Koreanisch) wählbar.

## Fehlerbehebungen
- HoYoverse-Downloads schlugen am Ende wegen einer falschen Größenprüfung fehl (entpackte statt Download-Größe).

## Runner
- Proton-Archive werden auf die Festplatte gestreamt statt komplett in den RAM geladen,
  mit Fortschrittsanzeige und SHA-512-Prüfung (sofern veröffentlicht). Entpacken blockiert nichts mehr.

## XXMI
- Startet unter Linux mit dem Proton/Runner und Prefix des jeweiligen Spiels (statt eines separaten Wine-Setups).
- „Ohne GUI starten“ wird jetzt beachtet; Spielzeit wird auch beim Start über XXMI erfasst.

## Aufgeräumt
- Einstellungen ohne Funktion entfernt (Download-Ordner, Preloads, Update-Kanal, Prüfen vor Start,
  Debug-Log, Mod-Profil, Mods standardmäßig aktiv, XXMI-Wine-Pfade).
- Einheitlicher User-Agent; neue Tests (Sophon-Install/Update/Reparatur und Direkt-Download gegen lokalen Testserver).

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

## Releases

- Release-Workflow baut bei jedem `v*`-Tag eine Windows-EXE (+ ZIP) und ein Linux-Paket
  mit `install.sh`/`uninstall.sh`, Desktop-Eintrag und Icon und veröffentlicht sie als GitHub-Release.
- Linux-Ein-Zeilen-Installer: `packaging/linux/get.sh`.

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
