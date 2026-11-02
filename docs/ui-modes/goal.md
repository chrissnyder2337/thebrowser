# UI modes: the goal

A copy-paste `/goal` prompt that builds every UI mode in [README.md](README.md)
and checks each against the browser it is modelled on. Paste it into a cloud
session on this repo, in auto mode. Claude works until the condition holds,
committing and pushing one batch of modes at a time; check progress with
`/goal`, and stop a run with `/goal clear`. Subagents drafting a mode use
[agent-brief.md](agent-brief.md).

The goal evaluator only reads what Claude prints, so every condition asks for
printed evidence. CI is still the real check: a PR that changes existing
goldens needs the maintainer's approval on the `oracle` environment.

```text
/goal Every UI mode in the Modes table of docs/ui-modes/README.md (all rows, including terminal-1992 and the "Later" and "Not planned" ones) is built on branch claude/90s-browser-ui-modes-r693xg, looks as close as possible to the browser it is modelled on, and is pushed with a draft PR titled "UI modes: every mode". Show as command output in this session:
(1) Each mode has its ui/chrome/<key>/ pages, UiMode arm, view heights, page defaults and era look, and `--ui=<key>` works. Re-architect wherever a faithful layout needs it: chrome below or beside the page, hidden views, inset and zoomed content (TV), per-mode menu bars with the original menus' names and items (grow menus.rs past file/navigate/options/help), side panels (DOS graphics, the Workspace hotlist), several content views or floating windows (Workspace, Cube workstation), and new retro looks where a mode's pages need one. docs/ui-modes/spec.md describes every new pattern.
(2) Fidelity. Per mode: a reference image of the original (docs/research/90s-browsers/images/, or one found online, used only for comparison, kept in the scratchpad, never committed) and a side-by-side of it with Lantern in that mode (the real app under Xvfb on a long Wikipedia article), saved to the scratchpad. A Sonnet subagent that did not build the mode compares the two and lists every difference in rows, controls and their order, labels, proportions, colours, fonts, bevels, status panes and page rendering. Fix and re-compare until the only differences left come from the branding rule (original icons, the Lantern lantern in activity boxes, no product names) or a missing browser feature, each listed under "Fidelity" in that mode's section of docs/ui-modes/modes.md. Print the reviewer's final verdict per mode.
(3) Tests. Every mode and look has the contract test and new goldens. Existing goldens and tests may be updated where the work needs it (the maintainer has approved this); list every changed existing golden or assertion in the PR description for the "oracle" environment approval. The settings page gains a Browser UI section and lists every retro look; tests/smoke.sh covers every look and every --ui mode.
(4) All exit 0: every check in the steward skill's "Before every push" section, tests/web/run.sh and tests/smoke.sh. /code-review and /security-review (tier 2) were run and every finding fixed or answered.
(5) A final table is printed: key | built | contract test | goldens | fidelity verdict | remaining differences, and docs/ui-modes/README.md shows every mode Built.
How to work: batches of 3-5 modes, one commit and push per batch, with the README status table updated each time. Draft each mode's pages, icons and looks with Sonnet subagents at medium effort, up to 5 running in parallel, from a brief (docs/ui-modes/agent-brief.md; extend it as patterns change); run the fidelity reviewers as Sonnet at medium effort too, and use Haiku at low effort for lookups and scans. If a usage limit stops subagents, keep going in the main session where possible and resume them after the reset. Keep the Rust, tier-2 files, builds, goldens and reviews in the main session. Follow the branding rule: faithful layout and wording, original art, no product names in anything that ships. Or stop after 300 turns and report what is left.
```
