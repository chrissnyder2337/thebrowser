//! UI modes: give the toolbar and status bar another 1990s browser's layout
//! (`ui/chrome/<mode>/`, docs/ui-modes/), switching without a restart.

use super::{
    pages, retro, settings,
    settings::{ModePages, Settings, UiMode},
    state::{Role, set_notice, with_shell},
    util::js_string,
};
use cef::*;
use std::cell::Cell;

thread_local! {
    // Kept out of the shell state: CEF asks for view sizes during layout,
    // which can happen while the shell is borrowed.
    static HEIGHTS: Cell<(i32, i32)> = Cell::new(heights(UiMode::Lantern));
}

/// Heights of the toolbar and status views, in DIP.
pub fn heights(mode: UiMode) -> (i32, i32) {
    match mode {
        UiMode::Lantern => (112, 24),
        UiMode::Workstation1993 => (104, 72),
        UiMode::Toolbar1996 => (118, 22),
        UiMode::Desktop1996 => (104, 22),
        UiMode::Applet1997 => (102, 22),
    }
}

const PAGE_DEFAULTS_JS: &str = include_str!("../../ui/chrome/page-defaults.js");

/// A mode's browser defaults for pages: rules (`ui/chrome/<mode>/pages.css`)
/// and the page color for pages with no background of their own.
fn page_defaults(mode: UiMode) -> Option<(&'static str, &'static str)> {
    match mode {
        UiMode::Lantern => None,
        UiMode::Workstation1993 => Some((
            include_str!("../../ui/chrome/workstation-1993/pages.css"),
            "#bfbfbf",
        )),
        UiMode::Toolbar1996 => Some((
            include_str!("../../ui/chrome/toolbar-1996/pages.css"),
            "#c0c0c0",
        )),
        UiMode::Desktop1996 => Some((
            include_str!("../../ui/chrome/desktop-1996/pages.css"),
            "#ffffff",
        )),
        UiMode::Applet1997 => Some((
            include_str!("../../ui/chrome/applet-1997/pages.css"),
            "#ffffff",
        )),
    }
}

/// Script that puts the mode's browser defaults under a content frame's page
/// (`ui/chrome/page-defaults.js`), or takes them out. None when there's
/// nothing to put in, so loads skip it.
pub fn page_defaults_script(settings: &Settings) -> Option<String> {
    let (css, background) = page_defaults(settings.ui_mode).filter(|_| settings.mode_defaults())?;
    let background = format!(":where(html) {{ background: {background}; }}");
    Some(defaults_call(css, &background))
}

/// Script that takes the browser defaults back out of a frame.
pub fn remove_page_defaults_script() -> String {
    defaults_call("", "")
}

fn defaults_call(css: &str, background: &str) -> String {
    format!(
        "{}({},{});",
        PAGE_DEFAULTS_JS.trim_end(),
        js_string(css),
        js_string(background)
    )
}

/// Heights for the mode the window is in now.
pub fn current_heights() -> (i32, i32) {
    HEIGHTS.with(Cell::get)
}

/// Size the views for `mode` from the next layout on.
pub fn use_heights(mode: UiMode) {
    HEIGHTS.with(|cell| cell.set(heights(mode)));
}

/// Switch to `mode` and remember it.
pub fn set_mode(mode: UiMode) {
    if with_shell(|shell| shell.settings.ui_mode) == mode {
        return;
    }
    let updated = with_shell(|shell| {
        shell.settings.ui_mode = mode;
        shell.settings
    });
    settings::save(&updated);
    apply();
    retro::refresh();
    set_notice(&format!("Browser UI: {}.", mode.name()));
}

/// Choose how pages look under the UI modes, and remember it.
pub fn set_pages(pages: ModePages) {
    let updated = with_shell(|shell| {
        shell.settings.mode_pages = pages;
        shell.settings
    });
    settings::save(&updated);
    retro::refresh();
    set_notice(&format!("{}.", pages.name()));
}

/// Load the current mode's chrome pages and lay the window out for them.
/// Each page sends `ready` once loaded, which brings the full state push.
pub fn apply() {
    let (mode, window, toolbar, status) = with_shell(|shell| {
        (
            shell.settings.ui_mode,
            shell.window.clone(),
            shell.toolbar.clone(),
            shell.status.clone(),
        )
    });
    use_heights(mode);
    for (role, browser, uri) in [
        (Role::Toolbar, &toolbar, pages::toolbar_uri(mode)),
        (Role::Status, &status, pages::status_uri(mode)),
    ] {
        let Some(frame) = browser.as_ref().and_then(|b| b.main_frame()) else {
            continue;
        };
        // Not ready until the new page says so; pushes wait until then.
        with_shell(|shell| match role {
            Role::Toolbar => shell.toolbar_ready = false,
            _ => shell.status_ready = false,
        });
        frame.load_url(Some(&CefString::from(uri.as_str())));
    }
    // The views' delegates size themselves from the new heights.
    for mut browser in [toolbar, status].into_iter().flatten() {
        if let Some(view) = browser_view_get_for_browser(Some(&mut browser)) {
            view.invalidate_layout();
        }
    }
    if let Some(window) = window {
        window.layout();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_defaults_follow_the_setting() {
        let mut settings = Settings {
            ui_mode: UiMode::Workstation1993,
            ..Settings::default()
        };
        let script = page_defaults_script(&settings).unwrap();
        assert!(script.starts_with("// Puts a UI mode's"));
        assert!(script.ends_with(r#",":where(html) { background: #bfbfbf; }");"#));
        settings.mode_pages = ModePages::AsIs;
        assert_eq!(page_defaults_script(&settings), None);
        settings.mode_pages = ModePages::EraLook;
        assert!(page_defaults_script(&settings).is_some());
        assert_eq!(page_defaults_script(&Settings::default()), None);
        assert!(remove_page_defaults_script().ends_with(r#"("","");"#));
        // Every rule a mode adds is one line with a zero-specificity
        // selector, so style attributes win.
        for mode in UiMode::ALL {
            if let Some((css, _)) = page_defaults(mode) {
                let css = css.split("*/").last().unwrap_or(css);
                for rule in css.lines().filter(|l| !l.trim().is_empty()) {
                    let (selector, body) = rule.split_once('{').expect(rule);
                    let selector = selector.trim();
                    assert!(body.trim_end().ends_with('}'), "{mode:?}: {rule}");
                    assert!(
                        selector.starts_with(":where(")
                            && selector.ends_with(')')
                            && selector.matches('(').count() == 1,
                        "{mode:?}: {rule}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_mode_has_room_for_its_chrome() {
        for mode in UiMode::ALL {
            let (toolbar, status) = heights(mode);
            assert!(toolbar >= 24 && status >= 20, "{mode:?}");
        }
        assert_eq!(heights(UiMode::Lantern), (112, 24));
    }
}
