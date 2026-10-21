//! Informational commands: `-lb`, `-pl` and `-cl`.

use crate::data::{BITRATES, ErrorKind, LOGOS};
use crate::error::{escape, print_exit};
use crate::rprintln;
use base64::Engine;
use std::time::Duration;

const CHANGELOG_URL: &str = "https://api.github.com/repos/pcrlnd/deew/contents/changelog.md";

pub fn list_bitrates() -> ! {
    for (codec, bitrates) in BITRATES {
        let joined = bitrates
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join("[white], [/white]");
        rprintln!(
            "[bold magenta]{}[/bold magenta]: [not bold color(231)]{}[/not bold color(231)]",
            codec,
            joined
        );
    }
    std::process::exit(0);
}

pub fn print_logos() -> ! {
    for (i, logo) in LOGOS.iter().enumerate() {
        rprintln!("logo {}:\n{}", i + 1, logo);
    }
    std::process::exit(0);
}

/// Keep the last 10 entries of the reversed changelog (so the 10 oldest
/// entries of the file, newest of those first), split into lines like the
/// Python version.
pub fn changelog_lines(content: &str) -> Vec<&str> {
    let mut entries: Vec<&str> = content.split("\n\n").collect();
    entries.reverse();
    let start = entries.len().saturating_sub(10);
    let kept = &entries[start..];
    let mut lines = Vec::new();
    for (i, entry) in kept.iter().enumerate() {
        if i > 0 {
            // the "\n\n" separator between entries
            lines.push("");
        }
        lines.extend(entry.split('\n'));
    }
    lines
}

/// Transform one changelog line into printable markup.
///
/// A trailing backslash (markdown line break) is dropped, `[` is escaped so
/// the raw text is never parsed as markup, `# ` headings become bold and
/// backticks alternate between opening and closing a yellow span.
pub fn changelog_markup(line: &str) -> String {
    let line = line.strip_suffix('\\').unwrap_or(line);
    let mut line = escape(line);
    if line.starts_with("# ") {
        line = format!(
            "[bold color(231)]{}[/bold color(231)]",
            line.replace("# ", "")
        );
    }
    let mut out = String::with_capacity(line.len());
    let mut open = false;
    for c in line.chars() {
        if c == '`' {
            out.push_str(if open {
                "[/bold yellow]"
            } else {
                "[bold yellow]"
            });
            open = !open;
        } else {
            out.push(c);
        }
    }
    format!("[not bold white]{out}[/not bold white]")
}

fn fetch_changelog() -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .into();
    let mut response = agent
        .get(CHANGELOG_URL)
        .header("User-Agent", format!("deew/{}", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|e| e.to_string())?;
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let encoded = json
        .get("content")
        .and_then(|c| c.as_str())
        .ok_or("no content field")?;
    let compact: String = encoded.split_whitespace().collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(compact)
        .map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

pub fn print_changelog() -> ! {
    let content = match fetch_changelog() {
        Ok(content) => content,
        Err(_) => print_exit(ErrorKind::Changelog, None),
    };
    for line in changelog_lines(&content) {
        rprintln!("{}", changelog_markup(line));
    }
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::strip;

    #[test]
    fn heading() {
        assert_eq!(
            changelog_markup("# deew 3.2.2:"),
            "[not bold white][bold color(231)]deew 3.2.2:[/bold color(231)][/not bold white]"
        );
    }

    #[test]
    fn backticks_alternate() {
        assert_eq!(
            changelog_markup("use `-b` and `-d` now"),
            "[not bold white]use [bold yellow]-b[/bold yellow] and [bold yellow]-d[/bold yellow] now[/not bold white]"
        );
        // an unbalanced backtick stays open, like in Python
        assert_eq!(
            changelog_markup("a ` b"),
            "[not bold white]a [bold yellow] b[/not bold white]"
        );
    }

    #[test]
    fn trailing_backslash_dropped() {
        assert_eq!(
            changelog_markup("line\\"),
            "[not bold white]line[/not bold white]"
        );
        assert_eq!(
            changelog_markup("a\\b"),
            "[not bold white]a\\b[/not bold white]"
        );
    }

    #[test]
    fn brackets_are_literal() {
        let m = changelog_markup("- fix `[x]` and [bold]no[/bold]");
        assert_eq!(strip(&m), "- fix [x] and [bold]no[/bold]");
        assert_eq!(strip(&changelog_markup("[a:0] stuff")), "[a:0] stuff");
    }

    #[test]
    fn heading_replaces_every_hash_space() {
        assert_eq!(
            changelog_markup("# a # b"),
            "[not bold white][bold color(231)]a b[/bold color(231)][/not bold white]"
        );
    }

    #[test]
    fn plain_and_empty_lines() {
        assert_eq!(changelog_markup(""), "[not bold white][/not bold white]");
        assert_eq!(
            changelog_markup("- item"),
            "[not bold white]- item[/not bold white]"
        );
    }

    #[test]
    fn keeps_last_ten_entries_reversed() {
        let entries: Vec<String> = (1..=12).map(|i| format!("# e{i}\n- x{i}")).collect();
        let content = entries.join("\n\n");
        let lines = changelog_lines(&content);
        // Python: reversed(entries)[-10:] == [e10, e9, ..., e1] joined by blank lines
        assert_eq!(lines.len(), 10 * 2 + 9);
        assert_eq!(lines[0], "# e10");
        assert_eq!(lines[1], "- x10");
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], "# e9");
        assert_eq!(*lines.last().unwrap(), "- x1");
    }

    #[test]
    fn fewer_than_ten_entries() {
        let lines = changelog_lines("# a\n- 1\n\n# b\n- 2");
        assert_eq!(lines, vec!["# b", "- 2", "", "# a", "- 1"]);
    }
}
