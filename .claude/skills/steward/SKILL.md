---
name: steward
description: Lantern's conventions for getting a pull request merged — the checks to run before pushing, how CI is set up and how it has failed before, review handling, merging, and the project's branding rule. Use when opening, updating, reviewing, fixing CI on, or merging a pull request in this repository.
---

# Stewarding Lantern pull requests

## Ground rules

- **`main` is protected.** Changes land only through pull requests, with these
  three checks passing on the PR's current head:
  `Linux (fmt, clippy, test, bundle)`, `Build (macos-latest)` and
  `Build (windows-latest)`. No approvals are required.
- **Merge with a merge commit.** Don't squash or rebase: the history keeps the
  separate steps (milestones, renames, fixes).
- **Start work from the latest `origin/main`.** Once a branch's PR has merged,
  restart the branch from `origin/main` for new work. Don't stack on merged
  history.
- **Never rewrite someone else's branch.** Other Claude sessions push
  `ccr-*` branches. To build on one, start your branch from it and add your
  commits on top, keeping its commits unchanged. Before merging, check that
  the source branch hasn't gained commits you'd leave behind:
  `git merge-base --is-ancestor origin/<their-branch> HEAD`.
- **Releases go through the `release` skill** (`/release`). Never tag by hand,
  and don't change the version in a feature PR.

## Branding rule

Lantern's look is the general late-1990s era, never a copy of a real product.
**Don't name things after real browsers or companies' products**: no
"Netscape", "Navigator", "Internet Explorer", "Mosaic", "Opera" and the like.
That covers user-facing labels, option keys and command-line values
(`--retro=<key>`), saved settings, file names, screenshots and badges.
Also avoid their logos, icons, throbber animations, and wording or layout
signatures (a "<browser> Now!" button, an "N" or "e" logo).
Describe the era instead: "Early web (1995)", "Web portal (1997)".
Fonts named in CSS font stacks are fine.
Research notes under `docs/research/` are the exception: they name real
browsers in order to describe them, and they never ship in the app.

## Risk tiers

Every PR has a tier; **the highest tier any changed path reaches wins**, and
a PR is tier 0 only if every path is low-risk. Say the tier in the PR
description.

| Tier | Examples | Gate |
|---|---|---|
| **0** | Typo and docs fixes, README screenshots, clippy fixes for new lints, pixel-art tweaks, new fixture pages. Nothing under `.github/`. | Merges automatically once CI is green, through the tier labeller (Phase 2), not by hand. |
| **1** | Retro looks, internal page styling, new toolbar commands, settings additions, dial-up profiles; Dependabot bumps that only change `uses:` lines. | The maintainer looks at the plan, screenshots and test changes, and skims the code. |
| **2** | `client.rs` request, popup and download handling; the `lantern.invalid` router in `commands.rs`; `util.rs` escaping and URL normalization; sandbox and helper setup; the CEF pin, `Cargo.toml`, `Cargo.lock`; `release.yml`, `bump-version.sh`; anything else under `.github/`; `tools/blackbox/`. | The maintainer reads the whole diff; also run `/security-review` in the session. Never auto-merged. |

## Oracle rules

The tests, golden screenshots and (later) black-box scenarios are the
oracle: they decide whether a change works, so a change doesn't get to
rewrite them to pass.

- **Existing goldens and existing assertions are not modified or deleted.**
  That covers files under `tests/golden/`, snapshot updates
  (`--update-snapshots`, `-u`) and weakening an existing test's assertions.
  Adding new goldens and new tests is fine, including the first goldens for
  new behaviour.
- **Overrides go through the maintainer.** If an existing golden or
  assertion really is wrong, say so in the PR and leave the change to the
  maintainer: they approve it on the `oracle` GitHub Environment or push it
  themselves. Never approve a deployment or environment yourself.
- The PreToolUse hook in `.claude/settings.json` asks before such edits.
  It's a speed bump, not permission: an approved prompt doesn't make the
  change allowed.

## Before every push

Run what CI runs; one validated push beats several speculative ones. Set up
CEF once (README → "CEF binaries"), then:

```sh
export CEF_PATH="$HOME/.local/share/cef"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
LD_LIBRARY_PATH="$CEF_PATH" cargo test --bin lantern
# Type-check the other platforms' code paths (no CEF build needed):
rustup target add x86_64-apple-darwin x86_64-pc-windows-msvc
for t in x86_64-apple-darwin x86_64-pc-windows-msvc; do
  CARGO_TARGET_DIR=target/xcheck cargo check --target "$t" --features cef/dox --bins --lib
done
```

Also run these when you touch the matching files:
- `.github/` → `actionlint .github/workflows/*.yml` (`pip install actionlint-py`);
- `tools/*.sh` → `shellcheck` (`pip install shellcheck-py`);
- `ui/**/*.js` → at least a syntax check (`node --check`, or
  `new Function(source)` for the look fragments in `ui/retro/`).

Then **run `/code-review` on your branch** and fix what it finds (or answer
it in the PR description) before pushing. For tier-2 changes also run
`/security-review`.

For UI changes, **run the app and look**. Unit tests don't cover the chrome
pages or the retro looks. On Linux without a display:
`Xvfb :99 &`, then
`DISPLAY=:99 target/debug/lantern --no-sandbox --disable-gpu --remote-debugging-port=9222 --url=<page>`
(add `--retro=<look>` for the looks). Inspect pages over the DevTools port,
and take screenshots with `import -window root`. Toolbar and status bar are
`data:` pages. Test on a real, long page such as a Wikipedia article.

## How CI is wired, and how it has failed

- `.github/actions/setup-cef` restores the CEF distribution from cache (or
  runs `export-cef-dir`) and sets `CEF_PATH`. Builds and `bundle-cef-app` rely
  on it.
- **`get_cef_dir()` … `unwrap()` panic in the bundler** means `CEF_PATH`
  wasn't set for that step.
- **Unit tests exit 127, "libcef.so: cannot open shared object file"**: on a
  warm `rust-cache` the CEF files aren't copied into `target/`, so the test
  step needs `LD_LIBRARY_PATH="$CEF_PATH"`. It passes cold and fails warm, so
  don't call it a flake.
- **Code that compiles on Linux but not Windows**: CEF enum wrappers differ by
  platform (e.g. `cef_event_flags_t` wraps `u32` on Linux and macOS, `i32` on
  Windows). The `cef/dox` cross-checks above catch these.
- PowerShell steps keep going after a failed native command. Check
  `$LASTEXITCODE` after each one.
- A red check is root-caused and fixed, never re-run hoping for green, and
  tests are never skipped or disabled to get past it.

## Reviews

- Make small, clearly correct asks right away: nits, renames, an added test, a
  local fix. Push them, and reply in one line on the thread naming the commit.
- Put larger or design-level asks to the user with a concrete proposal before
  building them.
- A bug you find in your own earlier commits on the PR gets fixed in the PR,
  not deferred.

## Merging

**Never merge a tier 1 or tier 2 PR**: they wait for the maintainer, who
merges them. Tier 0 PRs merge only through the tier labeller's auto-merge
once it exists (Phase 2); until then, leave every PR for the maintainer.
The steps below are for the maintainer, or for a merge they explicitly ask
for.

Merge only when all of these hold:
1. CI is green on the PR's **current head** (check the run's head commit,
   not just "a green run");
2. GitHub reports the PR as mergeable with no conflicts;
3. no review thread is waiting on you.

Then merge with a merge commit, pinned to the head you checked
(`expectedHeadSha` in the GitHub tools). Afterwards confirm CI on `main`
passes. If the PR bumped the version, follow the `release` skill to watch the
release publish.

## Pull request descriptions

Use **Why** (when it isn't obvious), **Changes**, and **Testing**. Testing
lists what was actually run and what it showed: commands, pages, numbers,
screenshots checked. Don't claim anything you didn't run, and say plainly
what wasn't tested, e.g. "type-checked for macOS and Windows, not run there".
