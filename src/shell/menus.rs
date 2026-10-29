//! Native menus opened from the chrome pages: the Retro menu, the Browser UI
//! menu and the menu bars some UI modes have. Native, because an HTML menu
//! would be clipped to its view. A page only names a menu; the items live
//! here, so the title channel never carries labels or actions.

use super::{
    commands, retro,
    settings::{RetroStyle, Settings, UiMode},
    state::{Role, defer, with_shell},
    ui_mode,
};
use cef::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    Retro,
    /// Just the Browser UI items (the keyboard shortcut).
    BrowserUi,
    File,
    Navigate,
    Options,
    Help,
}

impl Menu {
    /// Menus a page may open with the `menu` command.
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "file" => Some(Self::File),
            "navigate" => Some(Self::Navigate),
            "options" => Some(Self::Options),
            "help" => Some(Self::Help),
            _ => None,
        }
    }
}

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
    Style(RetroStyle),
    Mode(UiMode),
}

const PLAIN: [Item; 12] = [
    Item::Back,
    Item::Forward,
    Item::Reload,
    Item::Stop,
    Item::Home,
    Item::Search,
    Item::Bookmarks,
    Item::AddBookmark,
    Item::OpenUrl,
    Item::Settings,
    Item::About,
    Item::Splash,
];
const STYLE_BASE: i32 = 100;
const MODE_BASE: i32 = 300;
// Submenus need ids too; they never reach execute_command.
const RETRO_SUBMENU: i32 = 900;
const UI_SUBMENU: i32 = 901;

impl Item {
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
            _ => 1 + PLAIN.iter().position(|item| *item == self).unwrap_or(0) as i32,
        }
    }

    fn from_id(id: i32) -> Option<Self> {
        let index = |base: i32| usize::try_from(id - base).ok();
        if id >= MODE_BASE {
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
        Item::Style(style) => retro::set_style(style),
        Item::Mode(mode) => ui_mode::set_mode(mode),
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
        model.set_checked(id, (ctx.settings.retro_style == style).into());
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
        Menu::File => {
            add(model, Item::OpenUrl, "Open URL\u{2026}");
            add(model, Item::AddBookmark, "Add Bookmark");
            model.add_separator();
            add(model, Item::Settings, "Settings\u{2026}");
        }
        Menu::Navigate => {
            add(model, Item::Back, "Back");
            add(model, Item::Forward, "Forward");
            add(model, Item::Reload, "Reload");
            add(model, Item::Stop, "Stop");
            add(model, Item::Home, "Home");
            model.add_separator();
            add(model, Item::Search, "Search");
            add(model, Item::Bookmarks, "Bookmarks");
            model.set_enabled(Item::Back.id(), ctx.can_back.into());
            model.set_enabled(Item::Forward.id(), ctx.can_fwd.into());
            model.set_enabled(Item::Stop.id(), ctx.loading.into());
        }
        Menu::Options => {
            if let Some(sub) =
                model.add_sub_menu(RETRO_SUBMENU, Some(&CefString::from("Retro looks")))
            {
                add_styles(&sub, ctx);
            }
            add_modes_submenu(model, ctx);
            model.add_separator();
            add(model, Item::Settings, "Settings\u{2026}");
        }
        Menu::Help => add(model, Item::About, "About Lantern"),
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
pub fn show(role: Role, arg: &str) {
    let (key, anchor) = arg.split_once(',').unwrap_or((arg, ""));
    match Menu::from_key(key) {
        Some(menu) => show_at(menu, Some(role), retro::parse_point(anchor)),
        None => eprintln!("lantern: unknown menu {key:?}"),
    }
}

/// The Retro button's menu: `arg` is "x,y" in the toolbar.
pub fn show_retro(arg: &str) {
    show_at(Menu::Retro, Some(Role::Toolbar), retro::parse_point(arg));
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
        assert_eq!(Menu::from_key("file"), Some(Menu::File));
        assert_eq!(Menu::from_key("options"), Some(Menu::Options));
        for key in ["", "retro", "File", "browser-ui", "file,1,2"] {
            assert_eq!(Menu::from_key(key), None, "{key}");
        }
    }
}
