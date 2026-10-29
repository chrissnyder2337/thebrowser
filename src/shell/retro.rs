//! Retro looks: restyle every page in the content view like a 1990s site (a
//! GeoCities homepage, the early web of 1995, a '97 web portal), plus an optional
//! Flash-style splash intro, by running `ui/retro/*.js` in its frames. The
//! toolbar's Retro button opens a native menu to pick them (`menus.rs`).

use super::{
    settings,
    settings::RetroStyle,
    state::{content_browser, push_toolbar, set_notice, with_shell},
    util::{js_string, url_host},
};
use cef::*;

const RUNTIME: &str = include_str!("../../ui/retro/runtime.js");
const LOOKS: [(RetroStyle, &str); 3] = [
    (
        RetroStyle::GeoCities,
        include_str!("../../ui/retro/geocities.js"),
    ),
    (
        RetroStyle::EarlyWeb,
        include_str!("../../ui/retro/early-web.js"),
    ),
    (RetroStyle::Portal, include_str!("../../ui/retro/portal.js")),
];
const SPLASH: &str = include_str!("../../ui/retro/splash.js");

/// Script that switches a frame to `style`, defining the runtime and the
/// looks the first time it runs in a document. See `ui/retro/runtime.js`.
pub fn style_script(style: RetroStyle) -> String {
    if style == RetroStyle::Off {
        return r#"window.__lanternRetro&&window.__lanternRetro.apply("off");"#.to_owned();
    }
    let mut js = String::from(
        "(function(style){var R=window.__lanternRetro;if(!R){R=window.__lanternRetro=(",
    );
    js.push_str(RUNTIME.trim_end());
    js.push_str("\n)();");
    for (look, source) in LOOKS {
        js.push_str(&format!("R.define({},", js_string(look.key())));
        js.push_str(source.trim_end());
        js.push_str("\n);");
    }
    js.push_str("}R.apply(style);})(");
    js.push_str(&js_string(style.key()));
    js.push_str(");");
    js
}

fn splash_script(name: &str) -> String {
    format!("({}\n)({});", SPLASH.trim_end(), js_string(name))
}

const REMOVE_SPLASH: &str = "window.__lanternSplash&&window.__lanternSplash.remove();";

fn frame_url(frame: &Frame) -> String {
    CefString::from(&frame.url()).to_string()
}

/// Lantern's own pages (start page, settings, ...) are `data:` URLs. They keep
/// their own look, so the settings page always stays usable.
fn is_internal(url: &str) -> bool {
    url.starts_with("data:")
}

fn run(frame: &Frame, code: &str) {
    let code = CefString::from(code);
    frame.execute_java_script(Some(&code), Some(&CefString::from("")), 0);
}

fn run_in_content(code: &str, all_frames: bool) {
    let Some(browser) = content_browser() else {
        return;
    };
    if browser
        .main_frame()
        .is_some_and(|frame| is_internal(&frame_url(&frame)))
    {
        return;
    }
    if !all_frames {
        if let Some(frame) = browser.main_frame() {
            run(&frame, code);
        }
        return;
    }
    let mut ids = CefStringList::new();
    browser.frame_identifiers(Some(&mut ids));
    for id in ids {
        if let Some(frame) = browser.frame_by_identifier(Some(&CefString::from(id.as_str()))) {
            run(&frame, code);
        }
    }
}

/// A content frame started loading: style it as early as possible and, on
/// entering a new site, show the splash intro over it.
pub fn on_load_start(frame: &Frame) {
    let url = frame_url(frame);
    if is_internal(&url) {
        return;
    }
    let settings = with_shell(|shell| shell.settings);
    if settings.retro_style != RetroStyle::Off {
        run(frame, &style_script(settings.retro_style));
    }
    if !settings.splash_intro || frame.is_main() == 0 {
        return;
    }
    let Some(host) = url_host(&url).filter(|_| url.starts_with("http")) else {
        return;
    };
    let new_site = with_shell(|shell| {
        let new_site = shell.splash_host.as_deref() != Some(host.as_str());
        shell.splash_host = Some(host.clone());
        new_site
    });
    if new_site {
        run(frame, &splash_script(host.trim_start_matches("www.")));
    }
}

/// A content frame finished loading: style what arrived since the start.
pub fn on_load_end(frame: &Frame) {
    let style = with_shell(|shell| shell.settings.retro_style);
    if style != RetroStyle::Off && !is_internal(&frame_url(frame)) {
        run(frame, &style_script(style));
    }
}

/// Apply the current settings to the page on screen (after the settings page
/// or the menu changed them).
pub fn refresh() {
    let settings = with_shell(|shell| shell.settings);
    run_in_content(&style_script(settings.retro_style), true);
    if !settings.splash_intro {
        run_in_content(REMOVE_SPLASH, false);
    }
    push_toolbar();
}

fn update(change: impl FnOnce(&mut settings::Settings)) -> settings::Settings {
    let updated = with_shell(|shell| {
        change(&mut shell.settings);
        shell.settings
    });
    settings::save(&updated);
    refresh();
    updated
}

pub fn set_style(style: RetroStyle) {
    update(|s| s.retro_style = style);
    set_notice(match style {
        RetroStyle::Off => "Back to the modern web.",
        RetroStyle::GeoCities => "GeoCities mode on. Welcome to the information superhighway!",
        RetroStyle::EarlyWeb => "Early web: it's 1995 and style sheets haven't been invented.",
        RetroStyle::Portal => "Welcome to LanternNet, your start page for the World Wide Web.",
    });
}

pub fn set_splash(on: bool) {
    update(|s| s.splash_intro = on);
    // Show it on the next site entered, including the one on screen.
    with_shell(|shell| shell.splash_host = None);
    set_notice(if on {
        "Splash intros on. Reload or visit a site to see one."
    } else {
        "Splash intros off."
    });
}

/// Parse a chrome page's "x,y" menu anchor (CSS pixels in its view).
pub(super) fn parse_point(arg: &str) -> Point {
    let mut parts = arg
        .split(',')
        .map(|n| n.trim().parse::<f64>().unwrap_or(0.0) as i32);
    Point {
        x: parts.next().unwrap_or(0),
        y: parts.next().unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_are_calls() {
        let geocities = style_script(RetroStyle::GeoCities);
        assert!(geocities.starts_with("(function(style){"));
        assert!(geocities.ends_with(r#"})("geocities");"#));
        for (look, _) in LOOKS {
            assert!(geocities.contains(&format!("R.define(\"{}\",", look.key())));
        }
        assert!(style_script(RetroStyle::Off).contains(r#"apply("off")"#));
        let splash = splash_script("it's \"here\"");
        assert!(splash.starts_with("(// "));
        assert!(splash.ends_with(r#")("it's \"here\"");"#));
    }

    #[test]
    fn parses_menu_anchor() {
        let p = parse_point("612.5,56");
        assert_eq!((p.x, p.y), (612, 56));
        let p = parse_point("junk");
        assert_eq!((p.x, p.y), (0, 0));
    }
}
