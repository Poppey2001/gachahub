# Apply GachaHub v0.14.1 over v0.14.0 / older in-place trees

Extract this archive into the GachaHub project root, replacing existing files.
The patch contains a compatibility `src/components/Sidebar.tsx` so old source trees no longer fail on the removed `library` navigation section.

Recommended verification:

```bash
rm -rf src-tauri/target
npm install
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```
