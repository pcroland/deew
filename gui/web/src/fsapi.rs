//! Directory listing for the file browser (`GET /api/fs`).

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub dir: bool,
    pub hidden: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct FsListing {
    pub path: String,
    pub parent: Option<String>,
    pub roots: Vec<String>,
    pub entries: Vec<FsEntry>,
}

/// The file system roots: `/` on Unix, the existing drive letters on Windows.
pub fn roots() -> Vec<String> {
    #[cfg(windows)]
    {
        ('A'..='Z')
            .map(|letter| format!("{letter}:\\"))
            .filter(|root| std::path::Path::new(root).exists())
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec!["/".to_string()]
    }
}

fn is_hidden(name: &str, _meta: Option<&std::fs::Metadata>) -> bool {
    if name.starts_with('.') {
        return true;
    }
    #[cfg(windows)]
    if let Some(meta) = _meta {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        return meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
    }
    false
}

/// Lists `path` (the home directory when `None`).
pub fn list_dir(path: Option<&str>) -> Result<FsListing, String> {
    let requested: PathBuf = match path.filter(|p| !p.is_empty()) {
        Some(p) => PathBuf::from(p),
        None => std::env::home_dir().ok_or("cannot determine the home directory")?,
    };
    if !requested.is_absolute() {
        return Err(format!("not an absolute path: {}", requested.display()));
    }
    let dir = dunce::canonicalize(&requested)
        .map_err(|e| format!("cannot open {}: {e}", requested.display()))?;
    if !dir.is_dir() {
        return Err(format!("not a directory: {}", dir.display()));
    }
    let read =
        std::fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;

    let mut entries = Vec::new();
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let full = entry.path();
        // Follow symlinks; a broken link counts as a file.
        let meta = std::fs::metadata(&full)
            .ok()
            .or_else(|| entry.metadata().ok());
        let is_dir = meta.as_ref().is_some_and(|m| m.is_dir());
        // The hidden attribute belongs to the entry itself, not a link target.
        let own = entry.metadata().ok();
        entries.push(FsEntry {
            hidden: is_hidden(&name, own.as_ref().or(meta.as_ref())),
            path: full.to_string_lossy().into_owned(),
            name,
            dir: is_dir,
        });
    }
    entries.sort_by(|a, b| {
        b.dir
            .cmp(&a.dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(FsListing {
        parent: dir.parent().map(|p| p.to_string_lossy().into_owned()),
        path: dir.to_string_lossy().into_owned(),
        roots: roots(),
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_dirs_first_case_insensitive_and_flags_hidden() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path();
        std::fs::create_dir(p.join("beta")).unwrap();
        std::fs::create_dir(p.join("Alpha")).unwrap();
        std::fs::create_dir(p.join(".git")).unwrap();
        std::fs::write(p.join("b.wav"), b"").unwrap();
        std::fs::write(p.join("A.wav"), b"").unwrap();
        std::fs::write(p.join(".hidden"), b"").unwrap();

        let listing = list_dir(Some(p.to_str().unwrap())).unwrap();
        let names: Vec<_> = listing.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            [".git", "Alpha", "beta", ".hidden", "A.wav", "b.wav"]
        );
        let dirs: Vec<_> = listing.entries.iter().map(|e| e.dir).collect();
        assert_eq!(dirs, [true, true, true, false, false, false]);
        let hidden: Vec<_> = listing.entries.iter().map(|e| e.hidden).collect();
        assert_eq!(hidden, [true, false, false, true, false, false]);
        assert_eq!(
            listing.entries[1].path,
            listing.path.clone() + std::path::MAIN_SEPARATOR_STR + "Alpha"
        );
        assert!(listing.parent.is_some());
        assert!(!listing.roots.is_empty());
    }

    #[test]
    fn errors() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("f");
        std::fs::write(&file, b"").unwrap();
        assert!(list_dir(Some(file.to_str().unwrap())).is_err());
        assert!(list_dir(Some("relative/dir")).is_err());
        assert!(list_dir(Some(tmp.path().join("missing").to_str().unwrap())).is_err());
    }

    #[test]
    fn default_is_home() {
        if std::env::home_dir().is_some_and(|h| h.is_dir()) {
            assert!(list_dir(None).is_ok());
        }
    }

    #[cfg(unix)]
    #[test]
    fn root_has_no_parent() {
        assert_eq!(list_dir(Some("/")).unwrap().parent, None);
    }
}
