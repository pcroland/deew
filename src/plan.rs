//! Pure decisions of the encoding pipeline: argument validation, instance
//! count, channel layout, prompts and bitrate selection.

use crate::cli::Args;
use crate::config::{DefaultBitrates, MaxInstances};
use crate::data::{ErrorKind, allowed_bitrates};
use crate::error::print_exit;
use crate::probe::Probe;
use crate::util::{clamp_f, clamp_i, find_closest_allowed};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Dd,
    Ddp,
    Ac4,
    Thd,
}

impl Format {
    /// Parse an (already lowercased) format name.
    pub fn parse(s: &str) -> Option<Format> {
        Some(match s {
            "dd" => Format::Dd,
            "ddp" => Format::Ddp,
            "ac4" => Format::Ac4,
            "thd" => Format::Thd,
            _ => return None,
        })
    }

    /// Name shown in the summary table.
    pub fn summary_name(self) -> &'static str {
        match self {
            Format::Dd => "DD",
            Format::Ddp => "DDP",
            Format::Ac4 => "AC4",
            Format::Thd => "TrueHD",
        }
    }
}

/// An error that ends the program through `print_exit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fail {
    pub kind: ErrorKind,
    pub insert: Option<String>,
}

impl Fail {
    pub fn new(kind: ErrorKind) -> Fail {
        Fail { kind, insert: None }
    }

    pub fn with(kind: ErrorKind, insert: impl Into<String>) -> Fail {
        Fail {
            kind,
            insert: Some(insert.into()),
        }
    }

    pub fn exit(self) -> ! {
        print_exit(self.kind, self.insert.as_deref())
    }
}

/// `channel_number_to_name`. Only called with validated channel counts.
pub fn channel_name(channels: i64) -> &'static str {
    match channels {
        1 => "mono",
        2 => "stereo",
        6 => "5.1",
        8 => "7.1",
        _ => unreachable!("channel count {channels} was validated"),
    }
}

fn parse_int(text: &str) -> Result<i64, String> {
    text.trim()
        .parse::<i64>()
        .map_err(|_| format!("invalid number of instances: {text}"))
}

/// Number of simultaneous encodes. `arg` is `-in` (an empty string counts as
/// not given), `config` the `max_instances` setting. The result stays a
/// float, `50%` of 5 cpus is 2.5.
pub fn compute_instances(
    arg: Option<&str>,
    config: &MaxInstances,
    cpu: usize,
    dee_is_exe: bool,
) -> Result<f64, String> {
    let text = match arg.filter(|s| !s.is_empty()) {
        Some(s) => s.to_string(),
        None => match config {
            MaxInstances::Count(n) => n.to_string(),
            MaxInstances::Text(s) => s.clone(),
        },
    };
    let cpu = cpu as f64;
    let mut instances = if text.ends_with('%') {
        cpu * (parse_int(&text.replace('%', ""))? as f64 / 100.0)
    } else {
        parse_int(&text)? as f64
    };
    if dee_is_exe {
        instances = clamp_f(instances, 1.0, cpu - 2.0);
        instances = clamp_f(instances, 1.0, 6.0);
    } else {
        instances = clamp_f(instances, 1.0, cpu);
    }
    if instances == 0.0 {
        instances = 1.0;
    }
    Ok(instances)
}

/// Arguments after validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validated {
    /// Effective format (`-mo` forces ddp).
    pub format: Format,
    /// `None` when not given or 0 (Python truthiness).
    pub downmix: Option<i64>,
    pub dialnorm: i64,
    pub track_index: i64,
}

pub const DRC_PROFILES: [&str; 6] = [
    "film_light",
    "film_standard",
    "music_light",
    "music_standard",
    "speech",
    "none",
];

/// Validate the arguments in the order of the Python version.
pub fn validate_args(args: &Args, dee_is_exe: bool, linux: bool) -> Result<Validated, Fail> {
    let dialnorm = clamp_i(args.dialnorm, -31, 0);
    let track_index = args.track_index.max(0);
    let downmix = args.downmix.filter(|&d| d != 0);

    let mut format =
        Format::parse(&args.format.to_lowercase()).ok_or(Fail::new(ErrorKind::Format))?;
    if let Some(d) = downmix {
        if ![1, 2, 6].contains(&d) {
            return Err(Fail::new(ErrorKind::Downmix));
        }
        if format == Format::Thd {
            return Err(Fail::new(ErrorKind::ThdDownmix));
        }
    }
    if !DRC_PROFILES.contains(&args.drc.as_str()) {
        return Err(Fail::new(ErrorKind::Drc));
    }
    if !dee_is_exe && linux && format == Format::Thd {
        return Err(Fail::new(ErrorKind::LinuxThd));
    }
    if args.measure_only {
        format = Format::Ddp;
    }
    Ok(Validated {
        format,
        downmix,
        dialnorm,
        track_index,
    })
}

/// What the probed inputs have in common, and their bit depths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputInfo {
    pub samplerate: i64,
    pub channels: i64,
    /// Per input, normalised to 16, 24 or 32.
    pub bit_depths: Vec<u32>,
}

fn all_equal(list: &[i64]) -> bool {
    list.iter().all(|&x| x == list[0])
}

/// Sample rate and channel mismatch checks, bit depth normalisation. `probes` is not empty.
pub fn check_inputs(probes: &[Probe]) -> Result<InputInfo, Fail> {
    let rates: Vec<i64> = probes.iter().map(|p| p.sample_rate).collect();
    let channels: Vec<i64> = probes.iter().map(|p| p.channels).collect();
    if !all_equal(&rates) {
        return Err(Fail::with(
            ErrorKind::SampleMismatch,
            crate::util::py_int_list(&rates),
        ));
    }
    if !all_equal(&channels) {
        return Err(Fail::with(
            ErrorKind::ChannelMismatch,
            crate::util::py_int_list(&channels),
        ));
    }
    Ok(InputInfo {
        samplerate: rates[0],
        channels: channels[0],
        bit_depths: probes
            .iter()
            .map(|p| normalize_bit_depth(p.bit_depth))
            .collect(),
    })
}

pub fn normalize_bit_depth(depth: i64) -> u32 {
    match depth {
        16 | 24 | 32 => depth as u32,
        d if d < 16 => 16,
        d if 16 < d && d < 24 => 24,
        _ => 32,
    }
}

/// Output channel layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub downmix: Option<i64>,
    pub outchannels: i64,
    /// Value of the `downmix_config` element.
    pub downmix_config: &'static str,
}

/// Channel checks and downmix decisions, in the order of the Python version.
pub fn resolve_layout(format: Format, downmix: Option<i64>, channels: i64) -> Result<Layout, Fail> {
    if ![1, 2, 6, 8].contains(&channels) {
        return Err(Fail::new(ErrorKind::Channels));
    }
    if downmix.is_some_and(|d| d >= channels) {
        return Err(Fail::new(ErrorKind::DownmixMismatch));
    }
    let mut downmix = downmix;
    if downmix.is_none() && format == Format::Dd && channels == 8 {
        downmix = Some(6);
    }
    if format == Format::Ac4 && channels != 6 {
        return Err(Fail::new(ErrorKind::Ac4InputChannels));
    }
    if format == Format::Ac4 {
        downmix = Some(2);
    }
    if format == Format::Thd && channels == 1 {
        return Err(Fail::new(ErrorKind::ThdMonoInput));
    }
    Ok(Layout {
        downmix,
        outchannels: downmix.unwrap_or(channels),
        downmix_config: downmix.map_or("off", channel_name),
    })
}

/// A recommendation shown (`-np`) or asked about before encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notice {
    pub note: &'static str,
    pub question: &'static str,
}

/// The notices to show, in order.
pub fn notices(format: Format, outchannels: i64, measure_only: bool, dialnorm: i64) -> Vec<Notice> {
    let mut list = Vec::new();
    if [1, 2].contains(&outchannels) && matches!(format, Format::Dd | Format::Ddp) && !measure_only
    {
        list.push(Notice {
            note: "Consider using [bold cyan]qaac[/bold cyan] or [bold cyan]opus[/bold cyan] for [bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding.",
            question: "Consider using [bold cyan]qaac[/bold cyan] or [bold cyan]opus[/bold cyan] for [bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding, are you sure you want to use [bold cyan]DEE[/bold cyan]?",
        });
    }
    if outchannels == 2 && format == Format::Thd {
        list.push(Notice {
            note: "Consider using [bold cyan]FLAC[/bold cyan] for lossless [bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding.",
            question: "Consider using [bold cyan]FLAC[/bold cyan] for lossless [bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding, are you sure you want to use [bold cyan]DEE[/bold cyan]?",
        });
    }
    if dialnorm != 0 {
        list.push(Notice {
            note: "Consider leaving the dialnorm value at 0 (auto), setting it manually can be dangerous.",
            question: "Consider leaving the dialnorm value at 0 (auto), setting it manually can be dangerous, are you sure you want to do it?",
        });
    }
    list
}

/// Pick the bitrate: the requested one (or the configured default) rounded to
/// the closest value DEE accepts. `None` for TrueHD.
pub fn select_bitrate(
    format: Format,
    outchannels: i64,
    requested: Option<i64>,
    defaults: &DefaultBitrates,
    force_standard: bool,
    force_bluray: bool,
) -> Option<u32> {
    let requested = requested.filter(|&b| b != 0);
    let pick = |default: i64, table: &str| {
        Some(find_closest_allowed(
            requested.unwrap_or(default),
            allowed_bitrates(table),
        ))
    };
    match (format, outchannels) {
        (Format::Dd, 1) => pick(defaults.dd_1_0, "dd_10"),
        (Format::Dd, 2) => pick(defaults.dd_2_0, "dd_20"),
        (Format::Dd, 6) => pick(defaults.dd_5_1, "dd_51"),
        (Format::Ddp, 1) => pick(defaults.ddp_1_0, "ddp_10"),
        (Format::Ddp, 2) => pick(defaults.ddp_2_0, "ddp_20"),
        (Format::Ddp, 6) => pick(defaults.ddp_5_1, "ddp_51"),
        (Format::Ddp, 8) => {
            let table = if force_standard {
                "ddp_71_standard"
            } else if force_bluray {
                "ddp_71_bluray"
            } else {
                "ddp_71_combined"
            };
            pick(defaults.ddp_7_1, table)
        }
        (Format::Ac4, _) => pick(defaults.ac4_2_0, "ac4_20"),
        (Format::Thd, _) => None,
        // DD is downmixed to 5.1 at most, so this cannot happen.
        (Format::Dd, _) | (Format::Ddp, _) => {
            unreachable!("no {format:?} bitrates for {outchannels} channels")
        }
    }
}

/// `encoder_mode` of the DD/DDP job.
pub fn encoder_mode(
    format: Format,
    outchannels: i64,
    bitrate: u32,
    force_standard: bool,
    force_bluray: bool,
) -> &'static str {
    if format == Format::Dd {
        return "dd";
    }
    let mut mode = "ddp";
    if outchannels == 8 {
        mode = "ddp71";
        if bitrate > 1024 {
            mode = "bluray";
        }
        if force_standard {
            mode = "ddp71";
        }
        if force_bluray {
            mode = "bluray";
        }
    }
    mode
}

/// Extension of the output file.
pub fn output_extension(
    format: Format,
    bitrate: u32,
    force_standard: bool,
    force_bluray: bool,
) -> &'static str {
    match format {
        Format::Dd => "ac3",
        Format::Ac4 => "ac4",
        Format::Thd => "thd",
        Format::Ddp => {
            let mut ext = "ec3";
            if bitrate > 1024 {
                ext = "eb3";
            }
            if force_standard {
                ext = "ec3";
            }
            if force_bluray {
                ext = "eb3";
            }
            ext
        }
    }
}

/// The delay parsed from a `... DELAY -24ms.ext` file name, if it has one.
pub fn filename_delay(path: &str) -> Option<String> {
    use regex::Regex;
    use std::sync::LazyLock;
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^.+DELAY ([-|+]?[0-9]+m?s)\..+").expect("valid regex"));

    RE.captures(path).map(|c| {
        let delay = &c[1];
        if delay.starts_with(['-', '+']) {
            delay.to_string()
        } else {
            format!("+{delay}")
        }
    })
}

/// Delay of an input: `args.delay`, else parsed from the file name, else
/// `+0ms`.
pub fn input_delay(path: &str, arg: Option<&str>) -> String {
    if let Some(delay) = arg.filter(|d| !d.is_empty()) {
        return delay.to_string();
    }
    filename_delay(path).unwrap_or_else(|| "+0ms".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DefaultBitrates;

    fn defaults() -> DefaultBitrates {
        DefaultBitrates {
            dd_1_0: 128,
            dd_2_0: 256,
            dd_5_1: 640,
            ddp_1_0: 128,
            ddp_2_0: 256,
            ddp_5_1: 1024,
            ddp_7_1: 1536,
            ac4_2_0: 320,
        }
    }

    fn pct(s: &str) -> MaxInstances {
        MaxInstances::Text(s.to_string())
    }

    #[test]
    fn instances_percent_and_int() {
        let i = |arg: Option<&str>, cfg: MaxInstances, cpu, exe| {
            compute_instances(arg, &cfg, cpu, exe).unwrap()
        };
        assert_eq!(i(None, pct("50%"), 8, false), 4.0);
        assert_eq!(i(None, pct("50%"), 5, false), 2.5);
        assert_eq!(i(None, MaxInstances::Count(3), 8, false), 3.0);
        assert_eq!(i(None, pct("4"), 8, false), 4.0);
        assert_eq!(i(Some("2"), pct("50%"), 8, false), 2.0);
        assert_eq!(i(Some(""), pct("50%"), 8, false), 4.0);
        assert_eq!(i(Some("100%"), pct("50%"), 8, false), 8.0);
        assert_eq!(i(Some("250%"), pct("50%"), 8, false), 8.0);
        assert_eq!(i(Some("0"), pct("50%"), 8, false), 1.0);
        assert_eq!(i(Some("0%"), pct("50%"), 8, false), 1.0);
        assert_eq!(i(Some("-5"), pct("50%"), 8, false), 1.0);
    }

    #[test]
    fn instances_exe_clamps() {
        let i = |arg: &str, cpu| compute_instances(Some(arg), &pct("50%"), cpu, true).unwrap();
        assert_eq!(i("100%", 8), 6.0);
        assert_eq!(i("100%", 16), 6.0);
        assert_eq!(i("100%", 4), 2.0);
        assert_eq!(i("50%", 16), 6.0);
        assert_eq!(i("3", 16), 3.0);
        // cpu - 2 = 0: clamp(x, 1, 0) is 0, then clamp(0, 1, 6) is 1
        assert_eq!(i("100%", 2), 1.0);
        assert_eq!(i("4", 1), 1.0);
        assert_eq!(i("50%", 5), 2.5);
    }

    #[test]
    fn instances_errors() {
        assert!(compute_instances(Some("abc"), &pct("50%"), 8, false).is_err());
        assert!(compute_instances(Some("x%"), &pct("50%"), 8, false).is_err());
        assert!(compute_instances(Some("1.5"), &pct("50%"), 8, false).is_err());
        assert_eq!(
            compute_instances(Some(" 3 "), &pct("50%"), 8, false),
            Ok(3.0)
        );
    }

    fn args(f: impl FnOnce(&mut Args)) -> Args {
        let mut a = Args::default();
        f(&mut a);
        a
    }

    #[test]
    fn validation_order_and_values() {
        let v = validate_args(&args(|a| a.format = "DDP".into()), true, true).unwrap();
        assert_eq!(v.format, Format::Ddp);
        let v = validate_args(
            &args(|a| {
                a.dialnorm = -50;
                a.track_index = -3;
                a.downmix = Some(0);
            }),
            true,
            true,
        )
        .unwrap();
        assert_eq!((v.dialnorm, v.track_index, v.downmix), (-31, 0, None));
        let v = validate_args(&args(|a| a.dialnorm = 5), true, true).unwrap();
        assert_eq!(v.dialnorm, 0);

        let err = |a: Args, exe, linux| validate_args(&a, exe, linux).unwrap_err().kind;
        assert_eq!(
            err(args(|a| a.format = "mp3".into()), true, true),
            ErrorKind::Format
        );
        assert_eq!(
            err(args(|a| a.downmix = Some(3)), true, true),
            ErrorKind::Downmix
        );
        assert_eq!(
            err(args(|a| a.downmix = Some(-1)), true, true),
            ErrorKind::Downmix
        );
        assert_eq!(
            err(
                args(|a| {
                    a.downmix = Some(2);
                    a.format = "thd".into()
                }),
                true,
                true
            ),
            ErrorKind::ThdDownmix
        );
        assert_eq!(
            err(args(|a| a.drc = "loud".into()), true, true),
            ErrorKind::Drc
        );
        assert!(validate_args(&args(|a| a.drc = "none".into()), true, true).is_ok());
        assert_eq!(
            err(args(|a| a.format = "thd".into()), false, true),
            ErrorKind::LinuxThd
        );
        assert!(validate_args(&args(|a| a.format = "thd".into()), true, true).is_ok());
        assert!(validate_args(&args(|a| a.format = "thd".into()), false, false).is_ok());
        // format error comes before drc error
        assert_eq!(
            err(
                args(|a| {
                    a.format = "x".into();
                    a.drc = "y".into()
                }),
                true,
                true
            ),
            ErrorKind::Format
        );
    }

    #[test]
    fn measure_only_forces_ddp_after_checks() {
        let v = validate_args(
            &args(|a| {
                a.measure_only = true;
                a.format = "thd".into()
            }),
            true,
            true,
        )
        .unwrap();
        assert_eq!(v.format, Format::Ddp);
        // thd + native dee on linux is still rejected before the override
        let e = validate_args(
            &args(|a| {
                a.measure_only = true;
                a.format = "thd".into()
            }),
            false,
            true,
        )
        .unwrap_err();
        assert_eq!(e.kind, ErrorKind::LinuxThd);
    }

    fn probe(rate: i64, ch: i64, depth: i64) -> Probe {
        Probe {
            sample_rate: rate,
            channels: ch,
            duration: 2.0,
            bit_depth: depth,
        }
    }

    #[test]
    fn input_checks() {
        let info = check_inputs(&[probe(48000, 6, 24), probe(48000, 6, 24)]).unwrap();
        assert_eq!((info.samplerate, info.channels), (48000, 6));
        assert_eq!(info.bit_depths, [24, 24]);
        // different bit depths are fine, each normalised on its own
        let info = check_inputs(&[probe(48000, 6, 24), probe(48000, 6, 16)]).unwrap();
        assert_eq!(info.bit_depths, [24, 16]);
        let info = check_inputs(&[probe(48000, 6, 20), probe(48000, 6, 64)]).unwrap();
        assert_eq!(info.bit_depths, [24, 32]);

        let e = check_inputs(&[probe(48000, 6, 24), probe(44100, 6, 24)]).unwrap_err();
        assert_eq!(e, Fail::with(ErrorKind::SampleMismatch, "[48000, 44100]"));
        let e = check_inputs(&[probe(48000, 6, 24), probe(48000, 2, 24)]).unwrap_err();
        assert_eq!(e, Fail::with(ErrorKind::ChannelMismatch, "[6, 2]"));
        // the order of the checks
        let e = check_inputs(&[probe(48000, 6, 24), probe(44100, 2, 16)]).unwrap_err();
        assert_eq!(e.kind, ErrorKind::SampleMismatch);
    }

    #[test]
    fn bit_depth_normalisation() {
        for (d, want) in [
            (8, 16),
            (15, 16),
            (16, 16),
            (17, 24),
            (20, 24),
            (23, 24),
            (24, 24),
            (25, 32),
            (32, 32),
            (64, 32),
        ] {
            assert_eq!(normalize_bit_depth(d), want, "{d}");
        }
    }

    #[test]
    fn layouts() {
        let l = resolve_layout(Format::Ddp, None, 6).unwrap();
        assert_eq!(
            (l.downmix, l.outchannels, l.downmix_config),
            (None, 6, "off")
        );
        let l = resolve_layout(Format::Dd, None, 8).unwrap();
        assert_eq!(
            (l.downmix, l.outchannels, l.downmix_config),
            (Some(6), 6, "5.1")
        );
        let l = resolve_layout(Format::Ddp, None, 8).unwrap();
        assert_eq!((l.outchannels, l.downmix_config), (8, "off"));
        let l = resolve_layout(Format::Ddp, Some(2), 8).unwrap();
        assert_eq!((l.outchannels, l.downmix_config), (2, "stereo"));
        let l = resolve_layout(Format::Dd, Some(1), 8).unwrap();
        assert_eq!((l.outchannels, l.downmix_config), (1, "mono"));
        let l = resolve_layout(Format::Ac4, None, 6).unwrap();
        assert_eq!((l.outchannels, l.downmix_config), (2, "stereo"));

        let kind = |f, d, c| resolve_layout(f, d, c).unwrap_err().kind;
        assert_eq!(kind(Format::Ddp, None, 3), ErrorKind::Channels);
        assert_eq!(kind(Format::Ddp, Some(6), 6), ErrorKind::DownmixMismatch);
        assert_eq!(kind(Format::Ddp, Some(2), 2), ErrorKind::DownmixMismatch);
        assert_eq!(kind(Format::Ac4, None, 2), ErrorKind::Ac4InputChannels);
        assert_eq!(kind(Format::Ac4, None, 8), ErrorKind::Ac4InputChannels);
        assert_eq!(kind(Format::Thd, None, 1), ErrorKind::ThdMonoInput);
        assert!(resolve_layout(Format::Thd, None, 2).is_ok());
        // the channel check comes before the downmix check
        assert_eq!(kind(Format::Ddp, Some(6), 3), ErrorKind::Channels);
    }

    #[test]
    fn notice_list() {
        assert!(notices(Format::Ddp, 6, false, 0).is_empty());
        assert_eq!(notices(Format::Dd, 2, false, 0).len(), 1);
        assert!(notices(Format::Dd, 2, true, 0).is_empty());
        assert_eq!(notices(Format::Thd, 2, false, 0).len(), 1);
        assert!(notices(Format::Ac4, 2, false, 0).is_empty());
        let all = notices(Format::Ddp, 1, false, -27);
        assert_eq!(all.len(), 2);
        assert!(all[1].note.starts_with("Consider leaving the dialnorm"));
        assert_eq!(notices(Format::Thd, 2, false, -3).len(), 2,);
        assert_eq!(
            all[0].note,
            "Consider using [bold cyan]qaac[/bold cyan] or [bold cyan]opus[/bold cyan] for [bold yellow]mono[/bold yellow] and [bold yellow]stereo[/bold yellow] encoding."
        );
    }

    fn br(f: Format, ch: i64, req: Option<i64>, fs: bool, fb: bool) -> Option<u32> {
        select_bitrate(f, ch, req, &defaults(), fs, fb)
    }

    #[test]
    fn bitrate_defaults() {
        assert_eq!(br(Format::Dd, 1, None, false, false), Some(128));
        assert_eq!(br(Format::Dd, 2, None, false, false), Some(256));
        assert_eq!(br(Format::Dd, 6, None, false, false), Some(640));
        assert_eq!(br(Format::Ddp, 1, None, false, false), Some(128));
        assert_eq!(br(Format::Ddp, 2, None, false, false), Some(256));
        assert_eq!(br(Format::Ddp, 6, None, false, false), Some(1024));
        assert_eq!(br(Format::Ddp, 8, None, false, false), Some(1536));
        assert_eq!(br(Format::Ac4, 2, None, false, false), Some(320));
        assert_eq!(br(Format::Thd, 6, None, false, false), None);
        assert_eq!(br(Format::Thd, 6, Some(640), false, false), None);
        // 0 counts as not given
        assert_eq!(br(Format::Dd, 6, Some(0), false, false), Some(640));
    }

    type BitrateCase = (Format, i64, Option<i64>, bool, bool, u32);

    /// Expected values produced by running the selection code of the Python
    /// version: (format, output channels, requested, -fs, -fb, bitrate).
    #[rustfmt::skip]
    const PYTHON_BITRATES: &[BitrateCase] = &[
        (Format::Dd, 1, None, false, false, 128),
        (Format::Dd, 1, Some(0), false, false, 128),
        (Format::Dd, 1, Some(-5), false, false, 96),
        (Format::Dd, 1, Some(1), false, false, 96),
        (Format::Dd, 1, Some(100), false, false, 96),
        (Format::Dd, 1, Some(200), false, false, 192),
        (Format::Dd, 1, Some(244), false, false, 256),
        (Format::Dd, 1, Some(288), false, false, 256),
        (Format::Dd, 1, Some(900), false, false, 640),
        (Format::Dd, 1, Some(1100), false, false, 640),
        (Format::Dd, 1, Some(1200), false, false, 640),
        (Format::Dd, 1, Some(1700), false, false, 640),
        (Format::Dd, 1, Some(5000), false, false, 640),
        (Format::Dd, 2, None, false, false, 256),
        (Format::Dd, 2, Some(0), false, false, 256),
        (Format::Dd, 2, Some(-5), false, false, 96),
        (Format::Dd, 2, Some(1), false, false, 96),
        (Format::Dd, 2, Some(100), false, false, 96),
        (Format::Dd, 2, Some(200), false, false, 192),
        (Format::Dd, 2, Some(244), false, false, 256),
        (Format::Dd, 2, Some(288), false, false, 256),
        (Format::Dd, 2, Some(900), false, false, 640),
        (Format::Dd, 2, Some(1100), false, false, 640),
        (Format::Dd, 2, Some(1200), false, false, 640),
        (Format::Dd, 2, Some(1700), false, false, 640),
        (Format::Dd, 2, Some(5000), false, false, 640),
        (Format::Dd, 6, None, false, false, 640),
        (Format::Dd, 6, Some(0), false, false, 640),
        (Format::Dd, 6, Some(-5), false, false, 224),
        (Format::Dd, 6, Some(1), false, false, 224),
        (Format::Dd, 6, Some(100), false, false, 224),
        (Format::Dd, 6, Some(200), false, false, 224),
        (Format::Dd, 6, Some(244), false, false, 256),
        (Format::Dd, 6, Some(288), false, false, 256),
        (Format::Dd, 6, Some(900), false, false, 640),
        (Format::Dd, 6, Some(1100), false, false, 640),
        (Format::Dd, 6, Some(1200), false, false, 640),
        (Format::Dd, 6, Some(1700), false, false, 640),
        (Format::Dd, 6, Some(5000), false, false, 640),
        (Format::Ddp, 1, None, false, false, 128),
        (Format::Ddp, 1, Some(0), false, false, 128),
        (Format::Ddp, 1, Some(-5), false, false, 32),
        (Format::Ddp, 1, Some(1), false, false, 32),
        (Format::Ddp, 1, Some(100), false, false, 96),
        (Format::Ddp, 1, Some(200), false, false, 200),
        (Format::Ddp, 1, Some(244), false, false, 240),
        (Format::Ddp, 1, Some(288), false, false, 288),
        (Format::Ddp, 1, Some(900), false, false, 896),
        (Format::Ddp, 1, Some(1100), false, false, 1024),
        (Format::Ddp, 1, Some(1200), false, false, 1024),
        (Format::Ddp, 1, Some(1700), false, false, 1024),
        (Format::Ddp, 1, Some(5000), false, false, 1024),
        (Format::Ddp, 2, None, false, false, 256),
        (Format::Ddp, 2, Some(0), false, false, 256),
        (Format::Ddp, 2, Some(-5), false, false, 96),
        (Format::Ddp, 2, Some(1), false, false, 96),
        (Format::Ddp, 2, Some(100), false, false, 96),
        (Format::Ddp, 2, Some(200), false, false, 200),
        (Format::Ddp, 2, Some(244), false, false, 240),
        (Format::Ddp, 2, Some(288), false, false, 288),
        (Format::Ddp, 2, Some(900), false, false, 896),
        (Format::Ddp, 2, Some(1100), false, false, 1024),
        (Format::Ddp, 2, Some(1200), false, false, 1024),
        (Format::Ddp, 2, Some(1700), false, false, 1024),
        (Format::Ddp, 2, Some(5000), false, false, 1024),
        (Format::Ddp, 6, None, false, false, 1024),
        (Format::Ddp, 6, Some(0), false, false, 1024),
        (Format::Ddp, 6, Some(-5), false, false, 192),
        (Format::Ddp, 6, Some(1), false, false, 192),
        (Format::Ddp, 6, Some(100), false, false, 192),
        (Format::Ddp, 6, Some(200), false, false, 200),
        (Format::Ddp, 6, Some(244), false, false, 240),
        (Format::Ddp, 6, Some(288), false, false, 288),
        (Format::Ddp, 6, Some(900), false, false, 896),
        (Format::Ddp, 6, Some(1100), false, false, 1024),
        (Format::Ddp, 6, Some(1200), false, false, 1024),
        (Format::Ddp, 6, Some(1700), false, false, 1024),
        (Format::Ddp, 6, Some(5000), false, false, 1024),
        (Format::Ddp, 8, None, false, false, 1536),
        (Format::Ddp, 8, Some(0), false, false, 1536),
        (Format::Ddp, 8, Some(-5), false, false, 384),
        (Format::Ddp, 8, Some(1), false, false, 384),
        (Format::Ddp, 8, Some(100), false, false, 384),
        (Format::Ddp, 8, Some(200), false, false, 384),
        (Format::Ddp, 8, Some(244), false, false, 384),
        (Format::Ddp, 8, Some(288), false, false, 384),
        (Format::Ddp, 8, Some(900), false, false, 896),
        (Format::Ddp, 8, Some(1100), false, false, 1024),
        (Format::Ddp, 8, Some(1200), false, false, 1280),
        (Format::Ddp, 8, Some(1700), false, false, 1664),
        (Format::Ddp, 8, Some(5000), false, false, 1664),
        (Format::Ddp, 8, None, true, false, 1024),
        (Format::Ddp, 8, Some(0), true, false, 1024),
        (Format::Ddp, 8, Some(-5), true, false, 384),
        (Format::Ddp, 8, Some(1), true, false, 384),
        (Format::Ddp, 8, Some(100), true, false, 384),
        (Format::Ddp, 8, Some(200), true, false, 384),
        (Format::Ddp, 8, Some(244), true, false, 384),
        (Format::Ddp, 8, Some(288), true, false, 384),
        (Format::Ddp, 8, Some(900), true, false, 896),
        (Format::Ddp, 8, Some(1100), true, false, 1024),
        (Format::Ddp, 8, Some(1200), true, false, 1024),
        (Format::Ddp, 8, Some(1700), true, false, 1024),
        (Format::Ddp, 8, Some(5000), true, false, 1024),
        (Format::Ddp, 8, None, false, true, 1536),
        (Format::Ddp, 8, Some(0), false, true, 1536),
        (Format::Ddp, 8, Some(-5), false, true, 768),
        (Format::Ddp, 8, Some(1), false, true, 768),
        (Format::Ddp, 8, Some(100), false, true, 768),
        (Format::Ddp, 8, Some(200), false, true, 768),
        (Format::Ddp, 8, Some(244), false, true, 768),
        (Format::Ddp, 8, Some(288), false, true, 768),
        (Format::Ddp, 8, Some(900), false, true, 1024),
        (Format::Ddp, 8, Some(1100), false, true, 1024),
        (Format::Ddp, 8, Some(1200), false, true, 1280),
        (Format::Ddp, 8, Some(1700), false, true, 1664),
        (Format::Ddp, 8, Some(5000), false, true, 1664),
        (Format::Ddp, 8, None, true, true, 1024),
        (Format::Ddp, 8, Some(0), true, true, 1024),
        (Format::Ddp, 8, Some(-5), true, true, 384),
        (Format::Ddp, 8, Some(1), true, true, 384),
        (Format::Ddp, 8, Some(100), true, true, 384),
        (Format::Ddp, 8, Some(200), true, true, 384),
        (Format::Ddp, 8, Some(244), true, true, 384),
        (Format::Ddp, 8, Some(288), true, true, 384),
        (Format::Ddp, 8, Some(900), true, true, 896),
        (Format::Ddp, 8, Some(1100), true, true, 1024),
        (Format::Ddp, 8, Some(1200), true, true, 1024),
        (Format::Ddp, 8, Some(1700), true, true, 1024),
        (Format::Ddp, 8, Some(5000), true, true, 1024),
        (Format::Ac4, 2, None, false, false, 320),
        (Format::Ac4, 2, Some(0), false, false, 320),
        (Format::Ac4, 2, Some(-5), false, false, 64),
        (Format::Ac4, 2, Some(1), false, false, 64),
        (Format::Ac4, 2, Some(100), false, false, 112),
        (Format::Ac4, 2, Some(200), false, false, 144),
        (Format::Ac4, 2, Some(244), false, false, 256),
        (Format::Ac4, 2, Some(288), false, false, 256),
        (Format::Ac4, 2, Some(900), false, false, 320),
        (Format::Ac4, 2, Some(1100), false, false, 320),
        (Format::Ac4, 2, Some(1200), false, false, 320),
        (Format::Ac4, 2, Some(1700), false, false, 320),
        (Format::Ac4, 2, Some(5000), false, false, 320),
    ];

    #[test]
    fn bitrates_match_python() {
        for &(format, channels, requested, fs, fb, expected) in PYTHON_BITRATES {
            assert_eq!(
                br(format, channels, requested, fs, fb),
                Some(expected),
                "{format:?} {channels} {requested:?} fs={fs} fb={fb}"
            );
        }
    }

    #[test]
    fn encoder_modes() {
        let m = |f, ch, b, fs, fb| encoder_mode(f, ch, b, fs, fb);
        assert_eq!(m(Format::Dd, 6, 640, false, false), "dd");
        assert_eq!(m(Format::Dd, 6, 640, false, true), "dd");
        assert_eq!(m(Format::Ddp, 6, 1024, false, false), "ddp");
        assert_eq!(m(Format::Ddp, 2, 256, false, true), "ddp");
        assert_eq!(m(Format::Ddp, 8, 1024, false, false), "ddp71");
        assert_eq!(m(Format::Ddp, 8, 1536, false, false), "bluray");
        assert_eq!(m(Format::Ddp, 8, 1536, true, false), "ddp71");
        assert_eq!(m(Format::Ddp, 8, 384, false, true), "bluray");
        assert_eq!(m(Format::Ddp, 8, 1024, true, true), "bluray");
    }

    #[test]
    fn extensions() {
        let e = output_extension;
        assert_eq!(e(Format::Dd, 640, false, true), "ac3");
        assert_eq!(e(Format::Ac4, 320, false, false), "ac4");
        assert_eq!(e(Format::Thd, 0, true, true), "thd");
        assert_eq!(e(Format::Ddp, 1024, false, false), "ec3");
        assert_eq!(e(Format::Ddp, 1536, false, false), "eb3");
        assert_eq!(e(Format::Ddp, 1536, true, false), "ec3");
        assert_eq!(e(Format::Ddp, 384, false, true), "eb3");
        assert_eq!(e(Format::Ddp, 1536, true, true), "eb3");
        // like Python, independent of the channel layout
        assert_eq!(e(Format::Ddp, 256, false, true), "eb3");
    }

    #[test]
    fn delay_from_name() {
        assert_eq!(input_delay("a/b.mkv", None), "+0ms");
        assert_eq!(input_delay("a/b DELAY -24ms.wav", None), "-24ms");
        assert_eq!(input_delay("a/b DELAY +5ms.wav", None), "+5ms");
        assert_eq!(input_delay("a/b DELAY 5s.wav", None), "+5s");
        assert_eq!(input_delay("a/b DELAY 5s.wav", Some("-d")), "-d");
        assert_eq!(input_delay("a/b DELAY 5s.wav", Some("")), "+5s");
        assert_eq!(input_delay("a/b DELAY 5s", None), "+0ms");
        assert_eq!(input_delay("DELAY 5s.wav", None), "+0ms");
        assert_eq!(input_delay("x DELAY 1ms.a DELAY -2ms.wav", None), "-2ms");
        assert_eq!(input_delay("a/b.mkv", Some("+1s")), "+1s");
    }
}
