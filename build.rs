#[cfg(target_os = "windows")]
include!("src/shell/resources.rs");

fn main() {
    // On Linux, look for libcef.so next to the executable (as bundle-cef-app
    // lays it out) so the app runs without LD_LIBRARY_PATH.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-link-arg-bins=-Wl,-rpath,$ORIGIN");
    }

    #[cfg(target_os = "windows")]
    winres::WindowsResource::new()
        .set_icon_with_id("resources/win/lantern.ico", &IDI_LANTERN.to_string())
        .set_icon_with_id("resources/win/small.ico", &IDI_SMALL.to_string())
        .compile()
        .unwrap();
}
