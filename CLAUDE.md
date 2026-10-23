# Lantern: notes for Claude Code

- No Claude or AI attribution anywhere in this repository's history or on
  GitHub: no `Co-Authored-By: Claude …` trailers, no session links, and no
  "Generated with Claude Code" (or similar) lines in commit messages, pull
  request descriptions, review comments or issue comments.
  `.claude/settings.json` turns off the built-in attribution; this rule covers
  text written by hand.
- Pull-request conventions are in `.claude/skills/steward/SKILL.md`; releases
  go through `.claude/skills/release/SKILL.md`.

Architecture, build steps and the source layout are in [README.md](README.md);
don't repeat them here.

## Before every push

- Run every check in the steward skill's
  [**"Before every push"**](.claude/skills/steward/SKILL.md#before-every-push)
  section: rustfmt, clippy `-D warnings`, the unit tests, the macOS and
  Windows `cef/dox` cross-checks, and `actionlint`, `shellcheck` or
  `node --check` for the files you touched. In a cloud session,
  `tools/cloud-setup.sh` installs what they need (the SessionStart hook runs
  it when CEF is missing).
- **Run `/code-review` on your branch before every push**, and fix what it
  finds or answer it in the PR description.
- For UI changes, run the app and look (steward skill → Xvfb recipe).

## Invariants

- **The CEF pin moves only with its binaries.** `cef` in `Cargo.toml`,
  `Cargo.lock` and `CEF_CRATE_VERSION` in the workflows change together, in
  one PR ([cef-upgrade skill](.claude/skills/cef-upgrade/SKILL.md)).
- **All CEF calls stay on the UI thread.** Shell state is a thread-local
  `RefCell` (`state::with_shell`); CEF objects aren't `Send`. Never move them
  to another thread.
- **Chrome views stay locked down.** The toolbar and status views load only
  their own `data:` page and never open popups (`client.rs`
  `on_before_browse`).
- **Internal actions only go through `lantern.invalid`**
  (`commands::handle_action_url`). Don't add another channel from web
  content to Rust, and don't widen what the router accepts without a
  tier-2 review.
- **Every string that reaches HTML or JS goes through `util.rs` escaping**
  (`html_escape`, `js_string`, `encode_component`, the `Json` builder).
  Never `format!` untrusted text into markup or script.
- **Oracle rules** (steward skill): existing goldens and assertions aren't
  modified; overrides go through the maintainer.

## Risk tiers

The highest tier touched wins; details and gates are in the steward skill.

- **Tier 0**: typo and docs fixes, README screenshots, clippy fixes for new
  lints, pixel-art tweaks, new fixture pages. Nothing under `.github/`.
- **Tier 1**: retro looks, internal page styling, new toolbar commands,
  settings additions, dial-up profiles.
- **Tier 2**: `client.rs` request/popup/download handling, the
  `lantern.invalid` router, `util.rs` escaping and URL normalization,
  sandbox and helper setup, the CEF pin, `Cargo.toml`/`Cargo.lock`,
  `release.yml`, `bump-version.sh`, anything else under `.github/`.

Never merge tier 1–2 PRs; they wait for the maintainer.

## CEF sandbox

Chromium refuses to run as root without `--no-sandbox`, and inside Claude
Code's Linux sandbox CEF's own sandbox needs user namespaces. To run Lantern
here, launch it with `--no-sandbox --disable-gpu` (the steward recipe). The
alternative, `sandbox.allowNestedUserNamespaces`, weakens the Claude
sandbox's file protections, so don't enable it in shared settings. Neither
changes the shipped app: the `sandbox` feature stays on by default.

## House style

Match the surrounding code: its naming, idiom and comment density (short
doc comments that say why). Commit messages follow the log: a short
`Area: what changed` subject, and a body saying why when it isn't obvious. Skills:
[steward](.claude/skills/steward/SKILL.md),
[release](.claude/skills/release/SKILL.md),
[chrome-command](.claude/skills/chrome-command/SKILL.md),
[cef-upgrade](.claude/skills/cef-upgrade/SKILL.md). Security rules for edits:
[.claude/claude-security-guidance.md](.claude/claude-security-guidance.md).
