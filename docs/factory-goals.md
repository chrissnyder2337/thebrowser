# Factory goals

Copy-paste `/goal` prompts that implement the [software factory
plan](software-factory.md), one phase per goal and one pull request per
phase. Each condition is under the 4,000-character limit.

**How to run them**

1. Start a **new cloud session** on this repo for each goal, in **auto mode**.
2. Paste one goal. Claude works until the condition holds, then stops.
3. Review and merge that goal's PR, then do that phase's **"You do"** items
   before starting the next goal.
4. Goals 1→5 run in order. Each one needs the previous PR merged.

Keep in mind:
- The goal evaluator only reads what Claude prints in the session, so every
  condition asks for command output to be shown.
- CI is still the real check.
- Each goal ends with "or stop after N turns", so a stuck run doesn't use up
  your Pro usage. Check `/goal` for progress, and stop a run with
  `/goal clear`.

---

## Goal 1: Foundations (Phase 0)

```text
/goal Phase 0 of docs/software-factory.md is implemented on this session's branch and open as a draft PR titled "Factory phase 0: foundations". Show in this session, as command output:
(1) CLAUDE.md exists at the repo root, under 120 lines, linking to README sections instead of copying them, and covering: the pre-push checks, the invariants listed in Phase 0, the risk tiers, "run /code-review on your branch before every push", and the CEF sandbox note.
(2) A cloud-environment setup script at tools/cloud-setup.sh (apt: libglib2.0-dev ninja-build xvfb xdotool imagemagick; CEF exported to $CEF_PATH; cargo build warmed) and a SessionStart hook in .claude/settings.json that runs it when CEF is missing.
(3) .claude/settings.json also has a PreToolUse hook with onFailure "block" that refuses Edit/Write under tests/golden/ unless FACTORY_ORACLE_EDIT=1. Prove both cases with a test edit that is blocked and an allowed one, then revert the test edit.
(4) Skills .claude/skills/{steward,chrome-command,cef-upgrade}/SKILL.md written from this repo's real code paths (retro-look comes in goal 2), plus .claude/claude-security-guidance.md with the Lantern rules from Phase 0.
(5) .github/ISSUE_TEMPLATE/{feature,bug,chore}.yml asking for problem, observable acceptance criteria, out of scope and proposed tier; .github/labels.yml listing factory:ready, factory:in-progress, factory:needs-human, tier:0, tier:1, tier:2, oracle-change, review, blackbox.
(6) These all exit 0 after running tools/cloud-setup.sh: cargo fmt --check; cargo clippy --all-targets -- -D warnings; LD_LIBRARY_PATH="$CEF_PATH" cargo test --bin lantern.
(7) /code-review was run on the branch and every finding it reported was fixed or answered in the PR description.
(8) The PR description lists the "You do" items: create the labels from labels.yml, set tools/cloud-setup.sh as the cloud environment's setup script, run /auto-mode-setup once.
Constraints: no changes under src/ or ui/; release.yml and bump-version.sh are untouched; no secrets or tokens are added anywhere; README changes are limited to a link to CLAUDE.md. Or stop after 60 turns and report what is left.
```

**You do:** create the labels, set `tools/cloud-setup.sh` as the cloud
environment's setup script, merge.

---

## Goal 2: White-box checks (Phase 1.1–1.5)

```text
/goal Phase 1 items 1-5 of docs/software-factory.md are implemented and open as a draft PR titled "Factory phase 1: oracle". Show as command output in this session:
(1) Playwright tests (package.json plus tests/web/) using the preinstalled Chromium at /opt/pw-browsers, with fixture pages in tests/fixtures/ (tables, forms, floats, shadow DOM, iframe, a very long page). They inject ui/retro/runtime.js plus each look and assert: no page errors; footers stay at the end; button labels are readable; floating boxes are opaque; rescans restyle new nodes. Toolbar/status pages are tested with a stubbed lantern.update, checking each button's title-channel string.
(2) A cargo example (dump-pages) writes the internal pages to files, and Playwright screenshots them.
(3) Golden screenshots under tests/golden/ for chrome pages, internal pages and every look x fixture, created with FACTORY_ORACLE_EDIT=1 set only for that step.
(4) Rust table tests for the lantern.invalid router and title-channel parser with hostile input, plus proptest properties for escaping and URL normalization. proptest is a dev-dependency only.
(5) A --self-test=<file> flag that writes the final URL, title and load status and quits, and tests/smoke.sh that runs the release bundle under xvfb-run against a local fixture server, once per retro look, with CEF --no-sandbox.
(6) CI jobs for Playwright, smoke.sh, and an oracle-guard step that fails a PR changing src/ or ui/ together with tests/golden/ or existing assertions unless it has the oracle-change label.
(7) .claude/skills/retro-look/SKILL.md, and README Tests section updated.
(8) All exit 0 in this session: cargo fmt --check; cargo clippy --all-targets -- -D warnings; LD_LIBRARY_PATH="$CEF_PATH" cargo test --bin lantern; npx playwright test; tests/smoke.sh.
(9) Each new test was shown failing once against a deliberate break (then reverted), so none passes vacuously.
(10) /code-review run and findings addressed.
Constraints: product behavior is unchanged apart from the --self-test flag; no existing test is weakened or deleted; CI stays green on the pushed head (check the PR's checks). Or stop after 90 turns and report what is left.
```

**You do:** read the golden screenshots once to check they show the right
thing, then merge.

---

## Goal 3: Black-box tester (Phase 1.6)

```text
/goal Phase 1 item 6 of docs/software-factory.md is either implemented or marked disabled with evidence, and open as a draft PR titled "Factory phase 1.6: black-box tester". First, in this session: build the release bundle, start Xvfb, and run tools/blackbox/bb (start [--url=] [--retro=], shot, click X Y, type TEXT, key KEYS, stop; xdotool + ImageMagick). Start Lantern on a local fixture, take a screenshot, Read it and describe the toolbar, click Home, and Read a second screenshot showing the start page.
If that works, show: (a) .github/workflows/blackbox.yml runs on pull_request labeled "blackbox", plus weekly on main and workflow_dispatch. It uses GitHub Environment "blackbox", never runs for forks, has no checkout of this repo's source (only a sparse checkout of tools/blackbox/), downloads the Linux bundle artifact from the PR's CI run, checks out the private repo named by vars.SCENARIOS_REPO with secrets.SCENARIOS_DEPLOY_KEY, and runs anthropics/claude-code-action with secrets.CLAUDE_CODE_OAUTH_TOKEN. Allowed tools are limited to running ./bb and reading shots/. Verdicts follow the plan: run once, re-run failures twice, satisfied in 2 of 3, post a summary comment, upload screenshots as an artifact. (b) tools/blackbox/judge-prompt.md, which never reveals scenario text to the PR author. (c) tools/blackbox/mutants/*.patch making at least 4 broken builds (blank toolbar, retro look never applies, dial-up doesn't throttle, status bar frozen), each shown applying cleanly and breaking something visible in a bb screenshot. (d) docs/blackbox-scenarios.md: the scenario file format with ONE illustrative example only. Real scenarios are written by the maintainer in the private repo. (e) actionlint passes on blackbox.yml.
If CEF cannot run under Xvfb in this container, show the error output, commit nothing except a docs/software-factory.md update marking Phase 1.6 "[Pro: disabled]" with the reason, and state it.
Either way: /code-review run and findings addressed; the PR description lists the "You do" items: create the private repo with 5-10 scenarios, add the deploy key and the CLAUDE_CODE_OAUTH_TOKEN secret, set vars.SCENARIOS_REPO, create the "blackbox" environment with yourself as required reviewer, and run once against every mutant before trusting it.
Constraints: no scenario content beyond the one example is committed; no secrets in files; src/ changes are limited to anything the bb approach strictly needs. Or stop after 70 turns and report what is left.
```

**You do:** create `thebrowser-scenarios` (private) with your scenarios, add
the deploy key, the `CLAUDE_CODE_OAUTH_TOKEN` secret (from `claude
setup-token`), the `SCENARIOS_REPO` variable and the `blackbox` environment.
Run the workflow by hand against each mutant and confirm all of them fail.
Then merge.

---

## Goal 4: Intake, tiers and review (Phase 2)

```text
/goal Phase 2 of docs/software-factory.md is implemented and open as a draft PR titled "Factory phase 2: intake and tiers". Show as command output in this session:
(1) .github/tiers.yml mapping paths to tiers exactly as the plan's risk-tier table (tier 0 only when every changed path is on the tier-0 allowlist, else the highest tier touched), and a workflow that labels each PR tier:0/1/2 from its changed files. Its logic lives in a script with unit tests covering mixed-tier PRs and renames, run here and passing.
(2) An auto-merge workflow that enables GitHub auto-merge only for tier:0 PRs from this repo's branches once required checks pass; tier 1 and 2 never auto-merge.
(3) .github/workflows/claude-review.yml runs anthropics/claude-code-action with "/code-review --comment" on pull_request labeled "review", in GitHub Environment "review", never for forks, using secrets.CLAUDE_CODE_OAUTH_TOKEN, with PR text passed as data, never interpolated into run: lines.
(4) .claude/skills/factory-issue/SKILL.md: the plan gate (read the issue, post a 5-15 line plan with files, approach, proving tests and tier, wait for approval) and the implementation /goal template from the plan.
(5) docs/routines.md with ready-to-paste routine prompts for the plan step and for triage, each listing the trigger, model and connectors.
(6) actionlint passes on all workflows, and the tier script tests pass.
(7) /code-review run and findings addressed; CI green on the pushed head.
(8) The PR description lists the "You do" items: allow auto-merge in repo settings, create the "review" environment, create the plan and triage routines from docs/routines.md.
Constraints: no workflow triggers on pull_request_target, issue_comment from non-maintainers, or issues opened with secrets; third-party actions pinned to commit SHAs; no change to src/ or ui/. Or stop after 60 turns and report what is left.
```

**You do:** allow auto-merge, create the `review` environment, create the
two routines, merge. Then try the loop on two or three small real issues
before Goal 5.

---

## Goal 5: Background loops (Phase 3)

```text
/goal Phase 3 of docs/software-factory.md is implemented within the plan's Pro limits and open as a draft PR titled "Factory phase 3: background loops". Show as command output in this session:
(1) Plain GitHub Actions with no model: cef-watch.yml (weekly; detects a newer cef crate on crates.io and opens an issue labelled tier:2), release-smoke.yml (on release published; downloads the Linux asset, verifies SHA256SUMS.txt, runs tests/smoke.sh, opens an issue on failure), digest-data.yml (weekly; writes merged PRs by tier, reverts and first-push-green rate to a JSON artifact), and .github/dependabot.yml for cargo and github-actions (weekly).
(2) docs/routines.md extended with ready-to-paste routine prompts for: CEF upgrade (triggered by the cef-watch issue), weekly lint ratchet (one anti-pattern, one tiny PR, nothing if nothing found), monthly docs drift, weekly digest write-up from digest-data's artifact, and the black-box explorer marked "[Pro: disabled at first]" with its 30-minute cap. Each lists trigger, cadence, model and the narrowest permissions it needs.
(3) actionlint passes on all new workflows, and each workflow's script logic is shown working locally (for example cef-watch against the real crates.io API with the current pin, reporting "no update" or the newer version).
(4) /code-review run and findings addressed; CI green on the pushed head.
(5) docs/software-factory.md "First steps" updated to show which steps are done.
(6) The PR description lists the "You do" items: create each routine from docs/routines.md, and turn on the explorer only if /usage shows room.
Constraints: no loop runs more often than the plan's table allows; no new secrets; nothing changes src/ or ui/; actions pinned to commit SHAs. Or stop after 50 turns and report what is left.
```

**You do:** create the routines, merge. Phase 4 (moving categories to lower
tiers) is a monthly decision you make from the digest, not a goal.
