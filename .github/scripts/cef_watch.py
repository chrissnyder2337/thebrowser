#!/usr/bin/env python3
"""Check crates.io for a `cef` release newer than Cargo.toml's pin.

Run weekly by .github/workflows/cef-watch.yml (docs/software-factory.md,
Phase 3 → "CEF watcher"). Prints `pin=X.Y.Z` and `version=` lines for
$GITHUB_OUTPUT: version is the newest stable, unyanked release when it's
newer than the pin, and empty otherwise. The verdict also goes to stderr.
Versions compare without their build metadata (crates.io lists
`154.5.0+154.0.34`, Cargo.toml pins `=154.5.0`).

Usage: cef_watch.py [--cargo-toml Cargo.toml] [--versions versions.json]
       (without --versions it asks the crates.io API)
"""

import argparse
import json
import re
import sys
import urllib.request

API = "https://crates.io/api/v1/crates/cef/versions"
# crates.io asks API clients to name themselves.
USER_AGENT = "lantern-cef-watch (github.com/chrissnyder2337/thebrowser)"
STABLE = re.compile(r"(\d+)\.(\d+)\.(\d+)(?:\+[0-9A-Za-z.-]+)?")


def pinned(cargo_toml):
    """The version in Cargo.toml's `cef = { version = "=X.Y.Z", ... }` line."""
    match = re.search(r'^cef\s*=\s*\{[^}\n]*version\s*=\s*"=?([^"]+)"', cargo_toml, re.M)
    if not match:
        raise ValueError("Cargo.toml: no `cef = { version = ... }` line")
    return match.group(1)


def key(version):
    """(major, minor, patch) for a stable version, None for a pre-release."""
    match = STABLE.fullmatch(version)
    return tuple(int(n) for n in match.groups()) if match else None


def newer(pin, versions):
    """The newest stable, unyanked version above the pin, or None."""
    floor = key(pin)
    if floor is None:
        raise ValueError(f"pin {pin!r} isn't a plain X.Y.Z version")
    best = None
    for v in versions:
        k = key(v["num"])
        if k is None or v.get("yanked") or k <= floor:
            continue
        if best is None or k > key(best):
            best = v["num"]
    return best


def fetch():
    request = urllib.request.Request(API, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)["versions"]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cargo-toml", default="Cargo.toml")
    parser.add_argument("--versions", help="a saved crates.io versions response")
    args = parser.parse_args()
    with open(args.cargo_toml, encoding="utf-8") as f:
        pin = pinned(f.read())
    if args.versions:
        with open(args.versions, encoding="utf-8") as f:
            versions = json.load(f)["versions"]
    else:
        versions = fetch()
    found = newer(pin, versions)
    print(f"pin {pin}: " + (f"{found} is out" if found else "no update"), file=sys.stderr)
    print(f"pin={pin}")
    print(f"version={found or ''}")


if __name__ == "__main__":
    main()
