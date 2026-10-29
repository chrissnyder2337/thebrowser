# Plan: switchable browser UI modes

A plan for a **UI mode** switch: the toolbar and status bar change to the
look and layout of other 1990s browser families (the Windows shell browser,
the set-top TV box, the X workstation browser, the early Mac browser, the
Java browser and more), with a separate variant for each version whose
interface changed significantly.

Status: proposal, not started. Nothing here changes the app yet.

- [What a UI mode is](#what-a-ui-mode-is)
- [The branding rule](#the-branding-rule)
- [Families and variants](#families-and-variants)
- [Architecture](#architecture)
- [Testing](#testing)
- [Phases](#phases)
- [Risks](#risks)
- [Questions for the maintainer](#questions-for-the-maintainer)

## What a UI mode is

Lantern already has **retro looks** (`ui/retro/`), which restyle the *web
page*. A UI mode is the other half: it restyles the *browser around the
page*, meaning the toolbar view, the status view and how the window lays
them out. The two are independent. Any mode works with any look, the way
dial-up and the splash intro already combine with the looks.

A mode decides:

- the toolbar's markup, CSS and pixel-art icon set;
- the status bar's markup and CSS, or no status bar at all;
- view heights, whether the chrome sits above or below the page, and margins
  around the content view (the TV mode's overscan border);
- for the TV mode only, content zoom and arrow-key link navigation.

It does **not** replace the native window frame. The OS still draws the
title bar. A skinned frameless window (fake System 7 or NeXT title bars)
means reimplementing drag, resize, minimise and maximise on three platforms,
which is out of scope (see [Backlog](#backlog)).

## The branding rule

The steward skill's branding rule says Lantern never names anything after a
real browser, in labels, option keys, `--flags`, saved settings, file names
or screenshots, and never copies logos, throbbers or "layout signatures".
The request names Internet Explorer, WebTV, Mosaic, MacWWW and HotJava, so
the plan works like the retro looks do:

- **User-facing names describe the era and the kind of machine**, e.g.
  "Desktop (1996)", "Set-top TV (1996)", "Workstation (1993)". Keys look like
  `desktop-1996`.
- **Real names appear only under `docs/research/`.** That's the existing
  exception, and it's where the mapping from each mode to the browsers it
  draws on lives.
- **Each mode is an original design of its era's conventions**: bevel style,
  button size, band layout, fonts, palette, where things sit. It doesn't
  trace a screenshot. No logos, no lookalike throbbers (each mode gets an
  original lantern variant), and no product wording such as "Favorites",
  "Channels" or "Options" strings copied from a specific product.
- A unit test fails if a mode key or name contains a deny-listed product
  name (see [Testing](#testing)).

If the maintainer wants real names, that's a change to the branding rule
itself (the steward skill), not something this plan should work around. See
[Questions](#questions-for-the-maintainer).

## Families and variants

A **family** is a style of browser UI. A **variant** is one era of it, and a
variant exists only when the interface changed significantly: a new layout,
a new button style, bands added or removed. A version that only added
features doesn't get one. "Research" marks a variant that
`docs/research/90s-browsers/catalog.json` doesn't document well enough yet
(Phase 0 fills it in).

| Family (user-facing) | Draws on (research only) | Variants | What changes between them |
|---|---|---|---|
| **Lantern** (today's UI) | Mid-90s Windows browsers in general | `lantern` | The default, unchanged. |
| **Desktop** | Internet Explorer 1–5 (Windows) | `desktop-1995`, `desktop-1996`, `desktop-1997`, `desktop-1999` (research) | 1995: one small toolbar of 16 px icon buttons, no labels. 1996: a rebar of draggable bands (large labelled buttons, an Address band, a Links band) with grippers. 1997: flat buttons that raise on hover and stay grey until hovered, plus a side panel. 1999: only if research finds a real layout change; otherwise it merges into 1997. |
| **Desktop (Mac)** | Internet Explorer 2–4.5 (Mac) | `desktop-mac-1996`, `desktop-mac-1998` (research) | Mac-style buttons and fonts, a different band layout from the Windows builds. Only if research shows they differ enough from the Mac family below. |
| **Set-top TV** | WebTV Classic (1996), WebTV Plus (1997) | `tv-1996`, `tv-1997` (research) | No toolbar. A slim options strip at the bottom, very large type, content zoomed and inset from the screen edge, and a highlight box moved between links with the arrow keys. 1997: only if the Plus interface differed visibly. |
| **Workstation** | NCSA Mosaic for X 1.x–2.x, tkWWW, Arena | `workstation-1993`, `workstation-1994` (research) | Motif bevels, a row of text-only buttons, separate Title and URL fields, a corner activity indicator, Helvetica. Research decides what 2.x changed. |
| **Classic Mac** | MacWWW (Samba), Mac Mosaic | `mac-1992`, `mac-1994` (research) | 1992: one-bit black and white, Chicago-style type, almost no toolbar: navigation lives in menus. 1994: grey bevelled buttons in a row. |
| **Applet browser** | HotJava alpha (1995), HotJava 1.0/3.0 (1997–) | `applet-1995` (research), `applet-1997` | 1997: a conventional button row and fields drawn in the Java toolkit's look (`hotjava-3.png`). 1995: research, since the alpha looked very different. |

Candidates for later, from the catalog (see [Backlog](#backlog)):

- **Text terminal** (Lynx, Links, w3m): the chrome becomes a status line and
  key hints. Pairs with the Early web look.
- **Cube workstation** (WorldWideWeb, OmniWeb): needs a floating vertical
  menu.
- **Workspace** (Opera's MDI windows and hotlist side panel).
- **DOS graphics** (Arachne).

The current Lantern UI stays the default. Each variant is one PR, so the
family can grow one version at a time.

## Architecture

### Mode registry (Rust)

A new `src/shell/ui_mode.rs`, built like `RetroStyle` in `settings.rs`:

```rust
pub enum UiMode { Lantern, Desktop1995, Desktop1996, /* … */ }

impl UiMode {
    pub const ALL: [Self; N];
    pub fn key(self) -> &'static str;      // "desktop-1996"
    pub fn name(self) -> &'static str;     // "Desktop (1996)"
    pub fn family(self) -> Family;         // for the submenu
    pub fn layout(self) -> Layout;
}

pub struct Layout {
    pub toolbar_height: i32,     // 0 = no toolbar view
    pub status_height: i32,      // 0 = no status view
    pub chrome_below: bool,      // TV: options strip under the page
    pub content_inset: i32,      // TV: overscan border, in DIP
    pub content_zoom: f64,       // TV: CEF zoom level; 0.0 elsewhere
}
```

`Settings` gains `ui_mode` (`ui_mode=<key>` in `settings.txt`, `ui=<key>` in
`apply_query`), defaulting to `Lantern`. An unknown key falls back to the
default, as `retro_style` does. A `--ui=<key>` switch, parsed next to
`--retro=` in `app.rs`, picks a mode for one run without saving it. The
screenshot scripts and `--self-test` use it.

### Chrome pages: one controller, many skins

Today `ui/toolbar.html` and `ui/status.html` mix markup, CSS and the
controller script. The controller is the title channel, `lantern.update`,
the URL field's editing rules, drag-and-drop and the context-menu guard.
Splitting them keeps the security-relevant script in one place:

```
ui/chrome/toolbar.js            controller: send(), lantern.update, URL field, guards
ui/chrome/status.js             status controller
ui/chrome/lantern/toolbar.html  today's markup + CSS, with {{TOOLBAR_JS}}
ui/chrome/lantern/status.html
ui/chrome/desktop-1996/…        one folder per variant
```

The controller binds by convention and tolerates missing pieces:

- every `[data-cmd]` element sends its command on click, with an `x,y`
  anchor when it has `data-anchor` (menus);
- `#url`, `#go`, `#throbber`, `#links`, `#field`, `#back`, `#forward`,
  `#stop`, `#retro` and `#ui` are optional;
- `lantern.update` keeps today's JSON shape, and a skin can register a hook
  (`lantern.onUpdate(fn)`) for its own rendering, e.g. the TV strip's clock.

`pages.rs` builds `toolbar_uri(mode)` and `status_uri(mode)` from the
variant's folder, the controller and that variant's icon set. They're cached
per mode, since each is a large `data:` string. Icons stay original pixel art
in `ui/icons.txt`, with frames namespaced by variant (`frame desktop-1996.back`).
A variant can reuse another variant's frames.

### Switching at runtime

`ui_mode::set(mode)` (UI thread, like `retro::set_style`):

1. Saves settings and marks `toolbar_ready` and `status_ready` false.
2. Loads the new `data:` URI into the toolbar and status views. Each page
   sends `ready` as it does today, which triggers the full state push.
3. Re-lays out the window. The `BrowserViewDelegate` `preferred_size` and
   `minimum_size` read the current mode's `Layout` from shell state instead
   of the `TOOLBAR_HEIGHT` and `STATUS_HEIGHT` constants. Then it calls
   `set_visible` on a view whose height is 0, `reorder_child_view` when
   `chrome_below` changes, `inside_border_insets` on the box layout for
   `content_inset`, and `Window::layout()`. The window's minimum size
   follows the mode.
4. Applies or clears the content zoom (`BrowserHost::set_zoom_level`) and
   remembers the user's own zoom so leaving TV mode restores it.

Phase 2 starts with a short spike to confirm these calls in the `cef` crate
we pin (relayout after a `preferred_size` change, `reorder_child_view`,
`set_visible` on a `BrowserView`).

**The lockdown has to widen, carefully.** `client.rs` `on_before_browse`
lets a chrome view load only its first `data:` page or reload the same one,
so loading another mode's page is refused today. The change:

> A chrome view may load a `data:text/html` URL only if it is **exactly** the
> URI Rust built for that role and the current mode.

The check becomes a pure function,
`chrome_nav_allowed(role, url, current, expected)`, with unit tests. Web
content still can't steer a chrome view: it can't produce the expected URI,
and the views still never open popups. This is an invariant change in a
tier-2 file, so it gets its own review against
`.claude/claude-security-guidance.md`.

*Alternative considered:* destroy and recreate both `BrowserView`s on a
switch, which leaves the lockdown untouched. It costs more: new browser
handles in shell state, a visible flash, and close-handshake edge cases in
`can_close`. Keep it as the fallback if the reviewer rejects the narrower
rule.

### The switch itself

The switch has to be reachable from **every** mode, including the TV mode,
which has no toolbar buttons:

- **A "Browser UI" submenu in the Retro menu**: one submenu per family,
  with radio items for its variants (`MenuModel::add_sub_menu`).
- **A `ui-menu` title command** that opens the same native menu at an
  anchor. Each skin puts it somewhere that suits its era: a toolbar button,
  a menu-bar entry, or an item in the TV options strip. A contract test
  checks that every skin has a `[data-cmd="ui-menu"]` control.
- **A keyboard shortcut** (proposed: Ctrl+Shift+U) in `on_pre_key_event`,
  next to the existing ones, so a mode can never strand the user.
- **The settings page** (`ui/pages/settings.html`) gets a "Browser UI"
  select. It posts `ui=<key>` through the existing `lantern.invalid`
  settings action, so no new route is needed.

New title commands (`ui-menu` and, in Phase 6, `menu`) go through the
`chrome-command` skill, with the router change in `commands.rs` (tier 2).

### Menu bars (Phase 6)

The Workstation, Classic Mac and Desktop 1995 modes look wrong without a
menu bar (File, Edit, View, Go). An HTML drop-down would be clipped to the
toolbar view, the same reason the Retro menu is native, so:

- the skin draws the menu titles and sends `menu|<menu-id>,x,y`;
- Rust shows a native menu built from a **fixed Rust-side table** keyed by
  `menu-id`, whose items map to existing commands.

The page names a menu but never supplies items or labels, so the router
gains one enum-like argument and nothing more.

### What every mode must offer

At minimum: back, forward, reload or stop, home, a way to type a URL, the
loading indicator and the UI switch. Bookmarks, search and quick links are
optional per mode, since the keyboard shortcuts still work. Status text and
progress are optional, but a mode without a status bar has to show progress
somewhere (the TV strip shows a thin progress line).

### TV mode extras (Phase 5)

- Content zoom (about 1.4×) and an inset border painted in the window's
  background colour.
- Arrow-key link navigation with a drawn highlight box. This is a content
  script built like the retro runtime (closed shadow root, removed cleanly
  when you switch away). It follows the `retro-look` skill's rules and uses
  an original highlight colour and shape.
- The options strip at the bottom: Back, Home, Recent, Go to, Search,
  Options (opens the UI menu), a clock, and the modem indicator. Labels are
  generic.

## Testing

- **Rust unit tests**: settings parse, serialize and query for `ui_mode`;
  unique keys; every visible view has a non-zero height; branding deny-list
  over keys and names (`netscape`, `navigator`, `explorer`, `mosaic`,
  `opera`, `webtv`, `hotjava`, `macwww`, …); `chrome_nav_allowed`
  truth table; `toolbar_uri(mode)` is stable (same input, same URI).
- **Playwright** (`tests/web/chrome.spec.js` pattern): `dump-pages` writes
  every mode's toolbar and status page. A **contract test** loops over all
  modes and checks that the required controls exist, clicks send the right
  title commands, and a fixture state through `lantern.update` shows the URL,
  disabled buttons and the loading indicator.
- **Goldens**: new golden files per mode only. The `lantern` mode's existing
  goldens must still pass byte for byte after the Phase 1 refactor; that's
  the refactor's proof (oracle rule: existing goldens aren't modified).
- **The real app** under Xvfb (steward recipe): switch through every mode at
  runtime, check the relayout, the status view hiding, the chrome moving to
  the bottom, zoom restore, and that the mode survives a restart. Add
  screenshots to `docs/` as the retro looks did.
- **Black-box scenarios** (`docs/blackbox-scenarios.md`, tier 1): "switch to
  the TV mode and back using only the keyboard".

## Phases

Each phase is one or more `factory:ready` issues for the
[factory-issue](../.claude/skills/factory-issue/SKILL.md) loop. Tiers come
from `.github/tiers.yml`.

| Phase | Work | Tier |
|---|---|---|
| **0. Research** | Fill the catalog gaps marked "research" above: Desktop 1999, the Mac builds, TV 1996 vs 1997, Workstation 1.x vs 2.x, Classic Mac 1992 vs 1994, Applet 1995. For each variant, write a one-page spec under `docs/research/90s-browsers/ui-modes/`: layout sketch, heights, controls, palette, fonts, and a "won't copy" list. Confirm the variant list. | 0 |
| **1. Refactor** | Split the controller out of `toolbar.html` and `status.html`, and move today's UI to `ui/chrome/lantern/`. No visible change: existing goldens and chrome tests pass unchanged. | 1 |
| **2. Plumbing + first mode** | `ui_mode.rs`, settings, `--ui=`, runtime switch and relayout, the lockdown rule, the Browser UI submenu, `ui-menu`, the shortcut and the settings-page select, plus **Workstation (1993)** as the first new mode (text buttons, so the least pixel art) to prove switching end to end. | 2 |
| **3. One variant per family** | Desktop (1996), Classic Mac (1992), Applet browser (1997). One PR each: skin folder, icons, `UiMode` variant, goldens. | 1 (2 if a new command is needed) |
| **4. Other variants** | Desktop (1995) and (1997), Workstation (1994), Classic Mac (1994), the rest that Phase 0 confirmed. | 1 |
| **5. Set-top TV** | Layout extras (bottom chrome, inset, zoom), the link-navigation script, `tv-1996` and, if confirmed, `tv-1997`. | 2 |
| **6. Menu bars** | The `menu` command and the Rust menu table, then menu bars in the modes that need them. | 2 |

Phases 3 and 4 can run in parallel once Phase 2 lands, since each variant
touches only its own folder, its icon frames and one enum arm.

### Backlog

- Text terminal, Cube workstation, Workspace and DOS graphics families.
- A side panel for Desktop (1997), which needs a fourth view
  (`Role::Sidebar`) in a nested horizontal box layout.
- Frameless, fully skinned windows (fake title bars and window controls).
- "Match look": switching to a mode also offers its era's retro look.

## Risks

- **Lockdown widening.** It's the one security-relevant change, so it stays
  small and exact-match, with its own tests and review.
- **Pixel art is most of the work.** Each variant needs about ten original
  icons and a throbber. Text-button modes (Workstation, Classic Mac 1992)
  come first because they need the fewest.
- **Fonts.** Chicago, Helvetica, Charcoal and the Java toolkit's fonts aren't
  on every machine. Use CSS font stacks with good fallbacks and test on all
  three CI platforms' screenshots, not just Linux.
- **Branding drift.** Era-faithful designs slide toward copying. The deny-list
  test catches names; layout copying needs review, guided by each Phase 0
  spec's "won't copy" list.
- **Relayout in CEF Views.** Unverified in our `cef` crate version; the
  Phase 2 spike settles it before any skin work depends on it.
- **The TV mode on modern sites.** Zoomed, inset content on today's wide
  layouts can feel cramped. The zoom level stays a per-mode setting so it can
  be tuned.

## Questions for the maintainer

1. **Names.** The plan uses era names ("Desktop (1996)") because the branding
   rule forbids product names in the app. Keep that, or change the rule?
2. **Where the switch lives.** A submenu in the Retro menu, as proposed, or
   its own toolbar button in every mode?
3. **First families.** Proposed order: Workstation, Desktop, Classic Mac,
   Applet, then TV (it needs the most new machinery). Any change?
4. **Pairing.** Should picking a mode suggest its era's retro look, or stay
   fully independent?
