//! The top-level window and the three browser view delegates.

use super::state::{Role, with_shell};
use cef::*;
use std::cell::RefCell;

/// Window face gray (ARGB).
pub const FACE_GRAY: u32 = 0xFFC0C0C0;
pub const TOOLBAR_HEIGHT: i32 = 112;
pub const STATUS_HEIGHT: i32 = 24;

wrap_window_delegate! {
    pub struct LanternWindowDelegate {
        views: RefCell<Vec<(Role, BrowserView)>>,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1100, height: 780 }
        }

        fn minimum_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 520, height: TOOLBAR_HEIGHT + STATUS_HEIGHT + 120 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            let Some(window) = window else {
                return;
            };
            window.set_background_color(FACE_GRAY);

            let layout_settings = BoxLayoutSettings {
                horizontal: 0,
                cross_axis_alignment: AxisAlignment::STRETCH,
                ..Default::default()
            };
            let layout = window.set_to_box_layout(Some(&layout_settings));

            for (role, browser_view) in self.views.borrow().iter() {
                if *role != Role::Content {
                    browser_view.set_background_color(FACE_GRAY);
                }
                let mut view = View::from(browser_view);
                window.add_child_view(Some(&mut view));
                if let Some(layout) = layout.as_ref() {
                    let flex = if *role == Role::Content { 1 } else { 0 };
                    layout.set_flex_for_view(Some(&mut view), flex);
                }
            }

            window.set_title(Some(&CefString::from("Lantern")));
            set_window_icons(window);
            with_shell(|shell| shell.window = Some(window.clone()));
            window.show();

            if let Some((_, content)) = self.views.borrow().iter().find(|(r, _)| *r == Role::Content) {
                content.request_focus();
            }
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            self.views.borrow_mut().clear();
            with_shell(|shell| shell.window = None);
        }

        fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            // Ask each browser; CEF re-invokes the close once unload handlers finish.
            let browsers: Vec<Browser> = with_shell(|shell| {
                [Role::Content, Role::Toolbar, Role::Status]
                    .into_iter()
                    .filter_map(|role| shell.browser(role))
                    .collect()
            });
            let mut all = true;
            for browser in browsers {
                let closed = browser.host().map(|h| h.try_close_browser()).unwrap_or(1);
                all &= closed != 0;
            }
            all.into()
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

const ICON_SMALL_PNG: &[u8] = include_bytes!("../../ui/appicon/lantern_32.png");
const ICON_LARGE_PNG: &[u8] = include_bytes!("../../ui/appicon/lantern_256.png");

/// Window and taskbar icons. The macOS/Windows bundles carry their own icons,
/// but on Linux this is the only place the window gets one.
fn set_window_icons(window: &Window) {
    for (png, app) in [(ICON_SMALL_PNG, false), (ICON_LARGE_PNG, true)] {
        let Some(mut image) = image_create() else {
            return;
        };
        if image.add_png(1.0, Some(png)) == 0 {
            continue;
        }
        if app {
            window.set_window_app_icon(Some(&mut image));
        } else {
            window.set_window_icon(Some(&mut image));
        }
    }
}

wrap_browser_view_delegate! {
    pub struct LanternBrowserViewDelegate {
        role: Role,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            match self.role {
                Role::Toolbar => Size { width: 800, height: TOOLBAR_HEIGHT },
                Role::Status => Size { width: 800, height: STATUS_HEIGHT },
                Role::Content => Size { width: 800, height: 600 },
            }
        }

        fn minimum_size(&self, _view: Option<&mut View>) -> Size {
            match self.role {
                Role::Toolbar => Size { width: 0, height: TOOLBAR_HEIGHT },
                Role::Status => Size { width: 0, height: STATUS_HEIGHT },
                Role::Content => Size { width: 0, height: 0 },
            }
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}
