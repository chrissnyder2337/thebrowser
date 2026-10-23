---
name: cef-upgrade
description: Move Lantern to a new CEF / `cef` crate release — the Cargo.toml pin, Cargo.lock, CEF_CRATE_VERSION in both workflows, the README version line and the local CEF export — then rebuild, fix API drift on all three platforms and run the app. Use when asked to upgrade, bump or update CEF or Chromium, or when a new `cef` crate version is out.
---

# Upgrading CEF

The `cef` crate's generated bindings track one CEF release, so the crate pin
and the CEF binaries always move together, in one PR, and nothing else rides
along. A CEF upgrade is **tier 2**: it touches `Cargo.toml`, `Cargo.lock` and
`.github/`, so the maintainer reads the whole diff and it is never
auto-merged. Don't bump Lantern's own version here (that's the `release`
skill).

## 1. Pick the version

`cef` versions look like `154.5.0+154.0.34` (crate version + CEF version).
Check crates.io for the newest `cef` and that `export-cef-dir` and
`bundle-cef-app` (the `cef` crate's `bundle-cef-app` bin) exist at the same
version. Read the cef-rs release notes for API changes.

## 2. Move every pin together

| File | What |
|---|---|
| `Cargo.toml` | `cef = { version = "=<new>", … }` (keep the `=`) |
| `Cargo.lock` | `cargo update -p cef --precise <new>`; `cef-dll-sys` follows |
| `.github/workflows/ci.yml` | `env.CEF_CRATE_VERSION` |
| `.github/workflows/release.yml` | `env.CEF_CRATE_VERSION` (the one release.yml change allowed, and it needs the maintainer's eye) |
| `README.md` | "Building and running": the pin, CEF and Chromium versions, and the `bundle-cef-app` install line |

`CEF_CRATE_VERSION` keys the CEF and `bundle-cef-app` caches in
`.github/actions/setup-cef` and the workflows, so a missed one builds against
stale binaries. Check nothing is left:
`git grep -n '<old>' -- ':!docs/research'`.

## 3. Refresh CEF locally

`tools/cloud-setup.sh` reads the pin from `Cargo.toml` and re-exports CEF into
`$CEF_PATH` when the version it recorded differs, then rebuilds. Run it, then
confirm `$CEF_PATH/.lantern-cef-version` shows the new version.

## 4. Fix API drift

Build, then fix compile errors with the smallest change that keeps behaviour.
Common spots:

- the `wrap_*!` handler impls in `src/shell/client.rs`, `app.rs` and
  `window.rs` (callback signatures and enum names change between releases);
- `sys::cef_*` enums whose integer type differs by platform (see
  `handle_shortcut` in `client.rs`);
- settings fields in `src/shell/mod.rs` (`Settings`, `no_sandbox`);
- `pages::cef_versions()`, which parses `sys::CEF_VERSION` for the About
  page; its test `versions_parse` covers the format.

Keep every CEF call on the UI thread. Don't change sandbox or helper setup
unless the release requires it, and call out any change there in the PR.

## 5. Verify

- Every check in the steward skill's "Before every push", especially the
  macOS and Windows `cef/dox` cross-checks: they are the only signal for
  those platforms before CI.
- `actionlint .github/workflows/*.yml`.
- Bundle and run: `bundle-cef-app lantern -o target/bundle --release`, then
  start it under Xvfb (steward recipe) with `--no-sandbox --disable-gpu`.
  Load a long real page, each `--retro=` look, the About page (check the new
  CEF and Chromium versions show) and a download. Run the bundle smoke test
  too once it exists.
- In the PR's **Testing**, list what ran on Linux and say plainly that macOS
  and Windows were type-checked and built by CI, not run.
