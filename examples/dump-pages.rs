//! Writes Lantern's chrome pages (toolbar, status bar) and internal pages
//! (home, bookmarks, about, settings, error) to `<dir>/*.html`, and the script
//! Lantern runs for each retro look to `<dir>/retro-<look>.js`, so the
//! Playwright suite in tests/web can use them without CEF. Every UI mode's
//! chrome pages go to `toolbar-<mode>.html` and `status-<mode>.html`, with
//! the modes' keys and view heights in `ui-modes.json`, and each mode's page
//! defaults script to `mode-pages-<mode>.js` (`mode-pages-off.js` removes them).
//!
//! Usage: `LD_LIBRARY_PATH="$CEF_PATH" cargo run --example dump-pages -- <dir>`
//!
//! Version numbers are replaced with fixed text, so a release or CEF bump
//! doesn't change the golden screenshots.

#[allow(dead_code)]
#[path = "../src/shell/mod.rs"]
mod shell;

#[cfg(target_os = "macos")]
#[allow(dead_code)]
#[path = "../src/mac/mod.rs"]
mod mac;

use shell::{
    bookmarks::Bookmark,
    pages, retro,
    settings::{RetroStyle, Settings, UiMode},
    ui_mode,
    util::{Json, percent_decode},
};
use std::{fs, path::PathBuf, process::ExitCode};

/// Decode the `data:<mime>;base64,<percent-encoded base64>` URIs pages.rs builds.
fn decode_data_uri(uri: &str) -> Option<Vec<u8>> {
    let (_, payload) = uri.split_once(";base64,")?;
    let payload = percent_decode(payload);
    let mut out = Vec::with_capacity(payload.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0);
    for c in payload.bytes().filter(|&c| c != b'=') {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        acc = acc << 6 | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

fn main() -> ExitCode {
    let Some(dir) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: dump-pages <dir>");
        return ExitCode::FAILURE;
    };
    let bookmarks = [
        Bookmark::new(
            "Lantern on GitHub",
            "https://github.com/chrissnyder2337/thebrowser",
        ),
        Bookmark::new("Example Domain", "https://example.com/"),
        Bookmark::new(
            "<b>Not bold</b> & \"quoted\"",
            "https://example.org/?q=<script>&x=1",
        ),
    ];
    let settings = Settings::default();
    let mut pages = vec![
        ("toolbar".to_owned(), pages::toolbar_uri(UiMode::Lantern)),
        ("status".to_owned(), pages::status_uri(UiMode::Lantern)),
        ("home".to_owned(), pages::home_uri(&bookmarks)),
        ("home-empty".to_owned(), pages::home_uri(&[])),
        ("bookmarks".to_owned(), pages::bookmarks_uri(&bookmarks)),
        (
            "about".to_owned(),
            pages::about_uri("/home/user/.config/lantern"),
        ),
        ("settings".to_owned(), pages::settings_uri(&settings, false)),
        (
            "settings-saved".to_owned(),
            pages::settings_uri(&settings, true),
        ),
        (
            "error".to_owned(),
            pages::error_uri(
                "https://no-such-host.invalid/<x>",
                -105,
                "ERR_NAME_NOT_RESOLVED",
            ),
        ),
    ];
    let mut modes = vec![];
    for mode in UiMode::ALL {
        let key = mode.key();
        pages.push((format!("toolbar-{key}"), pages::toolbar_uri(mode)));
        pages.push((format!("status-{key}"), pages::status_uri(mode)));
        let (toolbar, status) = ui_mode::heights(mode);
        modes.push(Json::Obj(vec![
            ("key", Json::str(key)),
            ("name", Json::str(mode.name())),
            ("toolbarHeight", Json::Num(f64::from(toolbar))),
            ("statusHeight", Json::Num(f64::from(status))),
        ]));
    }
    let (cef, chromium) = pages::cef_versions();
    let mut fixed = vec![
        (env!("CARGO_PKG_VERSION").to_owned(), "X.Y.Z"),
        (cef, "CEF.VERSION"),
        (chromium, "CHROMIUM.VERSION"),
    ];
    // Longest first, so a version that contains another is replaced whole.
    fixed.retain(|(from, _)| !from.is_empty() && from != "unknown");
    fixed.sort_by_key(|(from, _)| std::cmp::Reverse(from.len()));
    if let Err(err) = fs::create_dir_all(&dir) {
        eprintln!("dump-pages: {}: {err}", dir.display());
        return ExitCode::FAILURE;
    }
    for (name, uri) in pages {
        let Some(html) = decode_data_uri(&uri) else {
            eprintln!("dump-pages: {name} is not a base64 data: URI");
            return ExitCode::FAILURE;
        };
        let mut html = String::from_utf8_lossy(&html).into_owned();
        for (from, to) in &fixed {
            html = html.replace(from.as_str(), to);
        }
        let path = dir.join(format!("{name}.html"));
        if let Err(err) = fs::write(&path, html) {
            eprintln!("dump-pages: {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
        println!("{}", path.display());
    }
    let mut scripts = vec![(
        "mode-pages-off.js".to_owned(),
        ui_mode::remove_page_defaults_script(),
    )];
    for mode in UiMode::ALL {
        let settings = Settings {
            ui_mode: mode,
            ..Settings::default()
        };
        if let Some(script) = ui_mode::page_defaults_script(&settings) {
            scripts.push((format!("mode-pages-{}.js", mode.key()), script));
        }
    }
    for (name, script) in scripts {
        let path = dir.join(name);
        if let Err(err) = fs::write(&path, script) {
            eprintln!("dump-pages: {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
        println!("{}", path.display());
    }
    let path = dir.join("ui-modes.json");
    if let Err(err) = fs::write(&path, Json::Arr(modes).render()) {
        eprintln!("dump-pages: {}: {err}", path.display());
        return ExitCode::FAILURE;
    }
    println!("{}", path.display());
    for style in RetroStyle::ALL {
        let path = dir.join(format!("retro-{}.js", style.key()));
        if let Err(err) = fs::write(&path, retro::style_script(style)) {
            eprintln!("dump-pages: {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
        println!("{}", path.display());
    }
    ExitCode::SUCCESS
}
