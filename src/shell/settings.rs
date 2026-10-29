//! User settings (the dial-up simulator and the retro looks), stored as `key=value`
//! lines in `settings.txt` next to the bookmarks.

use super::bookmarks::data_dir;
use std::{fs, io, path::Path};

/// How the dial-up simulator slows things down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialupMode {
    #[default]
    Off,
    /// Pages arrive at full speed, but images paint in line by line at modem speed.
    Images,
    /// Every byte the content view loads is throttled to modem speed.
    Network,
}

impl DialupMode {
    pub const ALL: [Self; 3] = [Self::Off, Self::Images, Self::Network];

    pub fn key(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Images => "images",
            Self::Network => "network",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.key() == key)
    }
}

/// Modem line speeds, with realistic effective throughput and latency.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModemSpeed {
    K14_4,
    K28_8,
    K33_6,
    #[default]
    K56,
}

impl ModemSpeed {
    pub const ALL: [Self; 4] = [Self::K14_4, Self::K28_8, Self::K33_6, Self::K56];

    pub fn key(self) -> &'static str {
        match self {
            Self::K14_4 => "14.4",
            Self::K28_8 => "28.8",
            Self::K33_6 => "33.6",
            Self::K56 => "56",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|speed| speed.key() == key)
    }

    /// Short label for the status bar, e.g. "28.8K".
    pub fn label(self) -> String {
        format!("{}K", self.key())
    }

    /// Effective download rate in bytes per second, after protocol overhead
    /// and line noise (a 56K modem rarely did better than about 44 kbit/s).
    pub fn download_bytes_per_sec(self) -> u32 {
        match self {
            Self::K14_4 => 1_600,
            Self::K28_8 => 3_200,
            Self::K33_6 => 3_800,
            Self::K56 => 5_500,
        }
    }

    /// Upload rate in bytes per second (V.90 56K modems still sent at 33.6K).
    pub fn upload_bytes_per_sec(self) -> u32 {
        match self {
            Self::K14_4 => 1_600,
            Self::K28_8 => 3_200,
            Self::K33_6 | Self::K56 => 3_800,
        }
    }

    /// Typical round-trip latency in milliseconds.
    pub fn latency_ms(self) -> u32 {
        match self {
            Self::K14_4 => 250,
            Self::K28_8 => 200,
            Self::K33_6 => 180,
            Self::K56 => 150,
        }
    }
}

/// A 1990s look Lantern can give every page (see `ui/retro/`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RetroStyle {
    #[default]
    Off,
    /// A GeoCities homepage: starfield, Comic Sans, rainbows, a hit counter.
    GeoCities,
    /// The early web of 1995: the page's own styles switched off.
    EarlyWeb,
    /// A 1997 web portal: a narrow column of small Arial, banner ad included.
    Portal,
}

impl RetroStyle {
    pub const ALL: [Self; 4] = [Self::Off, Self::GeoCities, Self::EarlyWeb, Self::Portal];

    pub fn key(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::GeoCities => "geocities",
            Self::EarlyWeb => "early-web",
            Self::Portal => "portal",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|style| style.key() == key)
    }

    /// Name for menus and the settings page.
    pub fn name(self) -> &'static str {
        match self {
            Self::Off => "Off (modern web)",
            Self::GeoCities => "GeoCities homepage (1998)",
            Self::EarlyWeb => "Early web (1995)",
            Self::Portal => "Web portal (1997)",
        }
    }
}

/// Which 1990s browser layout the toolbar and status bar wear (see
/// `ui/chrome/` and docs/ui-modes/). Names describe the era, never a product
/// (steward skill, "Branding rule").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UiMode {
    /// Lantern's own toolbar.
    #[default]
    Lantern,
    /// A Motif workstation browser: menu bar, Title and URL fields, and a row
    /// of text buttons under the page.
    Workstation1993,
    /// A mid-90s browser's grouped icon-and-label buttons, Location field and
    /// a row of directory buttons (here, the quick links).
    Toolbar1996,
    /// A mid-90s desktop browser: bands of large labelled buttons whose
    /// icons are grey until hovered, Address and Links bands.
    Desktop1996,
    /// A browser written for the Java platform: flat icon buttons with small
    /// labels and an Address row, in the toolkit's warm grey.
    Applet1997,
}

impl UiMode {
    pub const ALL: [Self; 5] = [
        Self::Lantern,
        Self::Workstation1993,
        Self::Toolbar1996,
        Self::Desktop1996,
        Self::Applet1997,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Lantern => "lantern",
            Self::Workstation1993 => "workstation-1993",
            Self::Toolbar1996 => "toolbar-1996",
            Self::Desktop1996 => "desktop-1996",
            Self::Applet1997 => "applet-1997",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.key() == key)
    }

    /// Name for menus.
    pub fn name(self) -> &'static str {
        match self {
            Self::Lantern => "Lantern (default)",
            Self::Workstation1993 => "Workstation (1993)",
            Self::Toolbar1996 => "Toolbar (1996)",
            Self::Desktop1996 => "Desktop (1996)",
            Self::Applet1997 => "Applet browser (1997)",
        }
    }

    /// The retro look that renders pages the way this mode's browsers did.
    pub fn era_look(self) -> RetroStyle {
        match self {
            Self::Lantern | Self::Desktop1996 | Self::Applet1997 => RetroStyle::Off,
            Self::Workstation1993 | Self::Toolbar1996 => RetroStyle::EarlyWeb,
        }
    }
}

/// How pages look under a UI mode, when no retro look is picked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModePages {
    /// As their sites made them.
    AsIs,
    /// With the mode's browser defaults (page color, fonts, link colors)
    /// wherever the site doesn't set its own (`ui/chrome/<mode>/pages.css`).
    #[default]
    Defaults,
    /// In the mode's era look (`UiMode::era_look`).
    EraLook,
}

impl ModePages {
    pub const ALL: [Self; 3] = [Self::AsIs, Self::Defaults, Self::EraLook];

    pub fn key(self) -> &'static str {
        match self {
            Self::AsIs => "as-is",
            Self::Defaults => "defaults",
            Self::EraLook => "era-look",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|pages| pages.key() == key)
    }

    /// Name for menus.
    pub fn name(self) -> &'static str {
        match self {
            Self::AsIs => "Pages as their sites made them",
            Self::Defaults => "Pages with this browser's defaults",
            Self::EraLook => "Pages in this browser's era look",
        }
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(value, "1" | "true" | "on" | "yes")
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    pub dialup_mode: DialupMode,
    pub modem_speed: ModemSpeed,
    pub retro_style: RetroStyle,
    /// Show a Flash-style splash intro when entering a site.
    pub splash_intro: bool,
    pub ui_mode: UiMode,
    pub mode_pages: ModePages,
}

impl Settings {
    fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "dialup_mode" => {
                    settings.dialup_mode = DialupMode::from_key(value).unwrap_or_default()
                }
                "modem_speed" => {
                    settings.modem_speed = ModemSpeed::from_key(value).unwrap_or_default()
                }
                "retro_style" => {
                    settings.retro_style = RetroStyle::from_key(value).unwrap_or_default()
                }
                "splash_intro" => settings.splash_intro = parse_bool(value),
                "ui_mode" => settings.ui_mode = UiMode::from_key(value).unwrap_or_default(),
                "mode_pages" => {
                    settings.mode_pages = ModePages::from_key(value).unwrap_or_default()
                }
                // Written by the first release of GeoCities mode.
                "geocities" if parse_bool(value) => settings.retro_style = RetroStyle::GeoCities,
                _ => {}
            }
        }
        settings
    }

    fn serialize(&self) -> String {
        format!(
            "dialup_mode={}\nmodem_speed={}\nretro_style={}\nsplash_intro={}\nui_mode={}\nmode_pages={}\n",
            self.dialup_mode.key(),
            self.modem_speed.key(),
            self.retro_style.key(),
            self.splash_intro,
            self.ui_mode.key(),
            self.mode_pages.key()
        )
    }

    /// The look pages get: the retro look the user picked, else the UI
    /// mode's era look if they asked for it.
    pub fn page_look(&self) -> RetroStyle {
        if self.retro_style == RetroStyle::Off && self.mode_pages == ModePages::EraLook {
            self.ui_mode.era_look()
        } else {
            self.retro_style
        }
    }

    /// Whether pages get the UI mode's browser defaults underneath.
    pub fn mode_defaults(&self) -> bool {
        self.mode_pages != ModePages::AsIs
    }

    /// Apply `dialup=<mode>&speed=<speed>&retro=<style>&splash=<0|1>` query
    /// pairs; unknown values are ignored. The settings page sends `splash=0`
    /// before its checkbox, so the last value wins.
    pub fn apply_query(&mut self, params: &[(String, String)]) {
        for (key, value) in params {
            match key.as_str() {
                "dialup" => {
                    if let Some(mode) = DialupMode::from_key(value) {
                        self.dialup_mode = mode;
                    }
                }
                "speed" => {
                    if let Some(speed) = ModemSpeed::from_key(value) {
                        self.modem_speed = speed;
                    }
                }
                "retro" => {
                    if let Some(style) = RetroStyle::from_key(value) {
                        self.retro_style = style;
                    }
                }
                "splash" => self.splash_intro = parse_bool(value),
                _ => {}
            }
        }
    }
}

fn load_from(path: &Path) -> Settings {
    match fs::read_to_string(path) {
        Ok(text) => Settings::parse(&text),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Settings::default(),
        Err(err) => {
            eprintln!("lantern: could not read {}: {err}", path.display());
            Settings::default()
        }
    }
}

fn save_to(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("txt.tmp");
    fs::write(&tmp, settings.serialize())?;
    fs::rename(&tmp, path)
}

pub fn load() -> Settings {
    load_from(&data_dir().join("settings.txt"))
}

pub fn save(settings: &Settings) {
    let path = data_dir().join("settings.txt");
    if let Err(err) = save_to(&path, settings) {
        eprintln!("lantern: could not save {}: {err}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_serialize() {
        let settings = Settings::parse(
            "dialup_mode=network\nmodem_speed=28.8\njunk\n retro_style = portal \r\nsplash_intro=true\n",
        );
        assert_eq!(
            settings,
            Settings {
                dialup_mode: DialupMode::Network,
                modem_speed: ModemSpeed::K28_8,
                retro_style: RetroStyle::Portal,
                splash_intro: true,
                ui_mode: UiMode::Lantern,
                mode_pages: ModePages::Defaults,
            }
        );
        assert_eq!(Settings::parse(&settings.serialize()), settings);
        assert_eq!(Settings::parse("dialup_mode=warp\n"), Settings::default());
        assert_eq!(Settings::parse("retro_style=bbs\n"), Settings::default());
    }

    #[test]
    fn reads_the_old_geocities_flag() {
        let old = Settings::parse("geocities=true\n");
        assert_eq!(old.retro_style, RetroStyle::GeoCities);
        assert_eq!(Settings::parse("geocities=false\n"), Settings::default());
        assert!(!old.serialize().contains("geocities=true"));
    }

    #[test]
    fn query_updates_known_values() {
        let mut settings = Settings::default();
        settings.apply_query(&[
            ("dialup".into(), "images".into()),
            ("speed".into(), "14.4".into()),
        ]);
        assert_eq!(settings.dialup_mode, DialupMode::Images);
        assert_eq!(settings.modem_speed, ModemSpeed::K14_4);
        settings.apply_query(&[("speed".into(), "9600".into())]);
        assert_eq!(settings.modem_speed, ModemSpeed::K14_4);
        // The unchecked-checkbox default comes first, the checkbox after it.
        settings.apply_query(&[
            ("retro".into(), "early-web".into()),
            ("splash".into(), "0".into()),
            ("splash".into(), "1".into()),
        ]);
        assert_eq!(settings.retro_style, RetroStyle::EarlyWeb);
        assert!(settings.splash_intro);
        settings.apply_query(&[
            ("retro".into(), "bbs".into()),
            ("splash".into(), "0".into()),
        ]);
        assert_eq!(settings.retro_style, RetroStyle::EarlyWeb);
        assert!(!settings.splash_intro);
    }

    #[test]
    fn ui_modes_round_trip() {
        for mode in UiMode::ALL {
            assert_eq!(UiMode::from_key(mode.key()), Some(mode));
            let saved = Settings {
                ui_mode: mode,
                ..Settings::default()
            }
            .serialize();
            assert_eq!(Settings::parse(&saved).ui_mode, mode);
        }
        assert_eq!(Settings::parse("ui_mode=kiosk\n").ui_mode, UiMode::Lantern);
    }

    #[test]
    fn page_look_prefers_the_users_look() {
        let mut settings = Settings {
            ui_mode: UiMode::Workstation1993,
            ..Settings::default()
        };
        assert_eq!(settings.mode_pages, ModePages::Defaults);
        assert!(settings.mode_defaults());
        assert_eq!(settings.page_look(), RetroStyle::Off);
        settings.mode_pages = ModePages::EraLook;
        assert_eq!(settings.page_look(), RetroStyle::EarlyWeb);
        settings.retro_style = RetroStyle::Portal;
        assert_eq!(settings.page_look(), RetroStyle::Portal);
        settings.mode_pages = ModePages::AsIs;
        assert!(!settings.mode_defaults());
        assert_eq!(UiMode::Lantern.era_look(), RetroStyle::Off);
        for pages in ModePages::ALL {
            assert_eq!(ModePages::from_key(pages.key()), Some(pages));
            let saved = Settings {
                mode_pages: pages,
                ..Settings::default()
            }
            .serialize();
            assert_eq!(Settings::parse(&saved).mode_pages, pages);
        }
    }

    /// The branding rule (steward skill): modes are named after an era and a
    /// kind of machine, never after a real browser.
    #[test]
    fn ui_modes_avoid_product_names() {
        let banned = [
            "netscape",
            "navigator",
            "explorer",
            "mosaic",
            "opera",
            "webtv",
            "hotjava",
            "macwww",
            "cello",
            "lynx",
            "arachne",
            "icab",
            "omniweb",
            "communicator",
        ];
        for mode in UiMode::ALL {
            for text in [mode.key(), mode.name()] {
                let text = text.to_ascii_lowercase();
                assert!(!banned.iter().any(|b| text.contains(b)), "{text}");
            }
        }
    }

    #[test]
    fn speeds_are_ordered() {
        let rates: Vec<_> = ModemSpeed::ALL
            .iter()
            .map(|s| s.download_bytes_per_sec())
            .collect();
        assert!(rates.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(ModemSpeed::K56.label(), "56K");
    }

    #[test]
    fn round_trip_on_disk() {
        let dir = std::env::temp_dir().join(format!("lantern-settings-{}", std::process::id()));
        let path = dir.join("settings.txt");
        assert_eq!(load_from(&path), Settings::default());
        let settings = Settings {
            dialup_mode: DialupMode::Images,
            modem_speed: ModemSpeed::K33_6,
            retro_style: RetroStyle::GeoCities,
            splash_intro: true,
            ui_mode: UiMode::Workstation1993,
            mode_pages: ModePages::EraLook,
        };
        save_to(&path, &settings).unwrap();
        assert_eq!(load_from(&path), settings);
        let _ = fs::remove_dir_all(dir);
    }
}
