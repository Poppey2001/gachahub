# GachaHub v0.13.2 release notes

This release stabilizes the v0.13 Runner Manager and connects it to the actual launch configuration.

### Fixed

- A runner selected in the manager/settings now keeps its real path instead of only its label.
- Proton launch no longer sends recommendation text such as "current GE-Proton" as `PROTONPATH`.
- Choosing a runner updates the matching runtime automatically.
- Wine runner selections use the detected Wine executable.
- Genshin post-install detection persists the detected runner path.
- Linux development reports port 1420 conflicts before Vite/Tauri fail and provides a safe stop helper.
- Common VS Code Snap GIO environment leakage is sanitized by the Linux dev helper.

### Upgrade

Apply the patch over v0.13.1, remove the old unused `src/components/Sidebar.tsx`, clear `src-tauri/target`, then rebuild.
