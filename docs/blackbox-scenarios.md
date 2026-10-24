# Black-box scenarios

The black-box tester (`.github/workflows/blackbox.yml`, Phase 1.6 of
[software-factory.md](software-factory.md)) judges Lantern against
plain-language user journeys kept in a **private** repository, named by the
`SCENARIOS_REPO` Actions variable. Implementing sessions never have that
repository attached, so nobody can code to the scenarios. This page
describes their format; the scenarios themselves are never committed here.

## Repository layout

```
scenarios/s01.md     one scenario per file
scenarios/s02.md
fixtures/…           pages the scenarios open, served at http://127.0.0.1:8000/
```

- **File names are opaque ids**: `s` and two or three digits (`s01.md`).
  The id is the only part of a scenario that appears in public places (the
  PR comment, the job summary, screenshot names), so it must not describe
  the scenario. `verdicts.py` refuses other names.
- **Fixtures** are static files. Lantern can't reach the internet during the
  test, so a scenario can only use pages from `fixtures/` and Lantern's own
  pages.
- Start with 5–10 scenarios, the ones that matter most.

## Scenario format

Markdown, written for a user, not a developer: name what they click and
what they should see, never selectors, function names or internals. The
judge only has the screen.

```markdown
# Title: one line

Fixtures: the pages it opens

## Steps
1. What the user does, in order: the toolbar button, menu item, setting or key.

## Expect
- One observation per line, each checkable in a screenshot.
- Say when timing matters ("within 5 seconds", "still partly drawn after 3 seconds").
```

Every **Expect** line must be visible on screen. "Satisfied" means the judge
saw all of them in screenshots it read; it is told to be literal.

## Example

The only scenario written down outside the private repository, and for that
reason not one of the real ones:

```markdown
# Dial-up paints images slowly

Fixtures: images.html (a page with a large photo)

## Steps
1. Start Lantern on images.html.
2. Click the modem panel in the status bar to open Settings.
3. Choose "Slow images" and "14.4K", and save.
4. Load images.html again.

## Expect
- About 3 seconds after loading, the photo is only partly drawn: a band at
  the top, the rest blank.
- The status bar shows 14.4K.
- Within a minute the whole photo is drawn.
```

## How a run uses them

`verdicts.py` builds the judge's brief from `tools/blackbox/judge-prompt.md`
and the scenarios. Each scenario runs once; one that isn't satisfied runs
twice more and passes only if both re-runs are satisfied (2 of 3). The
judge is told never to quote a scenario, and the workflow publishes only ids,
verdicts and screenshots.

**When scenarios change**, dispatch `mutants.yml`, then `blackbox.yml` once
for each broken build (its `build` input). Every broken build must fail at
least one scenario. If one passes, fix the scenario or the judge prompt
before trusting the tester again.
