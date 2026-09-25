# GachaHub v0.14.2

## Upgrade-tree repair

This release fixes an in-place upgrade problem where the v0.14.1 patch could be extracted from inside the project's `src/` directory. That produced a second source tree at `src/src/` while the old files in `src/` remained active.

Symptoms included TypeScript errors mentioning both paths at once, for example:

- `src/components/Sidebar.tsx` still using the removed `library` section.
- `src/components/settings/SettingsNav.tsx` still importing the removed `GameManifest` type.
- `src/lib/tauri.ts` still passing `LaunchGameRequest` directly to Tauri `invoke`.
- corrected files appearing under `src/src/...` and failing their relative imports.

v0.14.2 ships a repair script that moves any accidentally nested `src/src/` payload back into the real `src/`, removes the obsolete legacy `Sidebar.tsx`, and then applies the corrected v0.14.2 payload to the project root.

The v0.14 pause/resume download implementation remains unchanged.
