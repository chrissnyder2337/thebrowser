//! Lantern's browser shell: a Views window holding a toolbar, content and
//! status `BrowserView`, wired together through the handlers in this module.

use cef::*;

pub mod app;
pub mod art;
pub mod bookmarks;
pub mod client;
pub mod commands;
pub mod dialup;
pub mod pages;
pub mod resources;
pub mod retro;
pub mod selftest;
pub mod settings;
pub mod state;
pub mod util;
pub mod window;

#[cfg(target_os = "macos")]
pub type Library = library_loader::LibraryLoader;

#[cfg(not(target_os = "macos"))]
pub struct Library;

// Unused by `main` in Windows sandbox builds, which start from the bootstrap
// executable's `win.rs` entry point instead.
#[allow(dead_code)]
pub fn load_cef() -> Library {
    #[cfg(target_os = "macos")]
    let library = {
        let loader = library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), false);
        assert!(loader.load());
        loader
    };
    #[cfg(not(target_os = "macos"))]
    let library = Library;

    // Initialize the CEF API version.
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    #[cfg(target_os = "macos")]
    crate::mac::setup_lantern_application();

    library
}

// Unused by `main` in Windows sandbox builds, which start from the bootstrap
// executable's `win.rs` entry point instead.
#[allow(dead_code)]
pub fn run_main(main_args: &MainArgs, cmd_line: &CommandLine, sandbox_info: *mut u8) {
    let switch = CefString::from("type");
    let is_browser_process = cmd_line.has_switch(Some(&switch)) != 1;

    let ret = execute_process(Some(main_args), None, sandbox_info);

    if !is_browser_process {
        // Renderer, GPU and other subprocesses return here; they don't
        // initialize CEF themselves.
        assert!(ret >= 0, "cannot execute non-browser process");
        return;
    }
    assert_eq!(ret, -1, "cannot execute browser process");

    let mut app = app::LanternApp::new();

    // Persist cookies, logins and site data across runs.
    let cache_dir = bookmarks::cache_dir();
    if let Err(err) = std::fs::create_dir_all(&cache_dir) {
        eprintln!("lantern: could not create {}: {err}", cache_dir.display());
    }
    let cache_dir = CefString::from(cache_dir.to_string_lossy().as_ref());

    let settings = Settings {
        no_sandbox: !cfg!(feature = "sandbox") as _,
        root_cache_path: cache_dir.clone(),
        cache_path: cache_dir,
        persist_session_cookies: 1,
        background_color: window::FACE_GRAY,
        ..Default::default()
    };
    assert_eq!(
        initialize(
            Some(main_args),
            Some(&settings),
            Some(&mut app),
            sandbox_info,
        ),
        1,
        "CEF failed to initialize"
    );

    #[cfg(target_os = "macos")]
    let _delegate = crate::mac::setup_lantern_app_delegate();

    run_message_loop();

    shutdown();
}
