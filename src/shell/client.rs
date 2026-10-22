//! Per-role `Client` and its handlers. Each handler carries the [`Role`] of the
//! browser view it serves.

use super::{
    commands, dialup, geocities,
    state::{Role, internal_url_key, push_all, push_status, update_window_title, with_shell},
};
use cef::*;

#[cfg(target_os = "linux")]
type OsEvent<'a> = Option<&'a mut sys::XEvent>;
#[cfg(target_os = "windows")]
type OsEvent<'a> = Option<&'a mut sys::MSG>;
#[cfg(target_os = "macos")]
type OsEvent<'a> = *mut u8;

pub fn new_client(role: Role) -> Client {
    LanternClient::new(role)
}

wrap_client! {
    struct LanternClient {
        role: Role,
    }

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(LanternDisplayHandler::new(self.role))
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(LanternLifeSpanHandler::new(self.role))
        }

        fn load_handler(&self) -> Option<LoadHandler> {
            Some(LanternLoadHandler::new(self.role))
        }

        fn request_handler(&self) -> Option<RequestHandler> {
            Some(LanternRequestHandler::new(self.role))
        }

        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(LanternKeyboardHandler::new(self.role))
        }

        fn download_handler(&self) -> Option<DownloadHandler> {
            (self.role == Role::Content).then(LanternDownloadHandler::new)
        }
    }
}

fn is_main_frame(frame: &Option<&mut Frame>) -> bool {
    frame.as_ref().is_some_and(|f| f.is_main() != 0)
}

fn cef_text(text: Option<&CefString>) -> String {
    text.map(CefString::to_string).unwrap_or_default()
}

wrap_display_handler! {
    struct LanternDisplayHandler {
        role: Role,
    }

    impl DisplayHandler {
        fn on_address_change(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            url: Option<&CefString>,
        ) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            if self.role != Role::Content || !is_main_frame(&frame) {
                return;
            }
            let url = cef_text(url);
            with_shell(|shell| {
                if url.starts_with("data:") {
                    let label = shell.internal_urls.get(&internal_url_key(&url)).cloned();
                    if label.is_some() {
                        shell.internal_label = label;
                    }
                } else {
                    shell.internal_label = None;
                }
                shell.current_url = url;
            });
            push_all();
        }

        fn on_title_change(&self, _browser: Option<&mut Browser>, title: Option<&CefString>) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            let title = cef_text(title);
            match self.role {
                Role::Content => {
                    with_shell(|shell| shell.current_title = title);
                    update_window_title();
                }
                Role::Toolbar | Role::Status => {
                    commands::handle_chrome_title(self.role, &title);
                }
            }
        }

        fn on_status_message(&self, _browser: Option<&mut Browser>, value: Option<&CefString>) {
            if self.role != Role::Content {
                return;
            }
            let text = cef_text(value);
            // Don't leak internal data: URIs into the status bar.
            let text = if text.starts_with("data:") { String::new() } else { text };
            with_shell(|shell| shell.status_message = text);
            push_status();
        }

        fn on_loading_progress_change(&self, _browser: Option<&mut Browser>, progress: f64) {
            if self.role != Role::Content {
                return;
            }
            with_shell(|shell| shell.progress = progress);
            push_status();
        }
    }
}

wrap_life_span_handler! {
    struct LanternLifeSpanHandler {
        role: Role,
    }

    impl LifeSpanHandler {
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: ::std::os::raw::c_int,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: ::std::os::raw::c_int,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _no_javascript_access: Option<&mut ::std::os::raw::c_int>,
        ) -> ::std::os::raw::c_int {
            // Single-window browser: popups from the page open in the same view;
            // the chrome pages never get to open anything.
            if self.role == Role::Content {
                let url = cef_text(target_url);
                if !url.is_empty() {
                    super::state::defer(move || commands::navigate(&url));
                }
            }
            1
        }

        fn on_after_created(&self, browser: Option<&mut Browser>) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            let browser = browser.cloned();
            let settings = with_shell(|shell| {
                shell.set_browser(self.role, browser.clone());
                shell.open_browsers += 1;
                shell.settings
            });
            if self.role == Role::Content
                && let Some(browser) = browser.as_ref()
            {
                dialup::apply_network(browser, &settings);
            }
        }

        fn do_close(&self, _browser: Option<&mut Browser>) -> ::std::os::raw::c_int {
            with_shell(|shell| shell.is_closing = true);
            // Allow the close to proceed.
            0
        }

        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            let remaining = with_shell(|shell| {
                shell.set_browser(self.role, None);
                if self.role == Role::Toolbar {
                    shell.toolbar_ready = false;
                }
                if self.role == Role::Status {
                    shell.status_ready = false;
                }
                shell.open_browsers = shell.open_browsers.saturating_sub(1);
                shell.open_browsers
            });
            if remaining == 0 {
                with_shell(|shell| shell.window = None);
                quit_message_loop();
            }
        }
    }
}

wrap_load_handler! {
    struct LanternLoadHandler {
        role: Role,
    }

    impl LoadHandler {
        fn on_load_start(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            _transition_type: TransitionType,
        ) {
            if self.role != Role::Content {
                return;
            }
            // Dial-up "slow images" mode: install the image pacer in every frame.
            let settings = with_shell(|shell| shell.settings);
            if let (Some(frame), Some(script)) = (frame, dialup::image_script(&settings)) {
                let script = CefString::from(script.as_str());
                frame.execute_java_script(Some(&script), Some(&CefString::from("")), 0);
            }
        }

        fn on_loading_state_change(
            &self,
            _browser: Option<&mut Browser>,
            is_loading: ::std::os::raw::c_int,
            can_go_back: ::std::os::raw::c_int,
            can_go_forward: ::std::os::raw::c_int,
        ) {
            if self.role != Role::Content {
                return;
            }
            with_shell(|shell| {
                let was_loading = shell.is_loading;
                shell.is_loading = is_loading != 0;
                shell.can_back = can_go_back != 0;
                shell.can_fwd = can_go_forward != 0;
                if shell.is_loading && !was_loading {
                    shell.progress = 0.0;
                    shell.notice.clear();
                }
                if !shell.is_loading && was_loading {
                    shell.load_finished = true;
                    shell.progress = 1.0;
                }
            });
            push_all();
        }

        fn on_load_end(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            _http_status_code: ::std::os::raw::c_int,
        ) {
            if self.role != Role::Content {
                return;
            }
            if let Some(frame) = frame {
                geocities::on_frame_loaded(frame);
            }
        }

        fn on_load_error(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            error_code: Errorcode,
            error_text: Option<&CefString>,
            failed_url: Option<&CefString>,
        ) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            if self.role != Role::Content || !is_main_frame(&frame) {
                return;
            }
            // Aborted loads (downloads, stop button, superseded navigations) are not errors.
            let error_code = sys::cef_errorcode_t::from(error_code);
            if error_code == sys::cef_errorcode_t::ERR_ABORTED {
                return;
            }
            let failed_url = cef_text(failed_url);
            let error_text = cef_text(error_text);
            let error_code = error_code as i32;
            super::state::defer(move || commands::show_error(&failed_url, error_code, &error_text));
        }
    }
}

wrap_request_handler! {
    struct LanternRequestHandler {
        role: Role,
    }

    impl RequestHandler {
        fn on_before_browse(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _user_gesture: ::std::os::raw::c_int,
            _is_redirect: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            let url = request
                .map(|r| CefString::from(&r.url()).to_string())
                .unwrap_or_default();
            match self.role {
                Role::Content => commands::handle_action_url(&url).into(),
                Role::Toolbar | Role::Status => {
                    // The chrome pages may only load (or reload) their own data: URL.
                    let current = frame
                        .map(|f| CefString::from(&f.url()).to_string())
                        .unwrap_or_default();
                    let initial = current.is_empty() || current == "about:blank";
                    let allowed = url.starts_with("data:text/html") && (initial || url == current);
                    (!allowed).into()
                }
            }
        }
    }
}

wrap_keyboard_handler! {
    struct LanternKeyboardHandler {
        role: Role,
    }

    impl KeyboardHandler {
        fn on_pre_key_event(
            &self,
            _browser: Option<&mut Browser>,
            event: Option<&KeyEvent>,
            _os_event: OsEvent<'_>,
            _is_keyboard_shortcut: Option<&mut ::std::os::raw::c_int>,
        ) -> ::std::os::raw::c_int {
            let Some(event) = event else {
                return 0;
            };
            // Shortcuts work the same whichever view has focus.
            handle_shortcut(event).into()
        }
    }
}

mod vk {
    pub const ESCAPE: i32 = 0x1B;
    pub const HOME: i32 = 0x24;
    pub const LEFT: i32 = 0x25;
    pub const RIGHT: i32 = 0x27;
    pub const D: i32 = 0x44;
    pub const L: i32 = 0x4C;
    pub const R: i32 = 0x52;
    pub const F5: i32 = 0x74;
    pub const OEM_4: i32 = 0xDB; // [
    pub const OEM_6: i32 = 0xDD; // ]
}

fn handle_shortcut(event: &KeyEvent) -> bool {
    if event.type_ != KeyEventType::RAWKEYDOWN && event.type_ != KeyEventType::KEYDOWN {
        return false;
    }
    // The flag enum is backed by u32 on Linux/macOS but i32 on Windows.
    #[allow(clippy::unnecessary_cast)]
    let flag = |f: sys::cef_event_flags_t| event.modifiers & (f.0 as u32) != 0;
    let ctrl = flag(sys::cef_event_flags_t::EVENTFLAG_CONTROL_DOWN);
    let alt = flag(sys::cef_event_flags_t::EVENTFLAG_ALT_DOWN);
    let shift = flag(sys::cef_event_flags_t::EVENTFLAG_SHIFT_DOWN);
    let cmd = flag(sys::cef_event_flags_t::EVENTFLAG_COMMAND_DOWN);
    // The platform's primary accelerator modifier.
    let primary = if cfg!(target_os = "macos") { cmd } else { ctrl };
    let only_primary = primary && !alt && !shift;
    let only_alt = alt && !ctrl && !shift && !cmd;
    let no_mods = !ctrl && !alt && !shift && !cmd;

    let key = event.windows_key_code;
    let mac = cfg!(target_os = "macos");
    match key {
        vk::L if only_primary => commands::focus_url_bar(),
        vk::LEFT if only_alt => commands::go_back(),
        vk::RIGHT if only_alt => commands::go_forward(),
        vk::OEM_4 if mac && only_primary => commands::go_back(),
        vk::OEM_6 if mac && only_primary => commands::go_forward(),
        vk::F5 if no_mods => commands::reload(),
        vk::R if only_primary => commands::reload(),
        vk::D if only_primary => commands::add_bookmark(),
        vk::HOME if only_alt => commands::show_home(),
        vk::ESCAPE if no_mods => {
            // Esc only means "stop" while a page loads; otherwise let the page
            // (or the URL field, which uses it to revert) have it.
            if !with_shell(|shell| shell.is_loading) {
                return false;
            }
            commands::stop();
        }
        _ => return false,
    }
    true
}

wrap_download_handler! {
    struct LanternDownloadHandler;

    impl DownloadHandler {
        fn can_download(
            &self,
            _browser: Option<&mut Browser>,
            _url: Option<&CefString>,
            _request_method: Option<&CefString>,
        ) -> ::std::os::raw::c_int {
            1
        }

        fn on_before_download(
            &self,
            _browser: Option<&mut Browser>,
            _download_item: Option<&mut DownloadItem>,
            suggested_name: Option<&CefString>,
            callback: Option<&mut BeforeDownloadCallback>,
        ) -> ::std::os::raw::c_int {
            // Alloy cancels downloads without a handler: ask the user where to save.
            let Some(callback) = callback else {
                return 0;
            };
            let name = cef_text(suggested_name);
            callback.cont(Some(&CefString::from(name.as_str())), 1);
            super::state::set_notice(&format!("Downloading {name}\u{2026}"));
            1
        }

        fn on_download_updated(
            &self,
            _browser: Option<&mut Browser>,
            download_item: Option<&mut DownloadItem>,
            _callback: Option<&mut DownloadItemCallback>,
        ) {
            let Some(item) = download_item else {
                return;
            };
            if item.is_complete() != 0 {
                let path = CefString::from(&item.full_path()).to_string();
                super::state::set_notice(&format!("Download complete: {path}"));
            } else if item.is_canceled() != 0 {
                super::state::set_notice("Download cancelled.");
            }
        }
    }
}
