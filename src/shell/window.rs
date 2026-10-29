//! The top-level window, the panels its views sit in, and the browser view
//! delegates. The window is a column: the toolbar view; a middle row of the
//! side view and an inset panel (the frame view over the page); the status
//! view. A UI mode's `ui_mode::Layout` sizes, shows and hides them.

use super::{
    state::{Role, with_shell},
    ui_mode,
};
use cef::*;
use std::cell::RefCell;

/// Window face gray (ARGB).
pub const FACE_GRAY: u32 = 0xFFC0C0C0;

wrap_window_delegate! {
    pub struct LanternWindowDelegate {
        views: RefCell<Vec<(Role, BrowserView)>>,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1100, height: 780 }
        }

        fn minimum_size(&self, _view: Option<&mut View>) -> Size {
            let (layout, open) = ui_mode::current();
            let height = layout.toolbar + layout.status_height(open) + layout.frame
                + layout.inset[0] + layout.inset[2] + 2 * layout.well + 120;
            let width = 520 + layout.side + layout.inset[1] + layout.inset[3] + 2 * layout.well;
            Size { width, height }
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

            let views = self.views.borrow();
            let view_for = |role: Role| {
                views
                    .iter()
                    .find(|(r, _)| *r == role)
                    .map(|(_, v)| {
                        if role != Role::Content {
                            v.set_background_color(FACE_GRAY);
                        }
                        View::from(v)
                    })
            };
            let (Some(middle), Some(inset), Some(well)) =
                (
                    panel_create(None),
                    panel_create(Some(&mut InsetPanelDelegate::new(false))),
                    panel_create(Some(&mut InsetPanelDelegate::new(true))),
                )
            else {
                return;
            };
            let column = box_layout(window, false);
            let row = box_layout(&middle, true);
            add(window, column.as_ref(), view_for(Role::Toolbar), 0);
            add(window, column.as_ref(), Some(View::from(&middle)), 1);
            add(window, column.as_ref(), view_for(Role::Status), 0);
            add(&middle, row.as_ref(), view_for(Role::Side), 0);
            add(&middle, row.as_ref(), Some(View::from(&inset)), 1);
            inset.add_child_view(Some(&mut View::from(&well)));
            well.add_child_view(view_for(Role::Frame).as_mut());
            well.add_child_view(view_for(Role::Content).as_mut());
            let (layout, _) = ui_mode::current();
            lay_out_inset(&inset, &well, &layout, view_for(Role::Content));
            drop(views);
            with_shell(|shell| {
                shell.middle = Some(middle.clone());
                shell.inset = Some(inset.clone());
                shell.well = Some(well.clone());
            });

            window.set_title(Some(&CefString::from("Lantern")));
            set_window_icons(window);
            with_shell(|shell| shell.window = Some(window.clone()));
            // Hide the views this mode doesn't use and place the side view.
            ui_mode::relayout();
            window.show();

            if let Some((_, content)) = self.views.borrow().iter().find(|(r, _)| *r == Role::Content) {
                content.request_focus();
            }
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            self.views.borrow_mut().clear();
            with_shell(|shell| {
                shell.window = None;
                shell.middle = None;
                shell.inset = None;
                shell.well = None;
            });
        }

        fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            // Ask each browser; CEF re-invokes the close once unload handlers finish.
            let browsers: Vec<Browser> = with_shell(|shell| {
                Role::ALL
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

        // cef-rs defaults these to 0 (CEF's C++ defaults are true), which
        // would leave the window a fixed size.
        fn can_resize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            1
        }

        fn can_maximize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            1
        }

        fn can_minimize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            1
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

// Colors the inset panel (or, with `well`, the well panel) from the current
// layout. Views lose their background color on every theme change, so it's
// set again there.
wrap_panel_delegate! {
    struct InsetPanelDelegate {
        well: bool,
    }

    impl ViewDelegate {
        fn on_theme_changed(&self, view: Option<&mut View>) {
            if let Some(view) = view {
                let (layout, _) = ui_mode::current();
                view.set_background_color(if self.well { layout.well_color } else { layout.inset_color });
            }
        }
    }

    impl PanelDelegate {}
}

/// A view's size in the current layout; `across` is its size along the
/// other axis (the box layouts stretch it).
fn chrome_size(role: Role, across: i32) -> Size {
    let (layout, open) = ui_mode::current();
    match role {
        Role::Toolbar => Size {
            width: across,
            height: layout.toolbar,
        },
        Role::Status => Size {
            width: across,
            height: layout.status_height(open),
        },
        Role::Frame => Size {
            width: across,
            height: layout.frame,
        },
        Role::Side => Size {
            width: layout.side,
            height: across,
        },
        Role::Content => Size {
            width: across,
            height: if across > 0 { 600 } else { 0 },
        },
    }
}

fn box_layout(panel: &impl ImplPanel, horizontal: bool) -> Option<BoxLayout> {
    let settings = BoxLayoutSettings {
        horizontal: horizontal.into(),
        cross_axis_alignment: AxisAlignment::STRETCH,
        ..Default::default()
    };
    panel.set_to_box_layout(Some(&settings))
}

fn add(panel: &impl ImplPanel, layout: Option<&BoxLayout>, view: Option<View>, flex: i32) {
    let Some(mut view) = view else {
        return;
    };
    panel.add_child_view(Some(&mut view));
    if let Some(layout) = layout {
        layout.set_flex_for_view(Some(&mut view), flex);
    }
}

/// Lay out the inset panel for `layout`: a border around the well, and in
/// the well, a line around the frame view over the page.
pub fn lay_out_inset(inset: &Panel, well: &Panel, layout: &ui_mode::Layout, content: Option<View>) {
    let [top, left, bottom, right] = layout.inset;
    let border = |top, left, bottom, right| BoxLayoutSettings {
        horizontal: 0,
        inside_border_insets: Insets {
            top,
            left,
            bottom,
            right,
        },
        cross_axis_alignment: AxisAlignment::STRETCH,
        ..Default::default()
    };
    if let Some(outer) = inset.set_to_box_layout(Some(&border(top, left, bottom, right))) {
        outer.set_flex_for_view(Some(&mut View::from(well)), 1);
    }
    inset.set_background_color(layout.inset_color);
    let w = layout.well;
    if let (Some(inner), Some(mut content)) =
        (well.set_to_box_layout(Some(&border(w, w, w, w))), content)
    {
        inner.set_flex_for_view(Some(&mut content), 1);
    }
    well.set_background_color(layout.well_color);
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
        // Chrome sizes follow the UI mode (ui_mode::relayout).
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            chrome_size(self.role, 800)
        }

        fn minimum_size(&self, _view: Option<&mut View>) -> Size {
            chrome_size(self.role, 0)
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}
