//! Config file handling: locations, generation, display and loading.

use crate::data::{ErrorKind, LOGOS};
use crate::error::{error_line, escape, print_exit};
use crate::rprintln;
use crate::toml_highlight::highlight;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const CONFIG_TEMPLATE: &str = include_str!("config_template.toml");

/// Top-level keys that have to be present in the config.
pub const CONFIG_KEYS: [&str; 8] = [
    "ffmpeg_path",
    "ffprobe_path",
    "dee_path",
    "temp_path",
    "logo",
    "max_instances",
    "default_bitrates",
    "summary_sections",
];

/// `max_instances` is either a number or a string like `'50%'`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum MaxInstances {
    Count(i64),
    Text(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultBitrates {
    pub dd_1_0: i64,
    pub dd_2_0: i64,
    pub dd_5_1: i64,
    pub ddp_1_0: i64,
    pub ddp_2_0: i64,
    pub ddp_5_1: i64,
    pub ddp_7_1: i64,
    pub ac4_2_0: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SummarySections {
    pub deew_info: bool,
    pub binaries: bool,
    pub input_info: bool,
    pub output_info: bool,
    pub other: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub dee_path: String,
    pub temp_path: String,
    /// Only checked to be a number; the logo is printed from the raw table
    /// before the other keys are validated.
    #[serde(rename = "logo")]
    pub _logo: i64,
    pub max_instances: MaxInstances,
    pub default_bitrates: DefaultBitrates,
    pub summary_sections: SummarySections,
}

#[derive(Debug, Clone)]
pub struct Paths {
    /// The only config location: `config.toml` next to the executable.
    pub config: PathBuf,
    pub exe_dir: PathBuf,
}

impl Paths {
    /// Paths of the standalone build, based on the running executable.
    pub fn resolve() -> Paths {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));
        Paths {
            config: exe_dir.join("config.toml"),
            exe_dir,
        }
    }
}

/// `-c`: show the config location and the current config.
pub fn show_config(paths: &Paths) -> ! {
    rprintln!(
        "[bold cyan]Your config location:[/bold cyan]\n{}\n\n[bold cyan]Your current config:[/bold cyan]",
        escape(&paths.config.display().to_string())
    );
    if !paths.config.exists() {
        rprintln!("You don't have a config currently.");
        std::process::exit(0);
    }
    match std::fs::read_to_string(&paths.config) {
        Ok(content) => rprintln!("{}", highlight(&content)),
        Err(e) => error_line(&escape(&format!(
            "failed to read {}: {e}",
            paths.config.display()
        ))),
    }
    std::process::exit(0);
}

/// Write the template next to the executable and exit with `exit_code`:
/// 0 for an explicit `-gc` (it did what was asked), 1 when the config was
/// auto-created because it was missing (the requested run did not happen).
/// A failed write always exits 1 via `error_line`.
pub fn generate_config(paths: &Paths, exit_code: i32) -> ! {
    let target = &paths.config;
    if let Err(e) = std::fs::write(target, CONFIG_TEMPLATE) {
        error_line(&escape(&format!(
            "failed to write {}: {e}",
            target.display()
        )));
    }
    println!();
    rprintln!("{}", highlight(CONFIG_TEMPLATE));
    rprintln!(
        "\n[bold cyan]The above config has been created at:[/bold cyan]\n{}",
        escape(&target.display().to_string())
    );
    std::process::exit(exit_code);
}

fn read_table(path: &Path) -> Result<toml::Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    text.parse::<toml::Table>().map_err(|e| e.to_string())
}

/// Load the raw config table from the config file.
fn load_table(paths: &Paths) -> Result<(PathBuf, toml::Table), String> {
    let path = &paths.config;
    match read_table(path) {
        Ok(table) => Ok((path.clone(), table)),
        Err(e) => Err(format!("{}:\n{}", path.display(), e.trim_end())),
    }
}

/// Missing top-level keys, in config order.
pub fn missing_keys(table: &toml::Table) -> Vec<&'static str> {
    CONFIG_KEYS
        .into_iter()
        .filter(|k| !table.contains_key(*k))
        .collect()
}

/// Load the config (printing the logo), exiting with an error message if it
/// is unusable. Generates a config first if none exists.
pub fn load(paths: &Paths) -> Config {
    if !paths.config.exists() {
        rprintln!(
            "[bold yellow]config.toml[/bold yellow] [not bold white]is missing, creating one...[/not bold white]"
        );
        generate_config(paths, 1);
    }

    let (path, table) = match load_table(paths) {
        Ok(loaded) => loaded,
        Err(e) => error_line(&escape(&format!("failed to parse config.toml: {e}"))),
    };

    if let Some(logo) = table.get("logo").and_then(|v| v.as_integer())
        && 0 < logo
        && logo <= LOGOS.len() as i64
    {
        rprintln!("{}", LOGOS[(logo - 1) as usize]);
    }

    let missing = missing_keys(&table);
    if !missing.is_empty() {
        print_exit(
            ErrorKind::ConfigKey,
            Some(&format!(
                "[bold yellow]{}[/bold yellow]",
                missing.join("[not bold white], [/not bold white]")
            )),
        );
    }

    match table.try_into::<Config>() {
        Ok(config) => config,
        Err(e) => error_line(&escape(&format!(
            "invalid config {}: {}",
            path.display(),
            e.message()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_deserializes() {
        let config: Config = toml::from_str(CONFIG_TEMPLATE).unwrap();
        assert_eq!(config.ffmpeg_path, "ffmpeg");
        assert_eq!(config.dee_path, "dee.exe");
        assert_eq!(config.temp_path, "");
        assert_eq!(config.max_instances, MaxInstances::Text("50%".into()));
        assert_eq!(config.default_bitrates.ddp_5_1, 1024);
        assert_eq!(config.default_bitrates.ac4_2_0, 320);
        assert!(config.summary_sections.deew_info && config.summary_sections.other);
    }

    #[test]
    fn deew_web_section_is_optional() {
        assert!(!CONFIG_KEYS.contains(&"deew_web"));
        assert!(CONFIG_TEMPLATE.contains("[deew_web]"));
        let t = CONFIG_TEMPLATE
            .split("# Login for deew-web")
            .next()
            .unwrap();
        assert!(!t.contains("deew_web"));
        toml::from_str::<Config>(t).unwrap();
    }

    #[test]
    fn max_instances_number() {
        let t = CONFIG_TEMPLATE.replace("max_instances = '50%'", "max_instances = 4");
        let config: Config = toml::from_str(&t).unwrap();
        assert_eq!(config.max_instances, MaxInstances::Count(4));
    }

    #[test]
    fn missing_sub_key_is_an_error() {
        let t = CONFIG_TEMPLATE.replace("    ac4_2_0 = 320\n", "");
        let table: toml::Table = t.parse().unwrap();
        let e = table.try_into::<Config>().unwrap_err();
        assert!(e.message().contains("ac4_2_0"), "{}", e.message());
    }

    #[test]
    fn finds_missing_keys() {
        let table: toml::Table = "logo = 1\ntemp_path = ''\n".parse().unwrap();
        assert_eq!(
            missing_keys(&table),
            [
                "ffmpeg_path",
                "ffprobe_path",
                "dee_path",
                "max_instances",
                "default_bitrates",
                "summary_sections"
            ]
        );
        let full: toml::Table = CONFIG_TEMPLATE.parse().unwrap();
        assert!(missing_keys(&full).is_empty());
    }
}
