#!/usr/bin/env bash
set -euo pipefail
PORT="${GACHAHUB_DEV_PORT:-1420}"

if ! command -v lsof >/dev/null 2>&1; then
  echo "[GachaHub] lsof is required for the safe stop helper." >&2
  echo "[GachaHub] Inspect manually with: ss -ltnp | grep :$PORT" >&2
  exit 1
fi

mapfile -t pids < <(lsof -tiTCP:"$PORT" -sTCP:LISTEN 2>/dev/null | sort -u)
if ((${#pids[@]} == 0)); then
  echo "[GachaHub] Nothing is listening on port $PORT."
  exit 0
fi

stopped=0
for pid in "${pids[@]}"; do
  owner="$(ps -o user= -p "$pid" 2>/dev/null | xargs || true)"
  cmd="$(ps -o args= -p "$pid" 2>/dev/null || true)"
  if [[ "$owner" != "${USER:-$(id -un)}" ]]; then
    echo "[GachaHub] Refusing to stop PID $pid owned by $owner: $cmd" >&2
    continue
  fi
  if [[ "$cmd" != *vite* && "$cmd" != *gachahub* && "$cmd" != *tauri* && "$cmd" != *node* ]]; then
    echo "[GachaHub] Refusing to stop unrelated PID $pid: $cmd" >&2
    continue
  fi
  echo "[GachaHub] Stopping stale dev process $pid: $cmd"
  kill "$pid"
  stopped=1
done

if [[ "$stopped" -eq 0 ]]; then
  echo "[GachaHub] No GachaHub/Vite process was stopped." >&2
  exit 1
fi
