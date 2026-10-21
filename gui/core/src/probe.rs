//! Probing of the input files with ffprobe, mirroring the deew CLI
//! (`src/probe.rs`, `src/pipeline.rs` and `src/config.rs` of the root crate).

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Result for one input file. Either the three properties or `error`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileProbe {
    pub path: String,
    pub channels: Option<i64>,
    pub sample_rate: Option<i64>,
    /// Raw bit depth, before the CLI's normalisation to 16/24/32.
    pub bit_depth: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    /// The resolved ffprobe executable.
    pub ffprobe: String,
    pub files: Vec<FileProbe>,
}

/// The properties of the selected audio stream that the GUI needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamInfo {
    pub sample_rate: i64,
    pub channels: i64,
    pub bit_depth: i64,
}

/// Regular, non-hidden files of a directory (like `glob(dir/*)`), by name.
fn list_directory(dir: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| format!("{dir}{}{name}", std::path::MAIN_SEPARATOR))
        .collect()
}

/// Expands directories into their files exactly like the CLI does; other
/// inputs are kept as they are (a missing one is reported when probing).
pub fn expand_inputs(inputs: &[String]) -> Vec<String> {
    let mut list = Vec::new();
    for input in inputs {
        if Path::new(input).is_dir() {
            list.extend(list_directory(input));
        } else {
            list.push(input.clone());
        }
    }
    list
}

/// The ffprobe command line, the same as the CLI's.
pub fn probe_args(track_index: u32, file: &str) -> Vec<String> {
    [
        "-v",
        "quiet",
        "-select_streams",
        &format!("a:{track_index}"),
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
        file,
    ]
    .map(String::from)
    .to_vec()
}

/// ffprobe prints some numbers as JSON numbers and some as strings.
fn as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

/// Parses the JSON printed by ffprobe like the CLI does: `None` if it is not
/// JSON, has no first stream or lacks the sample rate or channel count.
pub fn parse_probe(json: &str) -> Option<StreamInfo> {
    let value: Value = serde_json::from_str(json).ok()?;
    let audio = value.get("streams").and_then(|s| s.get(0))?;
    let sample_rate = audio.get("sample_rate").and_then(as_i64)?;
    let channels = audio.get("channels").and_then(as_i64)?;
    let mut bit_depth = audio.get("bits_per_sample").and_then(as_i64).unwrap_or(0);
    if bit_depth == 0 {
        bit_depth = audio
            .get("bits_per_raw_sample")
            .and_then(as_i64)
            .unwrap_or(32);
    }
    Some(StreamInfo {
        sample_rate,
        channels,
        bit_depth,
    })
}

const PARSE_ERROR: &str =
    "couldn't be parsed by ffprobe (no such audio track, or not a media file)";

/// Runs ffprobe on `file`.
fn run_ffprobe(ffprobe: &Path, track_index: u32, file: &str) -> Result<StreamInfo, String> {
    if !Path::new(file).exists() {
        return Err("path does not exist".into());
    }
    let mut command = Command::new(ffprobe);
    command
        .args(probe_args(track_index, file))
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // The release GUI has no console; do not flash one for ffprobe.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command
        .output()
        .map_err(|e| format!("failed to run ffprobe: {e}"))?;
    if !output.status.success() {
        return Err(PARSE_ERROR.into());
    }
    parse_probe(&String::from_utf8_lossy(&output.stdout)).ok_or_else(|| PARSE_ERROR.into())
}

/// The only config location, like the CLI: `config.toml` next to the CLI
/// executable. `None` without a CLI.
pub fn config_path(cli_dir: Option<&Path>) -> Option<PathBuf> {
    cli_dir.map(|dir| dir.join("config.toml"))
}

/// Reads and parses the config; `None` if it is missing or unparsable.
fn read_config(path: &Path) -> Option<toml::Table> {
    std::fs::read_to_string(path).ok()?.parse().ok()
}

/// The `ffprobe_path` of the config (an unusable config counts as none),
/// else `ffprobe`.
pub fn ffprobe_name_from_config(config: Option<&Path>) -> String {
    config
        .and_then(read_config)
        .and_then(|table| {
            table
                .get("ffprobe_path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "ffprobe".to_string())
}

/// The `[default_bitrates]` table of a config text as key -> kbps. Empty if
/// the text is not TOML or has no such table; non-integer values are skipped.
pub fn parse_default_bitrates(text: &str) -> BTreeMap<String, i64> {
    text.parse::<toml::Table>()
        .ok()
        .and_then(|table| match table.get("default_bitrates") {
            Some(toml::Value::Table(t)) => Some(t.clone()),
            _ => None,
        })
        .map(|t| {
            t.into_iter()
                .filter_map(|(key, value)| value.as_integer().map(|n| (key, n)))
                .collect()
        })
        .unwrap_or_default()
}

/// The default bitrates of the config file (empty when there is none).
pub fn default_bitrates(config: Option<&Path>) -> BTreeMap<String, i64> {
    config
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|text| parse_default_bitrates(&text))
        .unwrap_or_default()
}

/// Finds ffprobe like the CLI: the name from the config, resolved via PATH.
pub fn resolve_ffprobe(config: Option<&Path>) -> Result<PathBuf, String> {
    let name = ffprobe_name_from_config(config);
    which::which(&name).map_err(|_| {
        format!("ffprobe was not found (looked for `{name}`); set ffprobe_path in the deew config")
    })
}

/// Probes every input (directories are expanded) with the given ffprobe.
pub fn probe_files(ffprobe: &Path, inputs: &[String], track_index: u32) -> ProbeResult {
    let files = expand_inputs(inputs)
        .into_iter()
        .map(|path| {
            let result = run_ffprobe(ffprobe, track_index, &path);
            match result {
                Ok(info) => FileProbe {
                    path,
                    channels: Some(info.channels),
                    sample_rate: Some(info.sample_rate),
                    bit_depth: Some(info.bit_depth),
                    error: None,
                },
                Err(error) => FileProbe {
                    path,
                    channels: None,
                    sample_rate: None,
                    bit_depth: None,
                    error: Some(error),
                },
            }
        })
        .collect();
    ProbeResult {
        ffprobe: ffprobe.display().to_string(),
        files,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn args_match_the_cli() {
        assert_eq!(
            probe_args(1, "a b.mkv"),
            [
                "-v",
                "quiet",
                "-select_streams",
                "a:1",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                "a b.mkv"
            ]
        );
    }

    #[test]
    fn typical_ffprobe_output() {
        let json = r#"{"streams":[{"index":0,"codec_name":"pcm_s24le","sample_rate":"48000",
            "channels":6,"bits_per_sample":24,"bits_per_raw_sample":"24","duration":"2.000000"}],
            "format":{"duration":"2.0"}}"#;
        assert_eq!(
            parse_probe(json),
            Some(StreamInfo {
                sample_rate: 48000,
                channels: 6,
                bit_depth: 24
            })
        );
    }

    #[test]
    fn numbers_in_either_form() {
        let json = r#"{"streams":[{"sample_rate":44100,"channels":"2","bits_per_sample":"16"}]}"#;
        assert_eq!(
            parse_probe(json),
            Some(StreamInfo {
                sample_rate: 44100,
                channels: 2,
                bit_depth: 16
            })
        );
    }

    #[test]
    fn bit_depth_defaults() {
        let json = r#"{"streams":[{"sample_rate":"48000","channels":2,"bits_per_sample":0}]}"#;
        assert_eq!(parse_probe(json).unwrap().bit_depth, 32);
        let json = r#"{"streams":[{"sample_rate":"48000","channels":2,
            "bits_per_sample":0,"bits_per_raw_sample":"20"}]}"#;
        assert_eq!(parse_probe(json).unwrap().bit_depth, 20);
    }

    #[test]
    fn unusable_output() {
        assert_eq!(parse_probe(""), None);
        assert_eq!(parse_probe("not json"), None);
        assert_eq!(parse_probe("{}"), None);
        assert_eq!(parse_probe(r#"{"streams":[]}"#), None);
        assert_eq!(parse_probe(r#"{"streams":[{"channels":2}]}"#), None);
        assert_eq!(parse_probe(r#"{"streams":[{"sample_rate":"1"}]}"#), None);
    }

    #[test]
    fn directories_expand_to_sorted_visible_files() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("in");
        std::fs::create_dir(&dir).unwrap();
        write(&dir.join("b.mkv"), "");
        write(&dir.join("a.mkv"), "");
        write(&dir.join(".hidden.mkv"), "");
        std::fs::create_dir(dir.join("sub")).unwrap();
        let single = tmp.path().join("single.wav");
        write(&single, "");
        let missing = tmp.path().join("missing.wav");

        let sep = std::path::MAIN_SEPARATOR;
        let d = dir.display();
        let inputs = vec![
            single.display().to_string(),
            d.to_string(),
            missing.display().to_string(),
        ];
        assert_eq!(
            expand_inputs(&inputs),
            vec![
                single.display().to_string(),
                format!("{d}{sep}a.mkv"),
                format!("{d}{sep}b.mkv"),
                missing.display().to_string(),
            ]
        );
    }

    #[test]
    fn ffprobe_name_comes_from_the_config() {
        let tmp = tempfile::tempdir().unwrap();
        let config = tmp.path().join("config.toml");
        let none = tmp.path().join("none.toml");
        write(&config, "logo = 1\nffprobe_path = \"from-config\"\n");
        assert_eq!(ffprobe_name_from_config(Some(&config)), "from-config");
        // An unparsable config counts as none.
        write(&config, "this is = not [ toml");
        assert_eq!(ffprobe_name_from_config(Some(&config)), "ffprobe");
        // A config without the key falls back to `ffprobe`.
        write(&config, "logo = 1\n");
        assert_eq!(ffprobe_name_from_config(Some(&config)), "ffprobe");
        assert_eq!(ffprobe_name_from_config(Some(&none)), "ffprobe");
        assert_eq!(ffprobe_name_from_config(None), "ffprobe");
    }

    #[test]
    fn default_bitrates_are_parsed() {
        let text =
            "logo = 1\n[default_bitrates]\ndd_5_1 = 640\nddp_7_1 = 1024\nbad = 'x'\nf = 1.5\n";
        let map = parse_default_bitrates(text);
        assert_eq!(map.len(), 2);
        assert_eq!(map["dd_5_1"], 640);
        assert_eq!(map["ddp_7_1"], 1024);
        assert!(parse_default_bitrates("").is_empty());
        assert!(parse_default_bitrates("not [ toml").is_empty());
        assert!(parse_default_bitrates("logo = 1\n").is_empty());
        assert!(parse_default_bitrates("default_bitrates = 3\n").is_empty());
    }

    #[test]
    fn default_bitrates_are_read_from_the_file() {
        let tmp = tempfile::tempdir().unwrap();
        let config = tmp.path().join("config.toml");
        assert!(default_bitrates(Some(&config)).is_empty());
        assert!(default_bitrates(None).is_empty());
        write(&config, "[default_bitrates]\nac4_2_0 = 256\n");
        assert_eq!(default_bitrates(Some(&config))["ac4_2_0"], 256);
    }

    #[test]
    fn ffprobe_is_resolved_from_the_config() {
        let tmp = tempfile::tempdir().unwrap();
        let fake = tmp.path().join(if cfg!(windows) {
            "myprobe.exe"
        } else {
            "myprobe"
        });
        write(&fake, "");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let config = tmp.path().join("config.toml");
        let toml_path = fake.display().to_string().replace('\\', "\\\\");
        write(&config, &format!("ffprobe_path = \"{toml_path}\"\n"));
        assert_eq!(resolve_ffprobe(Some(&config)).unwrap(), fake);

        write(&config, "ffprobe_path = 'definitely-not-installed-xyz'\n");
        let err = resolve_ffprobe(Some(&config)).unwrap_err();
        assert!(err.contains("definitely-not-installed-xyz"), "{err}");
    }

    #[test]
    fn config_is_next_to_the_cli() {
        assert_eq!(
            config_path(Some(Path::new("/cli"))),
            Some(Path::new("/cli").join("config.toml"))
        );
        assert_eq!(config_path(None), None);
    }

    #[test]
    fn missing_file_is_a_per_file_error() {
        let result = probe_files(
            Path::new("ffprobe-unused"),
            &["/no/such/file.mkv".into()],
            0,
        );
        assert_eq!(result.files.len(), 1);
        assert_eq!(
            result.files[0].error.as_deref(),
            Some("path does not exist")
        );
        assert_eq!(result.files[0].channels, None);
    }

    #[cfg(unix)]
    #[test]
    fn probes_with_a_fake_ffprobe() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let fake = tmp.path().join("ffprobe");
        // Prints a 5.1 stream for `a:0` and fails for any other track.
        write(
            &fake,
            "#!/bin/sh\nif [ \"$4\" = a:0 ]; then\necho '{\"streams\":[{\"sample_rate\":\"48000\",\"channels\":6,\"bits_per_sample\":0,\"bits_per_raw_sample\":\"24\"}]}'\nelse exit 1; fi\n",
        );
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        let media = tmp.path().join("x.mkv");
        write(&media, "");
        let inputs = [media.display().to_string()];

        let ok = probe_files(&fake, &inputs, 0);
        let file = &ok.files[0];
        assert_eq!(
            (file.channels, file.sample_rate, file.bit_depth, &file.error),
            (Some(6), Some(48000), Some(24), &None)
        );
        let bad = probe_files(&fake, &inputs, 1);
        assert_eq!(bad.files[0].error.as_deref(), Some(PARSE_ERROR));
    }
}
