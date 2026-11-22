#!/usr/bin/env bash
# GachaHub Linux installer.
#   ./install.sh            install for the current user (~/.local)
#   sudo ./install.sh --system   install for all users (/usr/local)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${1:-}" == "--system" ]]; then
    PREFIX="/usr/local"
else
    PREFIX="$HOME/.local"
fi

BIN_DIR="$PREFIX/bin"
APP_DIR="$PREFIX/share/applications"
ICON_DIR="$PREFIX/share/icons/hicolor/512x512/apps"

for f in gachahub gachahub.png gachahub.desktop; do
    [[ -f "$HERE/$f" ]] || { echo "Fehlt im Paket: $f" >&2; exit 1; }
done

echo "== GachaHub wird nach $PREFIX installiert =="
install -Dm755 "$HERE/gachahub"         "$BIN_DIR/gachahub"
install -Dm644 "$HERE/gachahub.png"     "$ICON_DIR/gachahub.png"
install -Dm644 "$HERE/gachahub.desktop" "$APP_DIR/gachahub.desktop"
if [[ -f "$HERE/uninstall.sh" ]]; then
    install -Dm755 "$HERE/uninstall.sh" "$PREFIX/share/gachahub/uninstall.sh"
    # remember where we installed, so uninstalling needs no flags
    sed -i "s|^INSTALLED_PREFIX=\"\"|INSTALLED_PREFIX=\"$PREFIX\"|" "$PREFIX/share/gachahub/uninstall.sh"
fi

command -v update-desktop-database >/dev/null && update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache  >/dev/null && gtk-update-icon-cache -q "$PREFIX/share/icons/hicolor" >/dev/null 2>&1 || true

echo "Fertig. GachaHub findest du jetzt im Anwendungsmenü oder startest es mit: gachahub"
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "Hinweis: $BIN_DIR ist nicht in deinem PATH – Start über das Menü funktioniert trotzdem." ;;
esac
echo "Deinstallieren: $PREFIX/share/gachahub/uninstall.sh"
