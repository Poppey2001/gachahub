# Game artwork packs

GachaHub v0.4 can load per-game artwork directly from this directory.

For every game, the recommended files are:

```text
public/game-assets/<game-id>/
├── background.webp   # recommended: 2560x1440 or larger
├── background.mp4    # optional animated background (muted/looped)
├── logo.webp         # transparent game logo
└── icon.webp         # square game icon, recommended 512x512
```

Known IDs:

- `arknights`
- `endfield`
- `genshin`
- `hsr`
- `zzz`
- `wuwa`
- `gfl2`

Missing files are safe: the UI falls back to the built-in gradient/initials design.

For repository releases, only include artwork that you have permission to redistribute.
