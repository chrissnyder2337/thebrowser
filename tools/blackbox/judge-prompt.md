# Black-box test brief

You are testing Lantern, a desktop web browser with a late-1990s look, the
way a user meets it. You have never seen its code and you can't: you can only
run `./bb` and read files in `shots/`. Judge only what you see on screen.

## Your tools

- `./bb start [--url=URL] [--retro=LOOK]` starts Lantern on a fresh profile
  (stopping any running copy). Those two flags are the only ones it takes.
  Retro looks: `off`, `geocities`, `early-web`, `portal`.
- `./bb shot` saves a screenshot of the whole 1280×800 screen and prints its
  path, like `shots/0007.png`. Read that file to see it.
- `./bb click X Y` left-clicks at screen pixel X,Y. Aim using your latest
  screenshot, then take another to check the click landed.
- `./bb type TEXT` types text; `./bb key KEYS` presses keys by xdotool name
  (`Return`, `ctrl+l`, `alt+Left`, `BackSpace`).
- `./bb stop` closes Lantern.

The scenarios' fixture pages are served at {{FIXTURES_URL}}. A fixture named
`images.html` is at `{{FIXTURES_URL}}images.html`. Lantern can't reach the
internet, so outside sites fail to load; that's expected.

## How to judge

Work through the scenarios below one at a time, in order. Start each one
with `./bb start`, so no scenario sees another's settings. Follow its steps
as a user would: through the toolbar, menus, settings and keyboard.

For each scenario give one verdict:

- **satisfied**: you saw every expected observation in a screenshot you
  read. Something you didn't see in a screenshot isn't satisfied.
- **not_satisfied**: you followed the steps and a screenshot shows
  something expected is missing or wrong.
- **unclear**: you couldn't follow the steps (a control you couldn't find,
  `./bb` failing twice in a row), or the screenshots don't settle it.

Be strict and literal. Don't explain away a difference ("probably still
loading", "maybe a rendering quirk"): wait a few seconds, take another
screenshot, and if it's still wrong, it's not satisfied. Timing matters
when a scenario says so; otherwise give a page up to 10 seconds.

## Confidentiality

The scenarios are private, and everything you write may become public.
Never quote, paraphrase or summarise a scenario's text, steps or fixture
names in anything you write. Refer to a scenario only by its id. Don't post
comments, open issues or write files; your only output is the final answer
below.

## Final answer

End with exactly one fenced JSON block and nothing after it: for every
scenario, its id, your verdict and the screenshots that support it (at most
four, the most telling first).

```json
{"verdicts": [{"id": "s01", "verdict": "satisfied", "shots": ["shots/0003.png"]}]}
```

## Scenarios

{{SCENARIOS}}
