//! Toolbar → Rust title-channel commands, `lantern.invalid` link actions and
//! the navigation helpers they share.

use super::{
    bookmarks, dialup, menus,
    pages::{self, ACTION_HOST},
    retro, settings,
    state::{
        Role, content_browser, defer, exec_js, internal_url_key, push_all, push_ready, push_status,
        push_toolbar, save_bookmarks, set_notice, with_shell,
    },
    ui_mode,
    util::{normalize_url_input, parse_query, percent_decode},
};
use cef::*;
use std::cell::Cell;

/// A parsed `lantern|<seq>|<command>|<arg>` title.
#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub arg: String,
}

pub fn parse_title(title: &str) -> Option<Command> {
    let mut parts = title.splitn(4, '|');
    if parts.next()? != "lantern" {
        return None;
    }
    let _seq = parts.next()?;
    let name = parts.next()?.to_owned();
    if name.is_empty() {
        return None;
    }
    let arg = percent_decode(parts.next().unwrap_or(""));
    Some(Command { name, arg })
}

/// Handle a title change from a chrome page. Returns whether it was a command.
pub fn handle_chrome_title(role: Role, title: &str) -> bool {
    let Some(command) = parse_title(title) else {
        return false;
    };
    match (role, command.name.as_str()) {
        (Role::Content, _) => return false,
        (_, "ready") => {
            with_shell(|shell| shell.set_ready(role, true));
            push_ready(role);
        }
        (Role::Toolbar | Role::Side | Role::Frame, name) => {
            run_chrome_command(role, name, &command.arg)
        }
        // Some UI modes put navigation buttons under the page
        // (ui/chrome/<mode>/status.html), but the status bar can't open
        // bookmarks' URLs or menus.
        (Role::Status, name) if STATUS_COMMANDS.contains(&name) => {
            run_chrome_command(role, name, &command.arg)
        }
        (Role::Status, name) => eprintln!("lantern: not a status bar command {name:?}"),
    }
    true
}

/// The commands the status bar may send.
const STATUS_COMMANDS: [&str; 14] = [
    "back",
    "forward",
    "reload",
    "stop",
    "home",
    "search",
    "bookmarks",
    "bookmark-add",
    "focus-url",
    "settings",
    "panel",
    // A typed address: the TV modes' Go to box is in their bottom panel.
    "go",
    // Opens one of the mode's own menus (menus.txt), for browsers whose
    // menu keys sat at the bottom (the text terminal's O)ptions).
    "menu",
    // The activity box (chrome.js sends it), in modes that keep it below the page.
    "about",
];

fn run_chrome_command(role: Role, name: &str, arg: &str) {
    match name {
        "back" => go_back(),
        "forward" => go_forward(),
        "reload" => reload(),
        "stop" => stop(),
        "home" => show_home(),
        "search" => show_search(),
        "go" => navigate_input(arg),
        "open" => navigate(arg),
        "bookmark-add" => add_bookmark(),
        "bookmarks" => show_bookmarks(),
        "about" => show_about(),
        "retro-menu" => menus::show_retro(role, arg),
        "menu" => menus::show(role, arg),
        "focus-url" => focus_url_bar(),
        "panel" => ui_mode::toggle_panel(),
        // The modem pane in the status bar.
        "settings" => show_settings(false),
        // The originals' toolbar buttons for what's otherwise in the menus.
        "print" => print(),
        "view-source" => view_source(),
        "zoom-in" => zoom(0.5),
        "zoom-out" => zoom(-0.5),
        "fullscreen" => toggle_fullscreen(),
        // A document window's close box (the Classic desktop modes).
        "close-window" => super::state::close_all_browsers(false),
        _ => eprintln!("lantern: unknown chrome command {name:?}"),
    }
}

fn with_content(f: impl FnOnce(&Browser)) {
    if let Some(browser) = content_browser() {
        f(&browser);
    }
}

pub fn go_back() {
    with_content(|b| {
        if b.can_go_back() != 0 {
            b.go_back();
        }
    });
}

pub fn go_forward() {
    with_content(|b| {
        if b.can_go_forward() != 0 {
            b.go_forward();
        }
    });
}

pub fn reload() {
    with_content(|b| b.reload());
}

pub fn stop() {
    with_content(|b| b.stop_load());
}

/// Print the page (the menus' Print item).
pub fn print() {
    with_content(|b| {
        if let Some(host) = b.host() {
            host.print();
        }
    });
}

/// Show the page's source (the menus' View Source item).
pub fn view_source() {
    // Not for Lantern's own data: pages, whose source is the generated page.
    if with_shell(|shell| shell.internal_label.is_some()) {
        return;
    }
    with_content(|b| {
        if let Some(frame) = b.main_frame() {
            frame.view_source();
        }
    });
}

/// Zoom the page by `step` levels; 0 resets it.
pub fn zoom(step: f64) {
    with_content(|b| {
        if let Some(host) = b.host() {
            let level = if step == 0.0 {
                0.0
            } else {
                host.zoom_level() + step
            };
            host.set_zoom_level(level.clamp(-4.0, 6.0));
        }
    });
}

/// Fill the screen with the window, or come back (the menus' Full Screen).
pub fn toggle_fullscreen() {
    if let Some(window) = with_shell(|shell| shell.window.clone()) {
        let on = window.is_fullscreen() == 0;
        window.set_fullscreen(on.into());
    }
}

pub fn focus_content() {
    with_content(|b| {
        if let Some(host) = b.host() {
            host.set_focus(1);
        }
    });
}

/// Focus the toolbar view and select the URL field.
pub fn focus_url_bar() {
    let role = pages::url_role(with_shell(|shell| shell.settings.ui_mode));
    // The TV modes keep the address in a panel that may be closed.
    let (layout, open) = ui_mode::current();
    if role == Role::Status && layout.status_open > 0 && !open {
        ui_mode::toggle_panel();
    }
    let Some(view) = with_shell(|shell| shell.browser(role)) else {
        return;
    };
    if let Some(host) = view.host() {
        host.set_focus(1);
    }
    exec_js(&view, "window.lantern&&lantern.focusUrl()");
}

/// Load a URL in the content view (a real web address, not an internal page).
pub fn navigate(url: &str) {
    let url = url.trim();
    if url.is_empty() {
        return;
    }
    if let Some(page) = internal_page_for(url) {
        page();
        return;
    }
    with_shell(|shell| shell.notice.clear());
    with_content(|b| {
        if let Some(frame) = b.main_frame() {
            frame.load_url(Some(&CefString::from(url)));
        }
    });
    focus_content();
}

/// Text typed into the URL bar.
pub fn navigate_input(text: &str) {
    if let Some(page) = internal_page_for(text.trim()) {
        page();
    } else if let Some(url) = normalize_url_input(text) {
        navigate(&url);
    }
}

fn internal_page_for(text: &str) -> Option<fn()> {
    match text.to_ascii_lowercase().as_str() {
        pages::HOME => Some(show_home),
        pages::BOOKMARKS => Some(show_bookmarks),
        pages::ABOUT => Some(show_about),
        pages::SETTINGS => Some(|| show_settings(false)),
        _ => None,
    }
}

/// Show a generated page in the content view, labelled `label` in the URL bar.
pub fn show_internal(label: &str, uri: String) {
    with_shell(|shell| {
        shell.internal_label = Some(label.to_owned());
        shell.notice.clear();
        // Bound the label map; pages are cheap to regenerate if a label is lost.
        if shell.internal_urls.len() > 64 {
            shell.internal_urls.clear();
        }
        shell
            .internal_urls
            .insert(internal_url_key(&uri), label.to_owned());
    });
    with_content(|b| {
        if let Some(frame) = b.main_frame() {
            frame.load_url(Some(&CefString::from(uri.as_str())));
        }
    });
    push_toolbar();
}

pub fn show_home() {
    let list = with_shell(|shell| shell.bookmarks.clone());
    show_internal(pages::HOME, pages::home_uri(&list));
    focus_content();
}

thread_local! {
    static SEARCH_SEQ: Cell<u32> = const { Cell::new(0) };
}

/// Home page with its search box focused (via a unique `#search-N` fragment).
pub fn show_search() {
    let list = with_shell(|shell| shell.bookmarks.clone());
    let seq = SEARCH_SEQ.with(|seq| {
        seq.set(seq.get().wrapping_add(1));
        seq.get()
    });
    let uri = format!("{}#search-{seq}", pages::home_uri(&list));
    show_internal(pages::HOME, uri);
    focus_content();
}

pub fn show_bookmarks() {
    let list = with_shell(|shell| shell.bookmarks.clone());
    show_internal(pages::BOOKMARKS, pages::bookmarks_uri(&list));
}

pub fn show_about() {
    let dir = bookmarks::data_dir().display().to_string();
    show_internal(pages::ABOUT, pages::about_uri(&dir));
}

/// The settings page; `saved` adds a "Settings saved" confirmation.
pub fn show_settings(saved: bool) {
    let current = with_shell(|shell| shell.settings);
    show_internal(pages::SETTINGS, pages::settings_uri(&current, saved));
}

/// Apply settings submitted from the settings page and remember them.
fn save_settings(params: &[(String, String)]) {
    let (before, updated, content) = with_shell(|shell| {
        let before = shell.settings.ui_mode;
        shell.settings.apply_query(params);
        (before, shell.settings, shell.content.clone())
    });
    if updated.ui_mode != before {
        ui_mode::apply();
    }
    settings::save(&updated);
    if let Some(content) = content {
        dialup::apply_network(&content, &updated);
    }
    push_status();
    retro::refresh();
    show_settings(true);
}

pub fn show_error(failed_url: &str, error_code: i32, error_text: &str) {
    show_internal(
        failed_url,
        pages::error_uri(failed_url, error_code, error_text),
    );
}

/// Re-render the current internal page if it lists bookmarks.
fn refresh_bookmark_pages() {
    match with_shell(|shell| shell.internal_label.clone()).as_deref() {
        Some(pages::HOME) => show_home(),
        Some(pages::BOOKMARKS) => show_bookmarks(),
        _ => {}
    }
}

pub fn add_bookmark() {
    enum Outcome {
        Added(String),
        Duplicate,
        NotBookmarkable,
    }
    let outcome = with_shell(|shell| {
        let url = shell.current_url.clone();
        if shell.internal_label.is_some() || url.is_empty() || url.starts_with("data:") {
            return Outcome::NotBookmarkable;
        }
        let title = shell.current_title.clone();
        if bookmarks::add(&mut shell.bookmarks, &title, &url) {
            Outcome::Added(
                shell
                    .bookmarks
                    .last()
                    .map(|b| b.title.clone())
                    .unwrap_or_default(),
            )
        } else {
            Outcome::Duplicate
        }
    });
    match outcome {
        Outcome::Added(title) => {
            save_bookmarks();
            push_toolbar();
            set_notice(&format!("Bookmark added: {title}"));
        }
        Outcome::Duplicate => set_notice("This page is already bookmarked."),
        Outcome::NotBookmarkable => set_notice("Lantern pages can't be bookmarked."),
    }
}

/// What a `https://lantern.invalid/<action>?…` link asks for.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Remove the bookmark with this URL (nothing, without a `url=`).
    Delete(Option<String>),
    Home,
    Bookmarks,
    About,
    OpenSettings,
    /// Apply the settings form's fields.
    SaveSettings(Vec<(String, String)>),
    /// An action Lantern doesn't know; the link is still swallowed.
    Unknown(String),
}

/// Parse an action link. `None` means `url` isn't one and loads normally.
pub fn parse_action_url(url: &str) -> Option<Action> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    if !host.eq_ignore_ascii_case(ACTION_HOST) {
        return None;
    }
    let path = path.split('#').next().unwrap_or("");
    let (action, query) = path.split_once('?').unwrap_or((path, ""));
    let params = parse_query(query);
    Some(match action {
        "delete" => Action::Delete(params.into_iter().find(|(k, _)| k == "url").map(|(_, v)| v)),
        "home" => Action::Home,
        "bookmarks" => Action::Bookmarks,
        "about" => Action::About,
        "open-settings" => Action::OpenSettings,
        "settings" => Action::SaveSettings(params),
        other => Action::Unknown(other.to_owned()),
    })
}

/// Handle a navigation to `https://lantern.invalid/...` from an internal page.
/// Returns true if `url` was such an action link (the navigation must be cancelled).
pub fn handle_action_url(url: &str) -> bool {
    let Some(action) = parse_action_url(url) else {
        return false;
    };
    // Navigating from inside on_before_browse is not safe; run actions later.
    match action {
        Action::Delete(Some(target)) => {
            defer(move || {
                let removed = with_shell(|shell| bookmarks::remove(&mut shell.bookmarks, &target));
                if removed {
                    save_bookmarks();
                    push_all();
                }
                refresh_bookmark_pages();
            });
        }
        Action::Delete(None) => {}
        Action::Home => defer(show_home),
        Action::Bookmarks => defer(show_bookmarks),
        Action::About => defer(show_about),
        Action::OpenSettings => defer(|| show_settings(false)),
        Action::SaveSettings(params) => defer(move || save_settings(&params)),
        Action::Unknown(other) => eprintln!("lantern: unknown action link {other:?}"),
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_titles() {
        assert_eq!(
            parse_title("lantern|7|go|rust%20lang%7Cx"),
            Some(Command {
                name: "go".into(),
                arg: "rust lang|x".into()
            })
        );
        assert_eq!(
            parse_title("lantern|1|ready"),
            Some(Command {
                name: "ready".into(),
                arg: String::new()
            })
        );
        assert_eq!(parse_title("Lantern Toolbar"), None);
        assert_eq!(parse_title("lantern|1|"), None);
        assert_eq!(parse_title("lanternX|1|go|x"), None);
    }

    #[test]
    fn hostile_titles() {
        let huge = "x".repeat(1 << 20);
        let cmd = |name: &str, arg: &str| {
            Some(Command {
                name: name.into(),
                arg: arg.into(),
            })
        };
        let cases: Vec<(String, Option<Command>)> = vec![
            // Only the exact prefix counts; titles a page might set don't.
            ("".into(), None),
            ("lantern".into(), None),
            ("lantern|".into(), None),
            ("LANTERN|1|go|x".into(), None),
            (" lantern|1|go|x".into(), None),
            ("<b>lantern</b>|1|go|x".into(), None),
            ("lantern|1||x".into(), None),
            // The argument keeps everything after the third bar, decoded once.
            ("lantern|1|go|a|b|c".into(), cmd("go", "a|b|c")),
            ("lantern|1|go|%2541".into(), cmd("go", "%41")),
            ("lantern|1|go|%ZZ%4".into(), cmd("go", "%ZZ%4")),
            ("lantern|1|go|%FF%FE".into(), cmd("go", "\u{FFFD}\u{FFFD}")),
            ("lantern|1|go|%00".into(), cmd("go", "\0")),
            (
                "lantern|1|go|javascript%3Aalert(1)".into(),
                cmd("go", "javascript:alert(1)"),
            ),
            (
                "lantern|1|go|%3Cscript%3E%22%27".into(),
                cmd("go", "<script>\"'"),
            ),
            // Names are passed on as given; unknown ones are ignored later.
            ("lantern|x|<img src=x>|".into(), cmd("<img src=x>", "")),
            ("lantern||ready".into(), cmd("ready", "")),
            (format!("lantern|1|go|{huge}"), cmd("go", &huge)),
        ];
        for (title, want) in cases {
            let shown: String = title.chars().take(40).collect();
            assert_eq!(parse_title(&title), want, "title {shown:?}");
        }
    }

    #[test]
    fn action_router() {
        let huge = "y".repeat(1 << 20);
        let pairs = |list: &[(&str, &str)]| {
            list.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Vec<_>>()
        };
        let cases: Vec<(String, Option<Action>)> = vec![
            ("https://lantern.invalid/home".into(), Some(Action::Home)),
            ("http://lantern.invalid/home".into(), Some(Action::Home)),
            ("https://LANTERN.INVALID/about".into(), Some(Action::About)),
            (
                "https://lantern.invalid/home#top".into(),
                Some(Action::Home),
            ),
            (
                "https://lantern.invalid/bookmarks?x=1".into(),
                Some(Action::Bookmarks),
            ),
            (
                "https://lantern.invalid/open-settings".into(),
                Some(Action::OpenSettings),
            ),
            (
                "https://lantern.invalid/delete?url=https%3A%2F%2Fa%2F%3Fq%3D1%26r%3D2".into(),
                Some(Action::Delete(Some("https://a/?q=1&r=2".into()))),
            ),
            (
                "https://lantern.invalid/delete?url=a&url=b".into(),
                Some(Action::Delete(Some("a".into()))),
            ),
            (
                "https://lantern.invalid/delete".into(),
                Some(Action::Delete(None)),
            ),
            (
                "https://lantern.invalid/delete?url=javascript%3Aalert(1)".into(),
                Some(Action::Delete(Some("javascript:alert(1)".into()))),
            ),
            (
                format!("https://lantern.invalid/delete?url={huge}"),
                Some(Action::Delete(Some(huge.clone()))),
            ),
            (
                "https://lantern.invalid/settings?retro=%3Cscript%3E&dialup=off".into(),
                Some(Action::SaveSettings(pairs(&[
                    ("retro", "<script>"),
                    ("dialup", "off"),
                ]))),
            ),
            // Unknown or oddly spelled actions are swallowed, never loaded.
            (
                "https://lantern.invalid".into(),
                Some(Action::Unknown(String::new())),
            ),
            (
                "https://lantern.invalid/HOME".into(),
                Some(Action::Unknown("HOME".into())),
            ),
            (
                "https://lantern.invalid/home/../about".into(),
                Some(Action::Unknown("home/../about".into())),
            ),
            (
                "https://lantern.invalid/%68ome".into(),
                Some(Action::Unknown("%68ome".into())),
            ),
            (
                "https://lantern.invalid/<script>".into(),
                Some(Action::Unknown("<script>".into())),
            ),
            // Anything that isn't exactly the action host loads normally.
            ("https://lantern.invalid.example.com/home".into(), None),
            ("https://example.com/lantern.invalid/home".into(), None),
            (
                "https://example.com/?u=https://lantern.invalid/home".into(),
                None,
            ),
            ("https://user@lantern.invalid/home".into(), None),
            ("https://lantern.invalid:443/home".into(), None),
            ("https://lantern%2Einvalid/home".into(), None),
            ("ftp://lantern.invalid/home".into(), None),
            ("file://lantern.invalid/home".into(), None),
            (
                "javascript:location='https://lantern.invalid/home'".into(),
                None,
            ),
            ("data:text/html,https://lantern.invalid/home".into(), None),
            (" https://lantern.invalid/home".into(), None),
            ("lantern:home".into(), None),
            ("".into(), None),
        ];
        for (url, want) in cases {
            let shown: String = url.chars().take(60).collect();
            assert_eq!(parse_action_url(&url), want, "url {shown:?}");
        }
    }
}
