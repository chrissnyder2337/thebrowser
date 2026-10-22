//! The dial-up simulator: modem-speed network throttling for the content
//! view, or a script that makes images paint in slowly.

use super::settings::{DialupMode, Settings};
use cef::*;

const IMAGES_JS: &str = include_str!("../../ui/dialup-images.js");

/// The "slow images" script for these settings, or `None` when that mode is off.
pub fn image_script(settings: &Settings) -> Option<String> {
    if settings.dialup_mode != DialupMode::Images {
        return None;
    }
    let speed = settings.modem_speed;
    Some(
        IMAGES_JS
            .replace("{{RATE}}", &speed.download_bytes_per_sec().to_string())
            .replace("{{LATENCY}}", &speed.latency_ms().to_string()),
    )
}

/// Turn Chromium's network emulation for `browser` on (network mode) or off.
/// The setting lives in the browser's DevTools session, so it holds across
/// navigations until it is changed again.
pub fn apply_network(browser: &Browser, settings: &Settings) {
    let Some(host) = browser.host() else {
        return;
    };
    let Some(mut params) = dictionary_value_create() else {
        return;
    };
    let speed = settings.modem_speed;
    let (latency, download, upload) = if settings.dialup_mode == DialupMode::Network {
        (
            f64::from(speed.latency_ms()),
            f64::from(speed.download_bytes_per_sec()),
            f64::from(speed.upload_bytes_per_sec()),
        )
    } else {
        // -1 throughput disables throttling.
        (0.0, -1.0, -1.0)
    };
    let key = |k: &str| CefString::from(k);
    params.set_bool(Some(&key("offline")), 0);
    params.set_double(Some(&key("latency")), latency);
    params.set_double(Some(&key("downloadThroughput")), download);
    params.set_double(Some(&key("uploadThroughput")), upload);
    let method = CefString::from("Network.emulateNetworkConditions");
    if host.execute_dev_tools_method(0, Some(&method), Some(&mut params)) == 0 {
        eprintln!("lantern: could not apply network emulation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::settings::ModemSpeed;

    #[test]
    fn image_script_only_in_images_mode() {
        let mut settings = Settings::default();
        assert!(image_script(&settings).is_none());
        settings.dialup_mode = DialupMode::Network;
        assert!(image_script(&settings).is_none());
        settings.dialup_mode = DialupMode::Images;
        settings.modem_speed = ModemSpeed::K28_8;
        let script = image_script(&settings).unwrap();
        assert!(script.ends_with("})(3200, 200);\n"), "{script}");
        assert!(!script.contains("{{"));
    }
}
