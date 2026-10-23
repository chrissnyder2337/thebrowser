#!/usr/bin/env bash
# Prepares a Linux cloud container for Lantern work: the apt packages CI uses,
# plus Xvfb, xdotool and ImageMagick for looking at the app; CEF exported to
# $CEF_PATH at the version pinned in Cargo.toml; the cross-check targets and
# linters from the steward skill; and a warm `cargo build`.
#
# Usage: tools/cloud-setup.sh   (idempotent; safe to re-run)
#
# Set it as the cloud environment's setup script. .claude/hooks/session-start.sh
# runs it as a fallback when CEF is missing.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

export CEF_PATH="${CEF_PATH:-$HOME/.local/share/cef}"
# The `cef` crate pin, e.g. 154.5.0. The CEF files must match it exactly.
cef_version="$(sed -n 's/^cef = { version = "=\([0-9.]*\)".*/\1/p' Cargo.toml)"
if [ -z "$cef_version" ]; then
  echo "cloud-setup: no exact cef pin found in Cargo.toml" >&2
  exit 1
fi

log() { echo "cloud-setup: $*" >&2; }

# 1. System packages.
packages=(libglib2.0-dev ninja-build xvfb xdotool imagemagick)
missing=()
for p in "${packages[@]}"; do
  dpkg-query -W -f='${Status}' "$p" 2>/dev/null | grep -q 'install ok installed' || missing+=("$p")
done
if [ "${#missing[@]}" -gt 0 ]; then
  sudo=()
  [ "$(id -u)" -eq 0 ] || sudo=(sudo)
  log "installing ${missing[*]}"
  "${sudo[@]}" apt-get update -q
  DEBIAN_FRONTEND=noninteractive "${sudo[@]}" apt-get install -yq "${missing[@]}"
fi

# 2. CEF, at the pinned version (as .github/actions/setup-cef does in CI).
marker="$CEF_PATH/.lantern-cef-version"
if [ ! -f "$CEF_PATH/libcef.so" ] || [ "$(cat "$marker" 2>/dev/null)" != "$cef_version" ]; then
  log "exporting CEF for cef $cef_version into $CEF_PATH"
  cargo install export-cef-dir --version "=$cef_version" --locked
  export-cef-dir --force "$CEF_PATH"
  echo "$cef_version" > "$marker"
fi

# Persist CEF_PATH for later shells and, under a SessionStart hook, the session.
line="export CEF_PATH=\"$CEF_PATH\""
grep -qxF "$line" "$HOME/.bashrc" 2>/dev/null || echo "$line" >> "$HOME/.bashrc"
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  echo "$line" >> "$CLAUDE_ENV_FILE"
fi

# 3. Targets and linters for the steward skill's "Before every push" checks.
rustup target add x86_64-apple-darwin x86_64-pc-windows-msvc
if ! command -v actionlint >/dev/null || ! command -v shellcheck >/dev/null; then
  pip install --quiet actionlint-py shellcheck-py
fi

# 4. Warm the build (cef-dll-sys compiles its C++ wrapper against $CEF_PATH).
cargo build --all-targets

log "done (CEF_PATH=$CEF_PATH)"
