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

/// How a mode lays out the window (see window.rs): the views' sizes in DIP,
/// where the side view goes, and the border around the page. A size of 0
/// hides that view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The toolbar view, full width at the top.
    pub toolbar: i32,
    /// The status view, full width at the bottom.
    pub status: i32,
    /// The status view's height while the mode's panel is open (the TV
    /// Options panel); 0 when the mode has none.
    pub status_open: i32,
    /// The side view's width, beside the page.
    pub side: i32,
    pub side_left: bool,
    /// The side view runs the window's full height, beside the toolbar and
    /// status views too, instead of only beside the page.
    pub side_full: bool,
    /// The frame view, right above the page and inside the inset.
    pub frame: i32,
    /// Border around the frame and page: top, left, bottom, right.
    pub inset: [i32; 4],
    /// The border's color (ARGB).
    pub inset_color: u32,
    /// A line drawn right around the frame and page, inside the inset (a
    /// sunken page edge): its width and color.
    pub well: i32,
    pub well_color: u32,
}

impl Layout {
    /// A toolbar over the page and a status bar under it.
    const fn bars(toolbar: i32, status: i32) -> Self {
        Self {
            toolbar,
            status,
            status_open: 0,
            side: 0,
            side_left: true,
            side_full: false,
            frame: 0,
            inset: [0; 4],
            inset_color: 0xFFC0C0C0,
            well: 0,
            well_color: 0xFF808080,
        }
    }

    /// The same, with a border of `color` around the page.
    const fn inset(self, inset: [i32; 4], color: u32) -> Self {
        Self {
            inset,
            inset_color: color,
            ..self
        }
    }

    /// The same, with a side view `width` wide on the left or right, beside
    /// the page or (`full`) the whole window.
    const fn side(self, width: i32, left: bool, full: bool) -> Self {
        Self {
            side: width,
            side_left: left,
            side_full: full,
            ..self
        }
    }

    /// The same, with a panel the status view grows to `height` for.
    const fn panel(self, height: i32) -> Self {
        Self {
            status_open: height,
            ..self
        }
    }

    /// The same, with a frame view `height` tall right above the page.
    const fn frame(self, height: i32) -> Self {
        Self {
            frame: height,
            ..self
        }
    }

    /// The same, with a `width` line of `color` right around the page.
    const fn well(self, width: i32, color: u32) -> Self {
        Self {
            well: width,
            well_color: color,
            ..self
        }
    }

    /// The status view's height, given whether the panel is open.
    pub fn status_height(&self, panel_open: bool) -> i32 {
        if panel_open && self.status_open > 0 {
            self.status_open
        } else {
            self.status
        }
    }
}

thread_local! {
    // Kept out of the shell state: CEF asks for view sizes during layout,
    // which can happen while the shell is borrowed.
    static CURRENT: Cell<(Layout, bool)> = const { Cell::new((Layout::bars(112, 24), false)) };
}

/// How `mode` lays out the window.
pub fn layout(mode: UiMode) -> Layout {
    match mode {
        UiMode::Lantern => Layout::bars(112, 24),
        UiMode::Workstation1993 => Layout::bars(106, 98)
            .inset([4, 6, 6, 6], 0xFFBFBFBF)
            .well(2, 0xFF676767),
        UiMode::Toolbar1996 => Layout::bars(118, 20),
        UiMode::Desktop1996 => Layout::bars(91, 20),
        UiMode::Applet1997 => Layout::bars(99, 22),
        UiMode::Terminal1999 => Layout::bars(40, 40),
        UiMode::Terminal1992 => Layout::bars(20, 80),
        UiMode::Pc1993 => Layout::bars(54, 22),
        UiMode::Toolbar1995 => Layout::bars(114, 22),
        UiMode::Desktop1995 => Layout::bars(80, 22).inset([0, 2, 2, 2], 0xFFC0C0C0),
        UiMode::Suite1997 => Layout::bars(127, 24),
        UiMode::Desktop1997 => Layout::bars(102, 22).well(2, 0xFF808080),
        UiMode::Desktop1999 => Layout::bars(100, 22),
        UiMode::Classic1992 => Layout::bars(20, 20)
            .frame(20)
            .inset([8, 8, 8, 8], 0xFF808080)
            .well(1, 0xFF000000),
        UiMode::Classic1994 => Layout::bars(20, 20)
            .frame(64)
            .inset([6, 10, 0, 10], 0xFF848484)
            .well(1, 0xFF000000),
        UiMode::Tv1996 => Layout::bars(0, 34)
            .panel(170)
            .inset([12, 18, 0, 18], 0xFF202020),
        UiMode::Dos1996 => Layout::bars(46, 20).side(150, false, true),
        UiMode::Cube1990 => Layout::bars(0, 22)
            .side(214, true, true)
            .frame(23)
            .inset([10, 10, 10, 10], 0xFF555555)
            .well(1, 0xFF000000),
        UiMode::Tv1997 => Layout::bars(0, 34)
            .panel(208)
            .inset([12, 18, 0, 18], 0xFF202020),
        UiMode::Workspace1997 => Layout::bars(48, 20)
            .side(210, true, false)
            .frame(46)
            .inset([4, 4, 4, 4], 0xFF808080)
            .well(2, 0xFFC0C0C0),
        UiMode::Workstation1996 => Layout::bars(100, 30)
            .inset([4, 6, 6, 6], 0xFFBFBFBF)
            .well(2, 0xFF676767),
        UiMode::WorkstationPc1995 => Layout::bars(86, 24),
        UiMode::Applet1995 => Layout::bars(62, 82),
        UiMode::Classic1999 => Layout::bars(20, 20)
            .frame(140)
            .inset([6, 10, 0, 10], 0xFF6666CC)
            .well(1, 0xFF000000),
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
            "#c0c0c0",
        )),
        UiMode::Applet1997 => Some((
            include_str!("../../ui/chrome/applet-1997/pages.css"),
            "#ffffff",
        )),
        UiMode::Terminal1999 => Some((
            include_str!("../../ui/chrome/terminal-1999/pages.css"),
            "#000000",
        )),
        UiMode::Terminal1992 => Some((
            include_str!("../../ui/chrome/terminal-1992/pages.css"),
            "#000000",
        )),
        UiMode::Pc1993 => Some((include_str!("../../ui/chrome/pc-1993/pages.css"), "#ffffe8")),
        UiMode::Toolbar1995 => Some((
            include_str!("../../ui/chrome/toolbar-1995/pages.css"),
            "#c0c0c0",
        )),
        UiMode::Desktop1995 => Some((
            include_str!("../../ui/chrome/desktop-1995/pages.css"),
            "#c0c0c0",
        )),
        UiMode::Suite1997 => Some((
            include_str!("../../ui/chrome/suite-1997/pages.css"),
            "#c0c0c0",
        )),
        UiMode::Desktop1997 => Some((
            include_str!("../../ui/chrome/desktop-1997/pages.css"),
            "#ffffff",
        )),
        UiMode::Desktop1999 => Some((
            include_str!("../../ui/chrome/desktop-1999/pages.css"),
            "#ffffff",
        )),
        UiMode::Classic1992 => Some((
            include_str!("../../ui/chrome/classic-1992/pages.css"),
            "#ffffff",
        )),
        UiMode::Classic1994 => Some((
            include_str!("../../ui/chrome/classic-1994/pages.css"),
            "#ffffff",
        )),
        UiMode::Tv1996 => Some((include_str!("../../ui/chrome/tv-1996/pages.css"), "#c8c8d8")),
        UiMode::Dos1996 => Some((
            include_str!("../../ui/chrome/dos-1996/pages.css"),
            "#ffffff",
        )),
        UiMode::Cube1990 => Some((
            include_str!("../../ui/chrome/cube-1990/pages.css"),
            "#ffffff",
        )),
        UiMode::Tv1997 => Some((include_str!("../../ui/chrome/tv-1997/pages.css"), "#c8c8d8")),
        UiMode::Workspace1997 => Some((
            include_str!("../../ui/chrome/workspace-1997/pages.css"),
            "#ffffff",
        )),
        UiMode::Workstation1996 => Some((
            include_str!("../../ui/chrome/workstation-1996/pages.css"),
            "#bfbfbf",
        )),
        UiMode::WorkstationPc1995 => Some((
            include_str!("../../ui/chrome/workstation-pc-1995/pages.css"),
            "#c0c0c0",
        )),
        UiMode::Applet1995 => Some((
            include_str!("../../ui/chrome/applet-1995/pages.css"),
            "#c0c0c0",
        )),
        UiMode::Classic1999 => Some((
            include_str!("../../ui/chrome/classic-1999/pages.css"),
            "#ffffff",
        )),
    }
}

const SCROLLBARS_JS: &str = include_str!("../../ui/chrome/scrollbars.js");

/// The scrollbars a mode's browser drew (`ui/chrome/scrollbars/<style>.css`):
/// its era's toolkit, or none for the text terminals and TV boxes. Lantern
/// keeps the system's.
pub fn scrollbars(mode: UiMode) -> Option<(&'static str, &'static str)> {
    macro_rules! style {
        ($name:literal) => {
            Some((
                $name,
                include_str!(concat!("../../ui/chrome/scrollbars/", $name, ".css")),
            ))
        };
    }
    match mode {
        UiMode::Lantern => None,
        UiMode::Workstation1993
        | UiMode::Workstation1996
        | UiMode::Applet1995
        | UiMode::Suite1997 => style!("motif"),
        UiMode::WorkstationPc1995
        | UiMode::Desktop1995
        | UiMode::Desktop1996
        | UiMode::Desktop1997
        | UiMode::Desktop1999
        | UiMode::Toolbar1995
        | UiMode::Toolbar1996
        | UiMode::Applet1997
        | UiMode::Workspace1997 => style!("win95"),
        UiMode::Pc1993 => style!("win31"),
        UiMode::Classic1992 => style!("mac1bit"),
        UiMode::Classic1994 => style!("system7"),
        UiMode::Classic1999 => style!("platinum"),
        UiMode::Cube1990 => style!("next"),
        UiMode::Dos1996 => style!("dos"),
        UiMode::Terminal1992 | UiMode::Terminal1999 | UiMode::Tv1996 | UiMode::Tv1997 => {
            style!("none")
        }
    }
}

/// Script that gives a content frame's page the mode's scrollbars, or takes
/// them out (an empty sheet).
pub fn scrollbars_script(mode: UiMode) -> String {
    let css = scrollbars(mode).map_or("", |(_, css)| css);
    format!("{}({});", SCROLLBARS_JS.trim_end(), js_string(css))
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

/// The layout the window is in now, and whether the mode's panel is open.
pub fn current() -> (Layout, bool) {
    CURRENT.with(Cell::get)
}

/// Size the views for `mode` from the next layout on.
pub fn use_layout(mode: UiMode, panel_open: bool) {
    CURRENT.with(|cell| cell.set((layout(mode), panel_open)));
}

/// Open or close the mode's panel (the `panel` command).
pub fn toggle_panel() {
    let (layout, open) = current();
    if layout.status_open == 0 {
        return;
    }
    with_shell(|shell| shell.panel_open = !open);
    relayout();
    super::state::push_status();
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
    let mode = with_shell(|shell| {
        shell.panel_open = false;
        shell.settings.ui_mode
    });
    for role in Role::CHROME {
        let Some(frame) = with_shell(|shell| shell.browser(role)).and_then(|b| b.main_frame())
        else {
            continue;
        };
        // Not ready until the new page says so; pushes wait until then.
        with_shell(|shell| shell.set_ready(role, false));
        frame.load_url(Some(&CefString::from(
            pages::chrome_uri(role, mode).as_str(),
        )));
    }
    relayout();
}

/// Lay the window out for the current mode: view sizes and visibility, the
/// side view's position, and the inset around the page.
pub fn relayout() {
    let (mode, panel_open, window, middle, main, inset, well, browsers) = with_shell(|shell| {
        let browsers: Vec<(Role, Browser)> = Role::ALL
            .into_iter()
            .filter_map(|role| shell.browser(role).map(|b| (role, b)))
            .collect();
        (
            shell.settings.ui_mode,
            shell.panel_open,
            shell.window.clone(),
            shell.middle.clone(),
            shell.main.clone(),
            shell.inset.clone(),
            shell.well.clone(),
            browsers,
        )
    });
    use_layout(mode, panel_open);
    let layout = layout(mode);
    let mut side_view = None;
    let mut content_view = None;
    for (role, mut browser) in browsers {
        let Some(view) = browser_view_get_for_browser(Some(&mut browser)) else {
            continue;
        };
        let size = match role {
            Role::Toolbar => layout.toolbar,
            Role::Status => layout.status_height(panel_open),
            Role::Side => layout.side,
            Role::Frame => layout.frame,
            Role::Content => 1,
        };
        view.set_visible((size > 0).into());
        // The views' delegates size themselves from the new layout.
        view.invalidate_layout();
        match role {
            Role::Side => side_view = Some(View::from(&view)),
            Role::Content => content_view = Some(View::from(&view)),
            _ => {}
        }
    }
    if let (Some(window), Some(main), Some(middle), Some(side)) = (&window, main, middle, side_view)
    {
        super::window::place_side(window, &main, &middle, side, &layout);
    }
    if let (Some(inset), Some(well)) = (inset, well) {
        super::window::lay_out_inset(&inset, &well, &layout, content_view);
    }
    if let Some(window) = window {
        window.layout();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_but_lantern_has_its_scrollbars() {
        for mode in UiMode::ALL {
            assert_eq!(
                scrollbars(mode).is_some(),
                mode != UiMode::Lantern,
                "{mode:?}"
            );
            if let Some((_, css)) = scrollbars(mode) {
                assert!(css.contains("::-webkit-scrollbar"), "{mode:?}");
            }
        }
        // Lantern's script takes the sheet back out.
        assert!(scrollbars_script(UiMode::Lantern).ends_with(r#"("");"#));
    }

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
            // A view is hidden or at least one text line tall; the status
            // view always shows (the status text and meter).
            let layout = layout(mode);
            for size in [layout.toolbar, layout.side, layout.frame] {
                assert!(size == 0 || size >= 20, "{mode:?}");
            }
            assert!(layout.status >= 20, "{mode:?}");
        }
        assert_eq!(layout(UiMode::Lantern), Layout::bars(112, 24));
    }
}
