# Lantern security rules

Lantern is a browser: every page it loads is hostile input. Apply these on
every edit, on top of the general guidance.

1. **Escape everything that reaches HTML or JS.** Text from pages, URLs,
   titles, bookmarks, settings, error strings and command arguments goes
   through `src/shell/util.rs` (`html_escape`, `js_string`,
   `encode_component`) or the `Json` builder before it lands in an internal
   page template, a `data:` URI or an `execute_java_script` call. `pages::fill`
   inserts values as-is (they must already be safe HTML), so escape each
   value first; never build markup or script with `format!` from unescaped
   text. Flag any new `innerHTML`, `eval` or
   `new Function` that takes data in `ui/`.
2. **Never widen `lantern.invalid` handling.** Internal pages reach Rust
   only through `https://lantern.invalid/<action>` links handled by
   `commands::handle_action_url` in the content view. Any web page can link
   there, not only Lantern's own pages, so an action must be harmless when
   a hostile page triggers it: it may only change Lantern's own state
   (bookmarks, settings, internal pages), never run code, touch files or
   reach the network. Don't accept new hosts or schemes, and don't add
   another channel from web content to Rust (JS bindings, `cefQuery`,
   process messages).
3. **Keep chrome views locked down.** The toolbar and status views may only
   load their own `data:text/html` page and never open popups
   (`client.rs`, `on_before_browse` and the life-span handler). The title
   channel is accepted only from those views.
4. **URL handling stays in `util::normalize_url_input`.** Don't let typed or
   page-supplied input navigate to `file:`, `javascript:`, `chrome:` or
   `devtools:` URLs by a new path; new schemes are a tier-2 change.
5. **Popups and downloads** keep going through the existing handlers: popups
   are redirected into the content view, downloads always show the native
   save dialog. No silent writes to disk.
6. **The CEF sandbox stays on in shipped builds.** `--no-sandbox` is for
   local and CI runs only; never make it a default or turn off the `sandbox`
   feature. Never start a release or black-box bundle with
   `--remote-debugging-port`.
7. **No secrets in the repository.** Workflows declare `permissions:`
   starting from `contents: read`, pin third-party Actions to a commit SHA,
   and never interpolate issue or PR text into a shell command or `${{ }}`
   expression.
