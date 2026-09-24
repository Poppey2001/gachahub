# GachaHub v0.13.2

## Runner selection and launch integration

- Connected the Runner Manager to global and per-game settings.
- Runner selectors now show detected installed runners instead of relying on free-form version text.
- Persist the concrete runner path together with the display version.
- Selecting Proton automatically selects the UMU runtime; selecting Wine automatically selects Wine.
- GachaHub-managed Proton builds are passed to `umu-run` through their exact `PROTONPATH`.
- Runner labels and compatibility recommendations are no longer passed as fake `PROTONPATH` values.
- System/Heroic/Lutris/Bottles Wine runners launch through their detected Wine binary.
- Compatibility presets no longer pretend that a recommended runner is already installed.
- Genshin post-install detection records both runner version and runner path.

## Development stability

- Added a Linux dev preflight for the fixed Vite port 1420 with a clear stale-process diagnostic.
- Added `npm run tauri:dev:stop` for safely stopping a stale GachaHub/Vite listener.
- The Linux dev helper isolates common VS Code Snap GIO/GTK overrides while keeping the NVIDIA/WebKitGTK workaround.
- Removed the unused legacy `Sidebar.tsx` from the full package.

## Compatibility

- Existing persisted settings are migrated to the new runner-path capable settings schema.
- Steam, Heroic, Lutris, Bottles and system runners stay read-only; only GachaHub-managed runner directories can be removed by GachaHub.

# v0.13.1

- Removed Rust compiler warnings in downloader, runner manager, post-install code and provider abstraction.
- Fixed download speed baseline initialization after HTTP resume negotiation.
- Removed unused legacy synchronous InstallPlan API; async ProviderDownloadPlan remains the active download API.

# GachaHub v0.13.0

## Runner Manager

- New Twintail-inspired two-pane Runner Manager.
- Runner families on the left and release versions on the right.
- Real remote release discovery for GE-Proton, DWProton, Proton-CachyOS, Proton-EM, Proton-Sarek and Proton-Wineland.
- Install and remove GachaHub-managed runner versions.
- Mark an installed version as the default runner.
- Detect Valve Proton / Steam runners and UMU-managed Proton without taking ownership of them.
- Managed runner directory: `~/.local/share/gachahub/runners`.
- Archive extraction for zip, tar.gz, tar.xz and tar.zst.
- Existing Steam/Heroic/Lutris/Bottles runners stay read-only.
- Runner UI translated in German, English, French and Spanish.
