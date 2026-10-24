//! The CEF `App` and browser-process handler: builds the window and its three views.

use super::{
    bookmarks, client, commands, pages, selftest,
    settings::{self, RetroStyle},
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

/// Whether a boolean switch like `--geocities` was given.
fn has_switch(command_line: &CommandLine, name: &str) -> bool {
    command_line.has_switch(Some(&CefString::from(name))) == 1
}

/// The value of a `--name=value` switch, if given and not blank.
fn switch_value(command_line: &CommandLine, name: &str) -> Option<String> {
    let value =
        CefString::from(&command_line.switch_value(Some(&CefString::from(name)))).to_string();
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// The `--url=` command-line value, if any.
fn url_switch(command_line: &CommandLine) -> Option<String> {
    switch_value(command_line, "url")
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
            let mut user_settings = settings::load();
            let command_line = command_line_get_global();
            // --retro=<style> (or --geocities) picks a look for this run without
            // saving it; it's only saved if changed in the menu or settings page.
            if let Some(command_line) = &command_line {
                let style = switch_value(command_line, "retro")
                    .and_then(|key| RetroStyle::from_key(&key))
                    .or_else(|| has_switch(command_line, "geocities").then_some(RetroStyle::GeoCities));
                if let Some(style) = style {
                    user_settings.retro_style = style;
                }
                // --self-test=<file> reports the first load to <file> and quits.
                if let Some(path) = switch_value(command_line, "self-test") {
                    selftest::enable(path.into());
                }
            }
            with_shell(|shell| {
                shell.bookmarks = list.clone();
                shell.settings = user_settings;
            });

            // The content view starts on the home page unless --url= was given.
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
