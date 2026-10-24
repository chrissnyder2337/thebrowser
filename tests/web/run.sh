#!/usr/bin/env bash
# Runs the Playwright suite (tests/web) inside the official Playwright image,
# at the tag matching @playwright/test in package.json, so golden screenshots
# render exactly as in CI. Extra arguments go to `playwright test`.
#
# Usage: tests/web/run.sh [playwright args...]
#
# Writes the internal pages first with `cargo run --example dump-pages`
# (needs CEF_PATH), unless LANTERN_PAGES points at pages already written.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

version="$(node -p 'require("./package.json").devDependencies["@playwright/test"]')"
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "run.sh: @playwright/test must be pinned to an exact version, not $version" >&2
  exit 1
fi
image="mcr.microsoft.com/playwright:v$version-noble"

if [ -z "${LANTERN_PAGES:-}" ]; then
  export CEF_PATH="${CEF_PATH:-$HOME/.local/share/cef}"
  LD_LIBRARY_PATH="$CEF_PATH" cargo run --quiet --example dump-pages -- target/pages > /dev/null
fi

# @playwright/test is plain JavaScript, so installing it on the host is fine;
# the browsers come from the image.
installed="$(node -p 'try { require("@playwright/test/package.json").version } catch { "" }')"
if [ "$installed" != "$version" ]; then
  PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm ci --no-audit --no-fund
fi

# No network: fixtures are served by the tests themselves.
exec docker run --rm --init --ipc=host --network none \
  -v "$root:/work" -w /work \
  -e CI -e LANTERN_PAGES \
  "$image" npx playwright test "$@"
