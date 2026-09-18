#!/usr/bin/env bash
set -euo pipefail

echo "== GachaHub Linux bootstrap =="
command -v node >/dev/null || { echo "Node.js fehlt."; exit 1; }
command -v npm >/dev/null || { echo "npm fehlt."; exit 1; }
command -v cargo >/dev/null || { echo "Rust/Cargo fehlt."; exit 1; }

npm install
printf '\nStarten mit:\n  npm run tauri:dev\n'
