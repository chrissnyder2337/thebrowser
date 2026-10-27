---
name: steward
description: Lantern's conventions for getting a pull request merged — the checks to run before pushing, how CI is set up and how it has failed before, review handling, merging, and the project's branding rule. Use when opening, updating, reviewing, fixing CI on, or merging a pull request in this repository.
---

# Stewarding Lantern pull requests

## Ground rules

- **`main` is protected.** Changes land only through pull requests, with these
  three checks passing on the PR's current head:
  `Linux (fmt, clippy, test, bundle)`, `Build (macos-latest)` and
  `Build (windows-latest)`. No approvals are required. Tier-0 auto-merge
  waits only on required checks, so `Playwright (web layer, goldens)`,
  `Bundle smoke test`, `Oracle guard` and `Tier script tests` should be
  required too (the maintainer sets this).
- **Merge with a merge commit.** Don't squash or rebase: the history keeps the
  separate steps (milestones, renames, fixes).
- **Start work from the latest `origin/main`.** Once a branch's PR has merged,
  restart the branch from `origin/main` for new work. Don't stack on merged
  history.
- **Never rewrite a branch you didn't create**: other sessions' branches
  (`ccr-*`, `claude/*`), the maintainer's, or Dependabot's. To build on one,
  start your branch from it and add your commits on top, keeping its commits
  unchanged. Before merging, check that the source branch hasn't gained
  commits you'd leave behind:
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

The examples below are a guide; the labeller's path lists in
`.github/tiers.yml` decide.

| Tier | Examples | Gate |
|---|---|---|
| **0** | Typo and docs fixes, README screenshots, new fixture pages. Nothing under `.github/`, nothing in `src/` or `ui/` (clippy fixes and pixel-art tweaks are tier 1 by path), and not the factory's own docs. | Auto-merge (merge commit) through the tier labeller once the required checks pass; see "Merging". |
| **1** | Retro looks, internal page styling, new toolbar commands, settings additions, dial-up profiles; Dependabot bumps that only change `uses:` lines. | The maintainer looks at the plan, screenshots and test changes, and skims the code. |
| **2** | `client.rs` request, popup and download handling; the `lantern.invalid` router in `commands.rs`; `util.rs` escaping and URL normalization; sandbox and helper setup; the CEF pin, `Cargo.toml`, `Cargo.lock`; `release.yml`, `bump-version.sh`; anything else under `.github/`; `tools/blackbox/`. | The maintainer reads the whole diff; also run `/security-review` in the session, and ask for `claude-review.yml` (below). Never auto-merged. |

### The tier labeller

`.github/workflows/tier.yml` sets the tier; you don't. For PRs into `main`,
on every `opened`, `synchronize`, `reopened`, `ready_for_review` and
`edited` event, it runs from the base branch (`pull_request_target`, base checkout only, no PR code), reads
`.github/tiers.yml` and `.github/scripts/tier.py` from there, takes the
changed files from the API, and replaces any old `tier:*` label with
`tier:0`, `tier:1` or `tier:2`. Which paths land where is in `tiers.yml`;
both sides of a rename count, a fixture page is tier 0 only when it's
new, and a Dependabot PR that only changes `uses:`
lines in workflows is tier 1.

- **Tier 0 auto-merges.** For a tier-0, non-draft PR from a branch in this
  repository, the same job runs `gh pr merge --auto --merge`, so GitHub
  merges it with a merge commit once the required checks pass. Fork PRs are
  labelled but never auto-merged.
- **Leaving tier 0 turns it off.** A push that makes the PR tier 1 or 2 gets
  the new label and `gh pr merge --disable-auto` (so does any tier-1 or
  tier-2 PR that has auto-merge on, and a PR moved off `main`).
  The maintainer merges tier 1 and 2 by hand, not through auto-merge. Keep tier-0 PRs to tier-0
  paths; split anything else into its own PR.
- If the label disagrees with your reading of the diff, say so in the PR;
  don't relabel by hand, and don't touch `tiers.yml` or the script in a
  feature PR (that makes it tier 2).
- Merges made by `GITHUB_TOKEN` don't start push workflows, so `ci.yml`
  also runs daily on `main`. After a tier-0 auto-merge, CI on `main` shows
  up in that run (or start **Run workflow** on `ci.yml`).

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
- **The lock is CI's `Oracle guard` check** (`tools/oracle-guard.sh`, run in
  `ci.yml`). It fails a PR that changes `src/` or `ui/` and also modifies or
  deletes an existing golden, deletes an existing test file, removes or
  changes an existing assertion, or changes the CI definition under
  `.github/workflows/` or `.github/actions/`. Its only override is the
  `Oracle override (maintainer approval)` job, which waits on the `oracle`
  GitHub Environment until the maintainer approves it. A label or a
  workflow input never skips it, and the guard isn't edited to pass.

## Before every push

Run what CI runs; one validated push beats several speculative ones. Set up
CEF once (`docs/development.md` → "CEF binaries"), then:

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

Then the web-layer and bundle tests:

```sh
tests/web/run.sh     # Playwright in the pinned official image (needs Docker); writes target/pages first
bundle-cef-app lantern -o target/bundle --release
tests/smoke.sh       # the bundle under xvfb-run, once per retro look, with --no-sandbox
```

`tests/web/run.sh` runs the suite with `docker run` in
`mcr.microsoft.com/playwright:v<version>-noble`, the version pinned for
`@playwright/test` in `package.json`, as CI does. Golden screenshots only
match in that image, so don't compare them on the host's browser.

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
The DevTools port is for implementing sessions only. The black-box tester
never gets it: `tools/blackbox/bb start` takes only `--url=` and `--retro=`,
and a change to `bb` must keep it that way.

## Black-box test

`blackbox.yml` runs the hidden scenarios against a PR's bundle
([docs/blackbox-scenarios.md](../../../docs/blackbox-scenarios.md)). It
holds the maintainer's Claude token and waits on the `blackbox` environment,
so only the maintainer starts it (**Run workflow** on `main`, with the PR
number). It is never triggered from a PR.

- **When to ask for it:** on tier-2 PRs, and on tier-1 PRs that change what
  users see (retro looks, chrome or internal pages, toolbar commands,
  dial-up). Not on tier 0, and not on every push: say in the PR description
  that it's due, once CI is green on the head you want tested.
- **A failing verdict** comes back as the PR comment's ids and screenshots
  only. Fix what the screenshots show; don't ask for the scenario text.
- **Checking the checker:** whenever scenarios or `tools/blackbox/` change,
  and monthly, dispatch `mutants.yml`, then `blackbox.yml` once per broken
  build. Each one must fail a scenario.

## Claude review

`claude-review.yml` runs Anthropic's `code-review` plugin with `--comment`
on a PR in GitHub Actions, on the maintainer's Claude token. The plugin is
pinned to a commit in `.claude-plugin/marketplace.json`. Like the black-box
test, only the maintainer starts it (**Run workflow** on `main`, with the PR
number), and it waits on the `review` environment. `main` is the workspace
and the PR head is data in `pr-head/`, never built or run. The reviewer can
read workspace files and run `gh pr view|diff|list|comment`, `gh issue
view|list` and `gh search`; it posts its findings as one PR comment (the
action's inline-comment tool only exists on PR events).

- **When to ask for it:** on every tier-2 PR, once CI is green on the head
  you want reviewed. Not on tier 0 or tier 1 (the in-session `/code-review`
  covers those). Say in the PR description that it's due.
- The plugin skips PRs that are closed, drafts, trivial, or already have a
  comment from Claude, so mark the PR ready first. It reviews a PR once: to
  review again after a substantial change, delete its earlier comment first.
- It runs several agents, some on Opus, so one review uses noticeably more
  Pro usage than an in-session `/code-review`.
- To see why a review posted nothing, start it with `show_result`, which
  adds the review's final text to the job summary (public on this repo).
- Its comments are review findings like any other: fix and push, or reply
  on the thread with why not.

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
(above); never merge one yourself or turn auto-merge on by hand.
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
