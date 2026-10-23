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
/goal These Phase 0 items from docs/software-factory.md are implemented on this session's branch and open as a draft PR titled "Factory phase 0: foundations". Show in this session, as command output:
(1) The existing CLAUDE.md is extended, not replaced: its no-attribution rule stays first and unchanged, it stays under 120 lines, links to README and skills instead of copying them, and adds: a pointer to the steward skill's "Before every push" checks, the invariants listed in Phase 0, the risk tiers, "run /code-review on your branch before every push", and the CEF sandbox note.
(2) A cloud-environment setup script at tools/cloud-setup.sh (apt: libglib2.0-dev ninja-build xvfb xdotool imagemagick; CEF exported to $CEF_PATH; cargo build warmed) and a SessionStart hook in the existing .claude/settings.json that runs it when CEF is missing. The existing attribution settings stay unchanged.
(3) .claude/settings.json also has a PreToolUse hook (matcher Edit|Write|Bash, onFailure "block") that returns an "ask" decision for edits to existing files under tests/golden/ for Bash commands containing --update-snapshots or writing into tests/golden/, and for any Bash or GitHub tool call that approves deployments, and allows new files there. Prove it by piping sample hook-input JSON into the hook script for: an existing-file edit (ask), a new-file write (allow), an --update-snapshots command (ask), an unrelated command (allow).
(4) New skills .claude/skills/{chrome-command,cef-upgrade}/SKILL.md written from this repo's real code paths (retro-look comes in goal 2); the existing steward skill gains only: the risk tiers; a merge rule (never merge tier 1-2 PRs, which wait for the maintainer, and tier 0 merges only through the tier labeller once it exists); pre-push /code-review; and the oracle rules (existing goldens and assertions are not modified; overrides go through the maintainer). Nothing already there is removed; plus .claude/claude-security-guidance.md with the Lantern rules from Phase 0.
(5) .github/ISSUE_TEMPLATE/{feature,bug,chore}.yml asking for problem, observable acceptance criteria, out of scope and proposed tier; .github/labels.yml listing factory:ready, factory:in-progress, factory:needs-human, tier:0, tier:1, tier:2.
(6) After running tools/cloud-setup.sh, every check in the steward skill's "Before every push" section exits 0, including the macOS and Windows cef/dox cross-checks, actionlint and shellcheck for the files touched.
(7) /code-review was run on the branch and every finding it reported was fixed or answered in the PR description.
(8) The PR description lists the "You do" items: create the labels from labels.yml; set tools/cloud-setup.sh as the cloud environment's setup script; run /auto-mode-setup and /doctor prompt-audit once; install the security-guidance plugin; later, run the scoped Claude Security scan and claude plugin eval on the skills, as Phase 0 describes. Those are not part of this goal.
Constraints: no changes under src/ or ui/; release.yml and bump-version.sh are untouched; no secrets or tokens are added anywhere; README changes are limited to a link to CLAUDE.md. Or stop after 60 turns and report what is left.
```

**You do:** create the labels, set `tools/cloud-setup.sh` as the cloud
environment's setup script, run `/auto-mode-setup` and `/doctor
prompt-audit`, install the security-guidance plugin, merge. The scoped
Claude Security scan and `claude plugin eval` can wait until after Goal 2.

---

## Goal 2: White-box checks (Phase 1.1–1.5)

```text
/goal Phase 1 items 1-5 of docs/software-factory.md are implemented and open as a draft PR titled "Factory phase 1: oracle". Show as command output in this session:
(1) Playwright tests (package.json plus tests/web/) with fixture pages in tests/fixtures/ (tables, forms, floats, shadow DOM, iframe, a very long page). They inject ui/retro/runtime.js plus each look and assert: no page errors; footers stay at the end; button labels are readable; floating boxes are opaque; rescans restyle new nodes. Toolbar/status pages are tested with a stubbed lantern.update, checking each button's title-channel string.
(2) A cargo example (dump-pages) writes the internal pages to files, and Playwright screenshots them.
(3) Golden screenshots under tests/golden/ (new files only) for chrome pages, internal pages and every look x fixture. Goldens are generated and compared inside the official Playwright Docker image at one pinned tag that matches @playwright/test in package.json, run via docker both here and in CI, with a small pixel tolerance. Show the same image tag passing locally.
(4) Rust table tests for the lantern.invalid router and title-channel parser with hostile input, plus proptest properties for escaping and URL normalization. proptest is a dev-dependency only.
(5) A --self-test=<file> flag that writes the final URL, title and load status and quits, and tests/smoke.sh that runs the release bundle under xvfb-run against a local fixture server, once per retro look, with CEF --no-sandbox.
(6) CI jobs, with explicit permissions: contents: read, for Playwright (in the pinned container), smoke.sh, and an oracle-guard step. The guard fails a PR that changes src/ or ui/ AND modifies or deletes existing files under tests/golden/ or existing assertions; its only override is a job in GitHub Environment "oracle" (maintainer approval), never a label. Added goldens and tests pass. Show the guard's script passing on this branch and failing on a local test branch that edits an existing golden.
(7) .claude/skills/retro-look/SKILL.md, and README Tests section updated. The steward skill's "Before every push" section adds the Playwright suite (in the pinned container) and tests/smoke.sh, and its oracle rules name the CI guard and the "oracle" environment override. Nothing else there changes.
(8) All exit 0 in this session: every check in the steward skill's "Before every push" section; the Playwright suite in the pinned container; tests/smoke.sh.
(9) Each new test was shown failing once against a deliberate break (then reverted), so none passes vacuously.
(10) /code-review run and findings addressed.
PR description "You do": create the "oracle" environment with yourself as required reviewer, and confirm the Claude GitHub app cannot approve deployments.
Constraints: product behavior is unchanged apart from the --self-test flag; no existing test is weakened or deleted; the oracle guard is not weakened to pass; CI stays green on the pushed head (check the PR's checks). Or stop after 90 turns and report what is left.
```

**You do:** read the golden screenshots once to check they show the right
thing, create the `oracle` environment with yourself as required reviewer,
check that the Claude GitHub app can't approve deployments, then merge.

---

## Goal 3: Black-box tester (Phase 1.6)

```text
/goal Phase 1 item 6 of docs/software-factory.md is either implemented or marked disabled with evidence, and open as a draft PR titled "Factory phase 1.6: black-box tester". First, in this session: build the release bundle, start Xvfb, and run tools/blackbox/bb (start [--url=] [--retro=], shot, click X Y, type TEXT, key KEYS, stop; xdotool + ImageMagick). bb start accepts only --url= and --retro= (never --remote-debugging-port or other debugging flags) and runs Lantern as a separate unprivileged user via env -i with only DISPLAY and a scratch HOME, with iptables allowing that user only localhost. Start Lantern on a local fixture, take a screenshot, Read it and describe the toolbar, click Home, and Read a second screenshot showing the start page. Show that a test env var set in bb's caller is absent in Lantern's /proc/<pid>/environ, and that curl as that user to an outside host fails.
IF THAT WORKS, show: (a) .github/workflows/blackbox.yml triggered only by workflow_dispatch (input: PR number) and a weekly schedule on main, never pull_request or pull_request_target. It has top-level permissions limited to contents: read, actions: read, pull-requests: write; uses GitHub Environment "blackbox"; sparse-checks-out only tools/blackbox/ from main; downloads the Linux bundle artifact from that PR's latest successful CI run; checks out the private repo vars.SCENARIOS_REPO with secrets.SCENARIOS_DEPLOY_KEY; and runs anthropics/claude-code-action with secrets.CLAUDE_CODE_OAUTH_TOKEN, allowed tools limited to running ./bb and reading shots/. Verdicts follow the plan: run once, re-run failures twice, satisfied in 2 of 3, post a summary comment, upload screenshots. (b) tools/blackbox/judge-prompt.md, which never reveals scenario text in comments. (c) tools/blackbox/mutants/*.patch with at least 4 broken builds (blank toolbar, retro look never applies, dial-up doesn't throttle, status bar frozen), each shown applying cleanly and breaking something visible in a bb screenshot; plus .github/workflows/mutants.yml (workflow_dispatch from main, no Claude, no secrets, contents: read) that applies each patch, builds the bundle and uploads it as artifact mutant-<name>, and a blackbox.yml input choosing the PR bundle or a mutant. (d) docs/blackbox-scenarios.md: the scenario format with ONE illustrative example only. The steward skill gains when to dispatch blackbox.yml (tier 2, and tier 1 that changes what users see) and a note that its run-the-app recipe's DevTools port is for implementing sessions only, never the tester. (e) actionlint passes on blackbox.yml. (f) The PR description lists the "You do" items: create the private repo with 5-10 scenarios, add the deploy key, the CLAUDE_CODE_OAUTH_TOKEN secret and vars.SCENARIOS_REPO, create the "blackbox" environment with yourself as required reviewer, and dispatch it once per mutant before trusting it.
IF CEF CANNOT RUN under Xvfb here: show the error output, commit only a docs/software-factory.md update marking Phase 1.6 "[Pro: disabled]" with the reason, and list no setup items.
Either way, /code-review run and findings addressed. Constraints: no scenario content beyond the one example is committed; no secrets in files; src/ changes are limited to anything bb strictly needs. Or stop after 70 turns and report what is left.
```

**You do (only if the goal built the tester):** create
`thebrowser-scenarios` (private) with your scenarios, and add the deploy key,
the `CLAUDE_CODE_OAUTH_TOKEN` secret (from `claude setup-token`), the
`SCENARIOS_REPO` variable and the `blackbox` environment. Dispatch the
workflow by hand once per mutant and confirm every one fails. Then merge. If
the goal marked the phase disabled, just merge.

---

## Goal 4: Intake, tiers and review (Phase 2)

```text
/goal Phase 2 of docs/software-factory.md is implemented and open as a draft PR titled "Factory phase 2: intake and tiers". Show as command output in this session:
(1) .github/tiers.yml mapping paths to tiers as the plan's risk-tier table states. Tier 0 only when every changed path is on the tier-0 allowlist, and nothing under .github/ is tier 0. Dependabot PRs changing only uses: lines in workflows are tier 1. Any other .github/ change, tools/blackbox/, Cargo.toml and Cargo.lock are tier 2. Otherwise the highest tier touched. The logic lives in a script with unit tests (mixed tiers, renames, uses-only bumps, a PR editing tiers.yml), run here and passing.
(2) .github/workflows/tier.yml on pull_request_target (opened, synchronize, reopened) that checks out ONLY the base branch (no PR ref), reads tiers.yml and the script from there, gets changed files from the GitHub API, replaces any old tier label with tier:0/1/2, and in the same job enables auto-merge (gh pr merge --auto --merge; the steward skill requires merge commits) only for tier:0 PRs from this repo's own branches and runs gh pr merge --disable-auto when the tier is 1 or 2. Permissions: contents: write, pull-requests: write (needed for auto-merge; acceptable because it never runs PR code and holds no Claude token).
(3) ci.yml gains a daily schedule run on main, because GITHUB_TOKEN merges don't trigger push workflows.
(4) .github/workflows/claude-review.yml triggered only by workflow_dispatch (input: PR number), in GitHub Environment "review", running anthropics/claude-code-action with "/code-review --comment" and secrets.CLAUDE_CODE_OAUTH_TOKEN. It checks out the PR head only as data, never builds or runs it, allows only read-only tools plus posting comments, and has explicit minimal permissions.
(5) The steward skill gains the tier labeller, tier-0 auto-merge with merge commits (auto-merge turns off if a PR leaves tier 0), and when to dispatch claude-review.yml. .claude/skills/factory-issue/SKILL.md: the plan gate (read the issue, post a 5-15 line plan with files, approach, proving tests and tier, wait for approval) and the implementation /goal template from the plan.
(6) docs/routines.md with ready-to-paste routine prompts for the plan step and for triage, each listing the trigger, model and connectors.
(7) actionlint passes on all workflows, and the tier script tests pass.
(8) /code-review run and findings addressed; CI green on the pushed head.
(9) The PR description lists the "You do" items: allow auto-merge in repo settings, keep main's branch protection to required checks with no required approving review, add the CLAUDE_CODE_OAUTH_TOKEN secret if not already added, create the "review" environment with yourself as required reviewer, create the plan and triage routines from docs/routines.md.
Constraints: no secret-holding job runs on pull_request, issue_comment or issues events; pull_request_target is used only by tier.yml and never checks out PR code; third-party actions pinned to commit SHAs; every workflow declares permissions; no change to src/ or ui/. Or stop after 60 turns and report what is left.
```

**You do:** allow auto-merge, keep `main`'s branch protection to required
checks (no required approving review), add the `CLAUDE_CODE_OAUTH_TOKEN` secret if
Goal 3 didn't, create the `review` environment, create the two routines,
merge. Then try the loop on two or three small real issues
before Goal 5.

---

## Goal 5: Background loops (Phase 3)

```text
/goal Phase 3 of docs/software-factory.md is implemented within the plan's Pro limits and open as a draft PR titled "Factory phase 3: background loops". Show as command output in this session:
(1) Plain GitHub Actions with no model: cef-watch.yml (weekly; detects a newer cef crate on crates.io and opens an issue labelled tier:2), release-smoke.yml (on workflow_run of the Release workflow completing successfully, since releases published with GITHUB_TOKEN don't fire release events; first exits unless a release for that run's version exists and was published after the run started, since Release also succeeds on build-only runs; then downloads the Linux asset, verifies SHA256SUMS.txt, runs tests/smoke.sh, opens an issue on failure), digest-data.yml (weekly; writes merged PRs by tier, reverts and first-push-green rate to a JSON artifact), and .github/dependabot.yml for cargo and github-actions (weekly).
(2) docs/routines.md extended with ready-to-paste routine prompts for: CEF upgrade (triggered by the cef-watch issue), weekly lint ratchet (one anti-pattern, one tiny PR, nothing if nothing found), monthly docs drift, weekly digest write-up from digest-data's artifact, and the black-box explorer marked "[Pro: disabled at first]" with its 30-minute cap. Each lists trigger, cadence, model and the narrowest permissions it needs.
(3) actionlint passes on all new workflows, and each workflow's script logic is shown working locally (for example cef-watch against the real crates.io API with the current pin, reporting "no update" or the newer version).
(4) /code-review run and findings addressed; CI green on the pushed head.
(5) docs/software-factory.md "First steps" updated to show which steps are done.
(6) The PR description lists the "You do" items: create each routine from docs/routines.md, and turn on the explorer only if /usage shows room.
Constraints: no loop runs more often than the plan's table allows; no new secrets; nothing changes src/ or ui/; actions pinned to commit SHAs; every workflow declares minimal permissions; issue-creating jobs use only issues: write. Or stop after 50 turns and report what is left.
```

**You do:** create the routines, merge. Phase 4 (moving categories to lower
tiers) is a monthly decision you make from the digest, not a goal.
