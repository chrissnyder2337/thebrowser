//! Small string helpers: percent coding, HTML/JS escaping and URL-bar normalization.
//!
//! These are deliberately dependency-free; they only need to be correct for the
//! narrow set of inputs Lantern produces and consumes.

use cef::*;

/// Build a `data:` URI the same way cefsimple does.
pub fn get_data_uri(data: &[u8], mime_type: &str) -> String {
    let data = CefString::from(&base64_encode(Some(data)));
    let uri = CefString::from(&uriencode(Some(&data), 0)).to_string();
    format!("data:{mime_type};base64,{uri}")
}

/// Decode `%XX` escapes (as produced by JavaScript's `encodeURIComponent`).
/// Malformed escapes are kept verbatim; invalid UTF-8 is replaced lossily.
pub fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                out.push(hi << 4 | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Like [`percent_decode`], but also turns `+` into a space (form encoding).
pub fn form_decode(input: &str) -> String {
    percent_decode(&input.replace('+', " "))
}

fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Equivalent of JavaScript's `encodeURIComponent`.
pub fn encode_component(input: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        match b {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => out.push(b as char),
            _ => {
                out.push('%');
                out.push(HEX[(b >> 4) as usize] as char);
                out.push(HEX[(b & 0xf) as usize] as char);
            }
        }
    }
    out
}

/// Escape text for insertion into HTML element content or a quoted attribute.
pub fn html_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Quote a string as a JSON/JavaScript string literal, safe to embed in a
/// script passed to `execute_java_script` or inside an inline `<script>`.
pub fn js_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    out.push('"');
    for c in input.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Minimal JSON value builder, enough for the toolbar/status update payloads.
pub enum Json {
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(&'static str, Json)>),
}

impl Json {
    pub fn str(s: impl Into<String>) -> Self {
        Json::Str(s.into())
    }

    pub fn render(&self) -> String {
        match self {
            Json::Bool(b) => b.to_string(),
            Json::Num(n) if n.is_finite() => n.to_string(),
            Json::Num(_) => "0".to_owned(),
            Json::Str(s) => js_string(s),
            Json::Arr(items) => {
                let items: Vec<_> = items.iter().map(Json::render).collect();
                format!("[{}]", items.join(","))
            }
            Json::Obj(fields) => {
                let fields: Vec<_> = fields
                    .iter()
                    .map(|(k, v)| format!("{}:{}", js_string(k), v.render()))
                    .collect();
                format!("{{{}}}", fields.join(","))
            }
        }
    }
}

const KNOWN_SCHEMES: &[&str] = &["http", "https", "file", "about", "data", "view-source"];

/// Return the scheme of `input` if it starts with one of the schemes the URL bar
/// passes through untouched.
fn known_scheme(input: &str) -> Option<&'static str> {
    let (scheme, _) = input.split_once(':')?;
    KNOWN_SCHEMES
        .iter()
        .copied()
        .find(|known| known.eq_ignore_ascii_case(scheme))
}

fn is_localhost(input: &str) -> bool {
    let host = input.split(['/', '?', '#']).next().unwrap_or("");
    match host.split_once(':') {
        Some((name, port)) => {
            name.eq_ignore_ascii_case("localhost")
                && !port.is_empty()
                && port.bytes().all(|b| b.is_ascii_digit())
        }
        None => host.eq_ignore_ascii_case("localhost"),
    }
}

/// Turn URL-bar text into something loadable:
/// a URL with a known scheme is used as-is, something host-like gets `https://`,
/// anything else becomes a DuckDuckGo search.
pub fn normalize_url_input(input: &str) -> Option<String> {
    let text = input.trim();
    if text.is_empty() {
        return None;
    }
    if known_scheme(text).is_some() {
        return Some(text.to_owned());
    }
    let has_space = text.chars().any(char::is_whitespace);
    if !has_space && (text.contains('.') || is_localhost(text)) {
        return Some(format!("https://{text}"));
    }
    Some(search_url(text))
}

pub fn search_url(query: &str) -> String {
    format!("https://duckduckgo.com/?q={}", encode_component(query))
}

/// Extract the host part of a URL (without port or credentials), lowercase.
pub fn url_host(url: &str) -> Option<String> {
    let (_, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let authority = authority.rsplit('@').next()?;
    let host = if let Some(stripped) = authority.strip_prefix('[') {
        stripped.split(']').next()?
    } else {
        authority.split(':').next()?
    };
    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}

/// Parse `a=1&b=2` into decoded pairs.
pub fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| match part.split_once('=') {
            Some((k, v)) => (form_decode(k), form_decode(v)),
            None => (form_decode(part), String::new()),
        })
        .collect()
}

/// Shorten long text for display, appending an ellipsis.
pub fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_owned()
    } else {
        let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_round_trip() {
        let s = "a b/c?d=é&x|y%";
        assert_eq!(percent_decode(&encode_component(s)), s);
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz%4"), "%zz%4");
        assert_eq!(percent_decode("%41%42"), "AB");
        assert_eq!(form_decode("a+b%2B"), "a b+");
    }

    #[test]
    fn escapes() {
        assert_eq!(html_escape("<a href=\"x\">&'"), "&lt;a href=&quot;x&quot;&gt;&amp;&#39;");
        assert_eq!(
            js_string("a\"b\\c\n</script>\u{2028}\u{1}"),
            "\"a\\\"b\\\\c\\n\\u003c/script>\\u2028\\u0001\""
        );
    }

    #[test]
    fn url_normalization() {
        assert_eq!(normalize_url_input("  "), None);
        assert_eq!(
            normalize_url_input("example.com").as_deref(),
            Some("https://example.com")
        );
        assert_eq!(
            normalize_url_input("localhost:8080/x").as_deref(),
            Some("https://localhost:8080/x")
        );
        assert_eq!(
            normalize_url_input("http://a.b").as_deref(),
            Some("http://a.b")
        );
        assert_eq!(
            normalize_url_input("about:blank").as_deref(),
            Some("about:blank")
        );
        assert_eq!(
            normalize_url_input("rust lang").as_deref(),
            Some("https://duckduckgo.com/?q=rust%20lang")
        );
        assert_eq!(
            normalize_url_input("lanterns").as_deref(),
            Some("https://duckduckgo.com/?q=lanterns")
        );
    }

    #[test]
    fn hosts() {
        assert_eq!(url_host("https://Ex.com:443/a").as_deref(), Some("ex.com"));
        assert_eq!(url_host("http://u:p@h/").as_deref(), Some("h"));
        assert_eq!(url_host("http://[::1]:80/").as_deref(), Some("::1"));
        assert_eq!(url_host("data:text/html,x"), None);
    }

    #[test]
    fn json() {
        let v = Json::Obj(vec![
            ("a", Json::Bool(true)),
            ("b", Json::Arr(vec![Json::str("x"), Json::Num(0.5)])),
        ]);
        assert_eq!(v.render(), r#"{"a":true,"b":["x",0.5]}"#);
    }

    #[test]
    fn queries() {
        assert_eq!(
            parse_query("url=https%3A%2F%2Fa.b%2F%3Fq%3D1&x"),
            vec![
                ("url".to_owned(), "https://a.b/?q=1".to_owned()),
                ("x".to_owned(), String::new())
            ]
        );
    }
}
