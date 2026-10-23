//! Builds the `data:text/html` URIs for the chrome pages (toolbar, status bar)
//! and the internal pages shown in the content view.

use super::{
    art::{with_icons, with_lantern},
    bookmarks::Bookmark,
    settings::{DialupMode, ModemSpeed, RetroStyle, Settings},
    util::{encode_component, get_data_uri, html_escape, truncate},
};
use std::fmt::Write;

pub const HOME: &str = "lantern:home";
pub const BOOKMARKS: &str = "lantern:bookmarks";
pub const ABOUT: &str = "lantern:about";
pub const SETTINGS: &str = "lantern:settings";

/// Host whose URLs internal pages use as "links back into Rust".
pub const ACTION_HOST: &str = "lantern.invalid";

const TOOLBAR_HTML: &str = include_str!("../../ui/toolbar.html");
const STATUS_HTML: &str = include_str!("../../ui/status.html");
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

pub fn toolbar_uri() -> String {
    let mut vars = vec![];
    with_icons(|sheet| {
        for name in [
            "back",
            "forward",
            "reload",
            "home",
            "search",
            "bookmarks",
            "mark",
            "stop",
            "retro",
        ] {
            let key = format!("ICON_{}", name.to_ascii_uppercase());
            let html = format!(
                "{}{}",
                sheet.svg(name, 1, "on"),
                sheet.embossed_svg(name, 1, "off")
            );
            vars.push((key, html));
        }
        vars.push(("ICON_LOCK".to_owned(), sheet.svg("lock", 1, "lock")));
        vars.push(("ICON_PAGE".to_owned(), sheet.svg("page", 1, "pg")));
    });
    with_lantern(|sheet| {
        vars.push(("LANTERN_IDLE".to_owned(), sheet.svg("idle", 2, "idle")));
        vars.push((
            "LANTERN_LIT".to_owned(),
            sheet.strip_svg(&["lit0", "lit1", "lit2", "lit3"], 2, "lit"),
        ));
    });
    let vars: Vec<(&str, String)> = vars.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    html_uri(&fill(TOOLBAR_HTML, &vars))
}

pub fn status_uri() -> String {
    let (lock, modem) = with_icons(|sheet| {
        (
            sheet.svg("lock", 1, "lock"),
            format!(
                "{}{}",
                sheet.svg("modem", 1, "on"),
                sheet.embossed_svg("modem", 1, "off")
            ),
        )
    });
    html_uri(&fill(
        STATUS_HTML,
        &[("ICON_LOCK", lock), ("ICON_MODEM", modem)],
    ))
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
        RetroStyle::Netscape => {
            "Switches every site's style sheets off, so pages show as plain HTML: \
             gray background, Times, blue and purple links. With a hit counter, of course."
        }
        RetroStyle::Portal => {
            "Squeezes pages into an 800x600-friendly column of small Arial with \
             colored heading bars, a channel bar, a search box and a blinking banner ad."
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
    let saved = if saved {
        r#"<p class="saved sunken">Settings saved.</p>"#.to_owned()
    } else {
        String::new()
    };
    let mut vars = common_vars();
    vars.push(("MODE_OPTIONS", modes));
    vars.push(("SPEED_OPTIONS", speeds));
    vars.push(("RETRO_OPTIONS", retro));
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
    fn versions_parse() {
        let (cef, chromium) = cef_versions();
        assert!(!cef.is_empty());
        assert!(chromium.chars().next().unwrap().is_ascii_digit());
    }
}
