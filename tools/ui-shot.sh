#!/usr/bin/env bash
# Screenshot Lantern in a UI mode under Xvfb, for fidelity checks
# (ui-mode skill). Runs the debug build with a scratch profile in an 800x600
# window, optionally clicks some points, and writes one PNG per state.
#
# Usage: tools/ui-shot.sh <mode> [--url=URL] [X,Y ...]
#   target/ui-shots/<mode>-0.png       after the page loads
#   target/ui-shots/<mode>-N.png       after the Nth click (window pixels)
#
# Environment: CEF_PATH (default ~/.local/share/cef), WAIT (seconds to let the
# page load, default 10), DISPLAY_NUM (default 99).
set -euo pipefail

mode="${1:?usage: tools/ui-shot.sh <mode> [--url=URL] [X,Y ...]}"
shift
url="https://en.wikipedia.org/wiki/World_Wide_Web"
if [[ "${1:-}" == --url=* ]]; then
  url="${1#--url=}"
  shift
fi
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/target/ui-shots"
profile="$(mktemp -d)"
export DISPLAY=":${DISPLAY_NUM:-99}"
mkdir -p "$out"

# Xvfb can die between runs; a dead display gives black captures.
if ! xdotool getdisplaygeometry >/dev/null 2>&1; then
  Xvfb "$DISPLAY" -screen 0 1280x1024x24 -nolisten tcp >/dev/null 2>&1 &
  for _ in $(seq 50); do
    xdotool getdisplaygeometry >/dev/null 2>&1 && break
    sleep 0.1
  done
fi

XDG_CONFIG_HOME="$profile" LD_LIBRARY_PATH="${CEF_PATH:-$HOME/.local/share/cef}" \
  "$root/target/debug/lantern" --no-sandbox --disable-gpu --ignore-certificate-errors \
  --ui="$mode" --url="$url" >"$profile/lantern.log" 2>&1 &
pid=$!
# Kill by PID: `pkill -f` matches this script's own command line too.
trap 'kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; rm -rf "$profile"' EXIT

win=""
for _ in $(seq 100); do
  # The main window's title is "<page title> - Lantern"; other windows
  # (helpers, the first blank title) don't match.
  win="$(xdotool search --name " - Lantern$" 2>/dev/null | head -1 || true)"
  [ -n "$win" ] && break
  sleep 0.2
done
if [ -z "$win" ]; then
  echo "ui-shot: no Lantern window; its log ends:" >&2
  tail -20 "$profile/lantern.log" >&2
  exit 1
fi
xdotool windowsize "$win" 800 600 windowmove "$win" 0 0
# Park the pointer off the window. Xvfb starts it at the screen's centre,
# which is inside an 800x600 window, and the tooltip under it ends up in the
# capture. xdotool can only move the pointer once a window exists.
xdotool mousemove 1200 1000
sleep "${WAIT:-10}"

shoot() {
  import -window root -crop 800x600+0+0 +repage "$1"
  # A dead display or a crashed renderer gives a flat image.
  if [ "$(convert "$1" -format '%[fx:standard_deviation<0.02]' info:)" = 1 ]; then
    echo "ui-shot: $1 looks blank" >&2
    exit 1
  fi
  echo "$1"
}

shoot "$out/$mode-0.png"
n=0
for point in "$@"; do
  n=$((n + 1))
  xdotool mousemove "${point%,*}" "${point#*,}" click 1
  sleep 2
  xdotool mousemove 1200 1000
  sleep 1
  shoot "$out/$mode-$n.png"
done
