#!/bin/bash
# Screenshot a text-mode browser in an 80x25 xterm on a private Xvfb display.
# Usage: tools/shoot-text-browser.sh <display-number> <out.png> <command...>
# Example: tools/shoot-text-browser.sh 91 images/w3m.png w3m fixtures/text-browser-sample.html
# Needs Xvfb, xterm, xdotool and ImageMagick's import.
set -eu
display=$1
out=$2
shift 2
title="shot-$display"
export DISPLAY=":$display"
Xvfb "$DISPLAY" -screen 0 1024x768x24 >/dev/null 2>&1 &
xvfb=$!
trap 'kill "$xvfb" 2>/dev/null || true' EXIT
sleep 2
xterm -T "$title" -geometry 80x25+0+0 -fa "DejaVu Sans Mono" -fs 11 -bg black -fg grey85 -e "$@" &
term=$!
# Let the browser draw, optionally press KEYS (xdotool key names, space
# separated, e.g. KEYS="Escape Return" opens Links' File menu), then grab only
# the xterm window.
sleep 6
window=$(xdotool search --name "$title" | head -1)
if [ -z "$window" ]; then
  echo "no xterm window titled $title appeared" >&2
  exit 1
fi
for key in ${KEYS:-}; do
  # xterm ignores synthetic events sent to a window, so focus it and type.
  xdotool windowfocus "$window" 2>/dev/null
  xdotool key "$key" 2>/dev/null
  sleep 1
done
import -window "$window" "$out"
kill "$term" 2>/dev/null || true
