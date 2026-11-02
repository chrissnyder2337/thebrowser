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
- its menu bar's menus and items (`menus.txt`);
- the window layout (`ui_mode::Layout`): view sizes, a side view, a frame
  view over the page, an inset border around the page, an expanding panel;
- how plain pages look (browser defaults, era look).

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
a menu: it sends `menu` with `<menu>,<x>,<y>`. Which menus a mode has, and
what's in them, is `ui/chrome/<mode>/menus.txt`, the original browser's menu
bar with its own names and items:

```
menu File                  a menu; its key is the label lower-cased, spaces as dashes
open-url Open Location...  an item that runs an action from menus.rs's ACTIONS table
- Mail Document...         an item the original had that Lantern can't do: shown greyed
---                        a separator
retro Retro                the Retro looks submenu
browser-ui Browser UI      the Browser UI submenu (one radio item per mode, page styles)
```

The actions are a fixed Rust table (`back forward reload stop home search
bookmarks add-bookmark open-url settings about splash print view-source
zoom-in zoom-out zoom-reset fullscreen close-window`), so the title channel
never carries labels or actions, and `menus.txt` is compiled in and parsed by
a unit test. A mode without a `menus.txt` gets the generic `file`,
`navigate`, `options` and `help` menus. Every mode's menus hold a Browser UI
submenu somewhere (the contract test checks a page can reach it).
### Window layout

The window is a column: the toolbar view, a middle row, the status view.
The middle row holds the **side view** and an **inset panel**; the inset
panel holds the **frame view** over the **content view**, inside a border.
`ui_mode::Layout` says, per mode, how big each piece is (0 hides it):

| Field | What |
|---|---|
| `toolbar`, `status` | Heights of the top and bottom views. |
| `status_open` | The status view's height while the mode's panel is open (`data-cmd="panel"` toggles it, chrome.js sets `html.panel-open`): the TV Options panel. |
| `panel_full` | The open panel takes the whole window and hides the page, like a set-top box's full-screen Setup page (Set-top TV 1997). A command sent from it, other than `panel`, `menu` and `focus-url`, closes it first so its result shows; keyboard focus moves to the panel and back to the page. |
| `side`, `side_left` | The side view's width and side: DOS graphics' button column, the Workspace hotlist. |
| `frame` | A view right over the page, inside the inset: a document window's title bar. |
| `inset`, `inset_color` | A border around frame and page (top, left, bottom, right): the TV's screen margin, a document window on a desktop. |
| `well`, `well_color` | A thin line right around frame and page, inside the inset: a sunken page edge (Workstation) or a window's black outline (Classic desktop). |

The inset and well panels have a delegate that sets their color again on
every theme change, which resets a view's background.

**Patterns built on this:**

- *Sunken page*: an inset in the face colour and a dark well (Workstation 1993).
- *Mac desktop* (Classic desktop modes): the toolbar view is the Mac menu
  bar; the inset is desktop grey; the well is the document window's black
  outline; the frame view is the window's title bar, with the browser's
  own toolbar rows under it when the original put them inside the window.

The side and frame views are chrome views like the toolbar and status bar
(`side.html`, `frame.html` in the mode's folder; same controller, same
lockdown, the toolbar's commands). Besides navigation, the toolbar, side and
frame views may send `print`, `view-source`, `zoom-in`, `zoom-out` and
`fullscreen`, for the originals' Print, Font and Fullscreen buttons. The
status view's narrower list includes `menu`, for browsers whose menu keys
sat at the bottom (the text terminal's O)ptions). Views' sizes are read from a
thread-local, not shell state, because CEF asks during layout.

### Switching at runtime

`ui_mode::set(mode)` saves the setting and calls `ui_mode::apply()`, which:

1. marks the chrome views (toolbar, status, side, frame) not ready;
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

### Pages under a mode

Each mode can also change how pages look, in two strengths. The setting is
`mode_pages` (saved in `settings.txt`, picked under **Browser UI** in the
menus):

| Key | Menu | What pages get |
|---|---|---|
| `as-is` | Pages as their sites made them | Nothing. |
| `defaults` (default) | Pages with this browser's defaults | On plain pages (no style sheets of their own), the mode's **browser defaults**: what that browser drew when a page didn't say, such as a grey page, Times and black text. |
| `era-look` | Pages in this browser's era look | The defaults, plus the mode's **era look**, a full retro look (`UiMode::era_look`) that overrides the site's styles. |

- **Browser defaults** are `ui/chrome/<mode>/pages.css` plus a page color
  in `ui_mode.rs`. `ui/chrome/page-defaults.js` adopts them as a
  constructed stylesheet, first in the list, in each content frame once it
  has loaded, and only if the page has **no style sheets of its own**:
  that's when an old browser's defaults showed, and it keeps modern sites
  exactly as they are (a per-property check can't, since a site's cascade
  layers lose to any unlayered rule). Every rule is one line with a
  `:where()` selector (checked by a unit test), so style attributes still
  win. The page color goes only on the top page, and only when its `html`
  and `body` set no background, so framed embeds stay see-through.
- **Era looks** are ordinary retro looks (`ui/retro/`). Workstation (1993)
  uses Early web (1995). A mode that needs a look that doesn't exist yet,
  such as the text terminal, adds one; a new retro look adds a choice to
  the settings page, so its golden screenshots need the maintainer's
  approval.
- **A retro look the user picks wins.** `Settings::page_look()` returns
  the user's retro look if there is one, and the mode's era look only when
  the look is Off and `mode_pages` is `era-look`. The toolbar's Retro
  button, the Retro menu and the self-test report show `page_look()`, and
  picking Off in the Retro menu (or `--retro=off`) turns an era look off
  too. A look that switches page styles off (Early web) switches the
  browser defaults off with them; it brings its own.
- Lantern's internal pages (`data:` URLs) are never touched, as with the
  retro looks.
- **The start page** takes the mode's look from its own sheet instead:
  `ui/chrome/<mode>/home.css`, added after the page's styles, restyles the
  shared `ui/pages/home.html` as a page that browser and its era would have
  shown at home (a plain grey document for the early workstations, a TV
  home screen, a text terminal's page). Only the look changes: the search
  box, bookmarks and tips stay. The sheet is self-contained (no `url()` or
  `@import`, unit-tested), and switching modes re-renders the start page if
  it's showing.
- **Modes that need their look.** A text terminal can't show a modern page,
  so for modes where `UiMode::needs_era_look()` is true the default
  `defaults` setting gives the era look too; `as-is` still turns it off.
### Scrollbars

Scrollbars belong to the browser, not the page, so each mode draws its era's
(`ui/chrome/scrollbars/<style>.css`, mapped in `ui_mode::scrollbars`):
Windows 3.1 and 95, Motif, one-bit Mac, System 7, Platinum, NeXT (both
arrows at the far end), DOS VGA, and none for the text terminals and TV
boxes. `ui/chrome/scrollbars.js` adopts the sheet into every content frame
when it starts and finishes loading and when the mode changes, last and with
`!important`, so a site's own scrollbar styles don't win; it also resets the
standard `scrollbar-color` and `scrollbar-width`, which would otherwise turn
the `::-webkit-scrollbar` rules off. Lantern keeps the system's.
`scrollbars.spec.js` has a golden per style.

### Shortcut

**Ctrl+Shift+M** (Cmd+Shift+M on macOS) opens the Browser UI menu at the top
left of the window, so no mode can strand the user.

### The settings page

The settings page has a **Browser UI** section: a radio per mode (with its
"modelled on" description and the trademark notice) and the three page
styles. Its Retro looks section lists every look, including the ones modes
added (Text terminal).

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
  names and keys avoid product names; `page_look` lets the user's look win;
  every browser-defaults rule is zero-specificity; `chrome_navigation_allowed`'s truth
  table; menu keys and command ids round-trip; every mode's templates fill
  without unknown placeholders.
- **Playwright**: `dump-pages` writes every mode's chrome pages
  (`toolbar-<key>.html`, `status-<key>.html`, `side-`, `frame-`) and
  `ui-modes.json` (keys, view sizes, menu keys). `ui-modes.spec.js` checks every mode's contract, that its
  buttons send the right title-channel messages, that `lantern.update`
  shows the state, and adds golden screenshots for each mode, its start
  page (`home-<key>.html`, which must fit without horizontal scrolling) and
  its open panel. The Lantern
  mode's existing goldens must keep passing unchanged.
- **Page defaults** (`mode-pages.spec.js`): on a plain fixture page the
  mode's defaults apply (with a golden); on a fixture that styles itself,
  nothing changes; switching them off removes the sheet.
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
