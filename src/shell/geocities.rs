//! GeoCities mode: restyles every page in the content view like a 1990s
//! personal homepage by running `ui/geocities.js` in its frames.

use super::{
    settings,
    state::{content_browser, push_toolbar, set_notice, with_shell},
};
use cef::*;

const SCRIPT: &str = include_str!("../../ui/geocities.js");

fn script(on: bool) -> String {
    format!("{}({on});", SCRIPT.trim_end())
}

fn run(frame: &Frame, on: bool) {
    let code = CefString::from(script(on).as_str());
    let script_url = CefString::from("");
    frame.execute_java_script(Some(&code), Some(&script_url), 0);
}

pub fn is_on() -> bool {
    with_shell(|shell| shell.settings.geocities)
}

/// Restyle a frame that just finished loading, if the mode is on.
pub fn on_frame_loaded(frame: &Frame) {
    if is_on() {
        run(frame, true);
    }
}

/// Apply the current mode to every frame of the content view.
fn apply_to_content(on: bool) {
    let Some(browser) = content_browser() else {
        return;
    };
    let mut ids = CefStringList::new();
    browser.frame_identifiers(Some(&mut ids));
    for id in ids {
        if let Some(frame) = browser.frame_by_identifier(Some(&CefString::from(id.as_str()))) {
            run(&frame, on);
        }
    }
}

pub fn set(on: bool) {
    let settings = with_shell(|shell| {
        shell.settings.geocities = on;
        shell.settings
    });
    settings::save(&settings);
    apply_to_content(on);
    push_toolbar();
    set_notice(if on {
        "GeoCities mode on. Welcome to the information superhighway!"
    } else {
        "GeoCities mode off."
    });
}

pub fn toggle() {
    set(!is_on());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_is_a_call() {
        let on = script(true);
        assert!(on.starts_with("// GeoCities mode"));
        assert!(on.ends_with("})(true);"));
        assert!(script(false).ends_with("})(false);"));
    }
}
