# GachaHub v0.14.2 repair patch

This patch is specifically for projects showing errors from both `src/...` and `src/src/...`.

The old v0.14.1 patch was applied from the wrong directory, so corrected files landed in a duplicate `src/src/` tree while the old files remained active.

## Apply

From anywhere:

```bash
bash apply-v0.14.2.sh /path/to/gachahub
```

The script creates a backup inside the project, removes the duplicate `src/src/` tree and obsolete `Sidebar.tsx`, then installs the corrected files into the canonical source tree.

Afterwards:

```bash
cd /path/to/gachahub
rm -rf src-tauri/target node_modules/.vite
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri:dev:linux
```
