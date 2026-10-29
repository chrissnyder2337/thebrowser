# How Lantern works

The views, how the chrome pages talk to Rust, the retro looks engine and the
source layout. To build and test Lantern, see [development.md](development.md).

## The window and its views

One top-level CEF Views window holds three `BrowserView`s in a vertical box
layout, all using the Alloy runtime style:

```
┌──────────────────────────────────────────────┐
│ toolbar  (112 DIP, data: page from ui/)      │
├──────────────────────────────────────────────┤
│ content  (flex 1, the web page)              │
├──────────────────────────────────────────────┤
│ status   (24 DIP, data: page from ui/)       │
└──────────────────────────────────────────────┘
```

The toolbar and status bar are ordinary HTML pages, which is the same idea as
building browser UI with web tech on top of Chromium. They talk to Rust like this:

- **Chrome page → Rust.** The page sets its own `document.title` to
  `lantern|<seq>|<command>|<encodeURIComponent(arg)>`. The toolbar's display
  handler sees it in `on_title_change`, decodes it and acts on the content
  browser. Commands: `ready`, `back`, `forward`, `reload`, `stop`, `home`,
  `search`, `go`, `open`, `bookmark-add`, `bookmarks`, `about`, `retro-menu`,
  `menu` (`<menu>,x,y`: a mode's File, Navigate, Options or Help menu),
  `focus-url` and `settings`. Both chrome views may send any of them, since
  some UI modes put buttons under the page.
- **Rust → chrome page.** `execute_java_script("lantern.update({...})")` pushes
  state (URL, nav buttons, loading, quick links; status text, progress, padlock).
- **Internal page → Rust.** The bookmarks, about and error pages link to
  `https://lantern.invalid/<action>?…`. The content view's `on_before_browse`
  handles these and cancels the navigation. `.invalid` is a reserved TLD, so
  these links can't collide with a real site.
- **Lockdown.** The toolbar and status views cancel every navigation except
  loading their own `data:` page, and they never open popups.

Each view has its own `Client`, which records the view's role (`Toolbar`,
`Content` or `Status`). Shell state lives in a thread-local `RefCell`, since
every callback runs on the CEF UI thread and CEF objects aren't `Send`.

Alloy style provides no browser UI of its own, so Lantern supplies the pieces
it needs: an error page (`on_load_error`, skipping `ERR_ABORTED`), a download
handler that shows the native save dialog, and popups redirected into the
content view.

## Retro looks

`src/shell/retro.rs` runs `ui/retro/runtime.js`, together with
the look definitions in `ui/retro/`, in every frame of the content view when a
load starts and when it ends, and in all frames right away when the look
changes. The styles are constructed stylesheets (`adoptedStyleSheets`), which
page Content Security Policies don't block. They also go into the page's open
shadow trees (custom elements). Floating boxes like menus and cookie banners
that had a solid background keep a solid one, so they stay readable.
Decorations live in closed shadow roots so the page can't restyle them.
Everything is rechecked as the page changes, and switching back to Off takes
it all out again without a reload. The splash intro is `ui/retro/splash.js`;
it runs as the page starts loading, so it covers the load the way the
originals did. The Retro menu is a native CEF menu (`Window::show_menu`)
anchored under the button, because an HTML menu would be clipped to the
toolbar view (`menus.rs`).

Research for new looks: [`docs/research/90s-browsers/`](research/90s-browsers/)
catalogs the browsers of the 1990s, version by version, with screenshots.

## UI modes

A UI mode swaps the toolbar and status bar for another 1990s browser's
layout. Every chrome page shares one controller, `ui/chrome/chrome.js`
(title channel, `lantern.update`), and each mode supplies only markup and
CSS in `ui/chrome/<mode>/`. The spec, the per-mode layouts and the status
of the work are in [`docs/ui-modes/`](ui-modes/).

## Source layout

```
src/main.rs, lib.rs, win.rs     entry points (from cefsimple; lib.rs/win.rs = Windows sandbox)
src/bin/lantern_helper.rs       macOS helper subprocess
src/mac/mod.rs                  NSApplication subclass + app delegate (from cefsimple)
src/shell/mod.rs                load_cef, run_main, CEF settings
src/shell/app.rs                App + BrowserProcessHandler, creates the window and views
src/shell/window.rs             window and BrowserView delegates (sizes, Alloy style, close)
src/shell/client.rs             per-role Client: display, load, life span, request, keyboard, download
src/shell/state.rs              Shell state, push helpers, UI-thread tasks
src/shell/commands.rs           title-channel commands, lantern.invalid actions, navigation
src/shell/pages.rs              data: URIs for chrome and internal pages
src/shell/art.rs                pixel-art text → inline SVG
src/shell/bookmarks.rs          bookmark TSV + per-OS data directory
src/shell/settings.rs           user settings (dial-up mode, modem speed profiles, retro look)
src/shell/dialup.rs             dial-up simulator: network emulation and the image pacer
src/shell/retro.rs              retro looks: running ui/retro/*.js, the look and splash settings
src/shell/menus.rs              native menus: Retro, Browser UI and the UI modes' menu bars
src/shell/ui_mode.rs            UI modes: view heights, switching and relayout
src/shell/selftest.rs           --self-test=<file>: report the first load and quit
src/shell/util.rs               percent/HTML/JS escaping, URL-bar normalization, tiny JSON
ui/chrome/chrome.js             controller shared by every chrome page (title channel, lantern.update)
ui/chrome/<mode>/               each UI mode's toolbar.html and status.html (markup and CSS), and its pages.css
ui/chrome/page-defaults.js      puts a UI mode's browser defaults (pages.css) under content pages
ui/retro/runtime.js             retro looks engine (sheets, shadow trees, floating boxes, rescans)
ui/retro/{geocities,early-web,portal,terminal}.js the looks
ui/retro/splash.js              Flash-style splash intro
ui/pages/*.html, common.css     internal page templates ({{PLACEHOLDER}} substitution)
ui/dialup-images.js             "slow images" pacer injected into content pages
ui/lantern.txt, icons.txt       pixel art: the lantern (5 frames) and the toolbar icons
ui/appicon/                     generated window icons
resources/mac, resources/win    bundle resources and generated .icns/.ico
tools/make_icons.py             regenerates all app icons from ui/lantern.txt
tools/bump-version.sh           bumps the version and opens the release pull request
tools/oracle-guard.sh           CI check: product changes may not rewrite existing goldens or assertions
tools/blackbox/                 black-box tester: the bb driver, judge prompt and broken-build patches (docs/blackbox-scenarios.md)
examples/dump-pages.rs          writes the chrome and internal pages to files for Playwright
tests/web/, tests/fixtures/     Playwright tests and the fixture pages they style
tests/golden/                   golden screenshots (new files only; see the steward skill)
tests/smoke.sh                  bundle smoke test under xvfb-run
.claude/skills/release/         Claude Code skill for cutting and debugging releases
.claude/skills/steward/         Claude Code skill: conventions for getting PRs merged
.claude/skills/retro-look/      Claude Code skill: changing or adding a retro look, with tests
```

Notes for Claude Code sessions are in [CLAUDE.md](../CLAUDE.md).

## Pixel art

All artwork is original and drawn as text grids in `ui/lantern.txt` and
`ui/icons.txt`: a palette of `= <char> <#rrggbb>` lines, then `frame <name>`
blocks. Rust turns each frame into a `shape-rendering: crispEdges` SVG at build
time (disabled buttons get an embossed gray/white version). The throbber is a
4-frame strip animated with CSS `steps(4)`. After editing the lantern, run
`python3 tools/make_icons.py` (needs Pillow) to regenerate the PNG, `.icns` and
`.ico` files.
