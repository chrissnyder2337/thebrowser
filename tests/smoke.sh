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
for look in off geocities early-web portal; do
  report="$work/$look.txt"
  # A fresh profile each run, so nothing is read from or saved to the user's.
  status=0
  XDG_CONFIG_HOME="$work/profile-$look" timeout 90 \
    xvfb-run -a -s "-screen 0 1024x768x24" \
    "$lantern" --no-sandbox --disable-gpu --url="$url" --retro="$look" --self-test="$report" \
    > "$work/$look.log" 2>&1 || status=$?
  want="$(printf 'url=%s\ntitle=%s\nstatus=ok 200\nretro=%s' "$url" "$title" "$look")"
  got="$(cat "$report" 2>/dev/null || true)"
  if [ "$status" -ne 0 ] || [ "$got" != "$want" ]; then
    echo "smoke: FAIL --retro=$look (exit $status)" >&2
    echo "--- expected"; echo "$want"; echo "--- got"; echo "$got"
    echo "--- log (last 20 lines)"; tail -20 "$work/$look.log"
    failed=1
  else
    echo "smoke: ok   --retro=$look: $(tr '\n' ' ' < "$report")"
  fi
done
exit "$failed"
