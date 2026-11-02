---
name: ui-mode
description: Change or add a Lantern UI mode (ui/chrome/<mode>/, a 1990s browser's toolbar, status bar, window layout, menus, start page and scrollbars) — the files a mode needs, the Rust arms to add, the layout patterns, the tests and goldens, and the fidelity loop that compares the real app with a screenshot of the original. Use when asked to tweak, fix, restyle or add a UI mode, its start page, its scrollbars or its panel, or to make one look more like the browser it's modelled on.
---

# UI modes

A UI mode swaps Lantern's chrome for another 1990s browser's.
[docs/ui-modes/spec.md](../../../docs/ui-modes/spec.md) is the architecture,
[modes.md](../../../docs/ui-modes/modes.md) what each mode is modelled on and
its **Fidelity** line, [agent-brief.md](../../../docs/ui-modes/agent-brief.md)
the brief for drafting a mode's pages. Don't repeat them; this skill is how to
work on them.

Work on a mode's pages and CSS is **tier 1**. Commands, the router or the
chrome lockdown (`commands.rs`, `client.rs`, `pages.rs`'s
`chrome_navigation_allowed`) make it tier 2. Follow the steward skill's
branding rule: faithful layout and ordinary wording, no product names in
anything shipped (comments and CSS `content:` text included), and all
artwork new. Docs may name the products, as long as they keep the trademark
notice.

## What a mode is made of

Under `ui/chrome/<key>/`:
- `toolbar.html` and `status.html`, plus `side.html` and `frame.html` where
  the layout uses them;
- `menus.txt`;
- `pages.css` (browser defaults: one `:where()` rule per line);
- `home.css`, the start page's look.

Pages end with `{{CHROME_JS}}` and fit their view exactly:
`body { height: 100vh; overflow: hidden }`, at the height `ui_mode::layout`
gives the view.

Rust arms for a new mode, all keyed by the `UiMode` variant:
- `settings.rs`: the variant, `ALL`, `key`, `name`, `era_look`, and
  `needs_era_look` if pages are unreadable without it;
- `ui_mode.rs`: `layout`, `page_defaults` (rules and page colour) and
  `scrollbars`;
- `pages.rs`: `mode_files!` (list `menus`, `side`, `frame` and `home` as the
  mode has them) and the description on the settings page;
- `tests/smoke.sh`: the mode's run.

The unit tests and `tests/web/ui-modes.spec.js` pick up a new mode from `ALL`
and `ui-modes.json` (written by `examples/dump-pages.rs`). Its new goldens
come from `tests/web/run.sh ui-modes.spec.js --update-snapshots`. Afterwards,
`git status` must show only new or this mode's goldens. A changed golden that
belongs to another mode is an oracle change: stop and find out why.

## Layout patterns (`ui_mode::Layout`)

`Layout::bars(toolbar, status)`, then any of:
- `.inset(edges, colour)`: a TV screen margin, or a desktop around a document
  window;
- `.well(width, colour)`: a sunken page edge, or a window outline;
- `.frame(h)`: a view over the page, used for a window title bar;
- `.side(w, left, full)`: a button column or a hotlist;
- `.panel(h)`: the status view grows into a panel;
- `.full_panel()`: the open panel takes the whole window.

A command from a full panel closes it first, so its result shows. A height
change in a page needs the same change in `layout`, or the bottom of the view
gets clipped. Grep the page's `#bar` and `height` after any restyle.

## The start page (`home.css`)

It restyles the shared `ui/pages/home.html`, using its existing classes:
`.panel`, `.caption`, `.head`, `fieldset`/`legend`, `ul.marks`, `.tips`.
`pages::home_uri` inserts it before `</head>`, after `fill`. It's
self-contained (no `url()`, `@import` or `{{`; a unit test checks this). Make
it look like the page that browser showed at home, drawn the way that browser
drew pages:
- no fieldset boxes or banners in browsers that drew plain pages;
- monospace for terminals;
- big type and tiles for TV.

The spec test fails if the page scrolls sideways at the size the mode leaves
for it. A stray `width: calc(100% + 2px)` was enough to fail it once.

Don't add a template placeholder for a mode-only value.
`templates_have_no_unknown_placeholders` lists every placeholder, and adding
one to that list changes an existing assertion, which is an oracle change.

## Icons

Frames go in `ui/icons.txt` with a per-mode prefix (`d96-back`). A mode that
needs more palette characters must use ones no other mode uses. Grep the
`= <char>` lines first, because two modes reusing a character recolours both.
Mark a colour `fill` (`= c #hex fill`) to keep it out of the embossed
disabled icon.

## Fidelity loop

1. **Find a real reference image** of the exact version. Good sources:
   - `docs/research/90s-browsers/`;
   - the product's own manual or release notes on the Wayback Machine (its
     CDX API lists captures: `web.archive.org/cdx/search/cdx?url=...`);
   - archive.org items.

   Wikimedia rate-limits (429). Keep references in the scratchpad and never
   commit them. Don't run downloaded installers to get a screenshot.
2. **Distrust recollection and host themes.** A reviewer's memory of a
   layout has been wrong. A reference captured on a later OS theme, such as
   XP's beige or a green scheme, isn't the original face colour. When no
   real capture of a version exists, say so in the Fidelity line and ask the
   maintainer. They have supplied screenshots before.
3. **Capture the real app:** `cargo build`, then
   `tools/ui-shot.sh <mode> [--url=URL] [X,Y ...]`. That writes
   `target/ui-shots/<mode>-0.png` after load, and one more per click (for
   panels and menus). Compare on a long page, such as the default Wikipedia
   article. Put it next to the reference:
   `convert ref.png -resize x450 r.png; convert target/ui-shots/<mode>-0.png -resize x450 a.png; convert r.png a.png +append side.png`.
4. **Review independently.** A model run that didn't build the mode lists
   every difference in rows, controls, order, labels, proportions, colours,
   fonts, bevels, status panes and page rendering. Each difference is marked
   fixable, branding, or missing feature. Fix the fixable ones and re-shoot.
   Repeat until only branding and missing-feature differences remain.
5. **Record the result** in the mode's Fidelity line in `modes.md`: the
   verdict, the number of rounds, what it was compared against, and what was
   left.

Capture pitfalls `ui-shot.sh` already handles; remember them if you script
your own:
- **Black captures:** Xvfb died. Restart it and check images aren't flat.
- **A stray tooltip:** Xvfb's pointer starts at the screen's centre, inside
  the window. Move it away after the window exists, because `xdotool` can't
  move it before.
- **Window not resized:** `xdotool search --name Lantern` matches helper
  windows. Wait for `" - Lantern$"`.
- **The script kills itself:** `pkill -f` matches its own command line. Kill
  by PID.

## Several modes at once

Batch 3–5 modes. Draft each mode's pages, icons and CSS with a subagent from
`agent-brief.md`, which writes files only. Keep the Rust, goldens, builds
and reviews in the main session. Give each subagent a renderer it can run
itself: a static render of the dumped page with the mode's CSS read fresh
from disk. That lets it iterate without rebuilding.
