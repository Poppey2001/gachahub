#!/usr/bin/env python3
"""Import local artwork into GachaHub's bundled game-asset layout.

Examples:
  python3 scripts/import-game-art.py genshin --background ~/Pictures/genshin.png
  python3 scripts/import-game-art.py arknights --logo ./logo.png --icon ./icon.png
  python3 scripts/import-game-art.py endfield --video ./loop.mp4
"""
from argparse import ArgumentParser
from pathlib import Path
from shutil import copy2
from PIL import Image

GAME_IDS = {'arknights', 'endfield', 'genshin', 'hsr', 'zzz', 'wuwa', 'gfl2'}

parser = ArgumentParser()
parser.add_argument('game_id', choices=sorted(GAME_IDS))
parser.add_argument('--background', type=Path)
parser.add_argument('--logo', type=Path)
parser.add_argument('--icon', type=Path)
parser.add_argument('--video', type=Path)
args = parser.parse_args()

root = Path(__file__).resolve().parents[1]
out = root / 'public' / 'game-assets' / args.game_id
out.mkdir(parents=True, exist_ok=True)


def image_to_webp(source: Path, target: Path, *, square: bool = False) -> None:
    if not source.exists():
        raise SystemExit(f'File not found: {source}')
    img = Image.open(source).convert('RGBA')
    if square:
        side = min(img.size)
        left = (img.width - side) // 2
        top = (img.height - side) // 2
        img = img.crop((left, top, left + side, top + side)).resize((512, 512), Image.Resampling.LANCZOS)
    img.save(target, 'WEBP', quality=92, method=6)
    print(f'Imported {source} -> {target}')

if args.background:
    image_to_webp(args.background.expanduser(), out / 'background.webp')
if args.logo:
    image_to_webp(args.logo.expanduser(), out / 'logo.webp')
if args.icon:
    image_to_webp(args.icon.expanduser(), out / 'icon.webp', square=True)
if args.video:
    source = args.video.expanduser()
    if not source.exists():
        raise SystemExit(f'File not found: {source}')
    copy2(source, out / 'background.mp4')
    print(f'Imported {source} -> {out / "background.mp4"}')

if not any((args.background, args.logo, args.icon, args.video)):
    raise SystemExit('Nothing to import. Use --background, --logo, --icon and/or --video.')
