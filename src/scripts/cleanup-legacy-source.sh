#!/usr/bin/env bash
set -euo pipefail

# The current launcher uses GameRail. The compatibility Sidebar shipped in the
# patch is safe to keep, but this helper can remove other known stale files from
# very old working trees without touching active source files.
find src -type f -name '*.orig' -delete 2>/dev/null || true
find src -type f -name '*~' -delete 2>/dev/null || true
printf 'Legacy source cleanup complete.\n'
