//! Interactive prompts, emulating `rich.prompt.Confirm`.

use crate::{rprint, rprintln};
use std::io::{BufRead, Write};

/// Read one line from stdin; exit with status 1 on EOF.
fn read_line() -> String {
    let mut line = String::new();
    match std::io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => {
            println!();
            std::process::exit(1);
        }
        Ok(_) => line,
    }
}

fn show_prompt(question: &str, choices: &str) {
    rprint!("{} [bold magenta][{}][/bold magenta]: ", question, choices);
    let _ = std::io::stdout().flush();
}

/// Port of `Confirm.ask(question)`.
pub fn confirm(question: &str) -> bool {
    loop {
        show_prompt(question, "y/n");
        match read_line().trim().to_lowercase().as_str() {
            "y" => return true,
            "n" => return false,
            _ => rprintln!("[red]Please enter Y or N"),
        }
    }
}
