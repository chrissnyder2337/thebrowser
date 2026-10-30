//! Builds the `data:text/html` URIs for the chrome pages (toolbar, status bar)
//! and the internal pages shown in the content view.

use super::{
    art::{with_icons, with_lantern},
    bookmarks::Bookmark,
    settings::{DialupMode, ModePages, ModemSpeed, RetroStyle, Settings, UiMode},
    state::Role,
    util::{encode_component, get_data_uri, html_escape, truncate},
};
use std::fmt::Write;

pub const HOME: &str = "lantern:home";
pub const BOOKMARKS: &str = "lantern:bookmarks";
pub const ABOUT: &str = "lantern:about";
pub const SETTINGS: &str = "lantern:settings";

/// Host whose URLs internal pages use as "links back into Rust".
pub const ACTION_HOST: &str = "lantern.invalid";

/// A UI mode's files (`ui/chrome/<key>/`): the toolbar and status pages, and
/// whichever of `menus.txt`, `side.html` and `frame.html` the mode has.
#[derive(Clone, Copy)]
struct ModeFiles {
    toolbar: &'static str,
    status: &'static str,
    menus: Option<&'static str>,
    side: Option<&'static str>,
    frame: Option<&'static str>,
}

macro_rules! mode_files {
    ($key:literal $(, $extra:ident)*) => {{
        #[allow(unused_mut)]
        let mut files = ModeFiles {
            toolbar: include_str!(concat!("../../ui/chrome/", $key, "/toolbar.html")),
            status: include_str!(concat!("../../ui/chrome/", $key, "/status.html")),
            menus: None,
            side: None,
            frame: None,
        };
        $(mode_files!(@ files, $key, $extra);)*
        files
    }};
    (@ $files:ident, $key:literal, menus) => {
        $files.menus = Some(include_str!(concat!("../../ui/chrome/", $key, "/menus.txt")))
    };
    (@ $files:ident, $key:literal, side) => {
        $files.side = Some(include_str!(concat!("../../ui/chrome/", $key, "/side.html")))
    };
    (@ $files:ident, $key:literal, frame) => {
        $files.frame = Some(include_str!(concat!("../../ui/chrome/", $key, "/frame.html")))
    };
}

/// What a chrome view a mode doesn't use shows (it's hidden anyway).
const BLANK_HTML: &str = "<!doctype html><meta charset=\"utf-8\"><title>Lantern</title><body style=\"margin:0;background:#c0c0c0\"></body>";
/// The controller every chrome page shares (title channel, `lantern.update`).
const CHROME_JS: &str = include_str!("../../ui/chrome/chrome.js");
const HOME_HTML: &str = include_str!("../../ui/pages/home.html");
const BOOKMARKS_HTML: &str = include_str!("../../ui/pages/bookmarks.html");
const ABOUT_HTML: &str = include_str!("../../ui/pages/about.html");
const ERROR_HTML: &str = include_str!("../../ui/pages/error.html");
const SETTINGS_HTML: &str = include_str!("../../ui/pages/settings.html");
const PAGE_CSS: &str = include_str!("../../ui/pages/common.css");

/// Replace `{{NAME}}` placeholders in one pass. Values must already be safe
/// HTML (use [`html_escape`] for text); inserted values are never re-scanned.
fn fill(template: &str, vars: &[(&str, String)]) -> String {
    let mut out = String::with_capacity(template.len() * 2);
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                let key = &after[..end];
                match vars.iter().find(|(k, _)| *k == key) {
                    Some((_, value)) => out.push_str(value),
                    None => {
                        debug_assert!(false, "unknown template placeholder {key}");
                        out.push_str(&rest[start..start + 2 + end + 2]);
                    }
                }
                rest = &after[end + 2..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn html_uri(html: &str) -> String {
    get_data_uri(html.as_bytes(), "text/html")
}

/// "154.0.34+g14c5a08+chromium-154.0.8037.98" → ("154.0.34+g14c5a08", "154.0.8037.98")
pub fn cef_versions() -> (String, String) {
    let raw = String::from_utf8_lossy(cef::sys::CEF_VERSION);
    let raw = raw.trim_end_matches('\0');
    match raw.split_once("+chromium-") {
        Some((cef, chromium)) => (cef.to_owned(), chromium.to_owned()),
        None => (raw.to_owned(), "unknown".to_owned()),
    }
}

fn logo(scale: usize) -> String {
    with_lantern(|sheet| sheet.svg("lit0", scale, "logo"))
}

fn common_vars() -> Vec<(&'static str, String)> {
    vec![
        ("VERSION", html_escape(env!("CARGO_PKG_VERSION"))),
        ("LOGO", logo(4)),
        ("PAGE_ICON", with_icons(|s| s.svg("page", 1, "pg"))),
        ("STYLE", PAGE_CSS.to_owned()),
        (
            "SETTINGS_URL",
            html_escape(&format!("https://{ACTION_HOST}/open-settings")),
        ),
    ]
}

fn mode_files(mode: UiMode) -> ModeFiles {
    match mode {
        UiMode::Lantern => mode_files!("lantern"),
        UiMode::Workstation1993 => mode_files!("workstation-1993", menus),
        UiMode::Toolbar1996 => mode_files!("toolbar-1996", menus),
        UiMode::Desktop1996 => mode_files!("desktop-1996", menus),
        UiMode::Applet1997 => mode_files!("applet-1997", menus),
        UiMode::Terminal1999 => mode_files!("terminal-1999", menus),
        UiMode::Terminal1992 => mode_files!("terminal-1992", menus),
        UiMode::Pc1993 => mode_files!("pc-1993", menus),
        UiMode::Toolbar1995 => mode_files!("toolbar-1995", menus),
        UiMode::Desktop1995 => mode_files!("desktop-1995", menus),
        UiMode::Suite1997 => mode_files!("suite-1997", menus),
        UiMode::Desktop1997 => mode_files!("desktop-1997", menus),
        UiMode::Desktop1999 => mode_files!("desktop-1999", menus),
        UiMode::Classic1992 => mode_files!("classic-1992", menus, frame),
        UiMode::Classic1994 => mode_files!("classic-1994", menus, frame),
        UiMode::Tv1996 => mode_files!("tv-1996", menus),
        UiMode::Dos1996 => mode_files!("dos-1996", menus, side),
        UiMode::Cube1990 => mode_files!("cube-1990", menus, side, frame),
        UiMode::Tv1997 => mode_files!("tv-1997", menus),
        UiMode::Workspace1997 => mode_files!("workspace-1997", menus, side, frame),
        UiMode::Workstation1996 => mode_files!("workstation-1996", menus),
    }
}

/// The chrome view holding `mode`'s address field (`#url`): the toolbar in
/// most modes, the page window's frame or the TV's bottom panel in others.
pub fn url_role(mode: UiMode) -> Role {
    let files = mode_files(mode);
    let has_url = |page: Option<&str>| page.is_some_and(|p| p.contains(r#"id="url""#));
    [
        (Role::Toolbar, Some(files.toolbar)),
        (Role::Status, Some(files.status)),
        (Role::Side, files.side),
        (Role::Frame, files.frame),
    ]
    .into_iter()
    .find(|(_, page)| has_url(*page))
    .map_or(Role::Toolbar, |(role, _)| role)
}

/// A UI mode's menu bar (`ui/chrome/<mode>/menus.txt`), if it has its own.
pub fn menus_text(mode: UiMode) -> Option<&'static str> {
    mode_files(mode).menus
}

/// Whether a chrome view (toolbar or status bar) may navigate to `url`: only
/// to an HTML `data:` page, and only its first page, a reload of the current
/// one, or exactly the page Rust built for it in the current UI mode
/// (`expected`, built only when needed). Web content can't produce that URI.
pub fn chrome_navigation_allowed(
    url: &str,
    current: &str,
    expected: impl FnOnce() -> String,
) -> bool {
    if !url.starts_with("data:text/html") {
        return false;
    }
    let initial = current.is_empty() || current == "about:blank";
    initial || url == current || url == expected()
}

/// Placeholders every chrome page may use: each icon frame as `ICON_<NAME>`
/// (normal and disabled svgs; `-` becomes `_`), the lock, page and modem
/// icons, the lantern for activity boxes, and the shared controller.
fn chrome_vars() -> Vec<(String, String)> {
    let mut vars = vec![];
    with_icons(|sheet| {
        for name in sheet.names() {
            let html = match name {
                "lock" => sheet.svg(name, 1, "lock"),
                "page" => sheet.svg(name, 1, "pg"),
                _ => format!(
                    "{}{}",
                    sheet.svg(name, 1, "on"),
                    sheet.embossed_svg(name, 1, "off")
                ),
            };
            let key = format!("ICON_{}", name.to_ascii_uppercase().replace('-', "_"));
            vars.push((key, html));
        }
    });
    with_lantern(|sheet| {
        vars.push(("LANTERN_IDLE".to_owned(), sheet.svg("idle", 2, "idle")));
        vars.push((
            "LANTERN_LIT".to_owned(),
            sheet.strip_svg(&["lit0", "lit1", "lit2", "lit3"], 2, "lit"),
        ));
    });
    vars.push(("CHROME_JS".to_owned(), CHROME_JS.trim_end().to_owned()));
    vars
}

fn fill_chrome(template: &str) -> String {
    let vars = chrome_vars();
    let vars: Vec<(&str, String)> = vars.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    html_uri(&fill(template, &vars))
}

pub fn toolbar_uri(mode: UiMode) -> String {
    fill_chrome(mode_files(mode).toolbar)
}

pub fn status_uri(mode: UiMode) -> String {
    fill_chrome(mode_files(mode).status)
}

/// The chrome views `mode` has a page of its own for.
pub fn chrome_views(mode: UiMode) -> Vec<Role> {
    let mut views = vec![Role::Toolbar, Role::Status];
    if mode_files(mode).side.is_some() {
        views.push(Role::Side);
    }
    if mode_files(mode).frame.is_some() {
        views.push(Role::Frame);
    }
    views
}

/// The page a chrome view shows in `mode` (a blank one if the mode doesn't
/// use that view).
pub fn chrome_uri(role: Role, mode: UiMode) -> String {
    match role {
        Role::Toolbar => toolbar_uri(mode),
        Role::Status => status_uri(mode),
        Role::Side => fill_chrome(mode_files(mode).side.unwrap_or(BLANK_HTML)),
        Role::Frame => fill_chrome(mode_files(mode).frame.unwrap_or(BLANK_HTML)),
        Role::Content => fill_chrome(BLANK_HTML),
    }
}

fn action_url(action: &str, url: &str) -> String {
    format!(
        "https://{ACTION_HOST}/{action}?url={}",
        encode_component(url)
    )
}

pub fn home_uri(bookmarks: &[Bookmark]) -> String {
    let mut list = String::new();
    for b in bookmarks {
        let _ = write!(
            list,
            r#"<li><a href="{url}" title="{url}">{title}</a></li>"#,
            url = html_escape(&b.url),
            title = html_escape(&truncate(&b.title, 60)),
        );
    }
    if list.is_empty() {
        list.push_str("<li><i>No bookmarks yet. Press Mark to add one.</i></li>");
    }
    let mut vars = common_vars();
    vars.push(("BOOKMARK_LIST", list));
    html_uri(&fill(HOME_HTML, &vars))
}

fn bookmark_rows(bookmarks: &[Bookmark]) -> String {
    let mut rows = String::new();
    for (i, b) in bookmarks.iter().enumerate() {
        let _ = write!(
            rows,
            concat!(
                r#"<tr class="{parity}"><td class="n">{n}</td><td>{title}</td>"#,
                r#"<td class="u" title="{url}">{url_short}</td>"#,
                r#"<td class="act"><a class="btn small" href="{url}">Open</a> "#,
                r#"<a class="btn small" href="{delete}">Delete</a></td></tr>"#
            ),
            parity = if i % 2 == 0 { "even" } else { "odd" },
            n = i + 1,
            title = html_escape(&truncate(&b.title, 70)),
            url = html_escape(&b.url),
            url_short = html_escape(&truncate(&b.url, 70)),
            delete = html_escape(&action_url("delete", &b.url)),
        );
    }
    if rows.is_empty() {
        rows.push_str(r#"<tr><td colspan="4" class="empty">No bookmarks.</td></tr>"#);
    }
    rows
}

pub fn bookmarks_uri(bookmarks: &[Bookmark]) -> String {
    let mut vars = common_vars();
    vars.push(("ROWS", bookmark_rows(bookmarks)));
    vars.push(("COUNT", bookmarks.len().to_string()));
    vars.push((
        "HOME_URL",
        html_escape(&format!("https://{ACTION_HOST}/home")),
    ));
    html_uri(&fill(BOOKMARKS_HTML, &vars))
}

pub fn about_uri(data_dir: &str) -> String {
    let (cef, chromium) = cef_versions();
    let mut vars = common_vars();
    vars.push(("CEF_VERSION", html_escape(&cef)));
    vars.push(("CHROMIUM_VERSION", html_escape(&chromium)));
    vars.push(("DATA_DIR", html_escape(data_dir)));
    vars.push((
        "HOME_URL",
        html_escape(&format!("https://{ACTION_HOST}/home")),
    ));
    html_uri(&fill(ABOUT_HTML, &vars))
}

fn mode_description(mode: DialupMode) -> (&'static str, &'static str) {
    match mode {
        DialupMode::Off => ("Off", "Pages load at your real connection speed."),
        DialupMode::Images => (
            "Slow images",
            "Text appears right away, but every picture paints in from the top, \
             a few lines at a time, as if it were still coming down the phone line.",
        ),
        DialupMode::Network => (
            "Slow network",
            "Everything the page loads is squeezed through a modem-sized pipe: \
             text trickles in, images arrive last, and big sites take minutes.",
        ),
    }
}

fn speed_description(speed: ModemSpeed) -> &'static str {
    match speed {
        ModemSpeed::K14_4 => "about 1.6 KB/s",
        ModemSpeed::K28_8 => "about 3.2 KB/s",
        ModemSpeed::K33_6 => "about 3.8 KB/s",
        ModemSpeed::K56 => "about 5.5 KB/s",
    }
}

fn retro_description(style: RetroStyle) -> &'static str {
    match style {
        RetroStyle::Off => "Pages look the way their designers made them.",
        RetroStyle::GeoCities => {
            "Starry backgrounds, Comic Sans, rainbow headings, blinking links, \
             an UNDER CONSTRUCTION sign, a hit counter and sparkles behind the mouse."
        }
        RetroStyle::EarlyWeb => {
            "Switches every site's style sheets off, so pages show as plain HTML: \
             gray background, Times, blue and purple links. With a hit counter, of course."
        }
        RetroStyle::Terminal => {
            "Draws every page the way a text-mode browser showed it in an 80-column \
             terminal: monospace on black, highlighted links, no pictures."
        }
        RetroStyle::SetTop => {
            "Draws pages the way a TV set-top box did: one column of large type on \
             grey-blue, links without underlines and a yellow box around the selected one."
        }
        RetroStyle::Portal => {
            "Squeezes pages into an 800x600-friendly column of small Arial with \
             colored heading bars, a channel bar, a search box and a blinking banner ad."
        }
    }
}

/// What a UI mode is, for the settings page. Prose may say what a mode is
/// modelled on; the page carries the trademark notice (steward skill).
fn ui_mode_description(mode: UiMode) -> &'static str {
    match mode {
        UiMode::Lantern => "Lantern's own toolbar, with big buttons and quick links.",
        UiMode::Workstation1993 => {
            "A Motif menu bar, Document Title and URL fields and a row of text buttons \
             under the page, modelled on NCSA Mosaic for X."
        }
        UiMode::Toolbar1996 => {
            "Grouped icon buttons, a Location field and directory buttons holding your \
             quick links, modelled on Netscape Navigator 2 and 3."
        }
        UiMode::Desktop1996 => {
            "Bands of large buttons, grey until you point at them, with Address and Links \
             bands, modelled on Internet Explorer 3."
        }
        UiMode::Applet1997 => {
            "Flat icon buttons with small labels in a warm grey toolkit look, modelled on \
             HotJava."
        }
        UiMode::Terminal1999 => {
            "A menu bar and URL prompt over the page and a status line under it, all \
             monospace on black, modelled on Links."
        }
        UiMode::Terminal1992 => {
            "The page title at the top and a status line, address prompt and key help \
             at the bottom, monospace on black, modelled on Lynx."
        }
        UiMode::Pc1993 => {
            "A Windows 3.1 menu bar, a strip with the page title, and the address in the \
             status line, modelled on Cello."
        }
        UiMode::Toolbar1995 => {
            "Icon buttons with labels, a Location field and directory buttons, with a \
             key in the status bar, modelled on Netscape Navigator 1."
        }
        UiMode::Desktop1995 => {
            "One row of small icon buttons and an Address row beside the activity box, \
             modelled on Internet Explorer 2."
        }
        UiMode::Suite1997 => {
            "Three toolbars with collapse tabs, a personal toolbar of your quick links and \
             a component bar, modelled on Netscape Communicator 4."
        }
        UiMode::Desktop1997 => {
            "The menu bar as a band, flat labelled buttons and Address and Links bands, \
             modelled on Internet Explorer 4."
        }
        UiMode::Desktop1999 => {
            "A menu band with a small activity box, flat labelled buttons and an Address \
             band with a Go button, modelled on Internet Explorer 5."
        }
        UiMode::Classic1992 => {
            "A one-bit menu bar and a document window with a striped title bar on a grey \
             desktop, modelled on MacWWW."
        }
        UiMode::Classic1994 => {
            "A menu bar over a document window holding the activity box, icon buttons and \
             a Link field, modelled on NCSA Mosaic for the Macintosh."
        }
        UiMode::Tv1996 => {
            "The page fills a TV screen; a slim bar at the bottom and an Options panel of \
             big rounded buttons, modelled on WebTV."
        }
        UiMode::Dos1996 => {
            "Title and URL rows and a full-height side panel with a big activity box, a \
             grid of icon buttons, shortcuts and system stats, modelled on Arachne."
        }
        UiMode::Cube1990 => {
            "A vertical application menu, Navigation and Open panels and a document window \
             with a black title bar, modelled on WorldWideWeb."
        }
        UiMode::Tv1997 => {
            "The page fills a TV screen; an Options panel with a column of buttons, a row \
             of icon tiles and the page title, modelled on WebTV Plus."
        }
        UiMode::Workspace1997 => {
            "A page window inside the main window, a Hot List panel of your quick links and \
             a zoom box, modelled on Opera 3."
        }
        UiMode::Workstation1996 => {
            "A Motif menu bar, a toolbar of icon buttons and a URL row over a sunken page, \
             modelled on NCSA Mosaic for X 2.7."
        }
    }
}

fn checked(on: bool) -> &'static str {
    if on { " checked" } else { "" }
}

pub fn settings_uri(settings: &Settings, saved: bool) -> String {
    let mut modes = String::new();
    for mode in DialupMode::ALL {
        let (name, description) = mode_description(mode);
        let _ = write!(
            modes,
            concat!(
                r#"<label class="opt"><input type="radio" name="dialup" value="{key}"{checked}>"#,
                r#"<span><b>{name}</b><br><small>{description}</small></span></label>"#
            ),
            key = mode.key(),
            checked = checked(settings.dialup_mode == mode),
            name = html_escape(name),
            description = html_escape(description),
        );
    }
    let mut speeds = String::new();
    for speed in ModemSpeed::ALL {
        let _ = write!(
            speeds,
            concat!(
                r#"<label class="speed"><input type="radio" name="speed" value="{key}"{checked}>"#,
                r#"<span><b>{label}</b> <small>{description}</small></span></label>"#
            ),
            key = speed.key(),
            checked = checked(settings.modem_speed == speed),
            label = html_escape(&speed.label()),
            description = html_escape(speed_description(speed)),
        );
    }
    let mut retro = String::new();
    for style in RetroStyle::ALL {
        let _ = write!(
            retro,
            concat!(
                r#"<label class="opt"><input type="radio" name="retro" value="{key}"{checked}>"#,
                r#"<span><b>{name}</b><br><small>{description}</small></span></label>"#
            ),
            key = style.key(),
            checked = checked(settings.retro_style == style),
            name = html_escape(style.name()),
            description = html_escape(retro_description(style)),
        );
    }
    let mut ui_modes = String::new();
    for mode in UiMode::ALL {
        let _ = write!(
            ui_modes,
            concat!(
                r#"<label class="opt"><input type="radio" name="ui" value="{key}"{checked}>"#,
                r#"<span><b>{name}</b><br><small>{description}</small></span></label>"#
            ),
            key = mode.key(),
            checked = checked(settings.ui_mode == mode),
            name = html_escape(mode.name()),
            description = html_escape(ui_mode_description(mode)),
        );
    }
    let mut mode_pages = String::new();
    for pages in ModePages::ALL {
        let _ = write!(
            mode_pages,
            r#"<label class="opt"><input type="radio" name="pages" value="{key}"{checked}><span>{name}</span></label>"#,
            key = pages.key(),
            checked = checked(settings.mode_pages == pages),
            name = html_escape(pages.name()),
        );
    }
    let saved = if saved {
        r#"<p class="saved sunken">Settings saved.</p>"#.to_owned()
    } else {
        String::new()
    };
    let mut vars = common_vars();
    vars.push(("MODE_OPTIONS", modes));
    vars.push(("SPEED_OPTIONS", speeds));
    vars.push(("RETRO_OPTIONS", retro));
    vars.push(("UI_OPTIONS", ui_modes));
    vars.push(("PAGES_OPTIONS", mode_pages));
    vars.push(("SPLASH_CHECKED", checked(settings.splash_intro).to_owned()));
    vars.push(("SAVED", saved));
    vars.push((
        "ACTION_URL",
        html_escape(&format!("https://{ACTION_HOST}/settings")),
    ));
    vars.push((
        "HOME_URL",
        html_escape(&format!("https://{ACTION_HOST}/home")),
    ));
    html_uri(&fill(SETTINGS_HTML, &vars))
}

pub fn error_uri(failed_url: &str, error_code: i32, error_text: &str) -> String {
    let retryable = ["http://", "https://", "file://"]
        .iter()
        .any(|scheme| failed_url.starts_with(scheme));
    let retry = if retryable {
        format!(
            r#"<a class="btn" href="{}">Retry</a>"#,
            html_escape(failed_url)
        )
    } else {
        String::new()
    };
    let error_text = if error_text.is_empty() {
        "UNKNOWN_ERROR"
    } else {
        error_text
    };
    let mut vars = common_vars();
    vars.push(("URL", html_escape(failed_url)));
    vars.push(("ERROR_TEXT", html_escape(error_text)));
    vars.push(("ERROR_CODE", error_code.to_string()));
    vars.push(("RETRY", retry));
    vars.push((
        "HOME_URL",
        html_escape(&format!("https://{ACTION_HOST}/home")),
    ));
    html_uri(&fill(ERROR_HTML, &vars))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_is_single_pass() {
        assert_eq!(
            fill("a{{X}}b{{Y}}c", &[("X", "{{Y}}".into()), ("Y", "1".into())]),
            "a{{Y}}b1c"
        );
        assert_eq!(fill("open {{ only", &[]), "open {{ only");
    }

    fn assert_filled(template: &str, vars: &[(&str, String)]) {
        let out = fill(template, vars);
        assert!(!out.contains("{{"), "unfilled placeholder in {out}");
    }

    #[test]
    fn templates_have_no_unknown_placeholders() {
        let mut vars = common_vars();
        for key in [
            "BOOKMARK_LIST",
            "ROWS",
            "COUNT",
            "HOME_URL",
            "CEF_VERSION",
            "CHROMIUM_VERSION",
            "DATA_DIR",
            "URL",
            "ERROR_TEXT",
            "ERROR_CODE",
            "RETRY",
            "MODE_OPTIONS",
            "SPEED_OPTIONS",
            "RETRO_OPTIONS",
            "UI_OPTIONS",
            "PAGES_OPTIONS",
            "SPLASH_CHECKED",
            "SAVED",
            "ACTION_URL",
        ] {
            vars.push((key, String::new()));
        }
        for template in [
            HOME_HTML,
            BOOKMARKS_HTML,
            ABOUT_HTML,
            ERROR_HTML,
            SETTINGS_HTML,
        ] {
            assert_filled(template, &vars);
        }
    }

    #[test]
    fn values_are_escaped() {
        let list = [Bookmark::new("<b>x</b>", "https://a/?q=\"1\"")];
        let rows = bookmark_rows(&list);
        assert!(!rows.contains("<b>x"));
        assert!(rows.contains("&lt;b&gt;x&lt;/b&gt;"));
        assert!(rows.contains("https://a/?q=&quot;1&quot;"));
        assert!(bookmarks_uri(&list).starts_with("data:text/html;base64,"));
        assert_eq!(
            action_url("delete", "https://a/?q=1&r=2"),
            "https://lantern.invalid/delete?url=https%3A%2F%2Fa%2F%3Fq%3D1%26r%3D2"
        );
    }

    #[test]
    fn every_ui_mode_has_its_chrome_pages() {
        for mode in UiMode::ALL {
            for uri in [toolbar_uri(mode), status_uri(mode)] {
                assert!(uri.starts_with("data:text/html;base64,"), "{mode:?}");
            }
            let files = mode_files(mode);
            for template in [
                Some(files.toolbar),
                Some(files.status),
                files.side,
                files.frame,
            ]
            .into_iter()
            .flatten()
            {
                assert!(template.contains("{{CHROME_JS}}"), "{mode:?}");
            }
            // Every mode but Lantern has the original's menu bar.
            assert_eq!(files.menus.is_some(), mode != UiMode::Lantern, "{mode:?}");
        }
        assert_ne!(
            toolbar_uri(UiMode::Lantern),
            toolbar_uri(UiMode::Workstation1993)
        );
        // Every mode has one address field, and focusing it finds it.
        assert_eq!(url_role(UiMode::Lantern), Role::Toolbar);
        assert_eq!(url_role(UiMode::Tv1996), Role::Status);
        assert_eq!(url_role(UiMode::Workspace1997), Role::Frame);
    }

    #[test]
    fn chrome_views_only_load_their_own_page() {
        let page = "data:text/html;base64,AAAA";
        let other = "data:text/html;base64,BBBB";
        let expected = || other.to_owned();
        // The first page, a reload, and the current mode's page.
        assert!(chrome_navigation_allowed(page, "", expected));
        assert!(chrome_navigation_allowed(page, "about:blank", expected));
        assert!(chrome_navigation_allowed(page, page, expected));
        assert!(chrome_navigation_allowed(other, page, expected));
        // Anything else, including another data: page or a web page.
        assert!(!chrome_navigation_allowed(
            "data:text/html;base64,CCCC",
            page,
            expected
        ));
        assert!(!chrome_navigation_allowed(
            "https://example.com/",
            "",
            expected
        ));
        assert!(!chrome_navigation_allowed(
            "data:image/svg+xml;base64,AAAA",
            "",
            expected
        ));
        assert!(!chrome_navigation_allowed(
            "https://lantern.invalid/home",
            page,
            expected
        ));
    }

    #[test]
    fn versions_parse() {
        let (cef, chromium) = cef_versions();
        assert!(!cef.is_empty());
        assert!(chromium.chars().next().unwrap().is_ascii_digit());
    }
}
