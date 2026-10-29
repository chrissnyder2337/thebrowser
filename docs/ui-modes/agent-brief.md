# Brief: build one Lantern UI mode's chrome pages (files only)

For subagents drafting a mode (docs/ui-modes/goal.md). `<scratchpad>` is the
session's scratchpad directory. The main session wires the mode into Rust,
runs the tests and commits; subagents only write the files below.

Lantern (repo at /home/user/thebrowser) is a CEF browser whose toolbar (above the page)
and status bar (below it) are small locked-down HTML pages. A **UI mode** swaps them for
another 1990s browser's layout. Read these first:

- docs/ui-modes/spec.md (how modes work) and docs/ui-modes/modes.md (your mode's section)
- ui/chrome/chrome.js (the shared controller; its top comment lists the conventions)
- ui/chrome/workstation-1993/{toolbar.html,status.html,pages.css}: a finished mode to copy
  the structure from
- ui/chrome/lantern/toolbar.html: Lantern's own toolbar (bevel CSS, throbber CSS)
- The mockup for your mode, if the session has one (layout reference only, not a
  template to paste), and the reference image the session gives you.
- Research screenshots, if your mode has one: docs/research/90s-browsers/images/
- The branding rule in .claude/skills/steward/SKILL.md ("Branding rule"): follow the real
  browser's layout, button set/order and ordinary wording faithfully, but NO product names
  anywhere in the files (not in labels, titles, comments or class names), no logos, no
  copies of its icons. Every icon is new pixel art; the activity box always shows the
  Lantern lantern.

## What you write

1. `ui/chrome/<key>/toolbar.html` and `ui/chrome/<key>/status.html`: complete, self-contained
   pages (inline CSS, no external resources, no JS of your own) that end with
   `<script>\n{{CHROME_JS}}\n</script>`, exactly like workstation-1993. Placeholders you may use
   anywhere: `{{CHROME_JS}}`, `{{LANTERN_IDLE}}` (an `<svg class="idle">` 32x48),
   `{{LANTERN_LIT}}` (an `<svg class="lit">` 128x48 strip of 4 frames; animate it inside a
   32x48 `overflow:hidden` box with `steps(4)` like Lantern's toolbar), `{{ICON_LOCK}}`,
   `{{ICON_MODEM}}` (two svgs, classes `on`/`off`, see workstation status CSS),
   `{{ICON_PAGE}}`, and for each icon frame you draw (see 3) `{{ICON_<NAME>}}`, which becomes
   two svgs: class `on` (normal) and class `off` (an embossed grey version for disabled
   buttons). Show `svg.on` normally and `svg.off` when the button has class `disabled`.
   `<NAME>` is the frame name upper-cased with `-` turned into `_`
   (frame `d96-back` -> `{{ICON_D96_BACK}}`). Nothing else in `{{...}}` form may appear.
2. `ui/chrome/<key>/pages.css`: the mode's browser defaults for plain pages. Exactly one
   comment block at the top, then one rule per line, every selector a single `:where(...)`
   with nothing after it (a unit test checks this). Usually just
   `:where(html) { color: #000; font-family: ...; }`. The page COLOR is not set here
   (state it in your notes; Rust adds it).
3. Icons: a file `<scratchpad>/icons-<key>.txt`
   in ui/icons.txt's format (read that file: palette lines `= <char> #rrggbb`, then
   `frame <name>` followed by rows of palette chars, `.` = transparent). Name every frame
   `<prefix>-<what>` with your prefix. You may use the existing palette characters
   (K N T t R r m S W M O o D g Y y) and, if you need more colours, ONLY your reserved
   characters; declare those at the top of your file. All frames of one size per role
   (e.g. all toolbar buttons 20x20). Draw clear, original 1990s-style icons.
4. `ui/chrome/<key>/menus.txt` (see the contract).
5. `<scratchpad>/notes-<key>.md`:
   the layout (toolbar and status view heights in px, and any side/frame/inset/panel) (the pages must fit exactly: body is
   `height: 100vh; overflow: hidden`), the page colour for pages.css, the era look
   (one of: off, early-web, portal, terminal) and anything you were unsure about.
   List every difference from the reference image you couldn't remove, and why.

## Layout beyond two bars

Most modes are a toolbar over the page and a status bar under it. If the original needs
more, say so in your notes as a layout (`ui_mode::Layout`, spec.md "Window layout"):
`status_open` (a panel the status view grows into, toggled by `data-cmd="panel"`; style it
with `html.panel-open`), `side` + `side_left` (a side view beside the page: write
`side.html`, same rules as the toolbar), `frame` (a view right above the page inside the
inset: write `frame.html`), `inset` [top,left,bottom,right] + `inset_color` (a border
around frame and page), `well` + `well_color` (a thin line right around the page inside the
inset: a sunken page edge). Chrome buttons may live in any of the views; the contract counts
them across all of them.

## The contract (tests check it)

- Across the two pages: `[data-cmd]` elements for `back`, `forward`, `reload`, `stop`,
  `home`; `#back`, `#forward`, `#stop` start with class `disabled`.
- The toolbar has the address field `<input id="url">` inside an element `id="field"`
  (gets class `secure` on https; show `{{ICON_LOCK}}` then) and may have `#go`.
- Exactly one `#msg` (status text, inside an element whose `title` gets set) and exactly
  one `#meter` (progress; chrome.js fills it with `<i>` blocks, `data-blocks="N"` = how
  many fill it; hidden until class `on`) across both pages; a `#lock-pane` and a
  `#modem-pane` (`data-cmd="settings"`, with `#modem-label`) in the status bar.
- **Menus are the original's.** Write `ui/chrome/<key>/menus.txt` with the original
  browser's menu bar: every menu in its order with its own name, and its items with their
  original labels (spec.md "Menus" has the format). An item Lantern can do names one of the
  actions (`back forward reload stop home search bookmarks add-bookmark open-url settings
  about splash print view-source zoom-in zoom-out zoom-reset fullscreen close-window`); an
  item it can't is `- <label>` (shown greyed). Put `browser-ui Browser UI` (and, if you like,
  `retro Retro looks`) at the end of the menu where the original kept its preferences
  (Options, View, Configure...). Aim for the real menus' items, not an invented list.
- Menu-bar items are `<button class="menu" data-cmd="menu" data-arg="<key>" data-anchor
  aria-haspopup="menu">`, where `<key>` is the menu's name lower-cased with spaces as
  dashes (`Go` -> `go`, `Window` -> `window`). Every menu in menus.txt gets its button, in
  order, and a button may only name a menu in menus.txt. A browser with no menu bar of its
  own (the TV modes) still needs one way to the Browser UI menu: a button sending
  `data-cmd="menu"` for a menus.txt menu holding `browser-ui`.
- Other commands you may use on buttons: `search`, `bookmarks`, `bookmark-add`,
  `focus-url`, `settings`, `panel` (the status bar may send only: back forward reload stop
  home search bookmarks bookmark-add focus-url settings panel go).
- `#throbber` (activity box) holds the lantern; clicking it opens About (chrome.js does it);
  class `loading` while a page loads: show the lit strip then.
- Optional: `#title` (page title text), `#links` with `<template id="page-icon">{{ICON_PAGE}}</template>`
  for quick links (buttons class `ql`, see Lantern's CSS), `#retro`.
- Buttons are real `<button>`s with `title` tooltips naming the keyboard shortcut where one
  exists (Alt+Left, Alt+Right, F5, Esc, Alt+Home, Ctrl+L, Ctrl+D).
- 800px wide must fit (it's the test viewport); wider windows stretch the address field.

## Checking your work (don't build Rust; don't commit; don't touch any other files)

Preview with the preinstalled Playwright (`require('playwright')`, chromium): write a small
script in the scratchpad that reads your HTML, replaces `{{CHROME_JS}}` with the contents of
ui/chrome/chrome.js and every other `{{...}}` with a placeholder grey `<svg>` box of the
right size, opens it at 800 x <your height>, calls
`lantern.update({url:"https://example.com/",title:"Example",canBack:true,canFwd:false,loading:true,secure:true,text:"Loading example.com...",progress:0.5,modem:"28.8K"})`,
and screenshots it to the scratchpad. Check there are no console errors, nothing overflows,
and it matches the mockup's layout. Report the screenshot paths.

Compare your screenshot with the reference image side by side (same scale) and fix
differences in rows, control order, labels, proportions, colours, fonts and bevels before
reporting.

Finish with a short report: files written, heights, page colour, era look, doubts.
