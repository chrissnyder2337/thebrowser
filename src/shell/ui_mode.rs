//! UI modes: give the toolbar and status bar another 1990s browser's layout
//! (`ui/chrome/<mode>/`, docs/ui-modes/), switching without a restart.

use super::{
    pages, settings,
    settings::UiMode,
    state::{Role, set_notice, with_shell},
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
    }
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
    set_notice(&format!("Browser UI: {}.", mode.name()));
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
    fn every_mode_has_room_for_its_chrome() {
        for mode in UiMode::ALL {
            let (toolbar, status) = heights(mode);
            assert!(toolbar >= 24 && status >= 20, "{mode:?}");
        }
        assert_eq!(heights(UiMode::Lantern), (112, 24));
    }
}
