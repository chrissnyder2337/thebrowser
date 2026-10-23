# Claude feature audit (May–October 2026)

An audit of what Anthropic shipped between late May and October 23, 2026,
against what Lantern uses today. It feeds the
[software factory plan](software-factory.md).

**What was read:**
- every "Added" entry in the Claude Code changelog from 2.1.150 (May 23) to
  2.1.309 (Oct 22);
- the weekly "What's new" digests (weeks 22–37; none were published after
  that, so the changelog covers mid-September onward);
- the Claude Platform release notes (May–Oct);
- Anthropic's engineering blog (May 25 and Oct 13 posts, plus the Mar 24
  harness post, which the first research pass missed);
- the Anthropic news page (Aug 27–Oct 22).

**What Lantern uses today:** one project skill (`release`), GitHub Actions CI,
and Claude Code cloud sessions. There's no `CLAUDE.md`, no
`.claude/settings.json`, no hooks, no plugins, no routines, and PR #8 had no
automated review at all.

---

## Adopt now: high value, low effort

| Feature | Shipped | Why it matters for Lantern | How |
|---|---|---|---|
| **Code Review** on PRs, with review levels | GA earlier; levels 2.1.304 (Oct 19); `@claude review always` / `stop` (2.1.307) | No PR has had an automated review. Levels map onto our risk tiers. | Install the Claude GitHub app's Code Review for this repo. Default **Standard**, **Deep** for tier-2 paths. `@claude review always` on agent PRs. |
| **`CLAUDE.md`** plus **`/doctor` trimming** and **`/doctor prompt-audit`** | 2.1.206 (Jul 9); 2.1.283 (Sep 25) | No project instructions exist yet. `prompt-audit` flags instructions written for older models, so run it on `.claude/skills/release` too. | Write `CLAUDE.md` (plan Phase 0), then run `/doctor` and `/doctor prompt-audit`. |
| **SessionStart hook**, with `reloadSkills: true` and **`onFailure: "block"`** | May 27; 2.1.295 (Oct 8) | Cloud agents can't reliably run clippy or tests without CEF installed. `onFailure: "block"` makes a required check fail closed instead of being silently skipped. | Install CEF and warm cargo in a SessionStart hook. Use `onFailure: "block"` on guard hooks. |
| **`/goal` + auto mode** (auto mode default since Aug 14) | May 12; check-ins and retries Aug–Sep | The built-in loop with a separate evaluator for the implementation step. | Already in the plan. In headless runs, add `--permission-prompts none` (2.1.259) so anything that would prompt is denied rather than hanging. |
| **`/auto-mode-setup`** | 2.1.304 (Oct 19) | Teaches auto mode what's normal here: CEF downloads, cargo and apt hosts, `bundle-cef-app`. That means fewer false blocks during unattended runs. | Run it once in a session that has done a full build. |
| **Sandbox: `allowNestedUserNamespaces`** | 2.1.305 (Oct 20) | The changelog names **Chromium** as the case for this. The bundle smoke test (Phase 1.4) starts CEF, and CEF's sandbox needs user namespaces inside Claude's Linux sandbox. | Turn it on only in settings used for smoke-test runs (it weakens the sandbox's file protections), or launch the test bundle with CEF's `--no-sandbox`. Note the tradeoff in `CLAUDE.md`. |
| **PreToolUse hook guarding the oracle** | Hooks long-standing; `onFailure: "block"` new | Blocks Edit/Write to `tests/golden/` and `scenarios/` unless the maintainer sets a flag. This enforces the plan's "agents don't rewrite the tests" rule at edit time, before CI ever sees it. | A command hook in `.claude/settings.json` that runs in every session, cloud included. |
| **Security-guidance plugin** | May 28 | Checks each edit for vulnerability patterns, reviews each turn, and does a deeper review on push. Lantern's escaping, URL handling and `lantern.invalid` routing are exactly the patterns it's for. | `/plugin install security-guidance@claude-plugins-official`, plus Lantern rules in `.claude/claude-security-guidance.md`. |
| **Cheaper current models** | Opus 5.5 (Sep 22, $4/$20, ~40% cheaper to run than Opus 5); Sonnet 5.5 (Sep 28); Haiku 5.5 (Oct 7, $0.10/$0.50) | Makes the factory much cheaper than the early-2026 cost figures suggest. | Opus 5.5 for planning and tier-2 work. Sonnet 5.5 for routine loops. Haiku 5.5 for triage subagents and the `/goal` evaluator. The Agent tool's `effort` parameter (2.1.292) sets effort per subagent. |

## Adopt in Phase 1–2

| Feature | Shipped | Use |
|---|---|---|
| **Computer use** (CLI preview Mar; Desktop background on macOS Sep); **in-app browser** (Jul); platform `computer_toolset` GA (Aug 19) | — | Lantern is a GUI app, and computer use lets an agent open it and look. Use it for evidence on visual PRs (screenshots of the retro looks in the real window), with Playwright and the Xvfb smoke test as the deterministic gate. It needs a display, so use the Desktop app locally; cloud containers don't have one. |
| **Scenario judge as an isolated subagent**: `omitClaudeMd` (2.1.271), `disallowed-tools` in frontmatter, `--restricted` (2.1.248) | Aug–Sep | The plan's holdout scenarios need a judge that doesn't share the implementer's context. Define a `scenario-judge` agent with `omitClaudeMd: true`, read-only tools, and Haiku or Sonnet. |
| **`claude plugin eval`** (2.1.269, Sep 11) with `--disallow-tools` (2.1.299) | Sep–Oct | Score each project skill (`release`, `steward`, `retro-look`, …) against a run without it, and keep only the skills that help. Skills in `.claude/skills` already load as a plugin (May). |
| **`/skill-doctor`** (2.1.261) | Sep 4 | Shows each skill's context cost and how often it's used, to prune with. |
| **Routines** (Apr), plus the "why the last run didn't start" banner (2.1.302) | — | The plan's triage, CEF watcher, lint ratchet, docs drift and digest loops. |
| **Ultraplan** (Apr) | — | The plan gate: comment on the agent's plan in a web editor, then run it. |
| **`claude ultrareview`** in CI (Apr; needs `--trust-workspace` since 2.1.309) | — | A deeper multi-agent bug hunt on tier-2 PRs, run from a workflow on maintainer request. Not on every push, since it's expensive. |
| **Claude Security plugin** (Jul 20) | — | A one-off multi-agent vulnerability scan of the whole codebase, before the factory starts running unattended. Findings turn into patches you choose to apply. |
| **Dynamic workflows** (May 28; size settings Jul; per-workflow model 2.1.296) | — | For one-off fan-out jobs, not the main loop. Examples: "audit every CEF callback for off-UI-thread access", "migrate to the next `cef` crate across all handlers", "add golden tests for every look × fixture". |
| **Artifacts** (Jun; MCP connectors Jul) | — | The weekly digest as a private page. Also "walk through this PR with the diff annotated" pages to speed up tier-1 skims. |

## Useful when running more than one agent

| Feature | Shipped | Use |
|---|---|---|
| **Agent view** (`claude agents`), background sessions, `claude attach` / `logs` | May–Oct | Watch several local sessions working on separate issues. |
| **Forked / background subagents by default** (Jun–Oct), `/fork` (Jul) | — | Side tasks inside one session without losing context. |
| **Cross-session messaging** (`SendMessage`, `ListAgents`, `notify_when_idle`) | Aug | Pass a finding from one session to another, for example "the CEF upgrade changed callback signatures". |
| **"You should know" built-in mod** (2.1.287, Oct 1) | — | A side agent that flags things you or Claude might miss. Cheap to try. |
| **Mods: `tool.check` with `decider: "person"`** (2.1.304) | — | A stronger version of the oracle guard for local sessions: only a person can approve edits to protected paths. The hook above is the version that also works in cloud sessions. |

## Measure the factory

| Feature | Shipped | Use |
|---|---|---|
| `/usage` breakdowns (by skill, subagent, plugin, MCP, loop, workflow) | May–Oct | Cost per loop for the weekly digest. |
| `/cost` prompt-cache lines (2.1.251, 2.1.260) | Aug–Sep | Spot loops that keep missing the cache and paying for it. |
| OpenTelemetry: repository attributes (2.1.269), `safety_event` (2.1.307), cache-write split (2.1.300) | Sep–Oct | Only worth it if you want dashboards. For one maintainer, `/usage` is enough. |

## Considered and not needed (for now)

- **Managed Agents** on the Claude Platform: scheduled deployments, Outcomes,
  webhooks, multiagent orchestration, `ant apply` agents-as-code, server-side
  `auto` permission policies (May–Oct). This is the route to a Vercel-style
  webhook-driven factory on your own infrastructure. For one maintainer,
  Claude Code routines and cloud sessions do the same job with no service to
  run. Revisit if Lantern gets outside contributors at volume.
- **Self-hosted environments** (Aug 7; Team/Enterprise only). These would give
  cloud sessions a machine with a real display or GPU for GUI tests. Only
  relevant if you're on those plans and Xvfb proves too limited.
- **AGENTS.md support** (2.1.277). Claude Code only reads `AGENTS.md` when
  there's no `CLAUDE.md`. Only useful if other agent tools also work on this
  repo.
- **`/design`** (Aug, research preview) for retro-look mockups. Optional. Note
  that standalone Claude Design closes December 14, 2026.
- **Claude Tag** (Slack), **gateway** and **managed-settings** features:
  organization administration, not relevant to a solo repo.

## Lessons from Anthropic's own posts

- **"Harness design for long-running application development" (Mar 24).**
  Separate planner, generator and evaluator agents, with the evaluator testing
  the running app in a browser rather than reading code. This is the same
  shape as the plan (plan gate → `/goal` implementation → Playwright and
  scenario judge), and it backs putting the most effort into the evaluator.
- **"How we contain Claude across products" (May 25).**
  - Deterministic boundaries over prompts: sandbox writes to the workspace,
    deny network by default, keep secrets out of the environment.
  - Treat every allowlisted domain as a capability, and treat cloned repos as
    hostile input.
  - Auto mode misses about 17% of overeager actions, so it's one layer inside
    the sandbox, not a replacement for it.
  - For Lantern, this means: no tokens in agent sessions beyond what each job
    needs, a strict network allowlist for routines, and the oracle guard as a
    hook rather than an instruction.
- **"Redesigning source control at Anthropic" (Oct 13).** Merges grew from about
  2,500 a day in May to over 30,000 in September, until version control itself
  became the bottleneck. At our scale the equivalent is **CI time and
  merging**. The Linux job builds CEF and bundles, and three runners must pass.
  Once more than one agent is running, add GitHub's merge queue and keep CI
  fast (cache CEF and `bundle-cef-app`, as CI already does).
- **"Donating Agent Skills to the Agentic AI Foundation" (Oct 22).** Skills
  are now an open standard adopted by 70+ platforms, so effort put into
  Lantern's skills isn't tied to one tool.

## Sources

- [Claude Code changelog](https://code.claude.com/docs/en/changelog) (2.1.150–2.1.309)
- [Claude Code: What's new](https://code.claude.com/docs/en/whats-new) (weeks 22–37)
- [Claude Platform release notes](https://platform.claude.com/docs/en/release-notes/overview)
- [Anthropic Engineering](https://www.anthropic.com/engineering): [How we contain Claude across products](https://anthropic.com/engineering/how-we-contain-claude); Redesigning source control at Anthropic (via [Simon Willison](https://simonwillison.net/2026/Oct/13/redesigning-source-control-at-anthropic/) and [The Register](https://www.theregister.com/a/5302943)); Harness design for long-running application development (via [InfoQ](https://infoq.com/news/2026/04/anthropic-three-agent-harness-ai/))
- [Anthropic News](https://www.anthropic.com/news) (Aug 27–Oct 22: Opus 5.5, Sonnet 5.5, Haiku 5.5, Fable 5.1, Agent Skills foundation)
- [Keep Claude working toward a goal](https://code.claude.com/docs/en/goal)
