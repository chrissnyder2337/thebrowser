#!/usr/bin/env bash
# Bundle smoke test: starts a Lantern bundle under xvfb-run, once per retro
# look, on a fixture page served from 127.0.0.1, and checks with
# --self-test=<file> that it opened the page and quit cleanly.
#
# Usage: tests/smoke.sh [bundle-dir]   (default target/bundle, built with
#        `bundle-cef-app lantern -o target/bundle --release`)
#
# CEF runs with --no-sandbox: Chromium refuses to run as root without it, and
# inside Claude Code's sandbox its own sandbox lacks user namespaces.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bundle="$(cd "${1:-$root/target/bundle}" && pwd)"
lantern="$bundle/lantern"
[ -x "$lantern" ] || { echo "smoke: no executable at $lantern" >&2; exit 1; }
command -v xvfb-run >/dev/null || { echo "smoke: xvfb-run not found (apt install xvfb)" >&2; exit 1; }

work="$(mktemp -d)"
server=""
# shellcheck disable=SC2329 # run by the EXIT trap
cleanup() {
  [ -n "$server" ] && kill "$server" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

# A static server on a free port, serving tests/fixtures.
port="$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')"
python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root/tests/fixtures" \
  > "$work/server.log" 2>&1 &
server=$!
up=0
for _ in $(seq 50); do
  if curl -sf "http://127.0.0.1:$port/tables.html" > /dev/null; then up=1; break; fi
  sleep 0.1
done
if [ "$up" -ne 1 ]; then
  echo "smoke: the fixture server didn't start on port $port" >&2
  cat "$work/server.log" >&2
  exit 1
fi

url="http://127.0.0.1:$port/tables.html"
title="Fixture: tables"
failed=0

# run <name> <expected retro> <expected ui> <lantern args...>
run() {
  local name="$1" retro="$2" ui="$3"
  shift 3
  local report="$work/$name.txt"
  # A fresh profile each run, so nothing is read from or saved to the user's.
  local status=0
  XDG_CONFIG_HOME="$work/profile-$name" timeout 90 \
    xvfb-run -a -s "-screen 0 1024x768x24" \
    "$lantern" --no-sandbox --disable-gpu --url="$url" "$@" --self-test="$report" \
    > "$work/$name.log" 2>&1 || status=$?
  local want got
  want="$(printf 'url=%s\ntitle=%s\nstatus=ok 200\nretro=%s\nui=%s' "$url" "$title" "$retro" "$ui")"
  got="$(cat "$report" 2>/dev/null || true)"
  if [ "$status" -ne 0 ] || [ "$got" != "$want" ]; then
    echo "smoke: FAIL $* (exit $status)" >&2
    echo "--- expected"; echo "$want"; echo "--- got"; echo "$got"
    echo "--- log (last 20 lines)"; tail -20 "$work/$name.log"
    failed=1
  else
    echo "smoke: ok   $*: $(tr '\n' ' ' < "$report")"
  fi
}

# Every retro look, in Lantern's own UI.
for look in off geocities early-web portal terminal; do
  run "look-$look" "$look" lantern --retro="$look"
done

# Every UI mode (settings.rs UiMode::ALL; a unit test checks the list), as
# <mode>:<the look its pages get by default>.
for entry in \
  lantern:off workstation-1993:off toolbar-1996:off desktop-1996:off \
  applet-1997:off terminal-1999:terminal \
  terminal-1992:terminal \
  pc-1993:off \
  toolbar-1995:off \
  desktop-1995:off \
  suite-1997:off \
  desktop-1997:off \
  desktop-1999:off \
  classic-1992:off \
  classic-1994:off; do
  mode="${entry%%:*}"
  run "ui-$mode" "${entry#*:}" "$mode" --ui="$mode"
done

exit "$failed"
