//! Toolbar → Rust title-channel commands, `lantern.invalid` link actions and
//! the navigation helpers they share.

use super::{
    bookmarks, dialup,
    pages::{self, ACTION_HOST},
    retro, settings,
    state::{
        Role, content_browser, defer, exec_js, internal_url_key, push_all, push_status,
        push_toolbar, save_bookmarks, set_notice, with_shell,
    },
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
        (Role::Toolbar, "ready") => {
            with_shell(|shell| shell.toolbar_ready = true);
            push_toolbar();
        }
        (Role::Status, "ready") => {
            with_shell(|shell| shell.status_ready = true);
            push_status();
        }
        (Role::Toolbar, name) => run_toolbar_command(name, &command.arg),
        // Clicking the modem pane in the status bar.
        (Role::Status, "settings") => show_settings(false),
        _ => {}
    }
    true
}

fn run_toolbar_command(name: &str, arg: &str) {
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
        "retro-menu" => retro::show_menu(arg),
        _ => eprintln!("lantern: unknown toolbar command {name:?}"),
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

pub fn focus_content() {
    with_content(|b| {
        if let Some(host) = b.host() {
            host.set_focus(1);
        }
    });
}

/// Focus the toolbar view and select the URL field.
pub fn focus_url_bar() {
    let Some(toolbar) = with_shell(|shell| shell.toolbar.clone()) else {
        return;
    };
    if let Some(host) = toolbar.host() {
        host.set_focus(1);
    }
    exec_js(&toolbar, "window.lantern&&lantern.focusUrl()");
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
    let (updated, content) = with_shell(|shell| {
        shell.settings.apply_query(params);
        (shell.settings, shell.content.clone())
    });
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

/// Handle a navigation to `https://lantern.invalid/...` from an internal page.
/// Returns true if `url` was such an action link (the navigation must be cancelled).
pub fn handle_action_url(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return false;
    };
    let (host, path) = rest.split_once('/').unwrap_or((rest, ""));
    if !host.eq_ignore_ascii_case(ACTION_HOST) {
        return false;
    }
    let path = path.split('#').next().unwrap_or("");
    let (action, query) = path.split_once('?').unwrap_or((path, ""));
    let params = parse_query(query);
    let param = |name: &str| {
        params
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    };

    // Navigating from inside on_before_browse is not safe; run actions later.
    match action {
        "delete" => {
            if let Some(target) = param("url") {
                defer(move || {
                    let removed =
                        with_shell(|shell| bookmarks::remove(&mut shell.bookmarks, &target));
                    if removed {
                        save_bookmarks();
                        push_all();
                    }
                    refresh_bookmark_pages();
                });
            }
        }
        "home" => defer(show_home),
        "bookmarks" => defer(show_bookmarks),
        "about" => defer(show_about),
        "open-settings" => defer(|| show_settings(false)),
        "settings" => {
            let params = params.clone();
            defer(move || save_settings(&params));
        }
        other => eprintln!("lantern: unknown action link {other:?}"),
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
}
