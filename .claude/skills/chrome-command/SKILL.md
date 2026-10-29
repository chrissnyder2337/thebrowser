---
name: chrome-command
description: Add or change a Lantern toolbar (or status bar) command end to end — the button in ui/chrome/lantern/toolbar.html, its pixel-art icon, the title-channel message, the Rust handler in src/shell/commands.rs, and state pushed back with lantern.update — plus the tests and checks. Use when asked to add a toolbar button, menu entry, keyboard shortcut or status-bar click that makes the browser do something.
---

# Adding a chrome command

The toolbar and status bar are `data:` pages built from `ui/chrome/lantern/toolbar.html` and
`ui/chrome/lantern/status.html`. They talk to Rust over the **title channel** and Rust talks
back with **`lantern.update({...})`** (`docs/architecture.md` → "The window and its views"). A new command
touches each hop below. New toolbar commands are **tier 1** (`CLAUDE.md`).
Anything that changes `lantern.invalid` actions, `client.rs` request handling
or `util.rs` escaping is **tier 2**: stop and say so in the PR.

## 1. The button (`ui/chrome/lantern/toolbar.html`)

Add a button next to its neighbours in the toolbar markup:

```html
<button class="big" id="history" data-cmd="history" title="Show history">{{ICON_HISTORY}}<span>History</span></button>
```

Every `.big` button already gets a click handler that calls
`send(b.dataset.cmd, "")` unless it has the `disabled` class, so a plain
command needs no new JS. For anything else (a link, the URL field, a command
with an argument) call `send("<command>", arg)`: it sets
`document.title = "lantern|<seq>|<command>|" + encodeURIComponent(arg)`. The
sequence number makes a repeated command a new title. Status-bar commands use
the same `send` in `ui/chrome/lantern/status.html`.

Labels follow the branding rule in the steward skill: describe the era, never
name a real browser.

## 2. The icon (`ui/icons.txt`, `src/shell/pages.rs`)

- Add a 24×24 `frame <name>` to `ui/icons.txt`, using the palette letters
  declared at the top of the file.
- Add `<name>` to the list in `pages::toolbar_uri()`. It fills `ICON_<NAME>`
  with the normal and embossed (disabled) SVGs.
- Add `<name>` to the icon list in `art::tests::sheets_are_well_formed` so the
  size is checked.

## 3. The handler (`src/shell/commands.rs`)

`client.rs`'s display handler passes chrome-page titles to
`commands::handle_chrome_title(role, title)`, which uses `parse_title` (it
percent-decodes the argument) and then:

- toolbar commands → `run_toolbar_command(name, arg)`: add a match arm there;
- status-bar commands → add a `(Role::Status, "<name>")` arm in
  `handle_chrome_title`.

Unknown names only log, so a typo fails quietly: check the app's stderr.
Rules for the handler:

- Act on the content view through the existing helpers (`with_content`,
  `navigate`, `navigate_input`, `show_internal`, …). Anything typed by the
  user goes through `navigate_input`, which applies `util::normalize_url_input`.
- Everything runs on the CEF UI thread and shell state lives in the
  thread-local `RefCell` behind `state::with_shell`. Don't hold a borrow across
  a call that can re-enter (navigation, `exec_js`); use `state::defer` for
  work that must run later.
- Treat `arg` as untrusted text. Never build JS or HTML from it by hand.

## 4. State back to the toolbar (`src/shell/state.rs`)

If the button needs state (enabled, latched, a label):

- add the field to `Shell` and to `Shell::toolbar_json()` using the `Json`
  builder (`Json::str` escapes; never `format!` strings into JSON or JS);
- call `push_toolbar()` (or `push_all()`) after it changes;
- handle the key in `window.lantern.update` in `ui/chrome/lantern/toolbar.html` with an
  `if ("<key>" in s)` block, like `canBack` or `retro`. Updates are partial,
  so only act on keys that are present.

## 5. Keyboard shortcut (optional, `src/shell/client.rs`)

Shortcuts are in `handle_shortcut`, with key codes in the `vk` module. Use
`primary` (Ctrl, or Cmd on macOS) for accelerators. The flag enum wraps `u32`
on Linux/macOS but `i32` on Windows, which is why the cross-checks exist.
Mention the shortcut in the button's `title`.

## 6. Tests and checks

- Unit-test any new pure helper next to it (`commands.rs` tests cover
  `parse_title`; add a case if the argument format is new).
- `cargo test --bin lantern` runs `templates_have_no_unknown_placeholders`
  and the sheet tests, which catch a missing icon or placeholder.
- Syntax-check the toolbar script: extract the `<script>` body and run
  `node --check`.
- Run the steward skill's "Before every push" checks, then **run the app and
  look** (steward → Xvfb recipe): click the button, check the content view
  and stderr, and take a screenshot of the toolbar for the PR.
- Update `docs/architecture.md` → "The window and its views", which lists the
  toolbar commands.
