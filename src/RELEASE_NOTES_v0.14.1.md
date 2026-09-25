# GachaHub v0.14.1 release notes

v0.14.1 is a TypeScript build-fix release for the v0.14 downloader milestone.

## Fixed

- Settings navigation now uses the current `Game` type instead of the removed legacy `GameManifest` type.
- The Tauri `launch_game` invocation now passes a plain invoke-argument object, satisfying the Tauri 2 `InvokeArgs` type.
- Legacy `src/components/Sidebar.tsx` from old in-place upgrades is no longer part of the clean full package. The v0.14.1 patch includes a compatibility shim so old working directories also compile immediately after extraction.

No downloader pause/resume behavior from v0.14.0 was removed or changed by this hotfix.
