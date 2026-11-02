//! `--self-test=<file>`: once the first page in the content view has loaded
//! (or failed to), write its URL, title and load status to `<file>` and quit.
//! `tests/smoke.sh` uses it to check that a bundle starts and loads a page,
//! once per retro look and once per UI mode.

use super::state::{close_all_browsers, defer, with_shell};
use std::{cell::RefCell, path::PathBuf};

thread_local! {
    static REPORT_TO: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

pub fn enable(path: PathBuf) {
    REPORT_TO.with(|target| *target.borrow_mut() = Some(path));
}

/// The content view's main frame finished loading with this HTTP status.
pub fn on_load_end(http_status: i32) {
    finish(format!("ok {http_status}"));
}

/// The content view's main frame failed to load.
pub fn on_load_error(code: i32, text: &str) {
    finish(format!("error {code} {text}"));
}

fn finish(status: String) {
    let Some(path) = REPORT_TO.with(|target| target.borrow_mut().take()) else {
        return;
    };
    // Let the title and the load's own callbacks land first.
    defer(move || {
        let (url, title, retro, ui) = with_shell(|shell| {
            (
                shell.current_url.clone(),
                shell.current_title.clone(),
                shell.settings.page_look().key(),
                shell.settings.ui_mode.key(),
            )
        });
        let text = format!("{}ui={ui}\n", report(&url, &title, &status, retro));
        if let Err(err) = std::fs::write(&path, text) {
            eprintln!("lantern: could not write {}: {err}", path.display());
        }
        close_all_browsers(true);
    });
}

/// `key=value` lines, one per field; line breaks in values become spaces.
fn report(url: &str, title: &str, status: &str, retro: &str) -> String {
    let one_line = |s: &str| s.replace(['\r', '\n'], " ");
    format!(
        "url={}\ntitle={}\nstatus={}\nretro={}\n",
        one_line(url),
        one_line(title),
        one_line(status),
        one_line(retro)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// tests/smoke.sh runs every UI mode and every look.
    #[test]
    fn smoke_test_covers_every_mode_and_look() {
        use crate::shell::settings::{RetroStyle, Settings, UiMode};
        let smoke = include_str!("../../tests/smoke.sh");
        for mode in UiMode::ALL {
            let look = Settings {
                ui_mode: mode,
                ..Settings::default()
            }
            .page_look();
            let entry = format!("{}:{}", mode.key(), look.key());
            assert!(smoke.contains(&entry), "tests/smoke.sh lacks {entry}");
        }
        let looks = smoke
            .lines()
            .find(|l| l.starts_with("for look in "))
            .unwrap_or("");
        for look in RetroStyle::ALL {
            assert!(
                looks.contains(look.key()),
                "tests/smoke.sh lacks look {look:?}"
            );
        }
    }

    #[test]
    fn report_is_one_field_per_line() {
        assert_eq!(
            report("http://127.0.0.1:1/a", "A\nB", "ok 200", "portal"),
            "url=http://127.0.0.1:1/a\ntitle=A B\nstatus=ok 200\nretro=portal\n"
        );
    }
}
