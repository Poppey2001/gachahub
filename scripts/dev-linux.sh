#!/usr/bin/env bash
set -euo pipefail

DEV_PORT="${GACHAHUB_DEV_PORT:-1420}"

# VS Code installed through Snap can leak its own GIO/GTK module paths into the
# integrated terminal. Tauri/WebKitGTK must use the host GIO modules instead.
if [[ "${SNAP_NAME:-}" == "code" || "${SNAP:-}" == *"/code/"* || "${GIO_EXTRA_MODULES:-}" == *"/snap/code/"* || "${GIO_EXTRA_MODULES:-}" == *"/snap/code/common/"* ]]; then
  if command -v pkg-config >/dev/null 2>&1; then
    host_gio_modules="$(pkg-config --variable=giomoduledir gio-2.0 2>/dev/null || true)"
    if [[ -n "$host_gio_modules" && -d "$host_gio_modules" ]]; then
      export GIO_EXTRA_MODULES="$host_gio_modules"
      echo "[GachaHub] VS Code Snap environment detected: using host GIO modules ($host_gio_modules)"
    else
      unset GIO_EXTRA_MODULES || true
      echo "[GachaHub] VS Code Snap environment detected: removed Snap GIO module override"
    fi
  else
    unset GIO_EXTRA_MODULES || true
  fi
  unset SNAP_LIBRARY_PATH GTK_PATH GTK_EXE_PREFIX GTK_DATA_PREFIX || true
fi

# WebKitGTK can render a blank Tauri window on some NVIDIA/Wayland systems.
if command -v nvidia-smi >/dev/null 2>&1 || lspci 2>/dev/null | grep -qi 'NVIDIA'; then
  export WEBKIT_DISABLE_DMABUF_RENDERER=1
  echo '[GachaHub] NVIDIA detected: WEBKIT_DISABLE_DMABUF_RENDERER=1'
fi

# Tauri expects Vite on a fixed dev URL. Fail early with a useful diagnostic
# instead of letting beforeDevCommand terminate with an opaque Vite error.
port_busy=0
if command -v ss >/dev/null 2>&1; then
  if ss -ltnH "sport = :$DEV_PORT" 2>/dev/null | grep -q .; then port_busy=1; fi
elif command -v lsof >/dev/null 2>&1; then
  if lsof -nP -iTCP:"$DEV_PORT" -sTCP:LISTEN 2>/dev/null | grep -q .; then port_busy=1; fi
fi

if [[ "$port_busy" -eq 1 ]]; then
  echo "[GachaHub] Dev port $DEV_PORT is already in use." >&2
  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"$DEV_PORT" -sTCP:LISTEN 2>/dev/null || true
  elif command -v fuser >/dev/null 2>&1; then
    fuser -v "$DEV_PORT/tcp" 2>/dev/null || true
  fi
  echo "[GachaHub] Stop the stale dev server with: ./scripts/stop-dev.sh" >&2
  exit 1
fi

exec npm run tauri:dev
