# Developing Lantern

Building Lantern from source, running its tests and cutting a release. For
how the browser is put together, see [architecture.md](architecture.md).

## Building and running

Lantern pins `cef = "=154.5.0"` (CEF 154.0.34, Chromium 154.0.8037.98). The
generated bindings track a specific CEF release, so upgrade the pin and the CEF
binaries together.

### 1. CEF binaries (optional, recommended)

The `cef-dll-sys` build script downloads CEF on its own, into its `OUT_DIR`. To
share one copy across builds, export it once and point `CEF_PATH` at it:

```sh
git clone --depth 1 https://github.com/tauri-apps/cef-rs.git
cargo run --manifest-path cef-rs/Cargo.toml -p export-cef-dir -- --force "$HOME/.local/share/cef"
export CEF_PATH="$HOME/.local/share/cef"
```

On Windows (PowerShell) use `$env:USERPROFILE/.local/share/cef` and
`$env:CEF_PATH=...`.

### 2. The bundling tool

`bundle-cef-app` builds Lantern and lays out a runnable app with the CEF files.
Install the version that matches the pinned crate:

```sh
cargo install cef --version "=154.5.0" --bin bundle-cef-app --locked
```

### 3. Build and run

**Linux**

```sh
bundle-cef-app lantern -o target/bundle            # add --release for a much smaller bundle
./target/bundle/lantern
```

The executable carries an `$ORIGIN` rpath, so it finds `libcef.so` next to itself.
Chromium refuses to run as root without `--no-sandbox`; as a normal user its own
sandbox works as usual.

**macOS**

```sh
bundle-cef-app lantern -o target/bundle -d Lantern -i org.example.lantern
open target/bundle/lantern.app
```

The bundle uses `lantern_helper` for CEF's subprocesses and copies
`resources/mac` (menu, strings, `lantern.icns`) into the app.

**Windows (PowerShell)**

```pwsh
bundle-cef-app lantern -o ./target/bundle
./target/bundle/lantern.exe
```

With the default `sandbox` feature the bundler builds the `cdylib` (entry point
`RunWinMain`) and pairs it with CEF's `bootstrap.exe`. Build with
`--no-default-features` for a plain executable. Icons are embedded by `build.rs`.

## Command-line options

- `--url=<address>`: open this instead of the start page. URL-bar rules apply,
  so `--url=example.com` works.
- `--retro=<look>`: use a retro look for this run without saving it (see the README's
  [Retro looks](../README.md#retro-looks)):
  `geocities`, `early-web`, `portal` or `off`. `--geocities` is short for
  `--retro=geocities`.
- `--ui=<mode>`: use a browser UI mode for this run without saving it (see
  [docs/ui-modes/](ui-modes/)): `lantern` or `workstation-1993`.
- `--self-test=<file>`: once the first page has loaded (or failed), write its
  URL, title and load status to `<file>` as `key=value` lines and quit. For
  `tests/smoke.sh`.

## Tests

```sh
cargo test --bin lantern          # unit, table and property tests (needs LD_LIBRARY_PATH="$CEF_PATH")
tests/web/run.sh                  # Playwright: retro looks, chrome and internal pages, goldens (Docker)
tests/smoke.sh target/bundle      # the release bundle under xvfb-run, once per retro look
tools/oracle-guard.sh origin/main # the CI check that protects existing goldens and assertions
```

- **Rust tests** cover URL normalization, percent and HTML/JS escaping, the
  title channel parser, the `lantern.invalid` action router (table tests with
  hostile input), template filling, bookmark and settings storage, the retro
  script builder and the pixel-art sheets. `proptest` properties check that
  escaping round-trips and stays inert, and that URL-bar normalization is
  stable.
- **Playwright** (`package.json`, `tests/web/`) needs no CEF.
  `cargo run --example dump-pages -- target/pages` writes the toolbar,
  status bar and internal pages, plus the script Lantern injects for each
  retro look. The tests inject each look into the fixture pages in
  `tests/fixtures/` (tables, forms, floats, shadow DOM, an iframe, a long
  page) and check for script errors, footers at the end of the page,
  readable buttons, opaque floating boxes and restyled late nodes. They
  drive the toolbar and status bar through `lantern.update` and check each
  button's title-channel message. Golden screenshots are in `tests/golden/`.
  `tests/web/run.sh` runs everything in the official Playwright Docker image
  whose tag matches `@playwright/test`, as CI does, because the goldens only
  match there.
- **Smoke test**: `tests/smoke.sh` serves `tests/fixtures/` on 127.0.0.1 and
  starts the bundle with `--no-sandbox --disable-gpu --self-test=<file>`,
  which writes the final URL, title and load status to the file and quits.

CI (`.github/workflows/ci.yml`) runs rustfmt, clippy, the unit tests and a
release bundle on Linux (uploaded as a build artifact), the Playwright suite
in the pinned container, the smoke test on that bundle, and builds on macOS
and Windows. It needs Rust 1.88 or newer. On pull requests the **Oracle
guard** fails a change that touches `src/` or `ui/` and also modifies or
deletes existing goldens or assertions, or changes the CI definition under
`.github/workflows/` or `.github/actions/`; only the maintainer's approval on
the `oracle` GitHub Environment overrides it.

## Releasing

`Cargo.toml`'s version decides the release. There's no separate tagging step:

```sh
tools/bump-version.sh minor --push     # or patch, major, or an exact 1.4.0
```

The script bumps `Cargo.toml` and `Cargo.lock` on a `release/vX.Y.Z` branch
started from `origin/main`, commits it, pushes it, and opens a pull request
(with the GitHub CLI `gh`, if installed; otherwise it prints the link). Add
`--dry-run` to preview it, or leave out `--push` to push it yourself.

Merging that pull request publishes the release: `.github/workflows/release.yml`
sees that `vX.Y.Z` doesn't exist yet, builds Linux x86_64 (`.tar.gz`), macOS
arm64 (`.zip` of `Lantern.app`) and Windows x86_64 (`.zip`), creates the tag,
and publishes the bundles under fixed names (so the README's Download links always
reach the newest release) with a `SHA256SUMS.txt` and generated notes.

- Pushing a tag by hand still works if it matches `Cargo.toml` at that commit.
  If not, the run stops with a "Tag does not match Cargo.toml" error and
  instructions.
- Running the Release workflow from the Actions tab builds the bundles as
  downloadable artifacts. Tick **publish** to release the version in
  `Cargo.toml` from that branch: useful to fill a release left without files.
- In Claude Code, `/release` (`.claude/skills/release/SKILL.md`) runs the whole
  process: picking the version from the commits, the bump PR, CI, merging, and
  checking the downloads.

The macOS app is ad-hoc signed (`codesign --sign -`) but not notarized, so
after downloading it, run `xattr -dr com.apple.quarantine Lantern.app` or allow
it under System Settings → Privacy & Security → Open Anyway.
