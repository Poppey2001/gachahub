# GachaHub architecture (v0.15, pure Rust)

```text
┌───────────────────────────── gachahub (crates/gachahub-app) ─────────────────────────────┐
│  eframe/egui immediate-mode UI                                                            │
│                                                                                           │
│  app.rs        state, message loop, game rail, toasts, debounced config saving            │
│  pages/        game stage · downloads · mods · runners · settings · catalog modal         │
│  theme.rs      palette, Inter fonts, gradient/glow painters, custom widgets               │
│  backend.rs    Tokio runtime + mpsc channel  ──►  Msg  ──►  UI (drained once per frame)   │
└───────────────────────────────────────────┬───────────────────────────────────────────────┘
                                            │ plain function calls / async fns
┌───────────────────────────── gachahub-core (crates/gachahub-core) ───────────────────────┐
│  games / compat      built-in manifests (games.json, compatibility.json)                  │
│  providers           HoYoverse metadata (direct packages), other publishers stubbed       │
│  download_manager    resumable HTTP-Range downloader, pause/cancel, MD5 verification      │
│  launcher            native (Windows) / umu-run · wine (+ gamemode, gamescope, inhibit)  │
│  runners             detect + install GE/DW/CachyOS/EM/Sarek Proton builds                │
│  mod_manager         library ↔ active folder, copy/symlink, archives, conflicts           │
│  xxmi                managed XXMI Launcher install/update + importer detection            │
│  postinstall         per-game detection profiles (Genshin)                                │
│  settings            Config { global, games, library } persisted as JSON                  │
│  events              CoreEvent + EventSink (replaces Tauri's AppHandle::emit)             │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

## Threading model

- The UI thread never blocks. Network work runs as Tokio tasks, filesystem scans and process
  spawns use `spawn_blocking`, file dialogs use `rfd`'s async API.
- Every result is a `backend::Msg` sent through a channel; the sender also calls
  `Context::request_repaint`, so the UI wakes up immediately.
- Long-running jobs (downloads, XXMI install) report progress through an `EventSink`
  closure that forwards `CoreEvent`s the same way.
- Launched games are tracked by a small thread waiting on the child process; when it exits
  the play time is added to the game's settings.

## Settings precedence

```text
manual per-game value  >  post-install detection (only fills unset fields)  >  global defaults
```

Per-game overrides are `Option<_>` fields; `None` means "inherit".
