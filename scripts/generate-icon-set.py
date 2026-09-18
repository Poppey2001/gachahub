#!/usr/bin/env python3
from pathlib import Path
import sys
from PIL import Image

if len(sys.argv) < 2:
    raise SystemExit('Usage: python3 scripts/generate-icon-set.py <square-png>')

source = Path(sys.argv[1])
root = Path(__file__).resolve().parents[1]
icons = root / 'src-tauri' / 'icons'
branding = root / 'public' / 'branding'
icons.mkdir(parents=True, exist_ok=True)
branding.mkdir(parents=True, exist_ok=True)

img = Image.open(source).convert('RGBA')
side = min(img.size)
left = (img.width - side) // 2
top = (img.height - side) // 2
img = img.crop((left, top, left + side, top + side))

for size, name in [
    (32, '32x32.png'),
    (64, '64x64.png'),
    (128, '128x128.png'),
    (256, '128x128@2x.png'),
    (256, '256x256.png'),
    (512, 'icon.png'),
]:
    img.resize((size, size), Image.Resampling.LANCZOS).save(icons / name, optimize=True)

img.resize((256, 256), Image.Resampling.LANCZOS).save(
    icons / 'icon.ico',
    sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)],
)

try:
    img.resize((1024, 1024), Image.Resampling.LANCZOS).save(icons / 'icon.icns')
except Exception:
    pass

img.resize((512, 512), Image.Resampling.LANCZOS).save(branding / 'gachahub-icon.png', optimize=True)
print(f'Generated icon set in {icons}')
