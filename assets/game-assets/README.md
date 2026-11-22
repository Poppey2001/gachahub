# Game artwork packs

GachaHub loads per-game artwork directly from this directory.

For every game, the recommended files are:

```text
assets/game-assets/<game-id>/   (or ~/.local/share/gachahub/game-assets/<game-id>/)
├── background.webp   # recommended: 2560x1440 or larger
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
