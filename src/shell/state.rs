//! Shell state shared by every callback, plus the Rust → chrome-page push helpers.
//!
//! All CEF callbacks Lantern implements run on the browser process UI thread, and
//! CEF ref-counted objects are not `Send`, so the state lives in a thread-local
//! `RefCell` rather than behind a global mutex.
//!
//! Re-entrancy rule: never call into CEF while the shell is borrowed. Some CEF
//! calls (navigation, closing) can synchronously trigger callbacks that borrow the
//! shell again. Copy what you need out of [`with_shell`], drop the borrow, then act.

use super::{
    bookmarks::{self, Bookmark},
    settings::{DialupMode, Settings},
    util::{self, Json},
};
use cef::*;
use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

/// Which of the three browser views a client/handler serves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Toolbar,
    Content,
    Status,
}

/// Number of bookmarks shown as quick links in the toolbar.
pub const QUICK_LINKS: usize = 8;

#[derive(Default)]
pub struct Shell {
    pub window: Option<Window>,
    pub toolbar: Option<Browser>,
    pub content: Option<Browser>,
    pub status: Option<Browser>,
    pub open_browsers: usize,
    pub is_closing: bool,

    pub current_url: String,
    pub current_title: String,
    pub is_loading: bool,
    pub can_back: bool,
    pub can_fwd: bool,
    pub progress: f64,
    /// Hover-link text from `on_status_message`.
    pub status_message: String,
    /// One-off message (e.g. "Bookmark added.") shown until the next navigation.
    pub notice: String,
    /// Whether a web page load has finished ("Page complete." vs "Ready.").
    pub load_finished: bool,

    /// What the URL bar shows while an internal `data:` page is displayed.
    pub internal_label: Option<String>,
    /// Labels of internal pages Lantern generated, keyed by a hash of their URL,
    /// so going back/forward to one still shows a friendly label.
    pub internal_urls: HashMap<u64, String>,

    pub bookmarks: Vec<Bookmark>,
    pub settings: Settings,
    pub toolbar_ready: bool,
    pub status_ready: bool,
}

thread_local! {
    static SHELL: RefCell<Shell> = RefCell::new(Shell::default());
}

/// Run `f` with exclusive access to the shell state. Do not call CEF inside `f`.
pub fn with_shell<R>(f: impl FnOnce(&mut Shell) -> R) -> R {
    debug_assert_ne!(currently_on(ThreadId::UI), 0);
    SHELL.with(|shell| f(&mut shell.borrow_mut()))
}

/// Key used for [`Shell::internal_urls`]. The fragment is ignored so that
/// `…#search` variants map to the same page.
pub fn internal_url_key(url: &str) -> u64 {
    let url = url.split('#').next().unwrap_or(url);
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    hasher.finish()
}

impl Shell {
    pub fn browser(&self, role: Role) -> Option<Browser> {
        match role {
            Role::Toolbar => self.toolbar.clone(),
            Role::Content => self.content.clone(),
            Role::Status => self.status.clone(),
        }
    }

    pub fn set_browser(&mut self, role: Role, browser: Option<Browser>) {
        match role {
            Role::Toolbar => self.toolbar = browser,
            Role::Content => self.content = browser,
            Role::Status => self.status = browser,
        }
    }

    /// What the URL bar should display.
    pub fn display_url(&self) -> String {
        self.internal_label
            .clone()
            .unwrap_or_else(|| self.current_url.clone())
    }

    pub fn is_secure(&self) -> bool {
        self.internal_label.is_none() && self.current_url.starts_with("https://")
    }

    pub fn status_text(&self) -> String {
        if !self.status_message.is_empty() {
            self.status_message.clone()
        } else if self.is_loading {
            match util::url_host(&self.current_url) {
                Some(host) if self.internal_label.is_none() => format!("Loading {host}\u{2026}"),
                _ => "Loading\u{2026}".to_owned(),
            }
        } else if !self.notice.is_empty() {
            self.notice.clone()
        } else if self.load_finished && self.internal_label.is_none() {
            "Page complete.".to_owned()
        } else {
            "Ready.".to_owned()
        }
    }

    fn toolbar_json(&self) -> String {
        let quick: Vec<_> = self
            .bookmarks
            .iter()
            .take(QUICK_LINKS)
            .map(|b| {
                Json::Obj(vec![
                    ("title", Json::str(&b.title)),
                    ("url", Json::str(&b.url)),
                ])
            })
            .collect();
        Json::Obj(vec![
            ("url", Json::str(self.display_url())),
            ("canBack", Json::Bool(self.can_back)),
            ("canFwd", Json::Bool(self.can_fwd)),
            ("loading", Json::Bool(self.is_loading)),
            ("secure", Json::Bool(self.is_secure())),
            ("bookmarks", Json::Arr(quick)),
            ("geocities", Json::Bool(self.settings.geocities)),
        ])
        .render()
    }

    fn status_json(&self) -> String {
        Json::Obj(vec![
            ("text", Json::str(self.status_text())),
            ("progress", Json::Num(self.progress.clamp(0.0, 1.0))),
            ("loading", Json::Bool(self.is_loading)),
            ("secure", Json::Bool(self.is_secure())),
            ("modem", Json::str(self.modem_label())),
            ("modemTip", Json::str(self.modem_tooltip())),
        ])
        .render()
    }

    /// Status bar label for the dial-up simulator ("" when it is off).
    fn modem_label(&self) -> String {
        match self.settings.dialup_mode {
            DialupMode::Off => String::new(),
            _ => self.settings.modem_speed.label(),
        }
    }

    fn modem_tooltip(&self) -> String {
        let speed = self.settings.modem_speed.label();
        match self.settings.dialup_mode {
            DialupMode::Off => "Dial-up simulator: off. Click to set it up.".to_owned(),
            DialupMode::Images => {
                format!("Dial-up simulator: images load at {speed}. Click to change.")
            }
            DialupMode::Network => {
                format!("Dial-up simulator: network limited to {speed}. Click to change.")
            }
        }
    }

    /// Window title for the current content title.
    pub fn window_title(&self) -> String {
        let title = self.current_title.trim();
        if title.is_empty() || title.starts_with("data:") {
            "Lantern".to_owned()
        } else {
            format!("{title} - Lantern")
        }
    }
}

/// Run a script in a browser's main frame.
pub fn exec_js(browser: &Browser, code: &str) {
    if let Some(frame) = browser.main_frame() {
        let code = CefString::from(code);
        let script_url = CefString::from("");
        frame.execute_java_script(Some(&code), Some(&script_url), 0);
    }
}

/// Push the full toolbar state (URL, nav buttons, quick links).
pub fn push_toolbar() {
    let target = with_shell(|shell| {
        let browser = shell.toolbar.clone()?;
        shell.toolbar_ready.then(|| (browser, shell.toolbar_json()))
    });
    if let Some((browser, json)) = target {
        exec_js(&browser, &format!("window.lantern&&lantern.update({json})"));
    }
}

/// Push the status bar state (text, progress, padlock).
pub fn push_status() {
    let target = with_shell(|shell| {
        let browser = shell.status.clone()?;
        shell.status_ready.then(|| (browser, shell.status_json()))
    });
    if let Some((browser, json)) = target {
        exec_js(&browser, &format!("window.lantern&&lantern.update({json})"));
    }
}

pub fn push_all() {
    push_toolbar();
    push_status();
}

/// Update the top-level window title from the current content title.
pub fn update_window_title() {
    let (window, title) = with_shell(|shell| (shell.window.clone(), shell.window_title()));
    if let Some(window) = window {
        window.set_title(Some(&CefString::from(title.as_str())));
    }
}

pub fn content_browser() -> Option<Browser> {
    with_shell(|shell| shell.content.clone())
}

/// Set the one-off status bar notice and refresh the status bar.
pub fn set_notice(text: &str) {
    with_shell(|shell| shell.notice = text.to_owned());
    push_status();
}

pub fn save_bookmarks() {
    let list = with_shell(|shell| shell.bookmarks.clone());
    bookmarks::save(&list);
}

/// Ask every browser to close. Posts to the UI thread if needed (macOS
/// `terminate:` can arrive from outside a CEF callback).
pub fn close_all_browsers(force_close: bool) {
    if currently_on(ThreadId::UI) == 0 {
        let mut task = CloseAllBrowsers::new(force_close);
        post_task(ThreadId::UI, Some(&mut task));
        return;
    }

    let browsers: Vec<Browser> = with_shell(|shell| {
        [Role::Content, Role::Toolbar, Role::Status]
            .into_iter()
            .filter_map(|role| shell.browser(role))
            .collect()
    });
    for browser in browsers {
        if let Some(host) = browser.host() {
            host.close_browser(force_close.into());
        }
    }
}

/// Bring the main window to the front (macOS dock-icon reopen, relaunch).
pub fn show_main_window() {
    if currently_on(ThreadId::UI) == 0 {
        let mut task = ShowMainWindow::new();
        post_task(ThreadId::UI, Some(&mut task));
        return;
    }

    if let Some(window) = with_shell(|shell| shell.window.clone()) {
        window.show();
        window.activate();
    }
}

/// Whether shutdown has started. Safe to call from any thread; only the UI
/// thread has the state, so other threads report `false`.
pub fn is_closing() -> bool {
    if currently_on(ThreadId::UI) == 0 {
        return false;
    }
    with_shell(|shell| shell.is_closing)
}

wrap_task! {
    struct CloseAllBrowsers {
        force_close: bool,
    }

    impl Task {
        fn execute(&self) {
            close_all_browsers(self.force_close);
        }
    }
}

wrap_task! {
    struct ShowMainWindow;

    impl Task {
        fn execute(&self) {
            show_main_window();
        }
    }
}

/// A one-shot closure, shared so the task wrapper can be `Clone`.
type Job = std::rc::Rc<RefCell<Option<Box<dyn FnOnce()>>>>;

wrap_task! {
    struct Deferred {
        job: Job,
    }

    impl Task {
        fn execute(&self) {
            if let Some(job) = self.job.borrow_mut().take() {
                job();
            }
        }
    }
}

/// Run `job` later on the UI thread, outside the current callback. Used when an
/// action (like starting a navigation) must not happen inside `on_before_browse`.
pub fn defer(job: impl FnOnce() + 'static) {
    let job: Box<dyn FnOnce()> = Box::new(job);
    let mut task = Deferred::new(std::rc::Rc::new(RefCell::new(Some(job))));
    post_task(ThreadId::UI, Some(&mut task));
}
