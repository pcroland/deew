//! The external programs (DEE, ffmpeg, ffprobe) and the temp directory.

use crate::data::ErrorKind;
use crate::error::{error_line, escape, print_exit};
use crate::util::rwpc;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

/// What is known about the DEE binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeeKind {
    /// A Windows executable (`MZ` header).
    pub is_exe: bool,
    /// A Windows executable run on another OS (WSL).
    pub nonnative: bool,
}

impl DeeKind {
    pub fn new(is_exe: bool, on_windows: bool) -> DeeKind {
        DeeKind {
            is_exe,
            nonnative: is_exe && !on_windows,
        }
    }
}

fn is_mz(header: &[u8]) -> bool {
    header.starts_with(b"MZ")
}

/// Check that the three binaries can be found and detect the kind of DEE.
/// Exits with `BinaryExist` for the first one that cannot be found.
pub fn check_binaries(dee: &str, ffmpeg: &str, ffprobe: &str) -> DeeKind {
    let mut dee_path = PathBuf::new();
    for (i, name) in [dee, ffmpeg, ffprobe].into_iter().enumerate() {
        match which::which(name) {
            Ok(path) if i == 0 => dee_path = path,
            Ok(_) => {}
            Err(_) => print_exit(ErrorKind::BinaryExist, Some(name)),
        }
    }
    let mut header = [0u8; 2];
    let is_exe = std::fs::File::open(&dee_path)
        .and_then(|mut f| f.read_exact(&mut header))
        .is_ok()
        && is_mz(&header);
    DeeKind::new(is_exe, cfg!(windows))
}

/// `os.path.abspath`: absolute and lexically normalised. On Windows both
/// separators are accepted and the result uses `\`.
pub fn abspath(path: &str, cwd: &Path) -> String {
    let joined = cwd.join(path);
    // `D:dir` (a drive without a root) is relative to the current directory
    // of drive D, which only the OS knows.
    #[cfg(windows)]
    let joined = if joined.has_root() {
        joined
    } else {
        std::path::absolute(&joined).unwrap_or(joined)
    };
    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                // `..` above the root stays at the root.
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out.to_string_lossy().into_owned()
}

/// `ntpath.join(base, 'deew')`.
fn ntpath_join_deew(base: &str) -> String {
    if base.is_empty() {
        "deew".to_string()
    } else if base.ends_with(['\\', '/']) {
        format!("{base}deew")
    } else {
        format!("{base}\\deew")
    }
}

/// Temp directory below the Windows `%TEMP%` given its path (the output of
/// PowerShell), as a path on this system.
pub fn windows_temp_to_local(powershell_output: &str) -> String {
    rwpc(&ntpath_join_deew(powershell_output.trim()))
}

fn windows_temp() -> Result<String, String> {
    let output = Command::new("powershell.exe")
        .args(["(gi $env:TEMP).fullname"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| {
            format!("couldn't run powershell.exe to find the Windows temp directory: {e}")
        })?;
    Ok(windows_temp_to_local(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

/// Whether `path` is on a Windows drive mounted by WSL (`/mnt/<letter>`).
fn on_windows_drive(path: &Path) -> bool {
    path.to_string_lossy()
        .strip_prefix("/mnt/")
        .and_then(|rest| rest.split('/').next())
        .is_some_and(|drive| drive.len() == 1 && drive.as_bytes()[0].is_ascii_alphabetic())
}

/// The temp directory: the configured one, else `temp` next to the
/// executable, else `%TEMP%\deew` (WSL with the executable outside a Windows
/// drive, which DEE can't reach). Absolute and normalised, not yet created.
pub fn temp_path(configured: &str, nonnative: bool, exe_dir: &Path) -> String {
    let chosen = if !configured.is_empty() {
        configured.to_string()
    } else if nonnative && !on_windows_drive(exe_dir) {
        match windows_temp() {
            Ok(path) => path,
            Err(message) => error_line(&escape(&message)),
        }
    } else {
        exe_dir.join("temp").to_string_lossy().into_owned()
    };
    let cwd = std::env::current_dir().unwrap_or_default();
    abspath(&chosen, &cwd)
}

/// Third word of the first line, without commas and `-static`, at most 30
/// characters.
pub fn parse_version_output(stdout: &str) -> Option<String> {
    let line = stdout.split('\n').next()?;
    let word = line.split(' ').nth(2)?;
    let version = word.replace(',', "").replace("-static", "");
    Some(if version.chars().count() > 30 {
        let head: String = version.chars().take(27).collect();
        format!("{head}...")
    } else {
        version
    })
}

pub const COULDNT_PARSE: &str = "[red]couldn't parse";

/// Version of a program (it is run with `args`); `None` if it cannot be
/// run or its output is not recognised.
pub fn version_of(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    parse_version_output(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mz_header() {
        assert!(is_mz(b"MZ\x90\x00"));
        assert!(!is_mz(b"#!"));
        assert!(!is_mz(b"M"));
        assert!(!is_mz(b""));
    }

    #[test]
    fn dee_kinds() {
        assert_eq!(
            DeeKind::new(true, false),
            DeeKind {
                is_exe: true,
                nonnative: true
            }
        );
        assert_eq!(
            DeeKind::new(true, true),
            DeeKind {
                is_exe: true,
                nonnative: false
            }
        );
        assert_eq!(
            DeeKind::new(false, false),
            DeeKind {
                is_exe: false,
                nonnative: false
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn abspath_normalises() {
        let cwd = Path::new("/home/u/work");
        assert_eq!(abspath("temp", cwd), "/home/u/work/temp");
        assert_eq!(abspath("./a/../b/./c", cwd), "/home/u/work/b/c");
        assert_eq!(abspath("../x", cwd), "/home/u/x");
        assert_eq!(abspath("/abs//path/", cwd), "/abs/path");
        assert_eq!(abspath("/a/b/../../..", cwd), "/");
        assert_eq!(abspath("/..", cwd), "/");
        assert_eq!(
            abspath("/mnt/c/Users/me/AppData/Local/Temp/deew", cwd),
            "/mnt/c/Users/me/AppData/Local/Temp/deew"
        );
    }

    #[cfg(windows)]
    #[test]
    fn abspath_on_windows() {
        let cwd = Path::new("C:\\Users\\me\\work");
        for (path, expected) in [
            ("temp", "C:\\Users\\me\\work\\temp"),
            ("C:\\dir\\file.wav", "C:\\dir\\file.wav"),
            ("C:/dir/file.wav", "C:\\dir\\file.wav"),
            ("c:/dir//sub/../x/", "c:\\dir\\x"),
            ("D:/tmp/", "D:\\tmp"),
            ("..\\x", "C:\\Users\\me\\x"),
            ("./a/../b", "C:\\Users\\me\\work\\b"),
            ("\\foo", "C:\\foo"),
            ("/foo/bar", "C:\\foo\\bar"),
            ("C:\\", "C:\\"),
            ("C:\\a\\b\\..\\..\\..", "C:\\"),
            ("C:\\..", "C:\\"),
            ("\\\\srv\\share\\x\\..\\y", "\\\\srv\\share\\y"),
        ] {
            assert_eq!(abspath(path, cwd), expected, "{path:?}");
        }
        // A drive-relative path of another drive is resolved by the OS.
        let other = abspath("D:foo", cwd);
        assert!(
            other.starts_with("D:\\") && other.ends_with("\\foo"),
            "{other}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn default_temp_is_next_to_the_executable_on_windows() {
        let exe_dir = Path::new("C:\\Program Files\\deew");
        assert_eq!(
            temp_path("", false, exe_dir),
            "C:\\Program Files\\deew\\temp"
        );
        assert_eq!(temp_path("D:/work/tmp/", false, exe_dir), "D:\\work\\tmp");
    }

    #[test]
    fn windows_temp_directory() {
        assert_eq!(
            windows_temp_to_local("C:\\Users\\me\\AppData\\Local\\Temp\r\n"),
            "/mnt/c/Users/me/AppData/Local/Temp/deew"
        );
        assert_eq!(windows_temp_to_local("D:\\"), "/mnt/d/deew");
        assert_eq!(windows_temp_to_local(""), "deew");
    }

    #[cfg(unix)]
    #[test]
    fn explicit_temp_path_wins() {
        let exe_dir = Path::new("/opt/deew");
        assert_eq!(temp_path("/data/tmp/", false, exe_dir), "/data/tmp");
        assert_eq!(temp_path("", false, exe_dir), "/opt/deew/temp");
        assert_eq!(temp_path("/data/tmp", true, exe_dir), "/data/tmp");
        assert_eq!(
            temp_path("", true, Path::new("/mnt/c/tools")),
            "/mnt/c/tools/temp"
        );
    }

    #[test]
    fn windows_drive_detection() {
        for (path, expected) in [
            ("/mnt/c", true),
            ("/mnt/c/tools", true),
            ("/mnt/D/tools", true),
            ("/mnt/wsl/tools", false),
            ("/mnt/", false),
            ("/home/me/.local/bin", false),
            ("/usr/local/bin", false),
        ] {
            assert_eq!(on_windows_drive(Path::new(path)), expected, "{path:?}");
        }
    }

    #[test]
    fn version_parsing() {
        assert_eq!(
            parse_version_output("dee, Version 5.2.1, Jun 10 2022\nmore").as_deref(),
            Some("5.2.1")
        );
        assert_eq!(
            parse_version_output("ffmpeg version 7.1.5 Copyright (c) 2000-2026\n").as_deref(),
            Some("7.1.5")
        );
        assert_eq!(
            parse_version_output("ffmpeg version 6.0-static https://x\n").as_deref(),
            Some("6.0")
        );
        let long = format!("ffmpeg version {} x", "a".repeat(40));
        assert_eq!(
            parse_version_output(&long),
            Some(format!("{}...", "a".repeat(27)))
        );
        let exact = format!("ffmpeg version {} x", "a".repeat(30));
        assert_eq!(parse_version_output(&exact), Some("a".repeat(30)));
        assert_eq!(parse_version_output(""), None);
        assert_eq!(parse_version_output("one two"), None);
    }

    #[test]
    fn missing_program_is_reported() {
        assert_eq!(version_of("/nonexistent/program", &[]), None);
    }
}
