#!/usr/bin/env bash
set -euo pipefail

INSTALL_PATH="${1:-$HOME/Games/GachaHub/Games/genshin}"
PREFIX_PATH="${2:-$HOME/.local/share/gachahub/prefixes/genshin}"

bool_cmd() {
  if command -v "$1" >/dev/null 2>&1; then printf 'true'; else printf 'false'; fi
}

find_genshin_exe() {
  local root="$1"
  [[ -d "$root" ]] || return 0
  find "$root" -maxdepth 5 -type f \( -iname 'GenshinImpact.exe' -o -iname 'YuanShen.exe' \) -print -quit 2>/dev/null || true
}

infer_prefix_from_exe() {
  local exe="$1"
  [[ -n "$exe" ]] || return 0
  local current
  current="$(dirname "$exe")"
  while [[ "$current" != "/" && -n "$current" ]]; do
    if [[ "$(basename "$current")" == "drive_c" ]]; then
      dirname "$current"
      return 0
    fi
    current="$(dirname "$current")"
  done
}

find_runner() {
  local roots=(
    "$HOME/.local/share/gachahub/runners"
    "$HOME/.steam/root/compatibilitytools.d"
    "$HOME/.local/share/Steam/compatibilitytools.d"
    "$HOME/.var/app/com.valvesoftware.Steam/data/Steam/compatibilitytools.d"
    "$HOME/.steam/steam/steamapps/common"
    "$HOME/.local/share/Steam/steamapps/common"
  )
  local patterns=('DWProton*' 'proton-cachyos*' 'Proton-CachyOS*' 'GE-Proton*' 'Proton Hotfix' 'Proton Experimental')
  local root pattern candidate
  for pattern in "${patterns[@]}"; do
    for root in "${roots[@]}"; do
      [[ -d "$root" ]] || continue
      candidate=$(find "$root" -maxdepth 1 -mindepth 1 -type d -name "$pattern" -printf '%f\n' 2>/dev/null | sort -V | tail -n1 || true)
      if [[ -n "$candidate" ]]; then
        printf '%s|%s' "$candidate" "$root/$candidate"
        return 0
      fi
    done
  done
  printf 'Auto (kompatibler Proton-Runner)|'
}

RUNNER_INFO="$(find_runner)"
RUNNER="${RUNNER_INFO%%|*}"
RUNNER_PATH="${RUNNER_INFO#*|}"
RUNTIME="proton"
if [[ "$RUNNER" == GE-Proton* ]]; then RUNTIME="ge-proton"; fi
GAME_EXE="$(find_genshin_exe "$INSTALL_PATH")"
INFERRED_PREFIX="$(infer_prefix_from_exe "$GAME_EXE")"
if [[ -n "$INFERRED_PREFIX" ]]; then PREFIX_PATH="$INFERRED_PREFIX"; fi
HAS_GAMEMODE="$(bool_cmd gamemoderun)"
HAS_GAMESCOPE="$(bool_cmd gamescope)"
HAS_MANGOHUD="$(bool_cmd mangohud)"
HAS_UMU="$(bool_cmd umu-run)"

# Stable key=value protocol parsed by the Tauri backend.
printf 'profileVersion=genshin-defaults-v1\n'
printf 'gameId=genshin\n'
printf 'platform=linux\n'
printf 'installPath=%s\n' "$INSTALL_PATH"
printf 'prefixPath=%s\n' "$PREFIX_PATH"
printf 'executablePath=%s\n' "$GAME_EXE"
printf 'runtime=%s\n' "$RUNTIME"
printf 'runnerVersion=%s\n' "$RUNNER"
printf 'runnerPath=%s\n' "$RUNNER_PATH"
printf 'launchArguments=\n'
printf 'environmentVariables=\n'
printf 'gamescopeEnabled=false\n'
printf 'gameModeEnabled=%s\n' "$HAS_GAMEMODE"
printf 'mangoHudEnabled=false\n'
printf 'preventSleep=true\n'
printf 'autoUpdate=true\n'
printf 'allowPreloads=true\n'
printf 'updateChannel=stable\n'
printf 'modsEnabled=false\n'
printf 'verifyBeforeLaunch=false\n'
printf 'xxmiEnabled=false\n'
printf 'hasGameMode=%s\n' "$HAS_GAMEMODE"
printf 'hasGamescope=%s\n' "$HAS_GAMESCOPE"
printf 'hasMangoHud=%s\n' "$HAS_MANGOHUD"
printf 'hasUmu=%s\n' "$HAS_UMU"
printf 'warning=%s\n' 'Keine Anti-Cheat- oder Schutzmechanismus-Workarounds werden automatisch gesetzt.'
if [[ -z "$GAME_EXE" ]]; then
  printf 'warning=%s\n' 'GenshinImpact.exe wurde im Installationspfad noch nicht gefunden. Der Pfad wird trotzdem als Standard übernommen.'
fi
