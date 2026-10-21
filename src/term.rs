//! Terminal size.

/// Width of the console, like rich: `COLUMNS`, else the terminal, else 80.
pub fn console_width() -> usize {
    if let Some(columns) = std::env::var("COLUMNS")
        .ok()
        .filter(|c| !c.is_empty() && c.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|c| c.parse::<usize>().ok())
    {
        return columns;
    }
    [console::Term::stdout(), console::Term::stderr()]
        .iter()
        .find_map(|t| t.size_checked())
        .map_or(80, |(_, cols)| usize::from(cols))
}
