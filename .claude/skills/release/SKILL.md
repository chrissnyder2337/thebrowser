---
name: release
description: Cut a new Lantern release (version bump PR → automatic GitHub Release with Linux, macOS and Windows downloads), or diagnose a release that failed. Use when the user asks to release, publish, ship, tag or bump the version, or asks why a release or tag didn't build.
---

# Releasing Lantern

## How releases work

`Cargo.toml`'s `version` is the single source of truth.

1. A pull request bumps the version (made by `tools/bump-version.sh`).
2. When it merges into `main`, `.github/workflows/release.yml` sees that tag
   `v<version>` doesn't exist yet. It builds the three bundles, creates the tag
   on the merge commit, and publishes the GitHub Release with generated notes.
3. Nobody tags by hand. A hand-pushed tag still works, but only if it matches
   `Cargo.toml` at that commit. Otherwise the run fails at **Plan** with
   "Tag does not match Cargo.toml".

Each release has `lantern-linux-x86_64.tar.gz`, `lantern-macos-arm64.zip`,
`lantern-windows-x86_64.zip` and `SHA256SUMS.txt`. The names carry no version,
so the README's `releases/latest/download/<asset>` links always work.

`main` is protected: changes land through pull requests with the CI checks
`Linux (fmt, clippy, test, bundle)`, `Build (macos-latest)` and
`Build (windows-latest)` passing.

## Cutting a release

1. **Check the current state.**
   ```sh
   git fetch --force --tags origin    # --force: a deleted-and-recreated tag updates
   git fetch origin main
   grep -m1 '^version' Cargo.toml                 # what main says now
   git describe --tags --abbrev=0 origin/main     # last tag reachable from main
   git log --oneline "$(git describe --tags --abbrev=0 origin/main)"..origin/main
   ```
   Also list GitHub releases (`gh release list`, or the GitHub API/tools) and
   look for a release with no assets. That's a leftover from a failed run;
   see Troubleshooting.

2. **Pick the version.** If the user named one, use it. Otherwise propose one
   from the commits since the last tag and confirm it with the user:
   - fixes only → `patch`;
   - new features or visible changes → `minor` (while below 1.0, use `minor`
     for breaking changes too);
   - `major` only when the user asks for it.

3. **Bump.** From a clean working tree:
   ```sh
   tools/bump-version.sh minor --push          # or patch / major / 1.4.0
   ```
   This creates `release/vX.Y.Z` from `origin/main`, updates `Cargo.toml` and
   `Cargo.lock`, commits "Release vX.Y.Z", pushes, and opens the PR with `gh`.
   Without `gh`, it prints a compare URL; open the PR with your GitHub tools.
   If this session may only push to one branch (a Claude Code cloud session),
   check out that branch and use `--no-branch` to add the bump on top of it
   (`--branch NAME` only starts a *fresh* branch from `origin/main`, and
   refuses one that already has other commits). Run `--dry-run` first to preview.

4. **Get the PR merged.** Wait for the three CI checks to go green, then merge
   it (merge commit). Don't merge while CI is red: fix the cause.

5. **Watch the release.** The merge to `main` starts the **Release** workflow:
   Plan → three Bundle jobs (about 5–10 minutes) → Publish vX.Y.Z. Then verify:
   - the release has all 4 assets;
   - each `https://github.com/<owner>/<repo>/releases/latest/download/<asset>`
     answers 200/206 after redirects, e.g.
     `curl -sSL -o /dev/null -w '%{http_code}\n' -r 0-0 <url>`.

6. **Report** the release URL, the version and the commits it contains.

## Troubleshooting

- **"Tag does not match Cargo.toml"**: a tag was pushed without bumping the
  version. Ask the user to delete the bad tag and its empty release (Releases →
  the release → Delete; then the tag under Tags). Then bump to that version
  with the script and merge. Tags can't be removed from a session that may only
  push its own branch, so the user does the deletion.
- **"vX.Y.Z already exists, nothing new to release"** in the Plan summary: the
  version wasn't bumped, or a stale tag holds that version. Bump again, or
  delete the stale tag and release, then run the Release workflow from the
  Actions tab on `main` with **publish** ticked.
- **A release exists with no assets**: rerun the Release workflow on the
  tagged commit with **publish** ticked. It uploads into the existing release.
- **`git fetch` says "would clobber existing tag"**: the local clone still has
  an old copy of a tag that was deleted and recreated on GitHub (a stale tag
  also makes `git log vX..main` compare against the wrong commit). Run
  `git fetch --force --tags origin`, which only updates local tags.
- **`bump-version.sh` refuses**: it requires a clean tree, a plain `X.Y.Z`
  version greater than the current one, and a tag that doesn't exist yet on
  GitHub. Read its message; it names the fix.
- **A Bundle job fails**: open its log (GitHub tools: job logs for the run).
  CEF problems usually show up in "Set up CEF" or the bundler (`get_cef_dir`
  panics mean `CEF_PATH` isn't set).

## Don't

- Don't edit the version in `Cargo.toml` by hand (that misses `Cargo.lock`).
  Use the script.
- Don't push a tag that disagrees with `Cargo.toml`.
- Don't delete or move a published release's tag unless the user asks: people
  may already have downloaded it.
