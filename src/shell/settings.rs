//! User settings (the dial-up simulator and GeoCities mode), stored as `key=value`
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    pub dialup_mode: DialupMode,
    pub modem_speed: ModemSpeed,
    /// Restyle every page like a 1990s GeoCities homepage.
    pub geocities: bool,
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
                "geocities" => settings.geocities = matches!(value, "1" | "true" | "on" | "yes"),
                _ => {}
            }
        }
        settings
    }

    fn serialize(&self) -> String {
        format!(
            "dialup_mode={}\nmodem_speed={}\ngeocities={}\n",
            self.dialup_mode.key(),
            self.modem_speed.key(),
            self.geocities
        )
    }

    /// Apply `dialup=<mode>&speed=<speed>` query pairs; unknown values are ignored.
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
        let settings =
            Settings::parse("dialup_mode=network\nmodem_speed=28.8\njunk\n geocities = on \r\n");
        assert_eq!(
            settings,
            Settings {
                dialup_mode: DialupMode::Network,
                modem_speed: ModemSpeed::K28_8,
                geocities: true,
            }
        );
        assert_eq!(Settings::parse(&settings.serialize()), settings);
        assert_eq!(Settings::parse("dialup_mode=warp\n"), Settings::default());
        assert!(!Settings::parse("geocities=false\n").geocities);
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
            geocities: true,
        };
        save_to(&path, &settings).unwrap();
        assert_eq!(load_from(&path), settings);
        let _ = fs::remove_dir_all(dir);
    }
}
