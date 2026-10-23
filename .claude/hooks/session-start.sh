#!/usr/bin/env bash
# SessionStart fallback for cloud sessions: runs tools/cloud-setup.sh when CEF
# is missing (the environment's setup script normally did it already), and
# exports CEF_PATH into the session either way.
set -euo pipefail

[ "${CLAUDE_CODE_REMOTE:-}" = "true" ] || exit 0

cef_path="${CEF_PATH:-$HOME/.local/share/cef}"
if [ ! -f "$cef_path/libcef.so" ]; then
  # Keep the build output out of the session's context.
  CEF_PATH="$cef_path" CLAUDE_ENV_FILE='' "$CLAUDE_PROJECT_DIR/tools/cloud-setup.sh" >&2
fi
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  echo "export CEF_PATH=\"$cef_path\"" >> "$CLAUDE_ENV_FILE"
fi
