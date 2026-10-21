//! Tiny line-based TOML highlighter producing markup for `markup::render`.
//! Stripping the markup always yields the input unchanged.

use crate::error::escape;

const COMMENT: &str = "color(102)";
const HEADER: &str = "bold color(81)";
const KEY: &str = "color(231)";
const EQUALS: &str = "color(197)";
const STRING: &str = "color(186)";
const NUMBER: &str = "color(141)";

/// Append `text` (escaped), wrapped in `style` unless that would be unsafe.
fn push(out: &mut String, text: &str, style: Option<&str>) {
    if text.is_empty() {
        return;
    }
    match style {
        // A backslash right before a tag would turn the tag into an escape.
        Some(s) if !out.ends_with('\\') && !text.ends_with('\\') => {
            out.push_str(&format!("[{s}]{}[/{s}]", escape(text)));
        }
        _ => out.push_str(&escape(text)),
    }
}

/// Length in bytes of the quoted string starting at `s[0]` (quote included).
/// Unterminated strings extend to the end of the line.
fn string_len(s: &str) -> usize {
    let b = s.as_bytes();
    let quote = b[0];
    let mut i = 1;
    while i < b.len() {
        if quote == b'"' && b[i] == b'\\' {
            i += 2;
            continue;
        }
        if b[i] == quote {
            return i + 1;
        }
        i += 1;
    }
    s.len()
}

fn is_bare(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+' | ':')
}

fn is_number(w: &str) -> bool {
    let t = w.strip_prefix(['+', '-']).unwrap_or(w);
    matches!(t, "inf" | "nan") || t.starts_with(|c: char| c.is_ascii_digit())
}

/// Highlight a value (scalar, array or inline table).
fn value(out: &mut String, s: &str) {
    let mut i = 0;
    let mut depth = 0usize;
    while i < s.len() {
        let rest = &s[i..];
        let c = rest.chars().next().unwrap();
        let len = match c {
            '#' => {
                push(out, rest, Some(COMMENT));
                return;
            }
            '"' | '\'' => {
                let n = string_len(rest);
                let is_key = depth > 0 && rest[n..].trim_start().starts_with('=');
                push(out, &rest[..n], Some(if is_key { KEY } else { STRING }));
                n
            }
            '[' | '{' => {
                depth += 1;
                push(out, &rest[..1], None);
                1
            }
            ']' | '}' => {
                depth = depth.saturating_sub(1);
                push(out, &rest[..1], None);
                1
            }
            '=' => {
                push(out, "=", Some(EQUALS));
                1
            }
            c if is_bare(c) => {
                let n = rest.find(|c| !is_bare(c)).unwrap_or(rest.len());
                let w = &rest[..n];
                let style = if depth > 0 && rest[n..].trim_start().starts_with('=') {
                    Some(KEY)
                } else if matches!(w, "true" | "false") || is_number(w) {
                    Some(NUMBER)
                } else {
                    None
                };
                push(out, w, style);
                n
            }
            c => {
                push(out, &rest[..c.len_utf8()], None);
                c.len_utf8()
            }
        };
        i += len;
    }
}

fn is_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Highlight `key = value`; returns false (writing nothing) if not parseable.
fn key_value(out: &mut String, s: &str) -> bool {
    let mut i = 0;
    let mut segs: Vec<(usize, usize, bool)> = Vec::new(); // (start, end, is_key)
    while i < s.len() {
        let rest = &s[i..];
        let c = rest.chars().next().unwrap();
        if c == '=' {
            break;
        }
        let (n, key) = match c {
            '"' | '\'' => {
                let n = string_len(rest);
                if n < 2 || !rest[..n].ends_with(c) {
                    return false;
                }
                (n, true)
            }
            c if is_key_char(c) => (rest.find(|c| !is_key_char(c)).unwrap_or(rest.len()), true),
            '.' | ' ' | '\t' => (1, false),
            _ => return false,
        };
        segs.push((i, i + n, key));
        i += n;
    }
    if i >= s.len() || !segs.iter().any(|s| s.2) {
        return false;
    }
    for (a, b, key) in segs {
        push(out, &s[a..b], key.then_some(KEY));
    }
    push(out, "=", Some(EQUALS));
    value(out, &s[i + 1..]);
    true
}

fn line(out: &mut String, l: &str) {
    let body = l.trim_start_matches([' ', '\t']);
    push(out, &l[..l.len() - body.len()], None);
    if body.starts_with('#') {
        push(out, body, Some(COMMENT));
    } else if body.starts_with('[') {
        let open = if body.starts_with("[[") { 2 } else { 1 };
        let close = if open == 2 { "]]" } else { "]" };
        match body[open..].find(close) {
            Some(p) if p > 0 => {
                let end = open + p + open;
                push(out, &body[..end], Some(HEADER));
                value(out, &body[end..]);
            }
            _ => push(out, body, None),
        }
    } else {
        let mut tmp = String::new();
        if key_value(&mut tmp, body) {
            out.push_str(&tmp);
        } else {
            push(out, body, None);
        }
    }
}

/// Highlight TOML text. `markup::strip(&highlight(t)) == t` for any `t`.
pub fn highlight(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for (n, l) in text.split('\n').enumerate() {
        if n > 0 {
            out.push('\n');
        }
        line(&mut out, l);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::{render, strip};

    fn check(t: &str) {
        assert_eq!(strip(&highlight(t)), t, "input: {t:?}");
        assert_eq!(render(&highlight(t), false), t, "input: {t:?}");
    }

    #[test]
    fn roundtrip_template() {
        check(include_str!("config_template.toml"));
    }

    #[test]
    fn roundtrip_tricky() {
        for t in [
            "a = \"x # not comment\" # comment",
            "[table]",
            "[[arr]]",
            "\"quoted.key\" = 'lit[eral]'",
            "n = -1_000",
            "f = 3.5e-2",
            "b = true",
            "arr = [1, \"two\", false]",
            "t = { x = 1 }",
            "@@@ [x",
            "",
            "    k = 'C:\\'",
            "k = \"unterminated \\",
            "k = \"\"\"multi\nline\"\"\"",
            "x = \\",
            "\\@ = 1 \\[b]",
            "a = [ \"\\\\\", 'x' ]\r\n[t] # c\r\n",
            "[unclosed",
            "é = \"ü\" # ö",
            "= 1",
            "[]",
            "[[]]",
            "k = [\"[bold]\"]",
        ] {
            check(t);
        }
    }

    #[test]
    fn colours() {
        let r = render(&highlight("    [t] # c\nk = \"s\"\nn = 5\n"), true);
        assert!(r.contains("\x1b[0m\x1b[1;38;5;81m[t]\x1b[0m"), "{r:?}");
        assert!(r.contains("\x1b[38;5;102m# c\x1b[0m"), "{r:?}");
        assert!(r.contains("\x1b[38;5;231mk\x1b[0m"), "{r:?}");
        assert!(r.contains("\x1b[38;5;197m=\x1b[0m"), "{r:?}");
        assert!(r.contains("\x1b[38;5;186m\"s\"\x1b[0m"), "{r:?}");
        assert!(r.contains("\x1b[38;5;141m5\x1b[0m"), "{r:?}");
    }

    #[test]
    fn hash_in_string_not_comment() {
        let r = render(&highlight("a = \"x # y\""), true);
        assert!(r.contains("\x1b[38;5;186m\"x # y\"\x1b[0m"), "{r:?}");
        assert!(!r.contains("102"));
    }
}
