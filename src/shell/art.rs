//! Renders the text pixel-art sheets in `ui/*.txt` to inline SVG.
//!
//! Sheet format: `= <char> <#rrggbb>` palette lines, then `frame <name>` followed
//! by rows of cells (`.` is transparent). `#` lines are comments.

use std::{collections::HashMap, fmt::Write};

pub const LANTERN: &str = include_str!("../../ui/lantern.txt");
pub const ICONS: &str = include_str!("../../ui/icons.txt");

pub struct Sheet {
    palette: HashMap<char, String>,
    frames: Vec<(String, Vec<Vec<char>>)>,
}

impl Sheet {
    pub fn parse(text: &str) -> Self {
        let mut palette = HashMap::new();
        let mut frames: Vec<(String, Vec<Vec<char>>)> = Vec::new();
        for line in text.lines() {
            let line = line.trim_end();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(entry) = line.strip_prefix("= ") {
                let mut parts = entry.split_whitespace();
                if let (Some(key), Some(color)) = (parts.next(), parts.next())
                    && let Some(key) = key.chars().next()
                {
                    palette.insert(key, color.to_owned());
                }
            } else if let Some(name) = line.strip_prefix("frame ") {
                frames.push((name.trim().to_owned(), Vec::new()));
            } else if let Some((_, rows)) = frames.last_mut() {
                rows.push(line.chars().collect());
            }
        }
        Self { palette, frames }
    }

    fn frame(&self, name: &str) -> &[Vec<char>] {
        self.frames
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, rows)| rows.as_slice())
            .unwrap_or_else(|| panic!("pixel art frame {name:?} missing"))
    }

    pub fn size(&self, name: &str) -> (usize, usize) {
        let rows = self.frame(name);
        (rows.iter().map(Vec::len).max().unwrap_or(0), rows.len())
    }

    /// Append `<rect>`s for one frame, merging horizontal runs of one color.
    /// `recolor` maps every opaque cell to a single color (for embossed icons).
    fn rects(&self, out: &mut String, name: &str, dx: usize, dy: usize, recolor: Option<&str>) {
        for (y, row) in self.frame(name).iter().enumerate() {
            let mut x = 0;
            while x < row.len() {
                let cell = row[x];
                let color = match recolor {
                    _ if cell == '.' => None,
                    Some(color) => Some(color),
                    None => self.palette.get(&cell).map(String::as_str),
                };
                let Some(color) = color else {
                    x += 1;
                    continue;
                };
                let start = x;
                while x < row.len() && row[x] != '.' && (recolor.is_some() || row[x] == cell) {
                    x += 1;
                }
                let _ = write!(
                    out,
                    r#"<rect x="{}" y="{}" width="{}" height="1" fill="{color}"/>"#,
                    start + dx,
                    y + dy,
                    x - start,
                );
            }
        }
    }

    /// One frame as an SVG `scale` screen pixels per cell.
    pub fn svg(&self, name: &str, scale: usize, class: &str) -> String {
        let (w, h) = self.size(name);
        let mut out = svg_open(w, h, scale, class);
        self.rects(&mut out, name, 0, 0, None);
        out.push_str("</svg>");
        out
    }

    /// Disabled look: the shape in dark gray over a white copy offset by one cell.
    pub fn embossed_svg(&self, name: &str, scale: usize, class: &str) -> String {
        let (w, h) = self.size(name);
        let mut out = svg_open(w + 1, h + 1, scale, class);
        self.rects(&mut out, name, 1, 1, Some("#ffffff"));
        self.rects(&mut out, name, 0, 0, Some("#808080"));
        out.push_str("</svg>");
        out
    }

    /// Several frames side by side (a sprite strip for CSS `steps()` animation).
    pub fn strip_svg(&self, names: &[&str], scale: usize, class: &str) -> String {
        let (w, h) = self.size(names[0]);
        let mut out = svg_open(w * names.len(), h, scale, class);
        for (i, name) in names.iter().enumerate() {
            self.rects(&mut out, name, i * w, 0, None);
        }
        out.push_str("</svg>");
        out
    }
}

fn svg_open(w: usize, h: usize, scale: usize, class: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" class="{class}" width="{}" height="{}" viewBox="0 0 {w} {h}" shape-rendering="crispEdges" aria-hidden="true">"#,
        w * scale,
        h * scale,
    )
}

thread_local! {
    static LANTERN_SHEET: Sheet = Sheet::parse(LANTERN);
    static ICON_SHEET: Sheet = Sheet::parse(ICONS);
}

pub fn with_lantern<R>(f: impl FnOnce(&Sheet) -> R) -> R {
    LANTERN_SHEET.with(f)
}

pub fn with_icons<R>(f: impl FnOnce(&Sheet) -> R) -> R {
    ICON_SHEET.with(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheets_are_well_formed() {
        let lantern = Sheet::parse(LANTERN);
        for name in ["idle", "lit0", "lit1", "lit2", "lit3"] {
            assert_eq!(lantern.size(name), (16, 24), "{name}");
        }
        let icons = Sheet::parse(ICONS);
        for name in [
            "back",
            "forward",
            "reload",
            "home",
            "search",
            "bookmarks",
            "mark",
            "stop",
        ] {
            assert_eq!(icons.size(name), (24, 24), "{name}");
        }
        for sheet in [&lantern, &icons] {
            for (name, rows) in &sheet.frames {
                for row in rows {
                    for cell in row {
                        assert!(
                            *cell == '.' || sheet.palette.contains_key(cell),
                            "{name}: unknown color {cell:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn merges_runs() {
        let sheet = Sheet::parse("= A #111111\n= B #222222\nframe t\nAAB.A\n");
        let svg = sheet.svg("t", 2, "c");
        assert!(svg.contains(r##"<rect x="0" y="0" width="2" height="1" fill="#111111"/>"##));
        assert!(svg.contains(r##"<rect x="2" y="0" width="1" height="1" fill="#222222"/>"##));
        assert!(svg.contains(r##"<rect x="4" y="0" width="1" height="1" fill="#111111"/>"##));
        assert!(svg.contains(r#"width="10" height="2""#));
    }
}
