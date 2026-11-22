#!/usr/bin/env bash
# One-line installer: downloads the latest GachaHub release and installs it.
#   curl -fsSL https://raw.githubusercontent.com/Poppey2001/gachahub/main/packaging/linux/get.sh | bash
set -euo pipefail
REPO="Poppey2001/gachahub"
ARCH="$(uname -m)"
[[ "$ARCH" == "x86_64" ]] || { echo "Aktuell gibt es nur x86_64-Builds (gefunden: $ARCH)." >&2; exit 1; }

URL="https://github.com/$REPO/releases/latest/download/gachahub-linux-x86_64.tar.gz"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Lade $URL"
curl -fL --progress-bar "$URL" -o "$TMP/gachahub.tar.gz"
tar -xzf "$TMP/gachahub.tar.gz" -C "$TMP"
bash "$TMP"/gachahub-linux-x86_64/install.sh "$@"
