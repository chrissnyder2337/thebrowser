//! Native menus opened from the chrome pages: the Retro menu, the Browser UI
//! menu and the menu bars some UI modes have. Native, because an HTML menu
//! would be clipped to its view. A page only names a menu; the items live
//! here, so the title channel never carries labels or actions.

use super::{
    commands, retro,
    settings::{ModePages, RetroStyle, Settings, UiMode},
    state::{Role, defer, with_shell},
    ui_mode,
};
use cef::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    Retro,
    /// Just the Browser UI items (the keyboard shortcut).
    BrowserUi,
    /// One of the current UI mode's menu-bar menus (`ui/chrome/<mode>/menus.txt`).
    Bar(UiMode, usize),
}

/// One entry of a mode's menu-bar menu.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Entry {
    Item(Item, String),
    /// A label the original had with no matching feature: shown, disabled.
    Inert(String),
    Separator,
    RetroLooks(String),
    BrowserUi(String),
}

/// A menu-bar menu: the key a page names it by, and its entries.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BarMenu {
    key: String,
    entries: Vec<Entry>,
}

/// Parse a mode's `menus.txt`:
///
/// ```text
/// menu File            a menu; pages name it by its lowercase label, spaces
///                      as dashes ("file")
/// back Back            an action, then its label
/// - Open Local...      a label with no matching feature (disabled)
/// ---                  a separator
/// retro Retro looks    the retro looks submenu
/// browser-ui Browser UI  the Browser UI submenu
/// ```
///
/// Blank lines and lines starting with `#` are skipped. Unknown actions are an
/// error, so a typo fails the unit test instead of shipping a dead item.
fn parse_bar(text: &str) -> Result<Vec<BarMenu>, String> {
    let mut menus: Vec<BarMenu> = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (word, label) = line.split_once(' ').unwrap_or((line, ""));
        let label = label.trim().to_owned();
        if word == "menu" {
            let key = label.to_ascii_lowercase().replace(' ', "-");
            menus.push(BarMenu {
                key,
                entries: Vec::new(),
            });
            continue;
        }
        let Some(menu) = menus.last_mut() else {
            return Err(format!("{line:?} comes before any menu"));
        };
        let entry = match word {
            "---" => Entry::Separator,
            "-" => Entry::Inert(label),
            "retro" => Entry::RetroLooks(label),
            "browser-ui" => Entry::BrowserUi(label),
            action => match Item::from_action(action) {
                Some(item) => Entry::Item(item, label),
                None => return Err(format!("unknown menu action {action:?}")),
            },
        };
        menu.entries.push(entry);
    }
    Ok(menus)
}

/// The menus.txt of each mode that has a menu bar of its own.
fn bar_text(mode: UiMode) -> Option<&'static str> {
    super::pages::menus_text(mode)
}

/// A mode's menu bar; modes without one get the generic File, Navigate,
/// Options and Help menus.
fn bar(mode: UiMode) -> Vec<BarMenu> {
    bar_text(mode)
        .and_then(|text| parse_bar(text).ok())
        .unwrap_or_else(|| parse_bar(GENERIC_MENUS).unwrap_or_default())
}

const GENERIC_MENUS: &str = "\
menu File
open-url Open URL\u{2026}
add-bookmark Add Bookmark
---
settings Settings\u{2026}
menu Navigate
back Back
forward Forward
reload Reload
stop Stop
home Home
---
search Search
bookmarks Bookmarks
menu Options
retro Retro looks
browser-ui Browser UI
---
settings Settings\u{2026}
menu Help
about About Lantern
";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Item {
    Back,
    Forward,
    Reload,
    Stop,
    Home,
    Search,
    Bookmarks,
    AddBookmark,
    OpenUrl,
    Settings,
    About,
    Splash,
    Print,
    ViewSource,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Fullscreen,
    CloseWindow,
    Style(RetroStyle),
    Mode(UiMode),
    Pages(ModePages),
}

/// Actions a menus.txt can name, and the items they run.
const ACTIONS: [(&str, Item); 19] = [
    ("back", Item::Back),
    ("forward", Item::Forward),
    ("reload", Item::Reload),
    ("stop", Item::Stop),
    ("home", Item::Home),
    ("search", Item::Search),
    ("bookmarks", Item::Bookmarks),
    ("add-bookmark", Item::AddBookmark),
    ("open-url", Item::OpenUrl),
    ("settings", Item::Settings),
    ("about", Item::About),
    ("splash", Item::Splash),
    ("print", Item::Print),
    ("view-source", Item::ViewSource),
    ("zoom-in", Item::ZoomIn),
    ("zoom-out", Item::ZoomOut),
    ("zoom-reset", Item::ZoomReset),
    ("fullscreen", Item::Fullscreen),
    ("close-window", Item::CloseWindow),
];

const PLAIN: [Item; ACTIONS.len()] = {
    let mut items = [Item::Back; ACTIONS.len()];
    let mut i = 0;
    while i < ACTIONS.len() {
        items[i] = ACTIONS[i].1;
        i += 1;
    }
    items
};
const STYLE_BASE: i32 = 100;
const MODE_BASE: i32 = 300;
const PAGES_BASE: i32 = 400;
// Inert items get ids from here; they're disabled, so they never run.
const INERT_BASE: i32 = 1000;
// Submenus need ids too; they never reach execute_command.
const RETRO_SUBMENU: i32 = 900;
const UI_SUBMENU: i32 = 901;

impl Item {
    fn from_action(action: &str) -> Option<Self> {
        ACTIONS
            .iter()
            .find(|(name, _)| *name == action)
            .map(|(_, item)| *item)
    }

    fn id(self) -> i32 {
        match self {
            Self::Style(style) => {
                STYLE_BASE
                    + RetroStyle::ALL
                        .iter()
                        .position(|s| *s == style)
                        .unwrap_or(0) as i32
            }
            Self::Mode(mode) => {
                MODE_BASE + UiMode::ALL.iter().position(|m| *m == mode).unwrap_or(0) as i32
            }
            Self::Pages(pages) => {
                PAGES_BASE + ModePages::ALL.iter().position(|p| *p == pages).unwrap_or(0) as i32
            }
            _ => 1 + PLAIN.iter().position(|item| *item == self).unwrap_or(0) as i32,
        }
    }

    fn from_id(id: i32) -> Option<Self> {
        let index = |base: i32| usize::try_from(id - base).ok();
        if id >= INERT_BASE {
            None
        } else if id >= PAGES_BASE {
            index(PAGES_BASE)
                .and_then(|i| ModePages::ALL.get(i))
                .map(|p| Self::Pages(*p))
        } else if id >= MODE_BASE {
            index(MODE_BASE)
                .and_then(|i| UiMode::ALL.get(i))
                .map(|m| Self::Mode(*m))
        } else if id >= STYLE_BASE {
            index(STYLE_BASE)
                .and_then(|i| RetroStyle::ALL.get(i))
                .map(|s| Self::Style(*s))
        } else {
            index(1).and_then(|i| PLAIN.get(i)).copied()
        }
    }
}

fn run(item: Item) {
    match item {
        Item::Back => commands::go_back(),
        Item::Forward => commands::go_forward(),
        Item::Reload => commands::reload(),
        Item::Stop => commands::stop(),
        Item::Home => commands::show_home(),
        Item::Search => commands::show_search(),
        Item::Bookmarks => commands::show_bookmarks(),
        Item::AddBookmark => commands::add_bookmark(),
        Item::OpenUrl => commands::focus_url_bar(),
        Item::Settings => commands::show_settings(false),
        Item::About => commands::show_about(),
        Item::Splash => retro::set_splash(!with_shell(|shell| shell.settings.splash_intro)),
        Item::Print => commands::print(),
        Item::ViewSource => commands::view_source(),
        Item::ZoomIn => commands::zoom(0.5),
        Item::ZoomOut => commands::zoom(-0.5),
        Item::ZoomReset => commands::zoom(0.0),
        Item::Fullscreen => commands::toggle_fullscreen(),
        Item::CloseWindow => super::state::close_all_browsers(false),
        Item::Style(style) => retro::set_style(style),
        Item::Mode(mode) => ui_mode::set_mode(mode),
        Item::Pages(pages) => ui_mode::set_pages(pages),
    }
}

wrap_menu_model_delegate! {
    struct ChromeMenuDelegate;

    impl MenuModelDelegate {
        fn execute_command(
            &self,
            _menu_model: Option<&mut MenuModel>,
            command_id: ::std::os::raw::c_int,
            _event_flags: EventFlags,
        ) {
            // Run after the menu has closed, outside its callback.
            if let Some(item) = Item::from_id(command_id) {
                defer(move || run(item));
            }
        }
    }
}

/// What the menus need to know about the browser.
struct Context {
    settings: Settings,
    can_back: bool,
    can_fwd: bool,
    loading: bool,
}

fn add(model: &MenuModel, item: Item, label: &str) {
    model.add_item(item.id(), Some(&CefString::from(label)));
}

fn add_styles(model: &MenuModel, ctx: &Context) {
    for style in RetroStyle::ALL {
        let id = Item::Style(style).id();
        model.add_radio_item(id, Some(&CefString::from(style.name())), 1);
        model.set_checked(id, (ctx.settings.page_look() == style).into());
    }
    model.add_separator();
    model.add_check_item(
        Item::Splash.id(),
        Some(&CefString::from("Flash-style splash intro")),
    );
    model.set_checked(Item::Splash.id(), ctx.settings.splash_intro.into());
}

fn add_modes(model: &MenuModel, ctx: &Context) {
    for mode in UiMode::ALL {
        let id = Item::Mode(mode).id();
        model.add_radio_item(id, Some(&CefString::from(mode.name())), 2);
        model.set_checked(id, (ctx.settings.ui_mode == mode).into());
    }
    model.add_separator();
    for pages in ModePages::ALL {
        let id = Item::Pages(pages).id();
        model.add_radio_item(id, Some(&CefString::from(pages.name())), 3);
        model.set_checked(id, (ctx.settings.mode_pages == pages).into());
    }
}

fn add_modes_submenu(model: &MenuModel, ctx: &Context) {
    if let Some(sub) = model.add_sub_menu(UI_SUBMENU, Some(&CefString::from("Browser UI"))) {
        add_modes(&sub, ctx);
    }
}

fn build(menu: Menu, model: &MenuModel, ctx: &Context) {
    match menu {
        Menu::Retro => {
            add_styles(model, ctx);
            model.add_separator();
            add_modes_submenu(model, ctx);
            model.add_separator();
            add(model, Item::Settings, "Retro settings\u{2026}");
        }
        Menu::BrowserUi => add_modes(model, ctx),
        Menu::Bar(mode, index) => {
            if let Some(menu) = bar(mode).get(index) {
                add_entries(model, &menu.entries, ctx);
            }
        }
    }
}

fn add_entries(model: &MenuModel, entries: &[Entry], ctx: &Context) {
    let mut inert = INERT_BASE;
    for entry in entries {
        match entry {
            Entry::Item(item, label) => {
                add(model, *item, label);
                let enabled = match item {
                    Item::Back => ctx.can_back,
                    Item::Forward => ctx.can_fwd,
                    Item::Stop => ctx.loading,
                    _ => true,
                };
                model.set_enabled(item.id(), enabled.into());
            }
            Entry::Inert(label) => {
                model.add_item(inert, Some(&CefString::from(label.as_str())));
                model.set_enabled(inert, 0);
                inert += 1;
            }
            Entry::Separator => {
                model.add_separator();
            }
            Entry::RetroLooks(label) => {
                if let Some(sub) =
                    model.add_sub_menu(RETRO_SUBMENU, Some(&CefString::from(label.as_str())))
                {
                    add_styles(&sub, ctx);
                }
            }
            Entry::BrowserUi(label) => {
                if let Some(sub) =
                    model.add_sub_menu(UI_SUBMENU, Some(&CefString::from(label.as_str())))
                {
                    add_modes(&sub, ctx);
                }
            }
        }
    }
}

/// Show `menu` at `point`, in `role`'s view coordinates (or the window's).
fn show_at(menu: Menu, role: Option<Role>, mut point: Point) {
    let (window, view_browser, ctx) = with_shell(|shell| {
        let ctx = Context {
            settings: shell.settings,
            can_back: shell.can_back,
            can_fwd: shell.can_fwd,
            loading: shell.is_loading,
        };
        (
            shell.window.clone(),
            role.and_then(|r| shell.browser(r)),
            ctx,
        )
    });
    // A view-anchored menu needs its view (it may be gone while closing).
    let Some(window) = window.filter(|_| role.is_none() || view_browser.is_some()) else {
        return;
    };
    let mut delegate = ChromeMenuDelegate::new();
    let Some(mut model) = menu_model_create(Some(&mut delegate)) else {
        return;
    };
    build(menu, &model, &ctx);
    match view_browser {
        Some(mut browser) => {
            if let Some(view) = browser_view_get_for_browser(Some(&mut browser)) {
                view.convert_point_to_screen(Some(&mut point));
            }
        }
        None => {
            window.convert_point_to_screen(Some(&mut point));
        }
    }
    window.show_menu(Some(&mut model), Some(&point), MenuAnchorPosition::TOPLEFT);
}

/// The `menu` command: `arg` is "<menu>,x,y", the anchor in `role`'s view.
/// `<menu>` names one of the current mode's menu-bar menus.
pub fn show(role: Role, arg: &str) {
    let (key, anchor) = arg.split_once(',').unwrap_or((arg, ""));
    let mode = with_shell(|shell| shell.settings.ui_mode);
    match bar(mode).iter().position(|menu| menu.key == key) {
        Some(index) => show_at(
            Menu::Bar(mode, index),
            Some(role),
            retro::parse_point(anchor),
        ),
        None => eprintln!("lantern: unknown menu {key:?}"),
    }
}

/// The keys of a mode's menus that hold the Browser UI submenu.
pub fn browser_ui_keys(mode: UiMode) -> Vec<String> {
    bar(mode)
        .into_iter()
        .filter(|menu| {
            menu.entries
                .iter()
                .any(|e| matches!(e, Entry::BrowserUi(_)))
        })
        .map(|menu| menu.key)
        .collect()
}

/// The menu keys a mode's pages may name (for tests and dump-pages).
pub fn bar_keys(mode: UiMode) -> Vec<String> {
    bar(mode).into_iter().map(|menu| menu.key).collect()
}

/// The Retro button's menu: `arg` is "x,y" in the toolbar.
pub fn show_retro(role: Role, arg: &str) {
    show_at(Menu::Retro, Some(role), retro::parse_point(arg));
}

/// The Browser UI menu at the window's top left (the keyboard shortcut).
pub fn show_browser_ui() {
    show_at(Menu::BrowserUi, None, Point { x: 8, y: 8 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_ids_round_trip() {
        let mut items: Vec<Item> = PLAIN.to_vec();
        items.extend(RetroStyle::ALL.map(Item::Style));
        items.extend(UiMode::ALL.map(Item::Mode));
        items.extend(ModePages::ALL.map(Item::Pages));
        let mut ids: Vec<i32> = items.iter().map(|item| item.id()).collect();
        for (item, id) in items.iter().zip(&ids) {
            assert_eq!(Item::from_id(*id), Some(*item));
            assert!(*id < RETRO_SUBMENU);
        }
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), items.len());
        for id in [0, -1, 99, 299, RETRO_SUBMENU, UI_SUBMENU, 5000] {
            assert_eq!(Item::from_id(id), None, "{id}");
        }
    }

    #[test]
    fn pages_can_only_name_their_menus() {
        let generic = bar_keys(UiMode::Lantern);
        assert_eq!(generic, ["file", "navigate", "options", "help"]);
        for key in ["", "retro", "File", "browser-ui", "file,1,2"] {
            assert!(!generic.iter().any(|k| k == key), "{key}");
        }
    }

    #[test]
    fn menus_files_parse() {
        let menus =
            parse_bar("# c\nmenu Go Places\nback Back\n- Open Local...\n---\nbrowser-ui UI\n")
                .unwrap();
        assert_eq!(menus[0].key, "go-places");
        assert_eq!(
            menus[0].entries,
            [
                Entry::Item(Item::Back, "Back".into()),
                Entry::Inert("Open Local...".into()),
                Entry::Separator,
                Entry::BrowserUi("UI".into()),
            ]
        );
        assert!(parse_bar("back Back\n").is_err());
        assert!(parse_bar("menu File\nlaunch-missiles Go\n").is_err());
        // Every mode's menus.txt parses, and every mode can reach the
        // Browser UI menu from its menus or its Retro button.
        for mode in UiMode::ALL {
            if let Some(text) = bar_text(mode) {
                let menus = parse_bar(text).unwrap_or_else(|e| panic!("{mode:?}: {e}"));
                assert!(!menus.is_empty(), "{mode:?}");
                assert!(
                    menus.iter().all(|m| !m.entries.is_empty()),
                    "{mode:?} has an empty menu"
                );
            }
        }
    }
}
