//! The CEF `App` and browser-process handler: builds the window and its three views.

use super::{
    bookmarks, client, commands, pages, settings,
    state::{Role, with_shell},
    util::normalize_url_input,
    window::{FACE_GRAY, LanternBrowserViewDelegate, LanternWindowDelegate},
};
use cef::*;
use std::cell::RefCell;

wrap_app! {
    pub struct LanternApp;

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(LanternBrowserProcessHandler::new())
        }
    }
}

/// The `--url=` command-line value, if any.
fn url_switch(command_line: &CommandLine) -> Option<String> {
    let url =
        CefString::from(&command_line.switch_value(Some(&CefString::from("url")))).to_string();
    let url = url.trim();
    (!url.is_empty()).then(|| url.to_owned())
}

fn create_browser_view(role: Role, url: &str) -> Option<BrowserView> {
    let mut client = client::new_client(role);
    let settings = BrowserSettings {
        background_color: if role == Role::Content {
            0xFFFFFFFF
        } else {
            FACE_GRAY
        },
        ..Default::default()
    };
    let mut delegate = LanternBrowserViewDelegate::new(role);
    browser_view_create(
        Some(&mut client),
        Some(&CefString::from(url)),
        Some(&settings),
        None,
        None,
        Some(&mut delegate),
    )
}

wrap_browser_process_handler! {
    struct LanternBrowserProcessHandler;

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);

            let list = bookmarks::load();
            let user_settings = settings::load();
            with_shell(|shell| {
                shell.bookmarks = list.clone();
                shell.settings = user_settings;
            });

            // The content view starts on the home page unless --url= was given.
            let command_line = command_line_get_global();
            let start = command_line.as_ref().and_then(url_switch);
            let content_url = match start.as_deref().and_then(normalize_url_input) {
                Some(url) => url,
                None => {
                    let uri = pages::home_uri(&list);
                    with_shell(|shell| {
                        shell.internal_label = Some(pages::HOME.to_owned());
                        shell
                            .internal_urls
                            .insert(super::state::internal_url_key(&uri), pages::HOME.to_owned());
                    });
                    uri
                }
            };

            let views: Vec<(Role, BrowserView)> = [
                (Role::Toolbar, pages::toolbar_uri()),
                (Role::Content, content_url),
                (Role::Status, pages::status_uri()),
            ]
            .into_iter()
            .filter_map(|(role, url)| create_browser_view(role, &url).map(|view| (role, view)))
            .collect();
            assert_eq!(views.len(), 3, "failed to create the browser views");

            // Create the window. It shows itself in on_window_created.
            let mut delegate = LanternWindowDelegate::new(RefCell::new(views));
            window_create_top_level(Some(&mut delegate));
        }

        fn on_already_running_app_relaunch(
            &self,
            command_line: Option<&mut CommandLine>,
            _current_directory: Option<&CefString>,
        ) -> ::std::os::raw::c_int {
            // A second launch shares our profile; reuse this window instead.
            super::state::show_main_window();
            if let Some(url) = command_line.and_then(|c| url_switch(c)) {
                commands::navigate_input(&url);
            }
            1
        }
    }
}
