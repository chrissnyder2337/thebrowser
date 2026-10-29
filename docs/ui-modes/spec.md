# UI modes: spec

How switchable browser UI modes work: what a mode is, the architecture, the
rules each mode follows and how they're tested. What's done and what's next
is in [README.md](README.md). Each mode's layout is in [modes.md](modes.md).

- [What a UI mode is](#what-a-ui-mode-is)
- [Naming and branding](#naming-and-branding)
- [Architecture](#architecture)
- [What every mode must offer](#what-every-mode-must-offer)
- [Testing](#testing)
- [Risks](#risks)

## What a UI mode is

Lantern's **retro looks** (`ui/retro/`) restyle the *web page*. A **UI mode**
restyles the *browser around the page*: the toolbar view, the status view
and how the window lays them out. The two are independent, so any mode
works with any look, as dial-up and the splash intro already do.

A mode decides:

- the toolbar's markup, CSS and pixel-art icons;
- the status bar's markup and CSS (it can also hold buttons, for browsers
  whose buttons sat under the page);
- the heights of the toolbar and status views;
- later, for some modes: chrome below the page, an inset border around the
  content view (the TV modes), content zoom and arrow-key link navigation.

A mode does **not** replace the native window frame: the OS still draws
the title bar. Faking System 7 or NeXT title bars would mean reimplementing
drag, resize, minimise and maximise on three platforms, so it's out of
scope.

A **family** is a style of browser UI (Desktop, Workstation, Classic Mac, ...).
A **variant** is one era of it, and a variant exists only when the interface
changed visibly: a new layout, new button style, bands added or removed.

## Naming and branding

The steward skill's branding rule (`.claude/skills/steward/SKILL.md`)
governs modes. In short:

- A mode **may follow a real browser's layout faithfully**: its rows and
  bands, its buttons and their order, where menus, fields, the activity box
  and the status bar sit, its bevels, colours and fonts, and ordinary
  wording (Back, Refresh, Favorites, Document URL:).
- A mode is **named by era and kind of machine**: "Desktop (1996)",
  key `desktop-1996`. Real names never appear in a mode's name, menu
  labels, keys, `--ui=` values or saved settings; the
  `ui_modes_avoid_product_names` test checks names and keys.
- **Descriptive prose may say what a mode is modelled on** ("modelled on
  Internet Explorer 3") in the README, docs, settings descriptions or the
  About page, always with the trademark notice from the rule.
- **All artwork is original.** No logos, icons, throbbers or screenshots of
  the originals; every activity box shows the Lantern lantern.

## Architecture

### The mode setting

`settings::UiMode` (next to `RetroStyle`) lists the modes, with `key()`,
`name()` and `from_key()`. It's saved as `ui_mode=<key>` in `settings.txt`;
an unknown key falls back to `lantern`. `--ui=<key>` picks a mode for one
run without saving it, like `--retro=`.

### Chrome pages: one controller, a folder per mode

```
ui/chrome/chrome.js             the controller every chrome page shares
ui/chrome/lantern/              Lantern's own toolbar.html and status.html
ui/chrome/<mode-key>/           each other mode's toolbar.html and status.html
```

`chrome.js` holds everything security-relevant or shared: the title channel
(`send`), `lantern.update`, the URL field's editing rules, drag-and-drop and
the context-menu guard. It binds by convention and every element is
optional (the comment at its top lists them):

- any `[data-cmd]` element sends its command on click unless it's
  `.disabled`; `data-arg` is the argument; `data-anchor` appends the
  element's bottom-left corner so a native menu can open under it;
- `#url`, `#go`, `#field`, `#title`, `#throbber`, `#back`, `#forward`,
  `#stop`, `#retro`, `#links`, and the status line's `#msg`, `#meter`
  (`data-blocks` sets its block count), `#lock-pane`, `#modem-pane`.

`pages.rs` builds each mode's pages from its folder, `chrome.js` and the
shared icon sheet. A mode's templates may use any of the placeholders the
Lantern pages use (`ICON_*`, `LANTERN_IDLE`, `LANTERN_LIT`, `CHROME_JS`).

### Menus

Menus are native (`Window::show_menu`), as the Retro menu already is,
because an HTML drop-down would be clipped to its view. A page only *names*
a menu: it sends `menu` with `<menu>,<x>,<y>` (`file`, `navigate`,
`options`, `help`). The items live in a fixed Rust table (`menus.rs`) and
map to existing commands, so the title channel never carries labels or
actions. The Retro menu and every mode's Options menu hold a **Browser UI**
submenu with one radio item per mode.

### Switching at runtime

`ui_mode::set(mode)` saves the setting and calls `ui_mode::apply()`, which:

1. marks the toolbar and status views not ready;
2. loads the mode's `data:` pages into them (each page sends `ready`, which
   brings the full state push);
3. re-lays out the window: the views' `preferred_size` and `minimum_size`
   read the current mode's heights from shell state, and the window is
   asked to lay out again.

**The chrome lockdown widens by exactly one case.** `client.rs`
`on_before_browse` used to let a chrome view load only its first `data:`
page or reload the same one. Now it may also load **exactly the page Rust
built for that view and the current mode**
(`pages::chrome_navigation_allowed`, unit-tested). Web content can't produce
that URI, and the views still never open popups. This is a tier-2 change.

### Shortcut

**Ctrl+Shift+U** (Cmd+Shift+U on macOS) opens the Browser UI menu at the top
left of the window, so no mode can strand the user.

### The settings page

The settings page doesn't list UI modes yet: adding a section changes its
existing golden screenshots, which only the maintainer may approve (steward
skill, "Oracle rules"). Until then, modes are picked from the menus, the
shortcut or `--ui=`.

## What every mode must offer

- Back, Forward, Reload, Stop, Home and a way to type an address.
- The loading indicator and the status text (in the status view or the
  toolbar).
- A way to the Browser UI menu: Lantern's Retro button, or an Options menu.
- Bookmarks, search and quick links are optional; their keyboard shortcuts
  still work.

The Playwright contract test (`tests/web/ui-modes.spec.js`) checks these for
every mode.

## Testing

- **Rust unit tests**: the setting round-trips; unknown keys fall back; mode
  names and keys avoid product names; `chrome_navigation_allowed`'s truth
  table; menu keys and command ids round-trip; every mode's templates fill
  without unknown placeholders.
- **Playwright**: `dump-pages` writes every mode's toolbar and status page
  (`toolbar-<key>.html`, `status-<key>.html`) and `ui-modes.json` (keys and
  view heights). `ui-modes.spec.js` checks every mode's contract, that its
  buttons send the right title-channel messages, that `lantern.update`
  shows the state, and adds golden screenshots for each mode. The Lantern
  mode's existing goldens must keep passing unchanged.
- **The real app** under Xvfb (steward skill recipe): switch modes at
  runtime, check the views resize and the mode survives a restart.

## Risks

- **The lockdown change** is the one security-relevant change: exact-match,
  unit-tested, reviewed as tier 2.
- **Pixel art is most of the work.** Each icon-heavy mode needs about ten
  original icons. Text-button modes come first.
- **Fonts.** Helvetica, Chicago, Charcoal and MS Sans Serif aren't on every
  machine; stacks need good fallbacks.
- **Branding drift.** Faithful layouts invite copied art. Reviews check that
  every icon and indicator is original.
- **Modes that fight modern sites.** The TV modes zoom and inset the page;
  their zoom stays a per-mode value so it can be tuned.
