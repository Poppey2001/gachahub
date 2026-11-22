#!/usr/bin/env bash
# Removes GachaHub. Your settings (~/.config/gachahub) and games are kept.
#   ./uninstall.sh            remove a per-user install (~/.local)
#   sudo ./uninstall.sh --system   remove a system-wide install (/usr/local)
set -euo pipefail
INSTALLED_PREFIX=""   # filled in by install.sh
if [[ -n "$INSTALLED_PREFIX" ]]; then
    PREFIX="$INSTALLED_PREFIX"
elif [[ "${1:-}" == "--system" ]]; then
    PREFIX="/usr/local"
else
    PREFIX="$HOME/.local"
fi
rm -f "$PREFIX/bin/gachahub" \
      "$PREFIX/share/applications/gachahub.desktop" \
      "$PREFIX/share/icons/hicolor/512x512/apps/gachahub.png"
rm -rf "$PREFIX/share/gachahub"
command -v update-desktop-database >/dev/null && update-desktop-database "$PREFIX/share/applications" >/dev/null 2>&1 || true
echo "GachaHub wurde entfernt. Einstellungen in ~/.config/gachahub bleiben erhalten."
