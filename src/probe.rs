//! ffprobe: argument list, output parsing and the process call.

use serde_json::Value;
use std::process::{Command, Stdio};

/// The properties of the selected audio stream that deew needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Probe {
    pub sample_rate: i64,
    pub channels: i64,
    /// Seconds, -1 if ffprobe did not report it.
    pub duration: f64,
    /// Raw bit depth (before normalisation).
    pub bit_depth: i64,
}

pub fn probe_args(ffprobe: &str, track_index: i64, file: &str) -> Vec<String> {
    [
        ffprobe,
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

fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

/// Parse the JSON printed by ffprobe. `Err` if it is not JSON, has no first
/// stream or lacks the sample rate or channel count.
pub fn parse_probe(json: &str) -> Result<Probe, ()> {
    let value: Value = serde_json::from_str(json).map_err(|_| ())?;
    let audio = value.get("streams").and_then(|s| s.get(0)).ok_or(())?;
    let sample_rate = audio.get("sample_rate").and_then(as_i64).ok_or(())?;
    let channels = audio.get("channels").and_then(as_i64).ok_or(())?;
    let duration = audio
        .get("duration")
        .and_then(as_f64)
        .filter(|d| d.is_finite())
        .unwrap_or(-1.0);
    let mut bit_depth = audio.get("bits_per_sample").and_then(as_i64).unwrap_or(0);
    if bit_depth == 0 {
        bit_depth = audio
            .get("bits_per_raw_sample")
            .and_then(as_i64)
            .unwrap_or(32);
    }
    Ok(Probe {
        sample_rate,
        channels,
        duration,
        bit_depth,
    })
}

/// Run ffprobe on `file`; `Err` if it fails or its output is unusable.
pub fn run(ffprobe: &str, track_index: i64, file: &str) -> Result<Probe, ()> {
    let args = probe_args(ffprobe, track_index, file);
    let output = Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| ())?;
    if !output.status.success() {
        return Err(());
    }
    parse_probe(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args() {
        assert_eq!(
            probe_args("ffprobe", 1, "a b.mkv"),
            [
                "ffprobe",
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
            Ok(Probe {
                sample_rate: 48000,
                channels: 6,
                duration: 2.0,
                bit_depth: 24
            })
        );
    }

    #[test]
    fn numbers_in_either_form() {
        let json = r#"{"streams":[{"sample_rate":44100,"channels":"2",
            "bits_per_sample":"16","duration":3}]}"#;
        assert_eq!(
            parse_probe(json),
            Ok(Probe {
                sample_rate: 44100,
                channels: 2,
                duration: 3.0,
                bit_depth: 16
            })
        );
    }

    #[test]
    fn defaults() {
        // no duration -> -1; bits_per_sample 0 -> raw -> 32
        let json = r#"{"streams":[{"sample_rate":"48000","channels":2,"bits_per_sample":0}]}"#;
        let p = parse_probe(json).unwrap();
        assert_eq!((p.duration, p.bit_depth), (-1.0, 32));
        let json = r#"{"streams":[{"sample_rate":"48000","channels":2,
            "bits_per_sample":0,"bits_per_raw_sample":"20","duration":"N/A"}]}"#;
        let p = parse_probe(json).unwrap();
        assert_eq!((p.duration, p.bit_depth), (-1.0, 20));
    }

    #[test]
    fn unusable_output() {
        assert_eq!(parse_probe(""), Err(()));
        assert_eq!(parse_probe("not json"), Err(()));
        assert_eq!(parse_probe("{}"), Err(()));
        assert_eq!(parse_probe(r#"{"streams":[]}"#), Err(()));
        assert_eq!(parse_probe(r#"{"streams":[{"channels":2}]}"#), Err(()));
        assert_eq!(parse_probe(r#"{"streams":[{"sample_rate":"1"}]}"#), Err(()));
    }
}
