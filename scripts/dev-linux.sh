#!/usr/bin/env bash
set -euo pipefail

# WebKitGTK can render a blank Tauri window on some NVIDIA/Wayland systems.
# Apply the DMA-BUF workaround only when an NVIDIA GPU/driver is detected.
if command -v nvidia-smi >/dev/null 2>&1 || lspci 2>/dev/null | grep -qi 'NVIDIA'; then
  export WEBKIT_DISABLE_DMABUF_RENDERER=1
  echo '[GachaHub] NVIDIA detected: WEBKIT_DISABLE_DMABUF_RENDERER=1'
fi

exec npm run tauri:dev
