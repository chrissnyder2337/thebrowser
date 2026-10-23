#!/usr/bin/env bash
# SessionStart fallback for cloud sessions: runs tools/cloud-setup.sh when CEF
# is missing or doesn't match the Cargo.toml pin (the environment's setup
# script normally did it already), and exports CEF_PATH into the session.
set -euo pipefail

[ "${CLAUDE_CODE_REMOTE:-}" = "true" ] || exit 0

cef_path="${CEF_PATH:-$HOME/.local/share/cef}"
pin="$(sed -n 's/^cef = { version = "=\([0-9.]*\)".*/\1/p' "$CLAUDE_PROJECT_DIR/Cargo.toml")"
if [ ! -f "$cef_path/libcef.so" ] || [ "$(cat "$cef_path/.lantern-cef-version" 2>/dev/null)" != "$pin" ]; then
  # Keep the build output out of the session's context.
  CEF_PATH="$cef_path" CLAUDE_ENV_FILE='' "$CLAUDE_PROJECT_DIR/tools/cloud-setup.sh" >&2
fi
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  echo "export CEF_PATH=\"$cef_path\"" >> "$CLAUDE_ENV_FILE"
fi
