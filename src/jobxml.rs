//! Construction of the DEE job XML.

use crate::data::{XML_AC4, XML_DD_DDP, XML_THD};
use crate::plan::{Format, encoder_mode, output_extension};
use crate::util::{basename, convert_delay_to_ms};
use crate::xml::{rename_element, set_text};

/// Everything the XML depends on except the input file and its delay.
#[derive(Debug, Clone)]
pub struct XmlParams<'a> {
    pub format: Format,
    pub outchannels: i64,
    /// Selected bitrate (0 for TrueHD, which has none).
    pub bitrate: u32,
    pub downmix_config: &'a str,
    pub drc: &'a str,
    pub dialnorm: i64,
    pub force_standard: bool,
    pub force_bluray: bool,
    /// Quoted output directory in the notation of DEE (see `util::wpc`).
    pub output_path: &'a str,
    /// Quoted temp directory in the notation of DEE.
    pub temp_path: &'a str,
}

/// The XML for one input. `delay` is a signed delay as accepted by
/// `convert_delay_to_ms`; `Err` if that rejects it.
pub fn build_xml(p: &XmlParams, input: &str, delay: &str) -> Result<String, ()> {
    let name = |ext: &str| basename(input, ext, true, false, true);
    let ext = output_extension(p.format, p.bitrate, p.force_standard, p.force_bluray);
    let (compensate, filter) = match p.format {
        Format::Dd | Format::Ddp => (true, "pcm_to_ddp"),
        Format::Ac4 => (true, "encode_to_ims_ac4"),
        Format::Thd => (false, "encode_to_dthd"),
    };
    let (_, delay_xml, delay_mode) = convert_delay_to_ms(delay, compensate)?;

    let mut xml = match p.format {
        Format::Dd | Format::Ddp => XML_DD_DDP,
        Format::Ac4 => XML_AC4,
        Format::Thd => XML_THD,
    }
    .to_string();
    let mut set = |path: &[&str], value: &str| xml = set_text(&xml, path, value);

    let filter_path = |rest: &[&'static str]| {
        let mut path = vec!["job_config", "filter", "audio", filter];
        path.extend_from_slice(rest);
        path
    };
    let output_name = match p.format {
        Format::Dd | Format::Ddp => "ec3",
        Format::Ac4 => "ac4",
        Format::Thd => "mlp",
    };
    let output_path = |leaf: &'static str| vec!["job_config", "output", output_name, leaf];

    match p.format {
        Format::Dd | Format::Ddp => {
            set(
                &["job_config", "output", "ec3", "storage", "local", "path"],
                p.output_path,
            );
            let mode = encoder_mode(
                p.format,
                p.outchannels,
                p.bitrate,
                p.force_standard,
                p.force_bluray,
            );
            set(&filter_path(&["encoder_mode"]), mode);
            set(&filter_path(&["downmix_config"]), p.downmix_config);
            set(&filter_path(&["data_rate"]), &p.bitrate.to_string());
            set(&filter_path(&["drc", "line_mode_drc_profile"]), p.drc);
            set(&filter_path(&["drc", "rf_mode_drc_profile"]), p.drc);
            set(&filter_path(&["custom_dialnorm"]), &p.dialnorm.to_string());
        }
        Format::Ac4 => {
            set(
                &["job_config", "output", "ac4", "storage", "local", "path"],
                p.output_path,
            );
            set(&filter_path(&["data_rate"]), &p.bitrate.to_string());
            for profile in [
                "ddp_drc_profile",
                "flat_panel_drc_profile",
                "home_theatre_drc_profile",
                "portable_hp_drc_profile",
                "portable_spkr_drc_profile",
            ] {
                set(&filter_path(&["drc", profile]), p.drc);
            }
        }
        Format::Thd => {
            set(
                &["job_config", "output", "mlp", "storage", "local", "path"],
                p.output_path,
            );
            for presentation in [
                "atmos_presentation",
                "presentation_8ch",
                "presentation_6ch",
                "presentation_2ch",
            ] {
                set(&filter_path(&[presentation, "drc_profile"]), p.drc);
            }
            set(&filter_path(&["custom_dialnorm"]), &p.dialnorm.to_string());
        }
    }
    set(
        &[
            "job_config",
            "input",
            "audio",
            "wav",
            "storage",
            "local",
            "path",
        ],
        p.temp_path,
    );
    set(&["job_config", "misc", "temp_dir", "path"], p.temp_path);

    set(
        &["job_config", "input", "audio", "wav", "file_name"],
        &basename(input, "wav", true, false, false),
    );
    set(&output_path("file_name"), &name(ext));
    set(&filter_path(&[delay_mode.xml_tag()]), &delay_xml);

    if p.format == Format::Dd {
        xml = rename_element(&xml, &["job_config", "output", "ec3"], "ac3");
    }
    Ok(xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(format: Format) -> XmlParams<'static> {
        XmlParams {
            format,
            outchannels: 6,
            bitrate: 640,
            downmix_config: "off",
            drc: "music_light",
            dialnorm: 0,
            force_standard: false,
            force_bluray: false,
            output_path: "\"/out\"",
            temp_path: "\"/tmp/deew\"",
        }
    }

    fn element(xml: &str, tag: &str) -> String {
        let open = format!("<{tag}>");
        let start = xml.find(&open).unwrap_or_else(|| panic!("no <{tag}>")) + open.len();
        let end = start + xml[start..].find(&format!("</{tag}>")).unwrap();
        xml[start..end].to_string()
    }

    #[test]
    fn ddp_51() {
        let mut p = params(Format::Ddp);
        p.bitrate = 1024;
        let xml = build_xml(&p, "/in/My Song.flac", "+0ms").unwrap();
        assert_eq!(element(&xml, "encoder_mode"), "ddp");
        assert_eq!(element(&xml, "data_rate"), "1024");
        assert_eq!(element(&xml, "downmix_config"), "off");
        assert_eq!(element(&xml, "line_mode_drc_profile"), "music_light");
        assert_eq!(element(&xml, "rf_mode_drc_profile"), "music_light");
        assert_eq!(element(&xml, "custom_dialnorm"), "0");
        // DDP is "+0ms" minus 16/3 ms: a negative delay -> `start`
        assert_eq!(element(&xml, "start"), "0:00:00.005333");
        assert_eq!(element(&xml, "prepend_silence_duration"), "0.0");
        assert!(xml.contains("<ec3 version=\"1\">"));
        assert!(xml.contains("<file_name>\"My Song.ec3\"</file_name>"));
        assert!(xml.contains("<file_name>\"My Song.wav\"</file_name>"));
        assert!(xml.contains("<path>\"/out\"</path>"));
        assert_eq!(xml.matches("<path>\"/tmp/deew\"</path>").count(), 2);
        assert!(xml.starts_with("<?xml version=\"1.0\"?>"));
        assert!(xml.contains("<!-- One of: not_indicated"));
    }

    #[test]
    fn dd_renames_the_output_element() {
        let xml = build_xml(&params(Format::Dd), "a.wav", "+0ms").unwrap();
        assert_eq!(element(&xml, "encoder_mode"), "dd");
        assert!(xml.contains("<ac3 version=\"1\">") && xml.contains("</ac3>"));
        assert!(!xml.contains("ec3"));
        assert!(xml.contains("<file_name>\"a.ac3\"</file_name>"));
    }

    #[test]
    fn ddp_71_modes_and_extensions() {
        let mut p = params(Format::Ddp);
        p.outchannels = 8;
        p.bitrate = 1536;
        let xml = build_xml(&p, "a.wav", "+0ms").unwrap();
        assert_eq!(element(&xml, "encoder_mode"), "bluray");
        assert!(xml.contains("\"a.eb3\""));
        p.bitrate = 1024;
        let xml = build_xml(&p, "a.wav", "+0ms").unwrap();
        assert_eq!(element(&xml, "encoder_mode"), "ddp71");
        assert!(xml.contains("\"a.ec3\""));
        p.force_bluray = true;
        let xml = build_xml(&p, "a.wav", "+0ms").unwrap();
        assert_eq!(element(&xml, "encoder_mode"), "bluray");
        assert!(xml.contains("\"a.eb3\""));
    }

    #[test]
    fn delay_modes() {
        let p = params(Format::Ddp);
        let xml = build_xml(&p, "a.wav", "+100ms").unwrap();
        assert_eq!(element(&xml, "prepend_silence_duration"), "0.094667");
        assert_eq!(element(&xml, "start"), "00:00:00.0");
        let xml = build_xml(&p, "a.wav", "-24ms").unwrap();
        assert_eq!(element(&xml, "start"), "0:00:00.029333");
        assert_eq!(element(&xml, "prepend_silence_duration"), "0.0");
        assert_eq!(build_xml(&p, "a.wav", "24ms"), Err(()));
    }

    #[test]
    fn ac4() {
        let mut p = params(Format::Ac4);
        p.bitrate = 320;
        p.outchannels = 2;
        p.drc = "speech";
        let xml = build_xml(&p, "/x/a DELAY -24ms.mka", "-24ms").unwrap();
        assert_eq!(element(&xml, "data_rate"), "320");
        for tag in [
            "ddp_drc_profile",
            "flat_panel_drc_profile",
            "home_theatre_drc_profile",
            "portable_hp_drc_profile",
            "portable_spkr_drc_profile",
        ] {
            assert_eq!(element(&xml, tag), "speech");
        }
        assert!(xml.contains("<ac4 version=\"1\">"));
        assert!(xml.contains("<file_name>\"a.ac4\"</file_name>"));
        assert!(xml.contains("<file_name>\"a DELAY -24ms.wav\"</file_name>"));
        assert_eq!(element(&xml, "start"), "0:00:00.029333");
        assert!(!xml.contains("custom_dialnorm"));
    }

    #[test]
    fn thd_has_no_delay_compensation() {
        let mut p = params(Format::Thd);
        p.dialnorm = -27;
        p.drc = "film_standard";
        let xml = build_xml(&p, "a.wav", "+0ms").unwrap();
        assert_eq!(element(&xml, "prepend_silence_duration"), "0.000000");
        assert_eq!(element(&xml, "start"), "first_frame_of_action");
        assert_eq!(element(&xml, "custom_dialnorm"), "-27");
        assert!(xml.contains("<mlp version=\"1\">"));
        assert!(xml.contains("<file_name>\"a.thd\"</file_name>"));
        assert_eq!(
            xml.matches("<drc_profile>film_standard</drc_profile>")
                .count(),
            4
        );
        let xml = build_xml(&p, "a.wav", "-1s").unwrap();
        assert_eq!(element(&xml, "start"), "0:00:01.0");
        let xml = build_xml(&p, "a.wav", "-1.5s").unwrap();
        assert_eq!(element(&xml, "start"), "0:00:01.500000");
    }

    #[test]
    fn wsl_notation_and_stripped_delay() {
        let mut p = params(Format::Ddp);
        p.output_path = "\"C:\\out\\\"";
        p.temp_path = "\"C:\\tmp\\\"";
        let xml = build_xml(&p, "/mnt/c/in/Ä b DELAY -24ms.wav", "-24ms").unwrap();
        assert!(xml.contains("<path>\"C:\\out\\\"</path>"));
        assert_eq!(xml.matches("<path>\"C:\\tmp\\\"</path>").count(), 2);
        assert!(xml.contains("<file_name>\"Ä b.ec3\"</file_name>"));
    }

    /// Flatten an XML document to `path -> text`, attributes as `path@name`.
    fn flatten(xml: &str) -> std::collections::BTreeMap<String, String> {
        let mut out = std::collections::BTreeMap::new();
        let mut stack: Vec<&str> = Vec::new();
        let mut rest = xml;
        while let Some(lt) = rest.find('<') {
            let text = rest[..lt].trim();
            if !text.is_empty() {
                let path = format!("/{}", stack.join("/"));
                out.insert(path, text.replace("&lt;", "<").replace("&gt;", ">"));
            }
            rest = &rest[lt..];
            if rest.starts_with("<?") {
                rest = &rest[rest.find("?>").unwrap() + 2..];
            } else if rest.starts_with("<!--") {
                rest = &rest[rest.find("-->").unwrap() + 3..];
            } else if rest.starts_with("</") {
                stack.pop();
                rest = &rest[rest.find('>').unwrap() + 1..];
            } else {
                let gt = rest.find('>').unwrap();
                let tag = &rest[1..gt];
                let name_end = tag.find(char::is_whitespace).unwrap_or(tag.len());
                stack.push(&tag[..name_end]);
                let mut attrs = tag[name_end..].trim();
                while let Some((name, after)) = attrs.split_once("=\"") {
                    let (value, after) = after.split_once('"').unwrap();
                    out.insert(
                        format!("/{}@{}", stack.join("/"), name.trim()),
                        value.to_string(),
                    );
                    attrs = after.trim();
                }
                rest = &rest[gt + 1..];
            }
        }
        out
    }

    /// The expected documents come from the Python implementation
    /// (`xmltodict.parse` of what its `save_xml` writes).
    #[test]
    fn documents_match_python() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/job_xml_cases.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let p = &case["params"];
            let s = |key: &str| p[key].as_str().unwrap().to_string();
            let b = |key: &str| p[key].as_bool().unwrap();
            let params = XmlParams {
                format: Format::parse(&s("format")).unwrap(),
                outchannels: p["outchannels"].as_i64().unwrap(),
                bitrate: p["bitrate"].as_u64().unwrap() as u32,
                downmix_config: &s("downmix_config"),
                drc: &s("drc"),
                dialnorm: p["dialnorm"].as_i64().unwrap(),
                force_standard: b("force_standard"),
                force_bluray: b("force_bluray"),
                output_path: &s("output_path"),
                temp_path: &s("temp_path"),
            };
            let input = s("input");
            let delay = crate::plan::input_delay(&input, p["delay_arg"].as_str());
            assert_eq!(delay, s("delay"), "{p}");
            let xml = build_xml(&params, &input, &delay).unwrap();
            let expected: std::collections::BTreeMap<String, String> = case["expected"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
                .collect();
            assert_eq!(flatten(&xml), expected, "{p}");
        }
    }
}
