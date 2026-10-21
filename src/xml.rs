//! Minimal in-place editing of the static DEE job templates. Editing the text
//! (instead of parsing and re-serialising) keeps the comments intact.

use std::ops::Range;

struct Element {
    /// Start of the `<name` open tag.
    open: usize,
    /// Text between the open tag and the matching `</name>`.
    content: Range<usize>,
}

/// Position of the next `<` in `xml[pos..end]` that is not inside a comment.
fn next_lt(xml: &str, mut pos: usize, end: usize) -> Option<usize> {
    while let Some(i) = xml[pos..end].find('<') {
        let i = pos + i;
        if xml[i..end].starts_with("<!--") {
            pos = xml[i..end].find("-->").map_or(end, |c| i + c + 3);
        } else {
            return Some(i);
        }
    }
    None
}

/// Name of the tag starting at `xml[lt..]` (`<name ...>` or `</name>`) and
/// whether it is a closing tag.
fn tag_name(xml: &str, lt: usize) -> (&str, bool) {
    let rest = &xml[lt + 1..];
    let (closing, rest) = rest.strip_prefix('/').map_or((false, rest), |r| (true, r));
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(rest.len());
    (&rest[..end], closing)
}

/// First element called `name` inside `range`, at any depth.
fn find_element(xml: &str, range: Range<usize>, name: &str) -> Option<Element> {
    let mut pos = range.start;
    while let Some(lt) = next_lt(xml, pos, range.end) {
        pos = lt + 1;
        if tag_name(xml, lt) != (name, false) {
            continue;
        }
        let gt = lt + xml[lt..range.end].find('>')?;
        assert!(
            !xml[..gt].ends_with('/'),
            "self-closing <{name}/> is not supported"
        );
        let content_start = gt + 1;
        // Find the matching end tag, skipping nested elements of the same name.
        let (mut depth, mut p) = (1, content_start);
        while let Some(lt2) = next_lt(xml, p, range.end) {
            p = lt2 + 1;
            if tag_name(xml, lt2).0 != name {
                continue;
            }
            if tag_name(xml, lt2).1 {
                depth -= 1;
                if depth == 0 {
                    return Some(Element {
                        open: lt,
                        content: content_start..lt2,
                    });
                }
            } else {
                depth += 1;
            }
        }
        return None;
    }
    None
}

fn locate(xml: &str, path: &[&str]) -> Element {
    let mut range = 0..xml.len();
    let mut found = None;
    for name in path {
        let el = find_element(xml, range, name)
            .unwrap_or_else(|| panic!("element path not found in template: {}", path.join("/")));
        range = el.content.clone();
        found = Some(el);
    }
    found.expect("element path must not be empty")
}

/// Replace the content of the element at `path` with `value`. `<` and `>` are
/// escaped but `&` is written raw, like the Python (`.replace('&amp;', '&')`).
/// Panics if the path does not exist: templates are static, so that is a bug.
pub fn set_text(xml: &str, path: &[&str], value: &str) -> String {
    let el = locate(xml, path);
    let value = value.replace('<', "&lt;").replace('>', "&gt;");
    format!(
        "{}{value}{}",
        &xml[..el.content.start],
        &xml[el.content.end..]
    )
}

/// Rename the element at `path` (start and end tag), keeping its attributes.
pub fn rename_element(xml: &str, path: &[&str], new_name: &str) -> String {
    let el = locate(xml, path);
    let old = path.last().expect("element path must not be empty");
    let name_start = el.open + 1;
    // The end tag is the last `</old` before the end of the content range.
    let close = el.content.end + 2;
    format!(
        "{}{new_name}{}{new_name}{}",
        &xml[..name_start],
        &xml[name_start + old.len()..close],
        &xml[close + old.len()..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{XML_AC4, XML_DD_DDP, XML_THD};

    /// Lines that differ between two documents (the edits never add lines).
    fn changed_lines<'a>(a: &'a str, b: &'a str) -> Vec<(&'a str, &'a str)> {
        assert_eq!(a.lines().count(), b.lines().count());
        a.lines().zip(b.lines()).filter(|(x, y)| x != y).collect()
    }

    const DATA_RATE: [&str; 5] = ["job_config", "filter", "audio", "pcm_to_ddp", "data_rate"];

    #[test]
    fn sets_value_and_keeps_comment() {
        let out = set_text(XML_DD_DDP, &DATA_RATE, "640");
        let changed = changed_lines(XML_DD_DDP, &out);
        assert_eq!(changed.len(), 1);
        assert!(
            changed[0]
                .0
                .starts_with("        <data_rate>-</data_rate>    <!-- One of: 32,")
        );
        assert!(
            changed[0]
                .1
                .starts_with("        <data_rate>640</data_rate>    <!-- One of: 32,")
        );
        assert_eq!(changed[0].0.replacen('-', "640", 1), changed[0].1);
    }

    #[test]
    fn start_does_not_match_starting_timecode() {
        let p = ["job_config", "filter", "audio", "pcm_to_ddp", "start"];
        let out = set_text(XML_DD_DDP, &p, "0:00:01.5");
        let changed = changed_lines(XML_DD_DDP, &out);
        assert_eq!(changed.len(), 1);
        assert!(changed[0].1.contains("<start>0:00:01.5</start>"));
        assert!(out.contains("<starting_timecode>off</starting_timecode>"));
    }

    #[test]
    fn same_named_paths_are_distinct() {
        let temp = ["job_config", "misc", "temp_dir", "path"];
        let out = set_text(XML_DD_DDP, &temp, "\"/tmp/x\"");
        let changed = changed_lines(XML_DD_DDP, &out);
        assert_eq!(
            changed,
            [("      <path>-</path>", "      <path>\"/tmp/x\"</path>")]
        );

        let wav = [
            "job_config",
            "input",
            "audio",
            "wav",
            "storage",
            "local",
            "path",
        ];
        let out = set_text(XML_DD_DDP, &wav, "\"/tmp/y\"");
        let changed = changed_lines(XML_DD_DDP, &out);
        assert_eq!(changed.len(), 1);
        assert!(
            changed[0]
                .1
                .starts_with("            <path>\"/tmp/y\"</path>    <!-- string -->")
        );

        let output = ["job_config", "output", "ec3", "storage", "local", "path"];
        let out = set_text(XML_DD_DDP, &output, "z");
        assert_eq!(changed_lines(XML_DD_DDP, &out).len(), 1);
        assert_eq!(out.matches("<path>-</path>").count(), 2);
    }

    #[test]
    fn tab_indented_line() {
        let p = [
            "job_config",
            "filter",
            "audio",
            "pcm_to_ddp",
            "custom_dialnorm",
        ];
        let out = set_text(XML_DD_DDP, &p, "-27");
        assert_eq!(
            changed_lines(XML_DD_DDP, &out),
            [(
                "\t\t<custom_dialnorm>-</custom_dialnorm>",
                "\t\t<custom_dialnorm>-27</custom_dialnorm>"
            )]
        );
    }

    #[test]
    fn renames_element() {
        let out = rename_element(XML_DD_DDP, &["job_config", "output", "ec3"], "ac3");
        let changed = changed_lines(XML_DD_DDP, &out);
        assert_eq!(
            changed,
            [
                ("    <ec3 version=\"1\">", "    <ac3 version=\"1\">"),
                ("    </ec3>", "    </ac3>"),
            ]
        );
        // Children are still reachable under the new name.
        let out = set_text(
            &out,
            &["job_config", "output", "ac3", "file_name"],
            "\"a.ac3\"",
        );
        assert!(
            out.contains("<file_name>\"a.ac3\"</file_name>    <!-- string -->\n      <storage>")
        );
    }

    #[test]
    fn escapes_angle_brackets_but_not_ampersand() {
        let p = ["job_config", "output", "ec3", "file_name"];
        let out = set_text(XML_DD_DDP, &p, "a & b <c>.ec3");
        assert!(out.contains("<file_name>a & b &lt;c&gt;.ec3</file_name>"));
    }

    #[test]
    #[should_panic(expected = "element path not found in template")]
    fn missing_path_panics() {
        set_text(XML_DD_DDP, &["job_config", "nope"], "x");
    }

    #[test]
    fn comments_are_not_searched() {
        let xml = "<a><!-- <b>no</b> --><b>yes</b></a>";
        assert_eq!(
            set_text(xml, &["a", "b"], "1"),
            "<a><!-- <b>no</b> --><b>1</b></a>"
        );
    }

    #[test]
    fn nested_same_name() {
        let xml = "<a><a>x</a>y</a>";
        assert_eq!(set_text(xml, &["a"], "1"), "<a>1</a>");
    }

    /// Every path the encode pipeline sets must exist in its template.
    #[test]
    fn pipeline_paths_exist() {
        let common = [
            "input/audio/wav/file_name",
            "input/audio/wav/storage/local/path",
            "misc/temp_dir/path",
        ];
        let cases: [(&str, &[&str]); 3] = [
            (
                XML_DD_DDP,
                &[
                    "output/ec3/file_name",
                    "output/ec3/storage/local/path",
                    "filter/audio/pcm_to_ddp/encoder_mode",
                    "filter/audio/pcm_to_ddp/downmix_config",
                    "filter/audio/pcm_to_ddp/data_rate",
                    "filter/audio/pcm_to_ddp/drc/line_mode_drc_profile",
                    "filter/audio/pcm_to_ddp/drc/rf_mode_drc_profile",
                    "filter/audio/pcm_to_ddp/custom_dialnorm",
                    "filter/audio/pcm_to_ddp/start",
                    "filter/audio/pcm_to_ddp/prepend_silence_duration",
                ],
            ),
            (
                XML_AC4,
                &[
                    "output/ac4/file_name",
                    "output/ac4/storage/local/path",
                    "filter/audio/encode_to_ims_ac4/data_rate",
                    "filter/audio/encode_to_ims_ac4/drc/ddp_drc_profile",
                    "filter/audio/encode_to_ims_ac4/drc/flat_panel_drc_profile",
                    "filter/audio/encode_to_ims_ac4/drc/home_theatre_drc_profile",
                    "filter/audio/encode_to_ims_ac4/drc/portable_hp_drc_profile",
                    "filter/audio/encode_to_ims_ac4/drc/portable_spkr_drc_profile",
                    "filter/audio/encode_to_ims_ac4/start",
                    "filter/audio/encode_to_ims_ac4/prepend_silence_duration",
                ],
            ),
            (
                XML_THD,
                &[
                    "output/mlp/file_name",
                    "output/mlp/storage/local/path",
                    "filter/audio/encode_to_dthd/atmos_presentation/drc_profile",
                    "filter/audio/encode_to_dthd/presentation_8ch/drc_profile",
                    "filter/audio/encode_to_dthd/presentation_6ch/drc_profile",
                    "filter/audio/encode_to_dthd/presentation_2ch/drc_profile",
                    "filter/audio/encode_to_dthd/custom_dialnorm",
                    "filter/audio/encode_to_dthd/start",
                    "filter/audio/encode_to_dthd/prepend_silence_duration",
                ],
            ),
        ];
        for (template, specific) in cases {
            for p in common.iter().chain(specific) {
                let path: Vec<&str> = std::iter::once("job_config").chain(p.split('/')).collect();
                let out = set_text(template, &path, "MARK");
                let changed = changed_lines(template, &out);
                assert_eq!(changed.len(), 1, "{p}");
                assert!(changed[0].1.contains(">MARK</"), "{p}");
            }
        }
    }
}
