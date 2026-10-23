# A software factory for Lantern

Research notes and a plan for turning Lantern's current way of working (one
maintainer, Claude sessions doing most of the coding) into a deliberate
"software factory".

See also: [Claude feature audit (May–October 2026)](claude-features-audit.md),
a list of recent Anthropic features this plan should use.

- [Part 1: What people mean by "software factory"](#part-1-what-people-mean-by-software-factory)
  - [How dated is this research?](#how-dated-is-this-research-checked-october-2026)
- [Part 2: Where Lantern stands today](#part-2-where-lantern-stands-today)
- [Part 3: The plan](#part-3-the-plan)
- [Sources](#sources)

---

## Part 1: What people mean by "software factory"

### The short definition

The term is old. Bemer proposed it in 1968, SDC ran a factory experiment in
1975, and Microsoft published a *Software Factories* book in 2004. In 2026 it
means something specific: **a system in which AI coding agents do most of the
building, while humans design the system, set direction and decide what
ships.**

Addy Osmani's vocabulary is the clearest:

| Term | Meaning |
|---|---|
| **Loop** | One agent doing one job on repeat: gather context, act, check, continue until a condition is met. |
| **Harness** | Everything around a loop: sandbox, tools, memory between runs, and the gates that define "done". |
| **Factory** | Many harnessed loops fed by a work queue, flowing through verification and a review gate to production. Humans watch the system rather than each line. |
| **Dark factory** | Code ships without any human reading it. Machines verify machines. |
| **Lit factory** | The same pipeline with human judgment kept upstream (plans, architecture) and at expensive review points. |

Most writers break a factory into the same five parts:

1. **Intake.** Issues, incidents, user requests and the roadmap feed a queue of
   well-specified work.
2. **Harness.** Agents pick up work in isolated environments with the context,
   tools and permissions they need.
3. **Automated verification.** CI, tests, static analysis, security scanning,
   and increasingly LLM reviewers and scenario judges. This step is cheap and
   scales.
4. **Review gate.** The one expensive step, because it needs human judgment.
5. **Deploy and feedback.** What happens in production feeds back into intake.

### The autonomy ladder

Dan Shapiro's "five levels", modeled on self-driving levels, have become the
shared yardstick:

| Level | Name | Human's role |
|---|---|---|
| 0 | Spicy autocomplete | Writes the code; accepts suggestions |
| 1 | Coding intern | Delegates snippets; reviews everything |
| 2 | Junior developer | Pair-programs; still reads every line |
| 3 | Developer | Most code is AI-written; human is a full-time reviewer |
| 4 | Engineering team | Human writes specs and plans; agents build |
| 5 | Dark factory | "A black box that turns specs into software" |

Pulumi and others put most serious teams at level 2 or 3, a few at 4, and treat
level 5 as an experiment that most teams "probably shouldn't try".

### The case studies

**StrongDM (the maximal, "dark" position).** A three-person team since July
2025 with two rules: *code must not be written by humans* and *code must not
be reviewed by humans*. What makes it work for them:

- **Scenarios as holdout sets.** End-to-end user stories live *outside* the
  codebase, hidden from the coding agents, and an LLM judges the software
  against them. They did this after watching agents write `return true` to
  pass tests and rewrite tests to match buggy code.
- **Satisfaction, not green/red.** The pass/fail suite is replaced by the
  fraction of scenario runs that probably satisfy a user.
- **Digital Twin Universe.** Agent-built behavioral clones of Okta, Jira,
  Slack and Google Docs/Drive/Sheets, so scenarios can run thousands of times
  an hour, including failure modes you can't trigger on the real services.
- **Spend.** Their benchmark: "if you haven't spent at least $1,000 on tokens
  today per human engineer, your software factory has room for improvement."

Simon Willison found the team convincing but called no-review "the most
interesting" and least obvious rule, given "how prone LLMs are to making
inhuman mistakes", and noted that $20k per engineer per month makes it as much
a business-model question as an engineering one. A Stanford Law commentary
questioned whether a team building *security* infrastructure should treat
human review "as an obstacle, not a safeguard".

**OpenAI "harness engineering" (level 4–5, with heavy mechanical
enforcement).** Three engineers steering Codex built an internal beta from an
empty repo over about five months: roughly a million lines, no hand-written
code, by their estimate in a tenth of the time. The lesson they draw is that
engineers' work moves to *designing environments, specifying intent and
building feedback loops*: an agent-written `AGENTS.md`, docs in the repo as the
system of record, and architecture rules enforced by linters and CI rather
than by reviewers. (OpenAI's own figures, not independently checked.)

**Vercel's AI SDK factory (lit, specialized, open source).** Built to cope
with 1,000+ open issues and ~800 PRs. Separate agents for triage, analysis,
implementation, review, docs and release backports, running in isolated
sandboxes with limited secrets because the inputs are untrusted public issues.
It reportedly writes 25–35% of merged PRs and closes 70–80% of issues. **A
human approves every merge.**

**Anthropic's long-running-agent harness (the mechanics).** Practical patterns
for agents that work across many sessions with no memory:

- An initializer session writes `init.sh`, a progress log and a feature list.
- The feature list is JSON (models are less tempted to rewrite it than
  Markdown), every item starts failing, and agents may only flip `passes`.
- One feature per session; commit and update the progress log at the end;
  start each session by reading the log and git history and running a smoke
  test.
- Agents marked features done after unit tests or `curl`. Telling them to test
  *as a user* with browser automation substantially improved results.

*Dated (late 2025):* Claude Code now handles much of this harness itself.
`/goal` keeps going until a separate model judges a condition met, Tasks
track work across sessions, models have 1M-token context with automatic
compaction, and sessions resume. The lessons that still hold are testing as a
user and starting each session from written state. You no longer need to
build the scaffolding (initializer agent, `feature_list.json`).

**Geoffrey Huntley's Ralph loop (the minimal position).** A bash loop that
feeds the same prompt file to an agent forever, with a fresh context each
time. One repo, one process, one task per iteration. The skill is in watching
the loop and fixing whatever made it fail so it can't recur. He argues against
multi-agent complexity until it's needed. Anthropic now ships it as a Claude
Code plugin.

*Dated (mid-2025):* Ralph's core idea, "keep going until it's actually done",
became a built-in feature in May 2026: **`/goal`**. The mechanics differ in
ways that matter:

- **A separate judge.** Ralph relies on the working agent to decide it's done
  (or never stops). With `/goal`, a separate small model checks your
  condition after every turn: "you can't trust a model to judge its own
  homework." That judge doesn't run tools. It only reads what the worker
  showed in the transcript, so it's a check on *stopping too early*, not an
  independent oracle.
- **Context.** Ralph throws the context away each iteration, which made sense
  with 2025 context windows. `/goal` continues the same session, using 1M-token
  context and auto-compaction, and restores the goal on resume.
- **Built-in safety rails.** It stops when the condition is judged impossible,
  clears on errors that need a human, stops if turns stop making progress, and
  checks in on stuck background work. In headless runs it's one line:
  `claude -p "/goal …"`. Paired with auto mode, it runs unattended.

What still holds from Ralph: one task per run, state written to files and git,
and the habit of watching the loop and fixing whatever caused a failure.

**Steve Yegge's Gas Town (the orchestration position).** Runs 10+ agents with
persistent worker identities, git worktrees per worker, a merge queue with
verification gates, and **Beads**, a git-backed issue tracker that holds task
state so a crashed session can resume. Users report that planning
(epics, dependencies, acceptance criteria) pays off most, that you should start
with 2–3 agents, and that the costs and fragility are real.

### The skeptics

- **Verification is the bottleneck, not generation.** This is the most common
  point. Dex Horthy (HumanLayer), in "Harness Engineering is not Enough: Why
  Software Factories Fail", describes running a fully unattended factory for
  about four months and then having to debug the result by hand. His numbers:
  agents stay reliable for roughly 3–10 steps and lose the thread past about
  20. "What we're really suffering from is a surplus of bad PRs." He recommends
  mostly deterministic workflows with LLM steps "sprinkled in at just the
  right points", and human review of *plans* before implementation (his
  Research → Plan → Implement framework). *Dated in part:* his factory ran on
  2025 models (it reportedly ended around November 2025), and the "3–10 steps"
  figure doesn't describe 2026 models running long `/goal` sessions. His point
  about verification and understanding is about the system, though, not the
  model, and Osmani repeated it in July 2026.
- **Comprehension debt.** Osmani: "the widening gap between how much code
  exists and how much any human still understands." Dark factories pile it up
  quietly, and the failure arrives late and gradually.
- **Tests share the agent's misunderstanding.** If the agent misreads a
  requirement, the code and the tests are both wrong and everything passes.
  StrongDM's hidden scenarios answer this; most setups don't.
- **AI amplifies whatever you already have.** The 2025 DORA report (~5,000
  respondents) found AI magnifies an organization's existing strengths and
  dysfunctions, with the biggest returns coming from platform quality, clear
  workflows and small batches rather than from the tools.
- **Measured productivity is murky.** METR's 2025 randomized trial found
  experienced open-source developers 19% *slower* with AI while believing they
  were 20% faster. Its 2026 follow-up couldn't produce a clean number because
  developers increasingly refused to work without AI. METR now calls the
  original result historical.
- **Review queues.** LinearB data (8.1M PRs, cited by Encore) says AI-authored
  PRs wait 4.6× longer for a first review.
- **Spec-driven development has limits.** Birgitta Böckeler (Thoughtworks)
  tried Kiro, spec-kit and Tessl. They were overkill for small changes (one bug
  became 4 user stories and 16 acceptance criteria), produced verbose specs
  that were harder to review than code, and agents still ignored parts of the
  spec. Kent Beck and Martin Fowler argue against writing the whole spec up
  front: specs should keep the feedback loop, not replace it.
- **Security.** "Comment and Control" (April 2026) showed Claude Code's
  security-review action, Gemini CLI and Copilot's agent could be hijacked by
  crafted PR titles, comments and issue bodies into running commands and
  leaking `GITHUB_TOKEN` and API keys. A factory that triggers on public
  issues is an injection surface.

### Where the sources agree

Despite the different positions, the same points come up again and again:

1. **The oracle decides how dark you can go.** Lights-out manufacturing waited
   decades after robots for automated measurement. Software is the same.
   Autonomy should be earned by checks that are cheap, fast and hard to fake.
2. **Choose dark or lit per kind of change, not for the whole project.**
   Dependency bumps behind strong gates can run unattended; auth, security,
   schema and release changes stay gated.
3. **Move human judgment upstream.** Reviewing a ten-line plan is cheaper than
   untangling a thousand-line diff.
4. **Keep loops short and give state a home outside the context window:**
   progress files, JSON task lists, issue trackers, git.
5. **Put the knowledge in the repo.** Agent instructions, architecture docs and
   skills are part of the product.
6. **Enforce mechanically.** Every review comment you make twice should become
   a lint, a test or a CI check.
7. **Measure outcomes, not volume.** Lead time, rework and reverts, escaped
   defects and cost per merged change beat "PRs opened" or "tokens burned".

### How dated is this research? (checked October 2026)

The sources cover about 15 months, and agent tooling changed a lot in that
time. Some advice is a workaround for limits that no longer exist. Ordered by
age:

| Source | When | Built on | What still holds / what's dated |
|---|---|---|---|
| Huntley, Ralph loop | May–Jul 2025 | 2025 agents, small context | Watching and fixing the loop still holds. The bash loop itself is replaced by `/goal` (see above). |
| METR productivity RCT | Jul 2025 (tools from early 2025) | Pre-agent IDE assistants | **Stale.** METR itself now calls it historical. Its follow-up couldn't get a clean number. |
| DORA 2025 | Surveyed Jun–Jul 2025 | Mostly chat and autocomplete use | The "AI amplifies your system" thesis is about organizations and probably still holds, but the data predates agents doing the work. |
| Böckeler on spec-driven tools | Oct 2025 | Kiro / spec-kit v1 | The tools have moved on. Her core point (heavy up-front specs are costly to review, so prefer small iterative steps) still holds. |
| Anthropic long-running harness | Late 2025 | Pre-`/goal`, pre-Tasks | Scaffolding is now built in (see above). Testing as a user still holds. |
| Shapiro's five levels | Jan 2026 | — | A vocabulary, not a technique. Still useful. |
| StrongDM factory | Feb 2026 (running since Jul 2025) | Early-2026 models | Hidden scenarios and an independent judge still hold. The $1,000/day figure reflects their scale and early-2026 pricing, not a target. |
| OpenAI harness engineering | Feb 2026 (work Aug 2025–Jan 2026) | GPT-5-era Codex | Docs in the repo and mechanical enforcement still hold. They're tool-agnostic. |
| Gas Town | Jan 2026 onward | Custom orchestrator | Mostly **superseded for us** by built-in orchestration (dynamic workflows, forked and background subagents, agent view, cross-session messaging). |
| "Comment and Control" injection research | Apr 2026 | Claude/Gemini/Copilot Actions | **Current.** Shapes the Security section. |
| Osmani, Light and Dark; Horthy's talk | Jul 2026 | Mid-2026 | Most recent statement of the skeptics' case. Still holds. |
| Vercel AI SDK factory | Aug 2026 | Current tooling | Most recent and closest to our situation (open source, untrusted input, human approves merges). |

Claude Code features from 2026 that change how a small team should build a
factory. Each one replaces something the earlier sources had to build
themselves:

| Feature (shipped) | What it replaces |
|---|---|
| Auto mode (Mar 2026; default since Aug) | `--dangerously-skip-permissions` / "YOLO mode" for unattended runs |
| Computer use in the CLI (Mar 2026), in-app browser (Jul) | Agents can't check a GUI themselves |
| Ultraplan (Apr 2026) | A hand-rolled "post a plan, wait for approval" step: draft a plan in the cloud, comment on it in a web editor, then run it |
| Routines (Apr 2026) | GitHub Actions + `claude -p` cron jobs: scheduled, GitHub-event or API-triggered cloud agents |
| `/ultrareview`, `/code-review`, Code Review levels (Apr–Oct 2026) | Custom review agents: Quick/Standard/Deep per repo, `@claude review always` |
| `/goal` (May 2026) | Ralph loops and "the agent says it's done" |
| Dynamic workflows (May 2026) | Custom multi-agent orchestrators for big fan-out jobs (audits, migrations) |
| Security-guidance plugin (May 2026) | A separate security pass: checks each edit, reviews each turn, does a deeper review on push |
| Forked and background subagents (Jun–Oct 2026) | Manually splitting work across sessions |
| `claude plugin eval` (Sep 2026) | Guessing whether a skill or `CLAUDE.md` rule actually helps: scores it against a no-plugin baseline |

**The takeaway.** Generating code, finishing a task and running unattended
have mostly been productized. What hasn't changed, and is now even more clearly
the bottleneck, is **an independent check that the result is correct**, plus
human judgment on intent and risk. The plan below therefore uses the built-in
features for the loop and puts our own effort into the oracle.

---

## Part 2: Where Lantern stands today

Lantern was built in two days, and 33 of its 42 commits were written by Claude.
It already sits around **level 3–4**: the maintainer specifies and merges,
agents build. Some factory parts already exist:

| Part | Today | Gap |
|---|---|---|
| Intake | Ad-hoc requests in Claude sessions | No issue templates, labels or queue; no spec format |
| Repo knowledge | Excellent README (architecture, source layout, protocols); one skill (`release`) | No `CLAUDE.md`, no skills for common work, no steward/babysit guidance |
| Environment | CI installs CEF via `.github/actions/setup-cef` | Cloud sessions have no SessionStart hook, so agents can't reliably run `cargo clippy`/`cargo test` (CEF is a large download) |
| Verification | rustfmt, clippy `-D warnings`, 24 unit tests on helpers, Linux bundle, macOS/Windows build | **Nothing checks the browser actually works.** No end-to-end test, no screenshots, no tests for `ui/retro/*.js` or the chrome pages, no smoke test of the bundle |
| Review gate | Maintainer reviews every PR; `main` protected | Every change is reviewed the same way regardless of risk |
| Release | Fully automated from `Cargo.toml` + `/release` skill | Good already |
| Feedback | None | No crash or usage signal; releases aren't smoke-tested |

**The main gap is the oracle.** Lantern is a GUI app on top of Chromium.
Today an agent can produce a change that passes every CI check and still
leaves a blank toolbar, broken retro look or a window that won't open. Until
that's fixed, every change needs a human to run the app, and the factory can't
safely speed up.

Lantern also has a few things that make a factory easier:

- **Most of the UI is web content.** The toolbar, status bar, internal pages
  and retro looks are HTML/CSS/JS. They can be tested in the Chromium and
  Playwright already installed in cloud sessions, without CEF.
- **The Rust ↔ UI protocol is narrow and documented** (title channel,
  `lantern.update(...)`, `lantern.invalid` actions), so contract tests are
  straightforward.
- **There's a `--url=` flag**, so a smoke test can point the real bundle at a
  local fixture page.

It also needs some caution. **It's a browser**: URL normalization, escaping,
the `lantern.invalid` action handler, popup and download handling, the
sandbox and the release pipeline are security-sensitive. Those stay lit.

---

## Part 3: The plan

### Design principles

1. **Lit factory, with dark lanes earned one at a time.** Following Osmani and
   Horthy rather than StrongDM: one maintainer can't run a DTU or spend
   $1,000 a day, and comprehension debt matters on a codebase you will own for
   years.
2. **Build the oracle before adding throughput.** Phases 1–2 are mostly about
   verification. More agents come after.
3. **Human time goes on intent and plans, not diffs.** The maintainer writes
   short specs and approves plans. Tiered review handles the rest.
4. **Mechanize repeated feedback.** If a review comment shows up twice, it
   becomes a lint, test or `CLAUDE.md` rule.
5. **Start with one loop.** Add parallel agents only when the queue and the
   oracle can absorb them.

### Phase 0: Foundations (about one day)

Give every agent session the same starting context and a working toolchain.

- **`CLAUDE.md`** at the repo root, short and linking to the README rather than
  repeating it:
  - the commands that must pass before a push (`cargo fmt --check`,
    `cargo clippy --all-targets -- -D warnings`,
    `LD_LIBRARY_PATH="$CEF_PATH" cargo test --bin lantern`, and the Phase 1
    checks once they exist);
  - invariants: the CEF pin moves only with its binaries; all CEF calls stay
    on the UI thread (thread-local `RefCell` state); chrome views stay locked
    down; internal actions only go through `lantern.invalid`; every string
    that reaches HTML or JS goes through `util.rs` escaping;
  - house style (match the surrounding code, comment density, commit message
    style) and the risk tiers below.
- **SessionStart hook** (via the `session-start-hook` skill) that installs the
  apt dependencies, restores or downloads CEF into a cached `CEF_PATH`, and
  warms `cargo build`, so a cloud session can run the full local check suite.
- **Skills** in `.claude/skills/`, written from what past sessions had to work
  out:
  - `steward`: how to drive a PR to green in this repo (which checks matter,
    how to read CEF build failures, when macOS/Windows-only failures are
    real);
  - `retro-look`: anatomy of a look in `ui/retro/`, how `runtime.js` handles
    shadow trees and floating boxes, how to add fixtures and golden shots
    (after Phase 1);
  - `chrome-command`: adding a toolbar command end to end (toolbar.html →
    title channel → `commands.rs` → `lantern.update`), with tests;
  - `cef-upgrade`: moving the `cef` pin and `CEF_CRATE_VERSION` together,
    regenerating, and running the smoke test.
- **Check that the skills earn their place** with `claude plugin eval`: a few
  representative tasks per skill, scored against a run without the skill. Drop
  or rewrite skills that don't beat the baseline. `/skill-doctor` shows what
  each one costs in context.
- **Security-guidance plugin**, with Lantern rules in
  `.claude/claude-security-guidance.md`: escape everything that reaches HTML
  or JS, never widen `lantern.invalid` handling, keep chrome views locked
  down.
- **Turn on Code Review** for the repo: Standard by default, Deep for tier-2
  paths, `@claude review always` on agent PRs. So far no PR has had an
  automated review.
- **Guard the oracle at edit time:** a PreToolUse hook in
  `.claude/settings.json` (with `onFailure: "block"`) that refuses edits to
  `tests/golden/` unless the maintainer sets a flag. CI's check (Phase 1.5)
  stays as the backstop. The black-box scenarios (Phase 1.6) aren't in this
  repo at all.
- **Tune for unattended runs:** run `/auto-mode-setup` once after a full
  build; use `--permission-prompts none` in headless runs; run
  `/doctor prompt-audit` on `CLAUDE.md` and the skills.
- **Issue templates** (`.github/ISSUE_TEMPLATE/`) for *feature*, *bug* and
  *chore*. Each asks for: the problem, acceptance criteria written as
  observable behavior ("with `--retro=portal`, the bookmarks page shows…"),
  out of scope, and a proposed risk tier.
- **Labels:** `factory:ready` (the maintainer approved the spec), `tier:0`,
  `tier:1`, `tier:2`, `factory:in-progress`, `factory:needs-human`.

### Phase 1: Build the oracle (about one to two weeks)

This is the most important phase. Each item below catches a class of breakage
CI misses today. Cheapest first:

1. **Web-layer tests in Playwright, no CEF needed.**
   - Fixture pages in `tests/fixtures/` (static HTML covering tables, forms,
     floats, shadow DOM, iframes, very long pages).
   - Inject `ui/retro/runtime.js` plus each look and assert on behavior: no
     JS errors, footers stay at the end (a past bug), button labels stay
     readable (a past bug), floating boxes are opaque (a past bug), and a
     rescan after DOM changes restyles new nodes.
   - Render `ui/toolbar.html` and `ui/status.html` with a stubbed
     `lantern.update(...)` and check that each command button produces the
     right title-channel string (`lantern|<seq>|<command>|<arg>`).
   - A small Rust test helper (or `cargo run --example dump-pages`) writes the
     data-URI internal pages (home, about, bookmarks, settings, error) to
     files so Playwright can screenshot them.
2. **Golden screenshots.** Playwright `toHaveScreenshot` for every chrome page,
   internal page, and look × fixture, stored in `tests/golden/`. Updating a
   golden image is an explicit act that a human sees in the PR (see "holdouts"
   below).
3. **Rust contract tests.** Table tests for the title-channel parser (already
   partly covered) and the `lantern.invalid` action router, including hostile
   input: oversized arguments, odd encodings, `javascript:` and `data:` URLs,
   and HTML in titles. Add `proptest` round-trip properties for escaping and
   URL normalization, since an agent will read those as tests it must not
   weaken.
4. **Bundle smoke test on Linux CI.** Under `xvfb-run`, start the release
   bundle with `--url=http://127.0.0.1:<port>/fixture.html` against a local
   static server and check that it opens a window, loads the page and exits
   cleanly. The simplest version needs a small debug flag, such as
   `--self-test=<file>`, which writes the final URL, title and load status to
   a file and quits. A screenshot of the X display can come later. The same
   test runs once per retro look. When an agent runs this inside Claude
   Code's Linux sandbox, CEF's own sandbox needs user namespaces. Either
   enable `sandbox.allowNestedUserNamespaces` (added Oct 2026 with Chromium
   as the named case; it weakens the sandbox's file protections) or launch
   the test bundle with `--no-sandbox`.
5. **Protect the white-box oracle.** `CODEOWNERS` makes `tests/golden/`
   maintainer-owned. A CI check fails any PR that changes both product code
   (`src/`, `ui/`) and the oracle (`tests/golden/`, existing test
   assertions), unless the PR has the `oracle-change` label, which only the
   maintainer applies. This is the cheap version of "agents can't rewrite the
   tests to match the bug".
6. **Black-box agent testing (StrongDM's holdouts, scaled down).** Everything
   above is written against the code, so it can share the implementer's
   misunderstanding. This layer tests Lantern the way a user meets it, with
   an agent that has never seen the code.
   - **Hidden scenarios.** Plain-language user journeys live in a separate
     **private repo**, `thebrowser-scenarios`. For example: "start Lantern,
     turn on dial-up at 14.4k, open the image fixture, see the images paint
     top to bottom"; "pick the Portal look, open the bookmarks page, the
     footer is at the bottom". Each scenario lists the fixture pages it needs
     and what a user would observe. Implementing sessions never have this
     repo attached, so agents can't code to the scenarios.
   - **Clean room.** A dedicated CI job (`blackbox.yml`) downloads only the
     built Linux bundle artifact from the PR's CI run, plus the scenarios and
     fixture pages, into an empty directory. It has no checkout of this repo,
     no `CLAUDE.md`, no diff and no test code. Network access is limited to
     the local fixture server and the Claude API.
   - **UI-only tester.** Lantern runs under Xvfb, and a computer-use agent
     (the API's `computer_toolset`, GA since August 2026, driven by a small
     Agent SDK script) sees screenshots and sends mouse and keyboard input.
     It gets nothing else: no file reads, no shell, no Playwright hooks. It
     may start Lantern with documented user-facing flags (`--url=`,
     `--retro=`) because users can too.
   - **Verdicts and gating.** Each scenario runs 3 times. A run returns
     *satisfied / not satisfied / unclear*, with the screenshots that
     support it. The PR check passes only when every scenario touched by the
     PR's tier is satisfied in at least 2 of 3 runs. Any *unclear* result,
     or a 2–1 split on a tier-2 PR, goes to the maintainer with the
     screenshots attached. The job posts a summary comment on the PR.
   - **Checking the checker.** Monthly, and whenever scenarios change, the
     job also runs against **deliberately broken builds**: a blank toolbar,
     a retro look that never applies, dial-up mode that doesn't throttle, a
     status bar that never updates. These are produced from a small set of
     patches the maintainer keeps in the scenarios repo. If the judge passes
     a broken build, the scenario or the judge prompt gets fixed before the
     check is trusted again.
   - **When it runs.** On tier 1–2 PRs after ordinary CI is green, and
     nightly on `main`. Not on every push or on tier-0 PRs, because
     computer-use runs are slow and cost more than ordinary CI.
   - **Security.** The job holds an Anthropic API key, so it runs only on PRs
     from this repo's own branches (never forks) and after the maintainer's
     `factory:ready` label. It gets a read-only `GITHUB_TOKEN` plus
     permission to post its comment, and a deploy key that can only read the
     scenarios repo.

   While the black-box job is being built, an agent can still run the real
   app and look at it with computer use in the Desktop app (cloud containers
   have no display). That's useful evidence to attach to a visual PR, but
   it's not a gate.

Once Phase 1 is done, an agent can show a change works without the maintainer
running the app. That's what unlocks the next phases.

### Phase 2: Intake and the main loop (about one week)

Wire the queue to the harness with a **plan gate** (Horthy's Research → Plan →
Implement):

1. The maintainer files or approves an issue and adds `factory:ready`. Only a
   maintainer-applied label starts work, never an outside comment or a newly
   opened issue (see Security).
2. **Plan.** A routine triggered by the label event reads the issue,
   `CLAUDE.md` and the relevant code, then drafts a short plan: files to
   touch, approach, the tests that will prove it, and the proposed tier. Five
   to fifteen lines, not a spec-kit document. Use Ultraplan, so the maintainer
   can comment on the plan in the web editor, or post the plan as an issue
   comment. The maintainer approves or corrects it. **This is the main human
   checkpoint.**
3. **Implement under a goal.** An agent in auto mode on its own branch, one
   issue per PR, runs a `/goal` built from the approved plan. For example:

   ```text
   /goal Every acceptance criterion in issue #N holds, shown by these all
   exiting 0 in this session: cargo fmt --check;
   cargo clippy --all-targets -- -D warnings;
   LD_LIBRARY_PATH="$CEF_PATH" cargo test --bin lantern; npx playwright test.
   For a bug, a new test failed before the fix and passes after it.
   No file under tests/golden/ is changed and no existing
   assertion is removed or loosened. Or stop after 40 turns and report what
   is left.
   ```

   The goal's evaluator stops the agent quitting early. It **doesn't**
   replace CI: it only reads what the agent printed, so the "don't touch the
   oracle" constraint is enforced again by the CI check from Phase 1.5. Keep
   a progress note in the PR description so another session can take over.
4. **Verify.** CI runs the Phase 1 oracle, plus Claude Code Review (Standard
   for tiers 0–1, Deep for tier 2), plus the black-box agent test (Phase 1.6)
   for tiers 1–2. A failing black-box verdict sends the agent only the
   scenario's *title* and the judge's screenshots, never the scenario text,
   so the holdout stays hidden. The
   implementing agent stewards its own PR to green under the `steward` skill
   (PR auto-fix covers routine CI breakage).
5. **Review by tier**, below.

Start with **one** implementing session at a time. Add a second only once the
plan-to-merge lead time is steady and the maintainer's review queue isn't
growing. Scale with built-in tools before building any of our own: parallel
cloud sessions or worktrees for separate issues, forked subagents within a
session, and a dynamic workflow for one-off fan-out jobs such as a
codebase-wide audit or a CEF API migration.

#### Risk tiers

| Tier | Examples | Gate |
|---|---|---|
| **0: dark lane** | Typo/docs fixes, README screenshots, CI action version bumps, clippy fixes for new lints, pixel-art tweaks, new fixture pages | Auto-merge when CI, the oracle and Claude Code Review are green. The maintainer reads the weekly digest, not each diff. |
| **1: skim** | Retro look changes, internal page styling, new toolbar commands, settings additions, dial-up profiles | Maintainer looks at the plan, the screenshot diff and the test changes, then skims the code. |
| **2: full review** | `client.rs` request, popup and download handling; `commands.rs` `lantern.invalid` router; `util.rs` escaping and URL normalization; sandbox and helper setup; the CEF pin; `release.yml`, `bump-version.sh`, CI config | Maintainer reads the whole diff. Claude Code Review runs at high effort, plus `/security-review`. Never auto-merged. |

The tier is mechanical: a CI step maps changed paths to the highest tier
touched and labels the PR, so an agent can't downgrade its own work. A PR
starts in tier 0 only if every path is on the tier-0 allowlist.

### Phase 3: Background loops (ongoing, after Phase 2 is stable)

Small, scheduled loops in Horthy's style: each opens at most one small PR,
and stops with nothing if there's nothing to do. Run them as Claude Code
routines (schedule or GitHub-event triggers), each with its own prompt and the
narrowest connectors and permissions it needs. Use plain scheduled GitHub
Actions only for steps that don't need a model (Dependabot, the release
download check).

| Loop | Cadence | Output |
|---|---|---|
| **Triage** | On issue open | Labels, duplicate check, a question if acceptance criteria are missing. Never starts work. |
| **CEF watcher** | Weekly | When a new `cef` crate appears, opens a tier-2 draft PR that moves the pin and `CEF_CRATE_VERSION`, with smoke-test results. |
| **Dependency and Actions bumps** | Weekly | Tier-0 PRs (Dependabot plus an agent fixing any breakage). |
| **Lint ratchet** | Nightly | Fixes one instance of one anti-pattern (for example an `unwrap()` in a callback, a missing escape, a TODO) and opens one tiny PR. |
| **Docs drift** | Weekly | Checks that the README's source layout, command list and protocol section match the code; opens a tier-0 fix. |
| **Black-box explorer** | Nightly | An unscripted version of the Phase 1.6 tester. In the same clean room, it tries to break Lantern the way a curious user would: odd URLs, very long pages, each retro look on tricky fixtures, dial-up mode at every speed, unusual settings combinations, rapid back/forward. It files an issue with screenshots and steps to reproduce for each distinct problem, deduplicated against open issues, and caps itself at a few issues a night. It never blocks a merge. Confirmed findings become new hidden scenarios. |
| **Release smoke** | On release | Downloads the published Linux asset, checks `SHA256SUMS.txt`, runs the smoke test, and files an issue if it fails. |
| **Weekly digest** | Weekly | One issue or artifact: what merged by tier, what the factory attempted and dropped, cost, and the metrics below. |

### Phase 4: Widen the dark lanes, carefully

Every month or so, look at the metrics and move **one category** of change to
a lower tier only if its track record supports it. For example, retro-look
changes could move from tier 1 to tier 0 after a month with no reverts and no
escaped regressions in that area, once golden coverage of the looks is good.
Tier 2 never goes dark.

### Security

Lantern is a public repository and a browser, so the factory must assume
hostile input.

- **No agent runs on untrusted triggers with secrets.** Agents start on
  maintainer actions (a label, a comment from the maintainer, a schedule),
  never on `issues: opened`, `pull_request_target` or outside comments. Triage
  can read public issues but runs with no secrets and a read-only token.
- **Never interpolate issue or PR text into a shell command or workflow
  expression.** Pass it as a file or environment variable the agent reads as
  data.
- **Least privilege per loop:** `contents: write` only for implementation
  jobs, no release permissions in any agent job, and the release workflow
  stays human-triggered by merging a version bump.
- **Tier 2 includes everything that touches security.** Escaping, URL
  handling, navigation interception, the sandbox and CI/release config always
  get a human read.
- **Pin third-party Actions to a commit SHA** in agent workflows.

### Metrics

Track these in the weekly digest (rolling 4 weeks). Don't use volume as a goal.

| Metric | Why |
|---|---|
| Lead time: issue `factory:ready` → merged | Is the factory actually faster? |
| First-push green rate | How good the harness and local checks are |
| Plan corrections per issue | Spec and context quality; a rising number means `CLAUDE.md` or the skills are missing something |
| Revert / follow-up-fix rate per tier | The safety signal for moving lanes darker |
| Escaped defects (bugs found after merge, by area) | Whether the oracle has gaps |
| Maintainer minutes per merged PR | The real bottleneck |
| Cost per merged PR | Sustainability; Simon Willison's $200/month is a more realistic anchor here than StrongDM's $1,000/day |
| Oracle coverage (looks × fixtures with goldens, scenarios passing) | Whether autonomy has been earned |
| Black-box satisfaction rate on `main`, and broken builds caught | Whether the black-box tester can be trusted. Any broken build it passes is a red flag. |
| Explorer findings: confirmed vs. noise | Whether the nightly explorer is worth its cost |

### What we're deliberately not doing

- **No full dark factory.** Without a strong oracle it just produces
  comprehension debt quickly, and the browser has security-sensitive surfaces.
- **No heavyweight spec-driven tooling** (spec-kit/Kiro-style document sets).
  A short issue with observable acceptance criteria plus a reviewed plan gets
  most of the value at a fraction of the review cost.
- **No custom harness or orchestrator.** No Ralph script, no
  `feature_list.json` initializer, no Gas Town. `/goal`, auto mode, routines,
  subagents and dynamic workflows cover these now. Our own engineering goes
  into the oracle, which no product feature replaces.
- **No swarm on day one.** One loop, then two, then more only if metrics
  justify it.
- **No digital twins.** Lantern has no third-party SaaS dependencies. The
  equivalent is local fixture pages and a local HTTP server (with throttling
  for the dial-up simulator).

### First steps, in order

1. Add `CLAUDE.md` and the SessionStart hook (Phase 0).
2. Add the Playwright harness with fixtures, retro-look behavior tests and
   chrome-page tests, and wire it into CI (Phase 1.1).
3. Add golden screenshots and the oracle-protection CI check (Phase 1.2, 1.5).
4. Add the `--self-test` flag and the Xvfb bundle smoke test (Phase 1.4).
5. Create the private `thebrowser-scenarios` repo with 5–10 scenarios and
   the broken-build patches. Build the clean-room `blackbox.yml` job, and
   check it catches every broken build before it gates anything (Phase 1.6).
6. Add issue templates, labels, the path-to-tier CI step and the plan-gate
   workflow (Phase 2).
7. Run the loop on a handful of real issues, adjust `CLAUDE.md` and the
   skills from what went wrong, then turn on the first background loops,
   including the black-box explorer.

---

## Sources

Definitions and the case for factories:

- Addy Osmani, [Software Factories, Light and Dark](https://addyosmani.com/blog/software-factories/)
- Dan Shapiro, [The Five Levels: from Spicy Autocomplete to the Dark Factory](https://www.danshapiro.com/blog/2026/01/the-five-levels-from-spicy-autocomplete-to-the-software-factory/), discussed by [Simon Willison](https://simonwillison.net/2026/Jan/28/the-five-levels/)
- StrongDM AI, [Software Factory](https://factory.strongdm.ai/), and Simon Willison, [How StrongDM's AI team build serious software without even looking at the code](https://simonwillison.net/2026/Feb/7/software-factory/)
- OpenAI, [Harness engineering: leveraging Codex in an agent-first world](https://openai.com/index/harness-engineering/)
- Vercel / Lars Grammel, "Building a software factory for AI SDK" ([mirror](https://www.plushcap.com/content/vercel/blog/vercel-building-a-software-factory-for-ai-sdk))
- Anthropic, [Effective harnesses for long-running agents](https://anthropic.com/engineering/effective-harnesses-for-long-running-agents)
- Geoffrey Huntley, [everything is a ralph loop](https://ghuntley.com/loop/)
- Steve Yegge's Gas Town, via [Cloud Native Now](https://cloudnativenow.com/features/gas-town-what-kubernetes-for-ai-coding-agents-actually-looks-like/) and [heise](https://www.heise.de/en/background/Full-Control-Gas-Town-Orchestrates-Ten-or-More-Coding-Agents-11178824.html?seite=2)
- Encore, [What Is an AI Software Factory?](https://encore.dev/articles/what-is-an-ai-software-factory) (history, LinearB review-wait figure)
- BCG Platinion, [The Agentic Software Factory](https://www.bcgplatinion.com/insights/the-agentic-software-factory)
- Pulumi, [The dark factory pattern](https://www.pulumi.com/blog/dark-factory-pattern-pulumi-autonomous-iac/)

Skepticism and evidence:

- Just Understanding Data, [Lights-On vs. Dark Software Factories](https://understandingdata.com/posts/lights-on-vs-dark-software-factories/)
- Stanford Law CodeX, [Built by Agents, Tested by Agents, Trusted by Whom?](https://law.stanford.edu/?p=557739)
- Google DORA, [2025 State of AI-assisted Software Development](https://www.dora.dev/research/ai)
- METR's 2025 developer-productivity RCT and 2026 redesign, via [scienceblog](https://scienceblog.com/?p=582102) and [Conffab](https://conffab.com/?p=10163)
- Birgitta Böckeler, [Understanding Spec-Driven-Development: Kiro, spec-kit, and Tessl](https://www.martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html)
- SecurityWeek, [Claude Code, Gemini CLI, GitHub Copilot agents vulnerable to prompt injection via comments](https://www.securityweek.com/claude-code-gemini-cli-github-copilot-agents-vulnerable-to-prompt-injection-via-comments/)

Current tooling (checked October 2026):

- Claude Code docs, [Keep Claude working toward a goal (`/goal`)](https://code.claude.com/docs/en/goal)
- Claude Code docs, [What's new](https://code.claude.com/docs/en/whats-new) (weekly digests: auto mode, routines, Ultraplan, `/ultrareview`, dynamic workflows, security-guidance plugin, `claude plugin eval`) and the [changelog](https://code.claude.com/docs/en/changelog) (Code Review levels, forked subagents by default)
- VentureBeat, [Claude Code's /goals separates the agent that works from the one that decides it's done](https://venturebeat.com/orchestration/claude-codes-goals-separates-the-agent-that-works-from-the-one-that-decides-its-done)
- Addy Osmani, [The Factory Model](https://addyosmani.com/blog/factory-model/) (Feb 2026)

Figures from StrongDM, OpenAI and Vercel are self-reported. Some details
(Vercel's percentages, Horthy's talk, Gas Town costs) come from secondary
write-ups and should be treated as approximate.
