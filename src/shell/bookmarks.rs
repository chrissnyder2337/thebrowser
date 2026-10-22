//! Bookmark storage: a plain `title\turl` TSV file in the per-OS config dir.

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

impl Bookmark {
    pub fn new(title: &str, url: &str) -> Self {
        let url = clean_field(url);
        let title = clean_field(title);
        let title = if title.is_empty() { url.clone() } else { title };
        Self { title, url }
    }
}

/// Tabs and newlines would break the file format, so fold them into spaces.
fn clean_field(text: &str) -> String {
    text.split(['\t', '\n', '\r'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_owned()
}

fn defaults() -> Vec<Bookmark> {
    vec![
        Bookmark::new("Wikipedia", "https://en.wikipedia.org/"),
        Bookmark::new("cef-rs on GitHub", "https://github.com/tauri-apps/cef-rs"),
        Bookmark::new("CEF Project", "https://bitbucket.org/chromiumembedded/cef"),
        Bookmark::new("DuckDuckGo", "https://duckduckgo.com/"),
        Bookmark::new("Rust", "https://www.rust-lang.org/"),
    ]
}

/// Lantern's per-user data directory:
/// macOS `~/Library/Application Support/Lantern`, Windows `%APPDATA%\Lantern`,
/// Linux `$XDG_CONFIG_HOME/lantern` (or `~/.config/lantern`).
pub fn data_dir() -> PathBuf {
    let home = || {
        env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
    };

    if cfg!(target_os = "macos") {
        home().join("Library/Application Support/Lantern")
    } else if cfg!(target_os = "windows") {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join("AppData").join("Roaming"))
            .join("Lantern")
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| home().join(".config"))
            .join("lantern")
    }
}

pub fn cache_dir() -> PathBuf {
    data_dir().join("cache")
}

fn bookmarks_path() -> PathBuf {
    data_dir().join("bookmarks.tsv")
}

fn parse(text: &str) -> Vec<Bookmark> {
    let mut list: Vec<Bookmark> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        let Some((title, url)) = line.split_once('\t') else {
            continue;
        };
        let bookmark = Bookmark::new(title, url);
        if !bookmark.url.is_empty() && !list.iter().any(|b| b.url == bookmark.url) {
            list.push(bookmark);
        }
    }
    list
}

fn serialize(list: &[Bookmark]) -> String {
    list.iter()
        .map(|b| format!("{}\t{}\n", b.title, b.url))
        .collect()
}

fn load_from(path: &Path) -> Vec<Bookmark> {
    match fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let list = defaults();
            if let Err(err) = save_to(path, &list) {
                eprintln!(
                    "lantern: could not seed bookmarks at {}: {err}",
                    path.display()
                );
            }
            list
        }
        Err(err) => {
            eprintln!("lantern: could not read {}: {err}", path.display());
            defaults()
        }
    }
}

fn save_to(path: &Path, list: &[Bookmark]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    // Write to a sibling file and rename, so a crash can't truncate the list.
    let tmp = path.with_extension("tsv.tmp");
    fs::write(&tmp, serialize(list))?;
    fs::rename(&tmp, path)
}

pub fn load() -> Vec<Bookmark> {
    load_from(&bookmarks_path())
}

pub fn save(list: &[Bookmark]) {
    let path = bookmarks_path();
    if let Err(err) = save_to(&path, list) {
        eprintln!("lantern: could not save {}: {err}", path.display());
    }
}

/// Append a bookmark unless one with the same URL exists. Returns whether it was added.
pub fn add(list: &mut Vec<Bookmark>, title: &str, url: &str) -> bool {
    let bookmark = Bookmark::new(title, url);
    if bookmark.url.is_empty() || list.iter().any(|b| b.url == bookmark.url) {
        return false;
    }
    list.push(bookmark);
    true
}

/// Remove the bookmark with this URL. Returns whether one was removed.
pub fn remove(list: &mut Vec<Bookmark>, url: &str) -> bool {
    let before = list.len();
    list.retain(|b| b.url != url);
    list.len() != before
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_serialize() {
        let list = parse("A\thttps://a/\nbad line\nB\thttps://b/\r\nA2\thttps://a/\n");
        assert_eq!(
            list,
            vec![
                Bookmark::new("A", "https://a/"),
                Bookmark::new("B", "https://b/")
            ]
        );
        assert_eq!(parse(&serialize(&list)), list);
    }

    #[test]
    fn add_strips_and_dedups() {
        let mut list = Vec::new();
        assert!(add(&mut list, "Hello\tthere\nworld", "https://x/"));
        assert_eq!(list[0].title, "Hello there world");
        assert!(!add(&mut list, "Again", "https://x/"));
        assert!(add(&mut list, "", "https://y/"));
        assert_eq!(list[1].title, "https://y/");
        assert!(remove(&mut list, "https://x/"));
        assert!(!remove(&mut list, "https://x/"));
    }

    #[test]
    fn round_trip_on_disk() {
        let dir = env::temp_dir().join(format!("lantern-test-{}", std::process::id()));
        let path = dir.join("bookmarks.tsv");
        let seeded = load_from(&path);
        assert_eq!(seeded, defaults());
        let mut list = seeded;
        add(&mut list, "New", "https://new.example/");
        save_to(&path, &list).unwrap();
        assert_eq!(load_from(&path), list);
        let _ = fs::remove_dir_all(dir);
    }
}
