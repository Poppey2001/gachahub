$ErrorActionPreference = "Stop"
Write-Host "== GachaHub Windows bootstrap =="
if (-not (Get-Command node -ErrorAction SilentlyContinue)) { throw "Node.js fehlt." }
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) { throw "npm fehlt." }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw "Rust/Cargo fehlt." }
npm install
Write-Host "`nStarten mit:`n  npm run tauri:dev"
