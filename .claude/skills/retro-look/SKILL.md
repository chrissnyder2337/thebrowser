---
name: retro-look
description: Change or add a Lantern retro look (ui/retro/*.js, run by ui/retro/runtime.js) — the look's anatomy, how the runtime handles shadow trees, floating boxes and rescans, the Rust wiring in retro.rs and settings.rs, fixture pages, the Playwright behaviour tests and new golden screenshots. Use when asked to tweak, fix or add a retro look, the splash intro, or a fixture page for them.
---

# Retro looks

A retro look restyles every page in the content view. Changes to a look are
**tier 1**; anything that also touches `client.rs`, `commands.rs` or
`util.rs` is tier 2 (steward skill → "Risk tiers"). Follow the branding rule:
describe the era, never name a real product.

## Anatomy

`src/shell/retro.rs` (`style_script`) wraps `ui/retro/runtime.js` and every
look into one script and runs it in each content frame when a load starts,
when it ends, and in all frames when the look changes. Applying the active
look again only rescans; `apply("off")` takes everything back out.

A look (`ui/retro/<key>.js`) is a factory `function (R) { return {...}; }`.
The fields are documented at the top of `runtime.js`:

- `css(p, all)`: rules, called once for the document (`p = "html body "`)
  and once for open shadow trees (`p = ""`). Use `!important`: page styles
  are often more specific.
- `pageCss`: rules for the document only (`html`, `body`).
- `solid`: background for floating boxes that had a solid one, or `null`.
- `unpinHeaders`: put sticky or fixed bars at the top back into the flow.
- `noPageStyles`: switch the page's own stylesheets off.
- `decorate()` / `undecorate()`: banners and footers on the top-level page
  only. `decorate()` runs on every scan, so it must check whether its
  decoration is still there. Build decorations with `R.overlay()` (a closed
  shadow root the page can't restyle), and call `R.keepLast(host)` for
  footers so content the parser adds later stays above them.

What the runtime does for you:

- **Constructed stylesheets** (`adoptedStyleSheets`), which page CSPs don't
  block, adopted into the document and into each open shadow root it finds.
- **Floating boxes**: on each scan it reads the page's layout with the look's
  sheets switched off and marks fixed, sticky and positive-z-index absolute
  elements with an opaque background (`data-lantern-retro-solid`), which
  the `solid` rule keeps solid.
- **Rescans**: a `MutationObserver` on the document and each shadow root
  schedules a scan (every 100 ms for the first 5 s, then at most once a
  second), so late nodes and new shadow trees get styled too.

A new look also needs: a `RetroStyle` variant with its key, name and
description in `src/shell/settings.rs`, an entry in `LOOKS` in `retro.rs`,
its menu item, and its description in `pages.rs` (`retro_description`).

## Tests

1. **Syntax**: `node --check ui/retro/runtime.js`; for a look fragment,
   `node -e 'new Function("return " + require("fs").readFileSync("ui/retro/<key>.js", "utf8"))'`.
2. **Playwright** (`tests/web/retro.spec.js`): `LOOKS` in
   `tests/web/helpers.js` lists the looks; add a new key there. The tests
   load each look through `dump-pages`' `retro-<key>.js`, the exact script
   Rust builds, so rebuild with `tests/web/run.sh` (it reruns dump-pages).
   Behaviour tests check: no page errors, footers at the end, button
   contrast ≥ 3, floating boxes solid (or in the flow for no-style looks),
   rescans restyling late nodes and shadow roots, and `off` removing
   everything.
3. **Fixtures**: new static pages go in `tests/fixtures/`, with a
   `<footer id="page-footer">` and `data-test-float` on boxes that should
   stay solid. Add the name to `FIXTURE_PAGES` in `helpers.js`.
4. **Goldens**: `tests/web/run.sh` writes a missing golden and fails that
   test once; run it again and it passes. Commit the new PNGs under
   `tests/golden/`. **Don't touch existing goldens**: if a change alters an
   existing screenshot, that's the oracle saying the look changed. Say so in
   the PR and leave the update to the maintainer (steward skill → "Oracle
   rules"); CI's `Oracle guard` blocks it otherwise. Animations are frozen at
   their start, so blink and marquee states are stable.
5. **Smoke**: `tests/smoke.sh` runs the bundle once per look (`--retro=`); a
   new look goes in its loop.
6. **Look at it**: run the app under Xvfb on a long real page with
   `--retro=<key>` (steward skill recipe) and take a screenshot.
