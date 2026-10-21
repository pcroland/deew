//! ffmpeg and DEE command lines (argument lists and the markup printed for
//! them). Everything here is pure.

use crate::error::escape;
use crate::plan::Format;

const SWAP_71: &str = "pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5";

/// How an input is converted to the intermediate wav. Inputs share their
/// sample rate and channels, so only the bit depth can differ between plans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioPlan {
    /// Bit depth of the intermediate (32 when resampling).
    pub bit_depth: u32,
    pub map_args: Vec<String>,
    pub map_print: String,
    pub swap_args: Vec<String>,
    pub swap_print: String,
    pub resample_args: Vec<String>,
    pub resample_print: String,
}

/// Target sample rate if the input has to be resampled.
fn resample_rate(format: Format, samplerate: i64) -> Option<&'static str> {
    match format {
        Format::Dd | Format::Ddp | Format::Ac4 if samplerate != 48000 => Some("48000"),
        Format::Thd if samplerate != 48000 && samplerate != 96000 => {
            Some(if samplerate < 72000 { "48000" } else { "96000" })
        }
        _ => None,
    }
}

pub fn audio_plan(
    format: Format,
    samplerate: i64,
    channels: i64,
    bit_depth: u32,
    track_index: i64,
) -> AudioPlan {
    let ti = track_index;
    let rate = resample_rate(format, samplerate);
    let bit_depth = if rate.is_some() { 32 } else { bit_depth };

    let (resample_args, resample_print) = match rate {
        Some(rate) => {
            let swap = if channels == 8 {
                format!("{SWAP_71},")
            } else {
                String::new()
            };
            (
                [
                    "-filter_complex",
                    &format!("[a:{ti}]{swap}aresample=resampler=soxr"),
                    "-ar",
                    rate,
                    "-precision",
                    "28",
                    "-cutoff",
                    "1",
                    "-dither_scale",
                    "0",
                ]
                .map(String::from)
                .to_vec(),
                format!(
                    "-filter_complex [bold color(231)]\"\\[a:{ti}]{swap}aresample=resampler=soxr\"[/bold color(231)] \
-ar [bold color(231)]{rate}[/bold color(231)] \
-precision [bold color(231)]28[/bold color(231)] \
-cutoff [bold color(231)]1[/bold color(231)] \
-dither_scale [bold color(231)]0[/bold color(231)] "
                ),
            )
        }
        None => (Vec::new(), String::new()),
    };

    let (swap_args, swap_print) = if channels == 8 && resample_args.is_empty() {
        (
            vec!["-filter_complex".to_string(), format!("[a:{ti}]{SWAP_71}")],
            format!("-filter_complex [bold color(231)]\"\\[a:{ti}]{SWAP_71}\"[/bold color(231)] "),
        )
    } else {
        (Vec::new(), String::new())
    };

    let (map_args, map_print) = if resample_args.is_empty() && swap_args.is_empty() {
        (
            vec!["-map".to_string(), format!("0:a:{ti}")],
            format!(
                "-map [bold color(231)]0:a[/bold color(231)][bold color(231)]:{ti}[/bold color(231)] "
            ),
        )
    } else {
        (Vec::new(), String::new())
    };

    AudioPlan {
        bit_depth,
        map_args,
        map_print,
        swap_args,
        swap_print,
        resample_args,
        resample_print,
    }
}

pub fn ffmpeg_args(ffmpeg: &str, plan: &AudioPlan, input: &str, wav: &str) -> Vec<String> {
    let mut args: Vec<String> = [ffmpeg, "-y", "-drc_scale", "0", "-i", input]
        .map(String::from)
        .to_vec();
    args.extend(plan.map_args.iter().cloned());
    args.push("-c".to_string());
    args.push(format!("pcm_s{}le", plan.bit_depth));
    args.extend(plan.swap_args.iter().cloned());
    args.extend(plan.resample_args.iter().cloned());
    args.extend(["-rf64", "always", wav].map(String::from));
    args
}

fn ffmpeg_print_with(plan: &AudioPlan, depth: &str, input: &str, output: &str) -> String {
    format!(
        "[bold cyan]ffmpeg[/bold cyan] -y -drc_scale [bold color(231)]0[/bold color(231)] \
-i [bold green]{input}[/bold green] \
{}-c [bold color(231)]{}[/bold color(231)] \
{}{}-rf64 [bold color(231)]always[/bold color(231)] \
[bold magenta]{output}[/bold magenta]",
        plan.map_print, depth, plan.swap_print, plan.resample_print
    )
}

/// The command as printed with `-la` (markup).
pub fn ffmpeg_print(plan: &AudioPlan, input: &str, wav: &str) -> String {
    let depth = format!("pcm_s{}le", plan.bit_depth);
    ffmpeg_print_with(plan, &depth, &escape(input), &escape(wav))
}

/// The codec of the short print: `pcm_s24le`, or for mixed bit depths the
/// distinct ones sorted, e.g. `pcm_s16le/pcm_s24le`.
fn short_codec(depths: &[u32]) -> String {
    let mut distinct = depths.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    let codecs: Vec<String> = distinct.iter().map(|d| format!("pcm_s{d}le")).collect();
    codecs.join("/")
}

/// The command as printed by default, with `[input]` and `[output]`.
/// `plans` are the plans of all inputs (not empty).
pub fn ffmpeg_print_short(plans: &[AudioPlan]) -> String {
    let depths: Vec<u32> = plans.iter().map(|p| p.bit_depth).collect();
    ffmpeg_print_with(&plans[0], &short_codec(&depths), "\\[input]", "\\[output]")
}

pub const INTERMEDIATE_EXISTS_PRINT: &str = "[green]Intermediate already exists[/green]";

pub fn dee_args(dee: &str, xml_input: &str, dee_is_exe: bool) -> Vec<String> {
    let mut args: Vec<String> = [
        dee,
        "--progress-interval",
        "500",
        "--diagnostics-interval",
        "90000",
        "-x",
        xml_input,
    ]
    .map(String::from)
    .to_vec();
    if !dee_is_exe {
        args.push("--disable-xml-validation".to_string());
    }
    args
}

fn validation_print(dee_is_exe: bool) -> &'static str {
    if dee_is_exe {
        ""
    } else {
        " --disable-xml-validation"
    }
}

pub fn dee_print(xml_input: &str, dee_is_exe: bool) -> String {
    format!(
        "[bold cyan]dee[/bold cyan] -x [bold magenta]{}[/bold magenta]{}",
        escape(xml_input),
        validation_print(dee_is_exe)
    )
}

pub fn dee_print_short(dee_is_exe: bool) -> String {
    format!(
        "[bold cyan]dee[/bold cyan] -x [bold magenta]\\[input][/bold magenta]{}",
        validation_print(dee_is_exe)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::strip;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    const SWAP: &str = "pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5";

    #[test]
    fn five_one_48k_maps_the_track() {
        let plan = audio_plan(Format::Ddp, 48000, 6, 16, 0);
        assert_eq!(
            ffmpeg_args("ffmpeg", &plan, "in.mkv", "/t/in.wav"),
            v(&[
                "ffmpeg",
                "-y",
                "-drc_scale",
                "0",
                "-i",
                "in.mkv",
                "-map",
                "0:a:0",
                "-c",
                "pcm_s16le",
                "-rf64",
                "always",
                "/t/in.wav"
            ])
        );
        let plan = audio_plan(Format::Ac4, 48000, 6, 24, 2);
        assert_eq!(plan.map_args, v(&["-map", "0:a:2"]));
        assert_eq!(plan.bit_depth, 24);
    }

    #[test]
    fn stereo_44k_resamples() {
        let plan = audio_plan(Format::Ddp, 44100, 2, 16, 0);
        assert_eq!(
            ffmpeg_args("ffmpeg", &plan, "in.flac", "/t/in.wav"),
            v(&[
                "ffmpeg",
                "-y",
                "-drc_scale",
                "0",
                "-i",
                "in.flac",
                "-c",
                "pcm_s32le",
                "-filter_complex",
                "[a:0]aresample=resampler=soxr",
                "-ar",
                "48000",
                "-precision",
                "28",
                "-cutoff",
                "1",
                "-dither_scale",
                "0",
                "-rf64",
                "always",
                "/t/in.wav"
            ])
        );
        assert!(plan.map_args.is_empty() && plan.swap_args.is_empty());
    }

    #[test]
    fn seven_one_48k_swaps_channels() {
        let plan = audio_plan(Format::Ddp, 48000, 8, 24, 1);
        assert_eq!(
            ffmpeg_args("ffmpeg", &plan, "in.mkv", "w.wav"),
            v(&[
                "ffmpeg",
                "-y",
                "-drc_scale",
                "0",
                "-i",
                "in.mkv",
                "-c",
                "pcm_s24le",
                "-filter_complex",
                &format!("[a:1]{SWAP}"),
                "-rf64",
                "always",
                "w.wav"
            ])
        );
        assert!(plan.map_args.is_empty() && plan.map_print.is_empty());
    }

    #[test]
    fn seven_one_44k_resamples_and_swaps() {
        let plan = audio_plan(Format::Ddp, 44100, 8, 24, 0);
        assert_eq!(plan.bit_depth, 32);
        assert!(plan.swap_args.is_empty() && plan.map_args.is_empty());
        assert_eq!(
            plan.resample_args[..2],
            v(&[
                "-filter_complex",
                &format!("[a:0]{SWAP},aresample=resampler=soxr")
            ])[..]
        );
    }

    #[test]
    fn thd_rates() {
        let rate = |sr, ch| audio_plan(Format::Thd, sr, ch, 24, 0);
        assert!(rate(48000, 6).resample_args.is_empty());
        assert!(rate(96000, 6).resample_args.is_empty());
        assert_eq!(rate(88200, 6).resample_args[3], "96000");
        assert_eq!(rate(192000, 6).resample_args[3], "96000");
        assert_eq!(rate(72000, 6).resample_args[3], "96000");
        assert_eq!(rate(71999, 6).resample_args[3], "48000");
        assert_eq!(rate(44100, 6).resample_args[3], "48000");
        assert_eq!(rate(88200, 6).bit_depth, 32);
        // DD/DDP/AC4 always go to 48 kHz, even from 96 kHz
        assert_eq!(
            audio_plan(Format::Dd, 96000, 6, 24, 0).resample_args[3],
            "48000"
        );
    }

    #[test]
    fn print_strings() {
        let plan = audio_plan(Format::Ddp, 48000, 6, 24, 0);
        assert_eq!(
            ffmpeg_print(&plan, "a [1].mkv", "/t/a [1].wav"),
            "[bold cyan]ffmpeg[/bold cyan] -y -drc_scale [bold color(231)]0[/bold color(231)] \
-i [bold green]a \\[1].mkv[/bold green] \
-map [bold color(231)]0:a[/bold color(231)][bold color(231)]:0[/bold color(231)] \
-c [bold color(231)]pcm_s24le[/bold color(231)] \
-rf64 [bold color(231)]always[/bold color(231)] \
[bold magenta]/t/a \\[1].wav[/bold magenta]"
        );
        assert_eq!(
            strip(&ffmpeg_print_short(std::slice::from_ref(&plan))),
            "ffmpeg -y -drc_scale 0 -i [input] -map 0:a:0 -c pcm_s24le -rf64 always [output]"
        );
        let plan = audio_plan(Format::Ddp, 44100, 8, 24, 0);
        assert_eq!(
            strip(&ffmpeg_print_short(std::slice::from_ref(&plan))),
            format!(
                "ffmpeg -y -drc_scale 0 -i [input] -c pcm_s32le -filter_complex \"[a:0]{SWAP},aresample=resampler=soxr\" -ar 48000 -precision 28 -cutoff 1 -dither_scale 0 -rf64 always [output]"
            )
        );
        let plan = audio_plan(Format::Ddp, 48000, 8, 24, 0);
        assert_eq!(
            strip(&ffmpeg_print_short(std::slice::from_ref(&plan))),
            format!(
                "ffmpeg -y -drc_scale 0 -i [input] -c pcm_s24le -filter_complex \"[a:0]{SWAP}\" -rf64 always [output]"
            )
        );
    }

    #[test]
    fn short_print_with_mixed_depths() {
        let plan = |d| audio_plan(Format::Ddp, 48000, 6, d, 0);
        let short = |plans: &[AudioPlan]| strip(&ffmpeg_print_short(plans));
        assert_eq!(
            short(&[plan(24), plan(16), plan(24)]),
            "ffmpeg -y -drc_scale 0 -i [input] -map 0:a:0 -c pcm_s16le/pcm_s24le -rf64 always [output]"
        );
        assert!(short(&[plan(32), plan(24), plan(16)]).contains("pcm_s16le/pcm_s24le/pcm_s32le"));
        assert!(short(&[plan(16), plan(16)]).contains("-c pcm_s16le -rf64"));
        // resampling forces 32 bit for every input
        let rs = |d| audio_plan(Format::Ddp, 44100, 6, d, 0);
        assert!(short(&[rs(16), rs(24)]).contains("-c pcm_s32le -filter_complex"));
    }

    #[test]
    fn dee_commands() {
        assert_eq!(
            dee_args("dee.exe", "C:\\t\\a.xml", true),
            v(&[
                "dee.exe",
                "--progress-interval",
                "500",
                "--diagnostics-interval",
                "90000",
                "-x",
                "C:\\t\\a.xml"
            ])
        );
        assert_eq!(
            dee_args("dee", "/t/a.xml", false).last().unwrap(),
            "--disable-xml-validation"
        );
        assert_eq!(
            strip(&dee_print("/t/a.xml", false)),
            "dee -x /t/a.xml --disable-xml-validation"
        );
        assert_eq!(strip(&dee_print_short(true)), "dee -x [input]");
        assert_eq!(
            strip(&dee_print_short(false)),
            "dee -x [input] --disable-xml-validation"
        );
    }
}
